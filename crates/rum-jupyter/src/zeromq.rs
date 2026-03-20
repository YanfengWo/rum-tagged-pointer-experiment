use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        mpsc::{SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use futures::{
    FutureExt, StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
    pin_mut, select,
};
use rum_runtime::Session;
use zeromq::{PubSocket, RepSocket, RouterSocket, ZmqMessage, prelude::*};

use crate::{
    KernelBootstrapConfig, KernelChannel, KernelQueuedTransport, KernelSocketDriver,
    KernelSocketSession, KernelSocketSessionConfig, KernelTransport, KernelTransportBackend,
    KernelTransportLayout, KernelWireMessage,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZeroMqBackendError {
    SocketCreation {
        kind: &'static str,
        message: String,
    },
    SocketSetup {
        endpoint: String,
        operation: &'static str,
        message: String,
    },
    InvalidWireFrames {
        channel: KernelChannel,
        message: String,
    },
}

pub trait ZeroMqSocket {
    fn bind(&mut self, endpoint: &str) -> Result<(), String>;
    fn connect(&mut self, endpoint: &str) -> Result<(), String>;
    fn send_multipart(&mut self, frames: Vec<Vec<u8>>);
    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>>;
}

pub trait ZeroMqSocketFactory {
    type Socket: ZeroMqSocket;

    fn router_socket(&mut self) -> Result<Self::Socket, String>;
    fn pub_socket(&mut self) -> Result<Self::Socket, String>;
    fn rep_socket(&mut self) -> Result<Self::Socket, String>;
}

struct ZeroMqSocketState<S> {
    shell: S,
    control: S,
    stdin: S,
    iopub: S,
    heartbeat: S,
    incoming_shell: VecDeque<KernelWireMessage>,
    incoming_control: VecDeque<KernelWireMessage>,
    incoming_stdin: VecDeque<KernelWireMessage>,
    heartbeat_incoming: VecDeque<Vec<u8>>,
}

impl<S> ZeroMqSocketState<S> {
    fn incoming_queue_mut(
        &mut self,
        channel: KernelChannel,
    ) -> Option<&mut VecDeque<KernelWireMessage>> {
        match channel {
            KernelChannel::Shell => Some(&mut self.incoming_shell),
            KernelChannel::Control => Some(&mut self.incoming_control),
            KernelChannel::IoPub => None,
            KernelChannel::Stdin => Some(&mut self.incoming_stdin),
        }
    }

    fn has_buffered_channel(&self, channel: KernelChannel) -> bool {
        match channel {
            KernelChannel::Shell => !self.incoming_shell.is_empty(),
            KernelChannel::Control => !self.incoming_control.is_empty(),
            KernelChannel::IoPub => false,
            KernelChannel::Stdin => !self.incoming_stdin.is_empty(),
        }
    }
}

pub struct ZeroMqSocketSession<S> {
    session_config: KernelSocketSessionConfig,
    delimiter: String,
    state: RefCell<ZeroMqSocketState<S>>,
}

impl<S: ZeroMqSocket> ZeroMqSocketSession<S> {
    pub fn from_bootstrap<F>(
        bootstrap_config: &KernelBootstrapConfig,
        factory: &mut F,
    ) -> Result<Self, ZeroMqBackendError>
    where
        F: ZeroMqSocketFactory<Socket = S>,
    {
        let session_config = bootstrap_config.socket_session_config();
        let mut shell =
            factory
                .router_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "router",
                    message,
                })?;
        let mut control =
            factory
                .router_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "router",
                    message,
                })?;
        let mut stdin =
            factory
                .router_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "router",
                    message,
                })?;
        let mut iopub =
            factory
                .pub_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "pub",
                    message,
                })?;
        let mut heartbeat =
            factory
                .rep_socket()
                .map_err(|message| ZeroMqBackendError::SocketCreation {
                    kind: "rep",
                    message,
                })?;

        setup_socket(&mut shell, &session_config.shell)?;
        setup_socket(&mut control, &session_config.control)?;
        setup_socket(&mut stdin, &session_config.stdin)?;
        setup_socket(&mut iopub, &session_config.iopub)?;
        setup_socket(&mut heartbeat, &session_config.heartbeat)?;

        Ok(Self {
            session_config,
            delimiter: bootstrap_config.wire_config().delimiter,
            state: RefCell::new(ZeroMqSocketState {
                shell,
                control,
                stdin,
                iopub,
                heartbeat,
                incoming_shell: VecDeque::new(),
                incoming_control: VecDeque::new(),
                incoming_stdin: VecDeque::new(),
                heartbeat_incoming: VecDeque::new(),
            }),
        })
    }
}

impl<S: ZeroMqSocket> KernelTransport for ZeroMqSocketSession<S> {
    fn send(&mut self, message: KernelWireMessage) {
        let frames = wire_message_to_frames(message);
        let mut state = self.state.borrow_mut();
        match frames.channel {
            KernelChannel::Shell => state.shell.send_multipart(frames.frames),
            KernelChannel::Control => state.control.send_multipart(frames.frames),
            KernelChannel::IoPub => state.iopub.send_multipart(frames.frames),
            KernelChannel::Stdin => state.stdin.send_multipart(frames.frames),
        }
    }
}

