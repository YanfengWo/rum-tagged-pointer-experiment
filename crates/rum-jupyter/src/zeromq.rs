use std::{cell::RefCell, collections::VecDeque};

use rum_runtime::Session;

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

    fn poll_channel_into_buffer(&self, channel: KernelChannel) -> bool {
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
        self.poll_channel_into_buffer(KernelChannel::Control)
            || self.poll_channel_into_buffer(KernelChannel::Shell)
            || self.poll_channel_into_buffer(KernelChannel::Stdin)
    }
}

impl<S: ZeroMqSocket> KernelSocketSession for ZeroMqSocketSession<S> {
    fn session_config(&self) -> &KernelSocketSessionConfig {
        &self.session_config
    }

    fn has_pending_channel(&self, channel: KernelChannel) -> bool {
        self.poll_channel_into_buffer(channel)
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

#[path = "zeromq_native.rs"]
mod zeromq_native;
pub use self::zeromq_native::{
    NativeZeroMqFactory, NativeZeroMqSocket, NativeZeroMqTransportBackend,
};
#[cfg(test)]
use self::zeromq_native::{frames_to_zmq_message, zmq_message_to_frames};

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        thread,
        time::{Duration, Instant},
    };

    use zeromq::{
        DealerSocket, ReqSocket, Socket, SocketOptions, SocketRecv, SocketSend, SubSocket,
        ZmqMessage, util::PeerIdentity,
    };

    use super::*;
    use crate::{
        KernelConnectionFile, KernelMessageHeader, KernelMetadata, KernelReplyStatus,
        KernelRequest, KernelRequestEnvelope, KernelResponse, decode_response_message,
    };

    #[path = "../../zeromq_fake_tests.rs"]
    mod fake;
    #[path = "../../zeromq_native_test_support.rs"]
    mod support;

    use self::support::{
        loopback_connection, signed_loopback_connection, test_runtime, wait_for_next_socket_event,
        wait_for_socket_event,
    };

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

