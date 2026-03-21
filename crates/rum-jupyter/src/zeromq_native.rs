use super::*;
use std::{
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
use zeromq::{PubSocket, RepSocket, RouterSocket, ZmqMessage, prelude::*};

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
                        let mut delivered = false;
                        for _ in 0..50 {
                            if send_half
                                .send(frames_to_zmq_message(frames.clone()))
                                .await
                                .is_ok()
                            {
                                delivered = true;
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        if !delivered {
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

pub(super) fn frames_to_zmq_message(frames: Vec<Vec<u8>>) -> ZmqMessage {
    let mut frames = frames.into_iter();
    let mut message = ZmqMessage::from(frames.next().unwrap_or_default());
    for frame in frames {
        message.push_back(Bytes::from(frame));
    }
    message
}

pub(super) fn zmq_message_to_frames(message: ZmqMessage) -> Vec<Vec<u8>> {
    message
        .into_vec()
        .into_iter()
        .map(|frame| frame.to_vec())
        .collect()
}