impl<S: ZeroMqSocket> KernelQueuedTransport for ZeroMqSocketSession<S> {
    fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage> {
        self.state
            .borrow_mut()
            .incoming_queue_mut(channel)
            .and_then(VecDeque::pop_front)
    }

    fn has_pending_incoming(&self) -> bool {
        let state = self.state.borrow();
        state.has_buffered_channel(KernelChannel::Shell)
            || state.has_buffered_channel(KernelChannel::Control)
            || state.has_buffered_channel(KernelChannel::Stdin)
    }
}

impl<S: ZeroMqSocket> KernelSocketSession for ZeroMqSocketSession<S> {
    fn session_config(&self) -> &KernelSocketSessionConfig {
        &self.session_config
    }

    fn has_pending_channel(&self, channel: KernelChannel) -> bool {
        let mut state = self.state.borrow_mut();
        if state.has_buffered_channel(channel) {
            return true;
        }

        let frames = match channel {
            KernelChannel::Shell => state.shell.try_recv_multipart(),
            KernelChannel::Control => state.control.try_recv_multipart(),
            KernelChannel::IoPub => None,
            KernelChannel::Stdin => state.stdin.try_recv_multipart(),
        };

        if let Some(frames) = frames {
            let message = frames_to_wire_message(channel, &self.delimiter, frames)
                .expect("ZeroMQ session should only surface valid Jupyter multipart frames");
            state
                .incoming_queue_mut(channel)
                .expect("channel-backed incoming queues exist")
                .push_back(message);
            true
        } else {
            false
        }
    }

    fn has_pending_heartbeat(&self) -> bool {
        let mut state = self.state.borrow_mut();
        if !state.heartbeat_incoming.is_empty() {
            return true;
        }

        if let Some(frames) = state.heartbeat.try_recv_multipart() {
            if let Some(frame) = frames.into_iter().next() {
                state.heartbeat_incoming.push_back(frame);
                return true;
            }
        }

        false
    }

    fn pop_heartbeat(&mut self) -> Option<Vec<u8>> {
        self.state.borrow_mut().heartbeat_incoming.pop_front()
    }

    fn send_heartbeat(&mut self, payload: Vec<u8>) {
        self.state
            .borrow_mut()
            .heartbeat
            .send_multipart(vec![payload]);
    }
}

pub struct ZeroMqTransportBackend<S> {
    bootstrap_config: KernelBootstrapConfig,
    transport_layout: KernelTransportLayout,
    socket_session_config: KernelSocketSessionConfig,
    transport: ZeroMqSocketSession<S>,
}

impl<S: ZeroMqSocket> ZeroMqTransportBackend<S> {
    pub fn from_factory<F>(
        bootstrap_config: KernelBootstrapConfig,
        factory: &mut F,
    ) -> Result<Self, ZeroMqBackendError>
    where
        F: ZeroMqSocketFactory<Socket = S>,
    {
        let transport_layout = bootstrap_config.transport_layout();
        let socket_session_config = bootstrap_config.socket_session_config();
        let transport = ZeroMqSocketSession::from_bootstrap(&bootstrap_config, factory)?;

        Ok(Self {
            bootstrap_config,
            transport_layout,
            socket_session_config,
            transport,
        })
    }

    pub fn event_pump(&self, session: Session) -> crate::KernelEventPump {
        self.bootstrap_config.event_pump(session)
    }

    pub fn socket_driver(&self, session: Session) -> KernelSocketDriver {
        self.bootstrap_config.socket_driver(session)
    }

    pub fn socket_session_config(&self) -> &KernelSocketSessionConfig {
        &self.socket_session_config
    }
}

impl<S> KernelTransportBackend for ZeroMqTransportBackend<S> {
    type Transport = ZeroMqSocketSession<S>;

    fn bootstrap_config(&self) -> &KernelBootstrapConfig {
        &self.bootstrap_config
    }

    fn transport_layout(&self) -> &KernelTransportLayout {
        &self.transport_layout
    }

    fn transport(&self) -> &Self::Transport {
        &self.transport
    }

    fn transport_mut(&mut self) -> &mut Self::Transport {
        &mut self.transport
    }
}

struct MultipartFrames {
    channel: KernelChannel,
    frames: Vec<Vec<u8>>,
}

fn wire_message_to_frames(message: KernelWireMessage) -> MultipartFrames {
    let mut frames = message.routing_ids;
    frames.push(message.delimiter.into_bytes());
    frames.push(message.signature.unwrap_or_default().into_bytes());
    frames.push(message.header_json.into_bytes());
    frames.push(message.parent_header_json.into_bytes());
    frames.push(message.metadata_json.into_bytes());
    frames.push(message.content_json.into_bytes());
    frames.extend(message.buffers);
    MultipartFrames {
        channel: message.channel,
        frames,
    }
}