    #[test]
    fn native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata() {
        let bootstrap = loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut shell = runtime.block_on(async {
            let mut shell = DealerSocket::new();
            shell
                .connect(&shell_endpoint)
                .await
                .expect("shell dealer should connect");
            shell
        });
        let mut iopub = runtime.block_on(async {
            let mut iopub = SubSocket::new();
            iopub
                .connect(&iopub_endpoint)
                .await
                .expect("iopub sub should connect");
            iopub
                .subscribe("")
                .await
                .expect("iopub sub should subscribe");
            iopub
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let execute_header = KernelMessageHeader::new(
            "execute-zmq-metadata-1",
            "frontend-session",
            "frontend",
            "execute_request",
        );
        let execute_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: execute_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
            })
            .expect("execute request should encode");
        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(execute_request).frames,
                ))
                .await
                .expect("shell dealer should send execute request");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let shell_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), shell.recv())
                .await
                .expect("shell reply should arrive before timeout")
                .expect("shell reply should decode")
        });
        let shell_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Shell,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(shell_reply_wire),
            )
            .expect("shell reply frames should convert"),
        )
        .expect("shell reply should decode");

        assert_eq!(shell_reply.header.username, "rum-kernel");
        assert_eq!(
            shell_reply.header.protocol_version,
            execute_header.protocol_version
        );
        assert_eq!(shell_reply.parent_header, Some(execute_header.clone()));
        assert_eq!(shell_reply.metadata.get("channel"), Some("shell"));
        assert_eq!(
            shell_reply.metadata.get("parent_msg_type"),
            Some("execute_request")
        );

        match shell_reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(1));
            }
            other => panic!("expected execute reply, got {other:?}"),
        }

        let mut iopub_messages = Vec::new();
        while iopub_messages.len() < 4 {
            let message = runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                    .await
                    .expect("iopub message should arrive before timeout")
                    .expect("iopub message should decode")
            });
            let envelope = crate::decode_iopub_message(
                &frames_to_wire_message(
                    KernelChannel::IoPub,
                    &bootstrap.wire_config().delimiter,
                    zmq_message_to_frames(message),
                )
                .expect("iopub frames should convert"),
            )
            .expect("iopub message should parse");
            iopub_messages.push(envelope);
        }

        for message in &iopub_messages {
            assert_eq!(message.header.username, "rum-kernel");
            assert_eq!(
                message.header.protocol_version,
                execute_header.protocol_version
            );
            assert_eq!(message.parent_header, Some(execute_header.clone()));
            assert_eq!(message.metadata.get("channel"), Some("iopub"));
            assert_eq!(
                message.metadata.get("parent_msg_type"),
                Some("execute_request")
            );
        }

        assert!(iopub_messages.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::Status(status)
                if status.execution_state == crate::KernelExecutionState::Busy
                    && status.lifecycle_state == crate::KernelLifecycleState::Running
        )));
        assert!(iopub_messages.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteInput(input)
                if input.code == "1" && input.execution_count == Some(1)
        )));
        assert!(iopub_messages.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteResult(result) if result.data.get("text/plain").is_some()
        )));
        assert!(iopub_messages.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::Status(status)
                if status.execution_state == crate::KernelExecutionState::Idle
                    && status.lifecycle_state == crate::KernelLifecycleState::Running
        )));
    }

    #[test]
    fn native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply() {
        let bootstrap = loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let stdin_endpoint = bootstrap.transport_layout().stdin.uri();
        let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
        let frontend_identity =
            PeerIdentity::from_str("frontend-zmq-stdin-client").expect("identity should parse");
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut shell = runtime.block_on(async {
            let mut options = SocketOptions::default();
            options.peer_identity(frontend_identity.clone());
            let mut shell = DealerSocket::with_options(options);
            shell
                .connect(&shell_endpoint)
                .await
                .expect("shell dealer should connect");
            shell
        });
        let mut stdin = runtime.block_on(async {
            let mut options = SocketOptions::default();
            options.peer_identity(frontend_identity);
            let mut stdin = DealerSocket::with_options(options);
            stdin
                .connect(&stdin_endpoint)
                .await
                .expect("stdin dealer should connect");
            stdin
        });
        let mut iopub = runtime.block_on(async {
            let mut iopub = SubSocket::new();
            iopub
                .connect(&iopub_endpoint)
                .await
                .expect("iopub sub should connect");
            iopub
                .subscribe("")
                .await
                .expect("iopub sub should subscribe");
            iopub
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let execute_header = KernelMessageHeader::new(
            "execute-zmq-stdin-1",
            "frontend-session",
            "frontend",
            "execute_request",
        );
        let execute_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: execute_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new(
                    "InputString[\"Name: \"]",
                )),
            })
            .expect("execute request should encode");
        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(execute_request).frames,
                ))
                .await
                .expect("shell dealer should send execute request");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let stdin_request_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), stdin.recv())
                .await
                .expect("stdin request should arrive before timeout")
                .expect("stdin request should decode")
        });
        let stdin_request_wire = frames_to_wire_message(
            KernelChannel::Stdin,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(stdin_request_wire),
        )
        .expect("stdin request frames should convert");
        let stdin_request = crate::decode_input_request_message(&stdin_request_wire)
            .expect("stdin request should parse");
        assert_eq!(stdin_request.request.prompt, "Name: ");

        let input_reply = codec
            .encode_input_reply_envelope(&crate::KernelInputReplyEnvelope {
                channel: KernelChannel::Stdin,
                header: KernelMessageHeader::new(
                    "input-zmq-reply-1",
                    "frontend-session",
                    "frontend",
                    "input_reply",
                ),
                parent_header: Some(stdin_request.header.clone()),
                metadata: KernelMetadata::new(),
                reply: crate::KernelInputReply::new("Ada"),
            })
            .expect("input reply should encode");
        runtime.block_on(async {
            stdin
                .send(frames_to_zmq_message(
                    wire_message_to_frames(input_reply).frames,
                ))
                .await
                .expect("stdin dealer should send input reply");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
        );

        let shell_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), shell.recv())
                .await
                .expect("shell reply should arrive before timeout")
                .expect("shell reply should decode")
        });
        let shell_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Shell,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(shell_reply_wire),
            )
            .expect("shell reply frames should convert"),
        )
        .expect("shell reply should decode");
        match shell_reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(1));
            }
            other => panic!("expected execute reply, got {other:?}"),
        }

        let mut iopub_payloads = Vec::new();
        while iopub_payloads.len() < 4 {
            let message = runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                    .await
                    .expect("iopub message should arrive before timeout")
                    .expect("iopub message should decode")
            });
            let envelope = crate::decode_iopub_message(
                &frames_to_wire_message(
                    KernelChannel::IoPub,
                    &bootstrap.wire_config().delimiter,
                    zmq_message_to_frames(message),
                )
                .expect("iopub frames should convert"),
            )
            .expect("iopub message should parse");
            iopub_payloads.push(envelope.payload);
        }

        assert!(iopub_payloads.iter().any(|payload| matches!(
            payload,
            crate::KernelIoPubPayload::Status(status)
                if status.execution_state == crate::KernelExecutionState::Busy
        )));
        assert!(iopub_payloads.iter().any(|payload| matches!(
            payload,
            crate::KernelIoPubPayload::ExecuteInput(input)
                if input.code == "InputString[\"Name: \"]"
        )));
        assert!(iopub_payloads.iter().any(|payload| matches!(
            payload,
            crate::KernelIoPubPayload::ExecuteResult(result)
                if result.data.get("text/plain") == Some("\"Ada\"")
        )));
        assert!(iopub_payloads.iter().any(|payload| matches!(
            payload,
            crate::KernelIoPubPayload::Status(status)
                if status.execution_state == crate::KernelExecutionState::Idle
        )));
    }

    #[test]
    fn native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown()
    {
        let bootstrap = loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let stdin_endpoint = bootstrap.transport_layout().stdin.uri();
        let control_endpoint = bootstrap.transport_layout().control.uri();
        let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
        let frontend_identity =
            PeerIdentity::from_str("frontend-zmq-multi-client").expect("identity should parse");
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut shell = runtime.block_on(async {
            let mut options = SocketOptions::default();
            options.peer_identity(frontend_identity.clone());
            let mut shell = DealerSocket::with_options(options);
            shell
                .connect(&shell_endpoint)
                .await
                .expect("shell dealer should connect");
            shell
        });
        let mut stdin = runtime.block_on(async {
            let mut options = SocketOptions::default();
            options.peer_identity(frontend_identity);
            let mut stdin = DealerSocket::with_options(options);
            stdin
                .connect(&stdin_endpoint)
                .await
                .expect("stdin dealer should connect");
            stdin
        });
        let mut control = runtime.block_on(async {
            let mut control = DealerSocket::new();
            control
                .connect(&control_endpoint)
                .await
                .expect("control dealer should connect");
            control
        });
        let mut iopub = runtime.block_on(async {
            let mut iopub = SubSocket::new();
            iopub
                .connect(&iopub_endpoint)
                .await
                .expect("iopub sub should connect");
            iopub
                .subscribe("")
                .await
                .expect("iopub sub should subscribe");
            iopub
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let first_execute_header = KernelMessageHeader::new(
            "execute-zmq-multi-1",
            "frontend-session",
            "frontend",
            "execute_request",
        );
        let first_execute = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: first_execute_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
            })
            .expect("first execute request should encode");
        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(first_execute).frames,
                ))
                .await
                .expect("shell dealer should send first execute");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let first_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), shell.recv())
                .await
                .expect("first shell reply should arrive before timeout")
                .expect("first shell reply should decode")
        });
        let first_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Shell,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(first_reply_wire),
            )
            .expect("first shell reply frames should convert"),
        )
        .expect("first shell reply should decode");
        match first_reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(1));
            }
            other => panic!("expected first execute reply, got {other:?}"),
        }

        let mut first_iopub = Vec::new();
        while first_iopub.len() < 4 {
            let message = runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                    .await
                    .expect("first iopub message should arrive before timeout")
                    .expect("first iopub message should decode")
            });
            first_iopub.push(
                crate::decode_iopub_message(
                    &frames_to_wire_message(
                        KernelChannel::IoPub,
                        &bootstrap.wire_config().delimiter,
                        zmq_message_to_frames(message),
                    )
                    .expect("first iopub frames should convert"),
                )
                .expect("first iopub message should parse"),
            );
        }
        assert!(first_iopub.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteInput(input)
                if input.code == "1" && input.execution_count == Some(1)
        )));
        assert!(first_iopub.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteResult(result)
                if result.data.get("text/plain") == Some("1")
        )));

        let second_execute_header = KernelMessageHeader::new(
            "execute-zmq-multi-2",
            "frontend-session",
            "frontend",
            "execute_request",
        );
        let second_execute = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: second_execute_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new(
                    "InputString[\"Name: \"]",
                )),
            })
            .expect("second execute request should encode");
        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(second_execute).frames,
                ))
                .await
                .expect("shell dealer should send second execute");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let stdin_request_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), stdin.recv())
                .await
                .expect("stdin request should arrive before timeout")
                .expect("stdin request should decode")
        });
        let stdin_request_wire = frames_to_wire_message(
            KernelChannel::Stdin,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(stdin_request_wire),
        )
        .expect("stdin request frames should convert");
        let stdin_request = crate::decode_input_request_message(&stdin_request_wire)
            .expect("stdin request should decode");
        assert_eq!(stdin_request.request.prompt, "Name: ");

        let input_reply = codec
            .encode_input_reply_envelope(&crate::KernelInputReplyEnvelope {
                channel: KernelChannel::Stdin,
                header: KernelMessageHeader::new(
                    "input-zmq-multi-1",
                    "frontend-session",
                    "frontend",
                    "input_reply",
                ),
                parent_header: Some(stdin_request.header.clone()),
                metadata: KernelMetadata::new(),
                reply: crate::KernelInputReply::new("Ada"),
            })
            .expect("stdin reply should encode");
        runtime.block_on(async {
            stdin
                .send(frames_to_zmq_message(
                    wire_message_to_frames(input_reply).frames,
                ))
                .await
                .expect("stdin dealer should send input reply");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
        );

        let second_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), shell.recv())
                .await
                .expect("second shell reply should arrive before timeout")
                .expect("second shell reply should decode")
        });
        let second_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Shell,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(second_reply_wire),
            )
            .expect("second shell reply frames should convert"),
        )
        .expect("second shell reply should decode");
        match second_reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(2));
            }
            other => panic!("expected second execute reply, got {other:?}"),
        }

        let mut second_iopub = Vec::new();
        while second_iopub.len() < 4 {
            let message = runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                    .await
                    .expect("second iopub message should arrive before timeout")
                    .expect("second iopub message should decode")
            });
            second_iopub.push(
                crate::decode_iopub_message(
                    &frames_to_wire_message(
                        KernelChannel::IoPub,
                        &bootstrap.wire_config().delimiter,
                        zmq_message_to_frames(message),
                    )
                    .expect("second iopub frames should convert"),
                )
                .expect("second iopub message should parse"),
            );
        }
        assert!(second_iopub.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteInput(input)
                if input.code == "InputString[\"Name: \"]" && input.execution_count == Some(2)
        )));
        assert!(second_iopub.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteResult(result)
                if result.data.get("text/plain") == Some("\"Ada\"")
        )));

        let third_execute_header = KernelMessageHeader::new(
            "execute-zmq-multi-3",
            "frontend-session",
            "frontend",
            "execute_request",
        );
        let third_execute = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: third_execute_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("2")),
            })
            .expect("third execute request should encode");
        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(third_execute).frames,
                ))
                .await
                .expect("shell dealer should send third execute");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let third_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), shell.recv())
                .await
                .expect("third shell reply should arrive before timeout")
                .expect("third shell reply should decode")
        });
        let third_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Shell,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(third_reply_wire),
            )
            .expect("third shell reply frames should convert"),
        )
        .expect("third shell reply should decode");
        match third_reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(3));
            }
            other => panic!("expected third execute reply, got {other:?}"),
        }

        let mut third_iopub = Vec::new();
        while third_iopub.len() < 4 {
            let message = runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                    .await
                    .expect("third iopub message should arrive before timeout")
                    .expect("third iopub message should decode")
            });
            third_iopub.push(
                crate::decode_iopub_message(
                    &frames_to_wire_message(
                        KernelChannel::IoPub,
                        &bootstrap.wire_config().delimiter,
                        zmq_message_to_frames(message),
                    )
                    .expect("third iopub frames should convert"),
                )
                .expect("third iopub message should parse"),
            );
        }
        assert!(third_iopub.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteInput(input)
                if input.code == "2" && input.execution_count == Some(3)
        )));
        assert!(third_iopub.iter().any(|message| matches!(
            &message.payload,
            crate::KernelIoPubPayload::ExecuteResult(result)
                if result.data.get("text/plain") == Some("2")
        )));

        let shutdown_header = KernelMessageHeader::new(
            "shutdown-zmq-multi-1",
            "frontend-session",
            "frontend",
            "shutdown_request",
        );
        let shutdown_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: shutdown_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Shutdown(crate::KernelShutdownRequest { restart: false }),
            })
            .expect("shutdown request should encode");
        runtime.block_on(async {
            control
                .send(frames_to_zmq_message(
                    wire_message_to_frames(shutdown_request).frames,
                ))
                .await
                .expect("control dealer should send shutdown request");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
        );

        let shutdown_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), control.recv())
                .await
                .expect("shutdown reply should arrive before timeout")
                .expect("shutdown reply should decode")
        });
        let shutdown_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Control,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(shutdown_reply_wire),
            )
            .expect("shutdown reply frames should convert"),
        )
        .expect("shutdown reply should decode");
        match shutdown_reply.response {
            KernelResponse::Shutdown(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert!(!response.restart);
                assert_eq!(
                    response.lifecycle_state,
                    crate::KernelLifecycleState::ShutdownRequested
                );
            }
            other => panic!("expected shutdown reply, got {other:?}"),
        }
    }

    #[test]
    fn native_zeromq_backend_frontend_style_control_interrupt_round_trip() {
        let bootstrap = loopback_connection();
        let codec = bootstrap.wire_codec();
        let control_endpoint = bootstrap.transport_layout().control.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut control = runtime.block_on(async {
            let mut control = DealerSocket::new();
            control
                .connect(&control_endpoint)
                .await
                .expect("control dealer should connect");
            control
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let interrupt_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "interrupt-zmq-1",
                    "frontend-session",
                    "frontend",
                    "interrupt_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Interrupt(crate::KernelInterruptRequest),
            })
            .expect("interrupt request should encode");
        runtime.block_on(async {
            control
                .send(frames_to_zmq_message(
                    wire_message_to_frames(interrupt_request).frames,
                ))
                .await
                .expect("control dealer should send interrupt request");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
        );

        let reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), control.recv())
                .await
                .expect("control reply should arrive before timeout")
                .expect("control reply should decode")
        });
        let reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Control,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(reply_wire),
            )
            .expect("control reply frames should convert"),
        )
        .expect("control reply should decode");
        match reply.response {
            KernelResponse::Interrupt(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(
                    response.lifecycle_state,
                    crate::KernelLifecycleState::InterruptRequested
                );
            }
            other => panic!("expected interrupt reply, got {other:?}"),
        }
    }

    #[test]
    fn native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages() {
        let bootstrap = loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let control_endpoint = bootstrap.transport_layout().control.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut shell = runtime.block_on(async {
            let mut shell = DealerSocket::new();
            shell
                .connect(&shell_endpoint)
                .await
                .expect("shell dealer should connect");
            shell
        });
        let mut control = runtime.block_on(async {
            let mut control = DealerSocket::new();
            control
                .connect(&control_endpoint)
                .await
                .expect("control dealer should connect");
            control
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let execute_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-zmq-shutdown-queued-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
            })
            .expect("queued execute request should encode");
        let shutdown_header = KernelMessageHeader::new(
            "shutdown-zmq-1",
            "frontend-session",
            "frontend",
            "shutdown_request",
        );
        let shutdown_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: shutdown_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Shutdown(crate::KernelShutdownRequest { restart: true }),
            })
            .expect("shutdown request should encode");

        runtime.block_on(async {
            control
                .send(frames_to_zmq_message(
                    wire_message_to_frames(shutdown_request).frames,
                ))
                .await
                .expect("control dealer should send shutdown request");
            tokio::time::sleep(Duration::from_millis(30)).await;
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(execute_request).frames,
                ))
                .await
                .expect("shell dealer should send queued execute request");
            tokio::time::sleep(Duration::from_millis(70)).await;
        });

        let first_event = wait_for_next_socket_event(&mut driver, backend.transport_mut());
        assert_eq!(
            first_event,
            crate::KernelSocketPollEvent::Channel(KernelChannel::Control)
        );

        let report = driver
            .pump_until_idle(backend.transport_mut())
            .expect("driver should stop cleanly after shutdown");
        assert_eq!(report.control_messages, 0);
        assert_eq!(report.shell_messages, 0);
        assert_eq!(report.stdin_messages, 0);
        assert_eq!(report.heartbeat_messages, 0);
        assert!(report.stopped_for_shutdown);
        assert!(driver.event_pump().adapter().shutdown_requested());
        assert_eq!(
            driver.event_pump().adapter().lifecycle_state(),
            crate::KernelLifecycleState::Shutdown
        );

        let control_reply_wire = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), control.recv())
                .await
                .expect("shutdown reply should arrive before timeout")
                .expect("shutdown reply should decode")
        });
        let control_reply = decode_response_message(
            &frames_to_wire_message(
                KernelChannel::Control,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(control_reply_wire),
            )
            .expect("shutdown reply frames should convert"),
        )
        .expect("shutdown reply should decode");
        assert_eq!(control_reply.header.username, "rum-kernel");
        assert_eq!(
            control_reply.header.protocol_version,
            shutdown_header.protocol_version
        );
        assert_eq!(control_reply.parent_header, Some(shutdown_header.clone()));

        match control_reply.response {
            KernelResponse::Shutdown(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert!(response.restart);
                assert_eq!(
                    response.lifecycle_state,
                    crate::KernelLifecycleState::ShutdownRequested
                );
            }
            other => panic!("expected shutdown reply, got {other:?}"),
        }

        let shell_timeout = runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(200), shell.recv()).await
        });
        assert!(
            shell_timeout.is_err(),
            "queued shell work should remain pending after shutdown"
        );
    }

    #[test]
    fn native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages() {
        let bootstrap = signed_loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("signed native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut shell = runtime.block_on(async {
            let mut shell = DealerSocket::new();
            shell
                .connect(&shell_endpoint)
                .await
                .expect("shell dealer should connect");
            shell
        });
        let mut iopub = runtime.block_on(async {
            let mut iopub = SubSocket::new();
            iopub
                .connect(&iopub_endpoint)
                .await
                .expect("iopub sub should connect");
            iopub
                .subscribe("")
                .await
                .expect("iopub sub should subscribe");
            iopub
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-zmq-signed-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
            })
            .expect("signed execute request should encode");
        assert!(request.signature.is_some());

        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(request).frames,
                ))
                .await
                .expect("shell dealer should send signed execute request");
        });

        wait_for_socket_event(
            &mut driver,
            backend.transport_mut(),
            crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        );

        let reply_wire = frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), shell.recv())
                    .await
                    .expect("signed shell reply should arrive before timeout")
                    .expect("signed shell reply should decode")
                    .into_vec()
                    .into_iter()
                    .map(|frame| frame.to_vec())
                    .collect::<Vec<_>>()
            }),
        )
        .expect("signed shell reply frames should convert");
        assert!(reply_wire.signature.is_some());
        let reply = codec
            .decode_response_message(&reply_wire)
            .expect("signed shell reply should validate and decode");
        match reply.response {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Ok);
                assert_eq!(response.execution_count, Some(1));
            }
            other => panic!("expected signed execute reply, got {other:?}"),
        }

        let mut saw_signed_execute_result = false;
        while !saw_signed_execute_result {
            let message = runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                    .await
                    .expect("signed iopub message should arrive before timeout")
                    .expect("signed iopub message should decode")
            });
            let wire = frames_to_wire_message(
                KernelChannel::IoPub,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(message),
            )
            .expect("signed iopub frames should convert");
            assert!(wire.signature.is_some());
            let envelope = codec
                .decode_iopub_message(&wire)
                .expect("signed iopub message should validate and decode");
            if matches!(
                envelope.payload,
                crate::KernelIoPubPayload::ExecuteResult(ref result)
                    if result.data.get("text/plain") == Some("1")
            ) {
                saw_signed_execute_result = true;
            }
        }
    }

    #[test]
    fn native_zeromq_backend_rejects_tampered_signed_requests_end_to_end() {
        let bootstrap = signed_loopback_connection();
        let codec = bootstrap.wire_codec();
        let shell_endpoint = bootstrap.transport_layout().shell.uri();
        let runtime = test_runtime();
        let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
            .expect("signed native zeromq backend should build");
        let mut driver = backend.socket_driver(Session::new());

        let mut shell = runtime.block_on(async {
            let mut shell = DealerSocket::new();
            shell
                .connect(&shell_endpoint)
                .await
                .expect("shell dealer should connect");
            shell
        });

        runtime.block_on(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let mut tampered_request = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-zmq-signed-bad-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
            })
            .expect("signed request should encode before tampering");
        tampered_request.signature = Some("bad-signature".to_string());

        runtime.block_on(async {
            shell
                .send(frames_to_zmq_message(
                    wire_message_to_frames(tampered_request).frames,
                ))
                .await
                .expect("shell dealer should send tampered signed request");
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match driver.pump_once(backend.transport_mut()) {
                Err(crate::KernelWireError::InvalidSignature) => break,
                Ok(None) | Ok(Some(_)) => {
                    assert!(
                        Instant::now() < deadline,
                        "timed out waiting for invalid-signature failure"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(other) => panic!("expected invalid signature error, got {other:?}"),
            }
        }

        let shell_timeout = runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(200), shell.recv()).await
        });
        assert!(
            shell_timeout.is_err(),
            "tampered signed request should not produce a shell reply"
        );
    }
}