fn frames_to_wire_message(
    channel: KernelChannel,
    delimiter: &str,
    frames: Vec<Vec<u8>>,
) -> Result<KernelWireMessage, ZeroMqBackendError> {
    let delimiter_bytes = delimiter.as_bytes();
    let Some(delimiter_index) = frames.iter().position(|frame| frame == delimiter_bytes) else {
        return Err(ZeroMqBackendError::InvalidWireFrames {
            channel,
            message: "missing delimiter frame".to_string(),
        });
    };

    if frames.len() < delimiter_index + 5 {
        return Err(ZeroMqBackendError::InvalidWireFrames {
            channel,
            message: "missing required signed payload frames".to_string(),
        });
    }

    let routing_ids = frames[..delimiter_index].to_vec();
    let payload_frames = &frames[delimiter_index + 1..];
    let signature = bytes_to_string(channel, "signature", &payload_frames[0])?;
    let header_json = bytes_to_string(channel, "header", &payload_frames[1])?;
    let parent_header_json = bytes_to_string(channel, "parent_header", &payload_frames[2])?;
    let metadata_json = bytes_to_string(channel, "metadata", &payload_frames[3])?;
    let content_json = bytes_to_string(channel, "content", &payload_frames[4])?;
    let buffers = payload_frames[5..].to_vec();

    Ok(KernelWireMessage {
        channel,
        routing_ids,
        delimiter: delimiter.to_string(),
        signature: if signature.is_empty() {
            None
        } else {
            Some(signature)
        },
        header_json,
        parent_header_json,
        metadata_json,
        content_json,
        buffers,
    })
}

fn bytes_to_string(
    channel: KernelChannel,
    frame_name: &str,
    bytes: &[u8],
) -> Result<String, ZeroMqBackendError> {
    String::from_utf8(bytes.to_vec()).map_err(|error| ZeroMqBackendError::InvalidWireFrames {
        channel,
        message: format!("{frame_name} frame was not valid utf-8: {error}"),
    })
}

fn setup_socket<S: ZeroMqSocket>(
    socket: &mut S,
    spec: &crate::KernelSocketSpec,
) -> Result<(), ZeroMqBackendError> {
    let endpoint = spec.uri();
    let result = match spec.bind_mode {
        crate::KernelSocketBindMode::Bind => socket.bind(&endpoint),
        crate::KernelSocketBindMode::Connect => socket.connect(&endpoint),
    };

    result.map_err(|message| ZeroMqBackendError::SocketSetup {
        endpoint,
        operation: match spec.bind_mode {
            crate::KernelSocketBindMode::Bind => "bind",
            crate::KernelSocketBindMode::Connect => "connect",
        },
        message,
    })
}

#[derive(Default)]
pub struct NativeZeroMqFactory;

pub type NativeZeroMqTransportBackend = ZeroMqTransportBackend<NativeZeroMqSocket>;

impl ZeroMqTransportBackend<NativeZeroMqSocket> {
    pub fn new_native(bootstrap_config: KernelBootstrapConfig) -> Result<Self, ZeroMqBackendError> {
        let mut factory = NativeZeroMqFactory;
        Self::from_factory(bootstrap_config, &mut factory)
    }
}

pub struct NativeZeroMqSocket(NativeZeroMqSocketInner);

enum NativeZeroMqSocketInner {
    Router(NativeRouterSocket),
    Pub(NativePubSocket),
    Rep(NativeRepSocket),
}

impl ZeroMqSocketFactory for NativeZeroMqFactory {
    type Socket = NativeZeroMqSocket;

    fn router_socket(&mut self) -> Result<Self::Socket, String> {
        Ok(NativeZeroMqSocket(NativeZeroMqSocketInner::Router(
            NativeRouterSocket::new(),
        )))
    }

    fn pub_socket(&mut self) -> Result<Self::Socket, String> {
        Ok(NativeZeroMqSocket(NativeZeroMqSocketInner::Pub(
            NativePubSocket::new(),
        )))
    }

    fn rep_socket(&mut self) -> Result<Self::Socket, String> {
        Ok(NativeZeroMqSocket(NativeZeroMqSocketInner::Rep(
            NativeRepSocket::new(),
        )))
    }
}

impl ZeroMqSocket for NativeZeroMqSocket {
    fn bind(&mut self, endpoint: &str) -> Result<(), String> {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.bind(endpoint),
            NativeZeroMqSocketInner::Pub(socket) => socket.bind(endpoint),
            NativeZeroMqSocketInner::Rep(socket) => socket.bind(endpoint),
        }
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.connect(endpoint),
            NativeZeroMqSocketInner::Pub(socket) => socket.connect(endpoint),
            NativeZeroMqSocketInner::Rep(socket) => socket.connect(endpoint),
        }
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.send_multipart(frames),
            NativeZeroMqSocketInner::Pub(socket) => socket.send_multipart(frames),
            NativeZeroMqSocketInner::Rep(socket) => socket.send_multipart(frames),
        }
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        match &mut self.0 {
            NativeZeroMqSocketInner::Router(socket) => socket.try_recv_multipart(),
            NativeZeroMqSocketInner::Pub(socket) => socket.try_recv_multipart(),
            NativeZeroMqSocketInner::Rep(socket) => socket.try_recv_multipart(),
        }
    }
}

enum ZeroMqLifecycleCommand {
    Bind {
        endpoint: String,
        reply: SyncSender<Result<(), String>>,
    },
    Connect {
        endpoint: String,
        reply: SyncSender<Result<(), String>>,
    },
    Start {
        reply: SyncSender<()>,
    },
}

enum NativeRouterSocketCommand {
    Lifecycle(ZeroMqLifecycleCommand),
    Send(Vec<Vec<u8>>),
}

enum NativePubSocketCommand {
    Lifecycle(ZeroMqLifecycleCommand),
    Send(Vec<Vec<u8>>),
}

enum NativeRepSocketCommand {
    Lifecycle(ZeroMqLifecycleCommand),
    Send(Vec<Vec<u8>>),
}

struct NativeRouterSocket {
    started: bool,
    handle: NativeRouterSocketHandle,
}

impl NativeRouterSocket {
    fn new() -> Self {
        Self {
            started: false,
            handle: NativeRouterSocketHandle::spawn(),
        }
    }

    fn bind(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.bind(endpoint)
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.connect(endpoint)
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.ensure_started();
        self.handle.send(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        self.ensure_started();
        self.handle.try_recv()
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.handle.start();
            self.started = true;
        }
    }
}

struct NativePubSocket {
    started: bool,
    handle: NativePubSocketHandle,
}

impl NativePubSocket {
    fn new() -> Self {
        Self {
            started: false,
            handle: NativePubSocketHandle::spawn(),
        }
    }

    fn bind(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.bind(endpoint)
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.connect(endpoint)
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.ensure_started();
        self.handle.send(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        None
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.handle.start();
            self.started = true;
        }
    }
}

struct NativeRepSocket {
    started: bool,
    handle: NativeRepSocketHandle,
}

impl NativeRepSocket {
    fn new() -> Self {
        Self {
            started: false,
            handle: NativeRepSocketHandle::spawn(),
        }
    }

    fn bind(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.bind(endpoint)
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.handle.connect(endpoint)
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.ensure_started();
        self.handle.send(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        self.ensure_started();
        self.handle.try_recv()
    }

    fn ensure_started(&mut self) {
        if !self.started {
            self.handle.start();
            self.started = true;
        }
    }
}

struct NativeRouterSocketHandle {
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
    command_tx: Option<UnboundedSender<NativeRouterSocketCommand>>,
    join_handle: Option<JoinHandle<()>>,
}

struct NativePubSocketHandle {
    command_tx: Option<UnboundedSender<NativePubSocketCommand>>,
    join_handle: Option<JoinHandle<()>>,
}

struct NativeRepSocketHandle {
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
    command_tx: Option<UnboundedSender<NativeRepSocketCommand>>,
    join_handle: Option<JoinHandle<()>>,
}

impl NativeRouterSocketHandle {
    fn spawn() -> Self {
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = unbounded();
        let incoming_clone = incoming.clone();
        let join_handle = Some(spawn_zeromq_worker("rum-zmq-router", async move {
            run_router_socket_worker(command_rx, incoming_clone).await;
        }));

        Self {
            incoming,
            command_tx: Some(command_tx),
            join_handle,
        }
    }

    fn bind(&self, endpoint: &str) -> Result<(), String> {
        self.send_lifecycle_with_result(ZeroMqLifecycleCommand::Bind {
            endpoint: endpoint.to_string(),
            reply: lifecycle_reply_channel().0,
        })
    }

    fn connect(&self, endpoint: &str) -> Result<(), String> {
        self.send_lifecycle_with_result(ZeroMqLifecycleCommand::Connect {
            endpoint: endpoint.to_string(),
            reply: lifecycle_reply_channel().0,
        })
    }

    fn send_lifecycle_with_result(&self, command: ZeroMqLifecycleCommand) -> Result<(), String> {
        let (reply, receiver) = lifecycle_reply_channel();
        let command = match command {
            ZeroMqLifecycleCommand::Bind { endpoint, .. } => {
                ZeroMqLifecycleCommand::Bind { endpoint, reply }
            }
            ZeroMqLifecycleCommand::Connect { endpoint, .. } => {
                ZeroMqLifecycleCommand::Connect { endpoint, reply }
            }
            ZeroMqLifecycleCommand::Start { .. } => unreachable!("start does not return Result"),
        };
        self.command_tx
            .as_ref()
            .expect("router worker sender should be present")
            .unbounded_send(NativeRouterSocketCommand::Lifecycle(command))
            .map_err(|_| "router worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "router worker did not acknowledge setup".to_string())?
    }

    fn start(&self) {
        let (reply, receiver) = sync_channel(1);
        if self
            .command_tx
            .as_ref()
            .expect("router worker sender should be present")
            .unbounded_send(NativeRouterSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Start { reply },
            ))
            .is_ok()
        {
            let _ = receiver.recv();
        }
    }

    fn send(&self, frames: Vec<Vec<u8>>) {
        if let Some(sender) = &self.command_tx {
            let _ = sender.unbounded_send(NativeRouterSocketCommand::Send(frames));
        }
    }

    fn try_recv(&self) -> Option<Vec<Vec<u8>>> {
        self.incoming
            .lock()
            .expect("router incoming queue should not be poisoned")
            .pop_front()
    }
}

impl Drop for NativeRouterSocketHandle {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl NativePubSocketHandle {
    fn spawn() -> Self {
        let (command_tx, command_rx) = unbounded();
        let join_handle = Some(spawn_zeromq_worker("rum-zmq-pub", async move {
            run_pub_socket_worker(command_rx).await;
        }));

        Self {
            command_tx: Some(command_tx),
            join_handle,
        }
    }

    fn bind(&self, endpoint: &str) -> Result<(), String> {
        self.send_lifecycle_with_result(ZeroMqLifecycleCommand::Bind {
            endpoint: endpoint.to_string(),
            reply: lifecycle_reply_channel().0,
        })
    }

    fn connect(&self, endpoint: &str) -> Result<(), String> {
        self.send_lifecycle_with_result(ZeroMqLifecycleCommand::Connect {
            endpoint: endpoint.to_string(),
            reply: lifecycle_reply_channel().0,
        })
    }

    fn send_lifecycle_with_result(&self, command: ZeroMqLifecycleCommand) -> Result<(), String> {
        let (reply, receiver) = lifecycle_reply_channel();
        let command = match command {
            ZeroMqLifecycleCommand::Bind { endpoint, .. } => {
                ZeroMqLifecycleCommand::Bind { endpoint, reply }
            }
            ZeroMqLifecycleCommand::Connect { endpoint, .. } => {
                ZeroMqLifecycleCommand::Connect { endpoint, reply }
            }
            ZeroMqLifecycleCommand::Start { .. } => unreachable!("start does not return Result"),
        };
        self.command_tx
            .as_ref()
            .expect("pub worker sender should be present")
            .unbounded_send(NativePubSocketCommand::Lifecycle(command))
            .map_err(|_| "pub worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "pub worker did not acknowledge setup".to_string())?
    }

    fn start(&self) {
        let (reply, receiver) = sync_channel(1);
        if self
            .command_tx
            .as_ref()
            .expect("pub worker sender should be present")
            .unbounded_send(NativePubSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Start { reply },
            ))
            .is_ok()
        {
            let _ = receiver.recv();
        }
    }

    fn send(&self, frames: Vec<Vec<u8>>) {
        if let Some(sender) = &self.command_tx {
            let _ = sender.unbounded_send(NativePubSocketCommand::Send(frames));
        }
    }
}

impl Drop for NativePubSocketHandle {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl NativeRepSocketHandle {
    fn spawn() -> Self {
        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = unbounded();
        let incoming_clone = incoming.clone();
        let join_handle = Some(spawn_zeromq_worker("rum-zmq-rep", async move {
            run_rep_socket_worker(command_rx, incoming_clone).await;
        }));

        Self {
            incoming,
            command_tx: Some(command_tx),
            join_handle,
        }
    }

    fn bind(&self, endpoint: &str) -> Result<(), String> {
        self.send_lifecycle_with_result(ZeroMqLifecycleCommand::Bind {
            endpoint: endpoint.to_string(),
            reply: lifecycle_reply_channel().0,
        })
    }

    fn connect(&self, endpoint: &str) -> Result<(), String> {
        self.send_lifecycle_with_result(ZeroMqLifecycleCommand::Connect {
            endpoint: endpoint.to_string(),
            reply: lifecycle_reply_channel().0,
        })
    }

    fn send_lifecycle_with_result(&self, command: ZeroMqLifecycleCommand) -> Result<(), String> {
        let (reply, receiver) = lifecycle_reply_channel();
        let command = match command {
            ZeroMqLifecycleCommand::Bind { endpoint, .. } => {
                ZeroMqLifecycleCommand::Bind { endpoint, reply }
            }
            ZeroMqLifecycleCommand::Connect { endpoint, .. } => {
                ZeroMqLifecycleCommand::Connect { endpoint, reply }
            }
            ZeroMqLifecycleCommand::Start { .. } => unreachable!("start does not return Result"),
        };
        self.command_tx
            .as_ref()
            .expect("rep worker sender should be present")
            .unbounded_send(NativeRepSocketCommand::Lifecycle(command))
            .map_err(|_| "rep worker is no longer running".to_string())?;
        receiver
            .recv()
            .map_err(|_| "rep worker did not acknowledge setup".to_string())?
    }

    fn start(&self) {
        let (reply, receiver) = sync_channel(1);
        if self
            .command_tx
            .as_ref()
            .expect("rep worker sender should be present")
            .unbounded_send(NativeRepSocketCommand::Lifecycle(
                ZeroMqLifecycleCommand::Start { reply },
            ))
            .is_ok()
        {
            let _ = receiver.recv();
        }
    }

    fn send(&self, frames: Vec<Vec<u8>>) {
        if let Some(sender) = &self.command_tx {
            let _ = sender.unbounded_send(NativeRepSocketCommand::Send(frames));
        }
    }

    fn try_recv(&self) -> Option<Vec<Vec<u8>>> {
        self.incoming
            .lock()
            .expect("rep incoming queue should not be poisoned")
            .pop_front()
    }
}

impl Drop for NativeRepSocketHandle {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

fn lifecycle_reply_channel() -> (
    SyncSender<Result<(), String>>,
    std::sync::mpsc::Receiver<Result<(), String>>,
) {
    sync_channel(1)
}

fn spawn_zeromq_worker<F>(name: &str, future: F) -> JoinHandle<()>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("zeromq worker runtime should build");
            runtime.block_on(future);
        })
        .expect("zeromq worker thread should spawn")
}

async fn run_router_socket_worker(
    mut command_rx: UnboundedReceiver<NativeRouterSocketCommand>,
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
) {
    let mut socket = RouterSocket::new();
    if !prepare_router_socket(&mut socket, &mut command_rx).await {
        return;
    }
    let (mut send_half, mut recv_half) = socket.split();

    loop {
        let next_command = command_rx.next().fuse();
        let next_message = recv_half.recv().fuse();
        pin_mut!(next_command, next_message);

        select! {
            command = next_command => {
                match command {
                    Some(NativeRouterSocketCommand::Send(frames)) => {
                        if send_half.send(frames_to_zmq_message(frames)).await.is_err() {
                            break;
                        }
                    }
                    Some(NativeRouterSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start { reply })) => {
                        let _ = reply.send(());
                    }
                    Some(NativeRouterSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind { reply, .. }))
                    | Some(NativeRouterSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect { reply, .. })) => {
                        let _ = reply.send(Err("router socket cannot be rebound after it starts receiving".to_string()));
                    }
                    None => break,
                }
            }
            message = next_message => {
                match message {
                    Ok(message) => {
                        incoming
                            .lock()
                            .expect("router incoming queue should not be poisoned")
                            .push_back(zmq_message_to_frames(message));
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

async fn prepare_router_socket(
    socket: &mut RouterSocket,
    command_rx: &mut UnboundedReceiver<NativeRouterSocketCommand>,
) -> bool {
    while let Some(command) = command_rx.next().await {
        match command {
            NativeRouterSocketCommand::Lifecycle(control) => {
                if handle_lifecycle_command(socket, control).await {
                    return true;
                }
            }
            NativeRouterSocketCommand::Send(_) => {}
        }
    }

    false
}

async fn run_pub_socket_worker(mut command_rx: UnboundedReceiver<NativePubSocketCommand>) {
    let mut socket = PubSocket::new();
    if !prepare_pub_socket(&mut socket, &mut command_rx).await {
        return;
    }

    while let Some(command) = command_rx.next().await {
        match command {
            NativePubSocketCommand::Send(frames) => {
                if socket.send(frames_to_zmq_message(frames)).await.is_err() {
                    break;
                }
            }
            NativePubSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start { reply }) => {
                let _ = reply.send(());
            }
            NativePubSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind { reply, .. })
            | NativePubSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect {
                reply, ..
            }) => {
                let _ = reply.send(Err(
                    "pub socket cannot be rebound after it starts sending".to_string()
                ));
            }
        }
    }
}

async fn prepare_pub_socket(
    socket: &mut PubSocket,
    command_rx: &mut UnboundedReceiver<NativePubSocketCommand>,
) -> bool {
    while let Some(command) = command_rx.next().await {
        match command {
            NativePubSocketCommand::Lifecycle(control) => {
                if handle_lifecycle_command(socket, control).await {
                    return true;
                }
            }
            NativePubSocketCommand::Send(_) => {}
        }
    }

    false
}

async fn run_rep_socket_worker(
    mut command_rx: UnboundedReceiver<NativeRepSocketCommand>,
    incoming: Arc<Mutex<VecDeque<Vec<Vec<u8>>>>>,
) {
    let mut socket = RepSocket::new();
    if !prepare_rep_socket(&mut socket, &mut command_rx).await {
        return;
    }

    loop {
        enum RepPhase1 {
            Continue,
            Stop,
            Request(ZmqMessage),
        }

        let phase = {
            let next_command = command_rx.next().fuse();
            let next_message = socket.recv().fuse();
            pin_mut!(next_command, next_message);

            select! {
                command = next_command => {
                    match command {
                        Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start { reply })) => {
                            let _ = reply.send(());
                            RepPhase1::Continue
                        }
                        Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind { reply, .. }))
                        | Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect { reply, .. })) => {
                            let _ = reply.send(Err("rep socket cannot be rebound after it starts receiving".to_string()));
                            RepPhase1::Continue
                        }
                        Some(NativeRepSocketCommand::Send(_)) => RepPhase1::Continue,
                        None => RepPhase1::Stop,
                    }
                }
                message = next_message => {
                    match message {
                        Ok(message) => RepPhase1::Request(message),
                        Err(_) => RepPhase1::Stop,
                    }
                }
            }
        };

        let request = match phase {
            RepPhase1::Continue => continue,
            RepPhase1::Stop => return,
            RepPhase1::Request(request) => request,
        };

        incoming
            .lock()
            .expect("rep incoming queue should not be poisoned")
            .push_back(zmq_message_to_frames(request));

        loop {
            match command_rx.next().await {
                Some(NativeRepSocketCommand::Send(frames)) => {
                    if socket.send(frames_to_zmq_message(frames)).await.is_err() {
                        return;
                    }
                    break;
                }
                Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Start {
                    reply,
                })) => {
                    let _ = reply.send(());
                }
                Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Bind {
                    reply,
                    ..
                }))
                | Some(NativeRepSocketCommand::Lifecycle(ZeroMqLifecycleCommand::Connect {
                    reply,
                    ..
                })) => {
                    let _ = reply.send(Err(
                        "rep socket cannot be rebound after it starts receiving".to_string(),
                    ));
                }
                None => return,
            }
        }
    }
}

async fn prepare_rep_socket(
    socket: &mut RepSocket,
    command_rx: &mut UnboundedReceiver<NativeRepSocketCommand>,
) -> bool {
    while let Some(command) = command_rx.next().await {
        match command {
            NativeRepSocketCommand::Lifecycle(control) => {
                if handle_lifecycle_command(socket, control).await {
                    return true;
                }
            }
            NativeRepSocketCommand::Send(_) => {}
        }
    }

    false
}

async fn handle_lifecycle_command<S: zeromq::Socket>(
    socket: &mut S,
    command: ZeroMqLifecycleCommand,
) -> bool {
    match command {
        ZeroMqLifecycleCommand::Bind { endpoint, reply } => {
            let _ = reply.send(
                socket
                    .bind(&endpoint)
                    .await
                    .map(|_| ())
                    .map_err(|error| error.to_string()),
            );
            false
        }
        ZeroMqLifecycleCommand::Connect { endpoint, reply } => {
            let _ = reply.send(
                socket
                    .connect(&endpoint)
                    .await
                    .map_err(|error| error.to_string()),
            );
            false
        }
        ZeroMqLifecycleCommand::Start { reply } => {
            let _ = reply.send(());
            true
        }
    }
}

fn frames_to_zmq_message(frames: Vec<Vec<u8>>) -> ZmqMessage {
    let mut frames = frames.into_iter();
    let mut message = ZmqMessage::from(frames.next().unwrap_or_default());
    for frame in frames {
        message.push_back(Bytes::from(frame));
    }
    message
}

fn zmq_message_to_frames(message: ZmqMessage) -> Vec<Vec<u8>> {
    message
        .into_vec()
        .into_iter()
        .map(|frame| frame.to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener,
        time::{Duration, Instant},
    };

    use zeromq::{DealerSocket, ReqSocket};

    use super::*;
    use crate::{
        KernelConnectionFile, KernelMessageHeader, KernelMetadata, KernelReplyStatus,
        KernelRequest, KernelRequestEnvelope, KernelResponse, decode_response_message,
    };

    #[derive(Debug, Default)]
    struct FakeZeroMqSocketState {
        bound: Vec<String>,
        connected: Vec<String>,
        incoming: VecDeque<Vec<Vec<u8>>>,
        outgoing: Vec<Vec<Vec<u8>>>,
    }

    #[derive(Clone, Debug, Default)]
    struct FakeZeroMqSocketHandle(Arc<Mutex<FakeZeroMqSocketState>>);

    impl FakeZeroMqSocketHandle {
        fn push_incoming(&self, frames: Vec<Vec<u8>>) {
            self.0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned")
                .incoming
                .push_back(frames);
        }

        fn take_outgoing(&self) -> Vec<Vec<Vec<u8>>> {
            let mut state = self
                .0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned");
            std::mem::take(&mut state.outgoing)
        }

        fn bound_endpoints(&self) -> Vec<String> {
            self.0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned")
                .bound
                .clone()
        }
    }

    struct FakeZeroMqSocket {
        state: FakeZeroMqSocketHandle,
    }

    impl ZeroMqSocket for FakeZeroMqSocket {
        fn bind(&mut self, endpoint: &str) -> Result<(), String> {
            self.state
                .0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned")
                .bound
                .push(endpoint.to_string());
            Ok(())
        }

        fn connect(&mut self, endpoint: &str) -> Result<(), String> {
            self.state
                .0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned")
                .connected
                .push(endpoint.to_string());
            Ok(())
        }

        fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
            self.state
                .0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned")
                .outgoing
                .push(frames);
        }

        fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
            self.state
                .0
                .lock()
                .expect("fake zeromq socket lock should not be poisoned")
                .incoming
                .pop_front()
        }
    }

    #[derive(Default)]
    struct FakeZeroMqFactory {
        router_sockets: Vec<FakeZeroMqSocketHandle>,
        pub_sockets: Vec<FakeZeroMqSocketHandle>,
        rep_sockets: Vec<FakeZeroMqSocketHandle>,
    }

    impl ZeroMqSocketFactory for FakeZeroMqFactory {
        type Socket = FakeZeroMqSocket;

        fn router_socket(&mut self) -> Result<Self::Socket, String> {
            let handle = FakeZeroMqSocketHandle::default();
            self.router_sockets.push(handle.clone());
            Ok(FakeZeroMqSocket { state: handle })
        }

        fn pub_socket(&mut self) -> Result<Self::Socket, String> {
            let handle = FakeZeroMqSocketHandle::default();
            self.pub_sockets.push(handle.clone());
            Ok(FakeZeroMqSocket { state: handle })
        }

        fn rep_socket(&mut self) -> Result<Self::Socket, String> {
            let handle = FakeZeroMqSocketHandle::default();
            self.rep_sockets.push(handle.clone());
            Ok(FakeZeroMqSocket { state: handle })
        }
    }

    fn test_connection() -> KernelBootstrapConfig {
        KernelBootstrapConfig::from_connection_file(
            KernelConnectionFile::parse_json(
                r#"{
                    "transport": "tcp",
                    "ip": "127.0.0.1",
                    "shell_port": 57503,
                    "iopub_port": 40885,
                    "stdin_port": 52597,
                    "control_port": 50160,
                    "hb_port": 42540,
                    "key": "",
                    "signature_scheme": "",
                    "kernel_name": "rum"
                }"#,
            )
            .expect("connection file should parse"),
        )
        .with_username("rum-kernel")
        .with_session_validation("frontend-session")
    }

    fn loopback_connection() -> KernelBootstrapConfig {
        let listeners = (0..5)
            .map(|_| TcpListener::bind("127.0.0.1:0").expect("ephemeral port should bind"))
            .collect::<Vec<_>>();
        let ports = listeners
            .iter()
            .map(|listener| {
                listener
                    .local_addr()
                    .expect("listener should expose a local address")
                    .port()
            })
            .collect::<Vec<_>>();
        drop(listeners);

        KernelBootstrapConfig::from_connection_file(KernelConnectionFile {
            transport: crate::KernelTransportKind::Tcp,
            ip: "127.0.0.1".to_string(),
            shell_port: ports[0],
            iopub_port: ports[1],
            stdin_port: ports[2],
            control_port: ports[3],
            hb_port: ports[4],
            key: "".to_string(),
            signature_scheme: crate::KernelSignatureScheme::None,
            kernel_name: Some("rum".to_string()),
        })
        .with_username("rum-kernel")
        .with_session_validation("frontend-session")
    }

    fn test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime should build")
    }

    fn wait_for_socket_event(
        driver: &mut KernelSocketDriver,
        transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
        expected: crate::KernelSocketPollEvent,
    ) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let event = driver
                .pump_once(transport)
                .expect("socket driver should keep pumping");
            if event == Some(expected) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for socket event {expected:?}"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn zeromq_backend_binds_expected_socket_endpoints() {
        let bootstrap = test_connection();
        let mut factory = FakeZeroMqFactory::default();
        let backend = ZeroMqTransportBackend::from_factory(bootstrap.clone(), &mut factory)
            .expect("backend should build");

        assert_eq!(
            backend.socket_session_config().shell.kind.as_str(),
            "router"
        );
        assert_eq!(
            factory.router_sockets[0].bound_endpoints(),
            vec!["tcp://127.0.0.1:57503"]
        );
        assert_eq!(
            factory.router_sockets[1].bound_endpoints(),
            vec!["tcp://127.0.0.1:50160"]
        );
        assert_eq!(
            factory.router_sockets[2].bound_endpoints(),
            vec!["tcp://127.0.0.1:52597"]
        );
        assert_eq!(
            factory.pub_sockets[0].bound_endpoints(),
            vec!["tcp://127.0.0.1:40885"]
        );
        assert_eq!(
            factory.rep_sockets[0].bound_endpoints(),
            vec!["tcp://127.0.0.1:42540"]
        );
    }

    #[test]
    fn zeromq_backend_round_trips_execute_over_fake_router_socket() {
        let bootstrap = test_connection();
        let codec = bootstrap.wire_codec();
        let mut factory = FakeZeroMqFactory::default();
        let mut backend = ZeroMqTransportBackend::from_factory(bootstrap.clone(), &mut factory)
            .expect("backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-zmq-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1 + 1")),
            })
            .expect("request should encode");
        factory.router_sockets[0].push_incoming(wire_message_to_frames(request).frames);

        assert_eq!(
            driver
                .pump_once(backend.transport_mut())
                .expect("driver should pump"),
            Some(crate::KernelSocketPollEvent::Channel(KernelChannel::Shell))
        );

        let reply_frames = factory.router_sockets[0].take_outgoing();
        assert_eq!(reply_frames.len(), 1);
        let reply_wire = frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            reply_frames[0].clone(),
        )
        .expect("reply frames should convert");
        let reply = decode_response_message(&reply_wire).expect("reply should decode");
        match reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(1));
            }
            other => panic!("expected execute response, got {other:?}"),
        }
    }

    #[test]
    fn native_zeromq_backend_round_trips_execute_over_loopback_sockets() {
        let bootstrap = loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut dealer = runtime.block_on(async {
            let mut dealer = DealerSocket::new();
            dealer
                .connect(&shell_endpoint)
                .await
                .expect("dealer should connect");
            dealer
        });

        let request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-zmq-real-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1 + 1")),
            })
            .expect("request should encode");
        runtime.block_on(async {
            dealer
                .send(frames_to_zmq_message(
                    wire_message_to_frames(request).frames,
                ))
                .await
                .expect("dealer should send request");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let reply_frames = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), dealer.recv())
                .await
                .expect("dealer should receive reply before timeout")
                .expect("dealer reply should decode")
                .into_vec()
                .into_iter()
                .map(|frame| frame.to_vec())
                .collect::<Vec<_>>()
        });
        let reply_wire = frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            reply_frames,
        )
        .expect("reply frames should convert");
        let reply = decode_response_message(&reply_wire).expect("reply should decode");
        match reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(1));
            }
            other => panic!("expected execute response, got {other:?}"),
        }
    }

    #[test]
    fn native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket() {
        let bootstrap = loopback_connection();
        let heartbeat_endpoint = bootstrap.transport_layout().heartbeat.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut heartbeat = runtime.block_on(async {
            let mut heartbeat = ReqSocket::new();
            heartbeat
                .connect(&heartbeat_endpoint)
                .await
                .expect("req socket should connect");
            heartbeat
        });
        runtime.block_on(async {
            heartbeat
                .send(ZmqMessage::from(Vec::from("hb")))
                .await
                .expect("heartbeat request should send");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Heartbeat,
        );

        let reply = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), heartbeat.recv())
                .await
                .expect("heartbeat reply should arrive before timeout")
                .expect("heartbeat reply should decode")
        });
        assert_eq!(zmq_message_to_frames(reply), vec![b"hb".to_vec()]);
    }
}
