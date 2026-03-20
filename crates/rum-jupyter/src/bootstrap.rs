use rum_runtime::Session;

use crate::{
    KernelAdapter, KernelChannel, KernelConnectionFile, KernelEventPump, KernelSocketSessionConfig,
    KernelTransportKind, KernelWireCodec, KernelWireConfig, LocalFrontendConnection,
    LocalSocketSession, MockSocketTransport,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelTransportEndpoint {
    pub transport: KernelTransportKind,
    pub ip: String,
    pub port: u16,
}

impl KernelTransportEndpoint {
    pub fn uri(&self) -> String {
        format!("{}://{}:{}", self.transport.as_str(), self.ip, self.port)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelTransportLayout {
    pub shell: KernelTransportEndpoint,
    pub control: KernelTransportEndpoint,
    pub stdin: KernelTransportEndpoint,
    pub iopub: KernelTransportEndpoint,
    pub heartbeat: KernelTransportEndpoint,
}

impl KernelTransportLayout {
    pub fn endpoint(&self, channel: KernelChannel) -> &KernelTransportEndpoint {
        match channel {
            KernelChannel::Shell => &self.shell,
            KernelChannel::Control => &self.control,
            KernelChannel::Stdin => &self.stdin,
            KernelChannel::IoPub => &self.iopub,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelBootstrapConfig {
    pub connection: KernelConnectionFile,
    pub username: String,
    pub session_id_validation: Option<String>,
}

impl KernelBootstrapConfig {
    pub fn from_connection_file(connection: KernelConnectionFile) -> Self {
        Self {
            connection,
            username: "kernel".to_string(),
            session_id_validation: None,
        }
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn with_session_validation(mut self, session_id: impl Into<String>) -> Self {
        self.session_id_validation = Some(session_id.into());
        self
    }

    pub fn transport_layout(&self) -> KernelTransportLayout {
        let endpoint = |port| KernelTransportEndpoint {
            transport: self.connection.transport,
            ip: self.connection.ip.clone(),
            port,
        };

        KernelTransportLayout {
            shell: endpoint(self.connection.shell_port),
            control: endpoint(self.connection.control_port),
            stdin: endpoint(self.connection.stdin_port),
            iopub: endpoint(self.connection.iopub_port),
            heartbeat: endpoint(self.connection.hb_port),
        }
    }

    pub fn wire_config(&self) -> KernelWireConfig {
        let mut config =
            KernelWireConfig::default().with_signature_policy(self.connection.signature_policy());
        if let Some(session_id) = &self.session_id_validation {
            config.session_id = Some(session_id.clone());
        }
        config
    }

    pub fn wire_codec(&self) -> KernelWireCodec {
        KernelWireCodec::new(self.wire_config())
    }

    pub fn event_pump(&self, session: Session) -> KernelEventPump {
        KernelEventPump::new(
            KernelAdapter::with_username(session, self.username.clone()),
            self.wire_codec(),
        )
    }
}

pub trait KernelTransportBackend {
    type Transport;

    fn bootstrap_config(&self) -> &KernelBootstrapConfig;
    fn transport_layout(&self) -> &KernelTransportLayout;
    fn transport(&self) -> &Self::Transport;
    fn transport_mut(&mut self) -> &mut Self::Transport;
}

#[derive(Debug)]
pub struct MockTransportBackend {
    bootstrap_config: KernelBootstrapConfig,
    transport_layout: KernelTransportLayout,
    transport: MockSocketTransport,
}

impl MockTransportBackend {
    pub fn new(bootstrap_config: KernelBootstrapConfig) -> Self {
        let transport_layout = bootstrap_config.transport_layout();

        Self {
            bootstrap_config,
            transport_layout,
            transport: MockSocketTransport::default(),
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        KernelBootstrapConfig,
        KernelTransportLayout,
        MockSocketTransport,
    ) {
        (self.bootstrap_config, self.transport_layout, self.transport)
    }

    pub fn event_pump(&self, session: Session) -> KernelEventPump {
        self.bootstrap_config.event_pump(session)
    }
}

impl KernelTransportBackend for MockTransportBackend {
    type Transport = MockSocketTransport;

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

#[derive(Debug)]
pub struct LocalSocketTransportBackend {
    bootstrap_config: KernelBootstrapConfig,
    transport_layout: KernelTransportLayout,
    socket_session_config: KernelSocketSessionConfig,
    transport: LocalSocketSession,
    frontend: LocalFrontendConnection,
}

impl LocalSocketTransportBackend {
    pub fn new(bootstrap_config: KernelBootstrapConfig) -> Self {
        let transport_layout = bootstrap_config.transport_layout();
        let socket_session_config = bootstrap_config.socket_session_config();
        let (transport, frontend) = LocalSocketSession::new(socket_session_config.clone());

        Self {
            bootstrap_config,
            transport_layout,
            socket_session_config,
            transport,
            frontend,
        }
    }

    pub fn frontend(&self) -> LocalFrontendConnection {
        self.frontend.clone()
    }

    pub fn event_pump(&self, session: Session) -> KernelEventPump {
        self.bootstrap_config.event_pump(session)
    }

    pub fn socket_session_config(&self) -> &KernelSocketSessionConfig {
        &self.socket_session_config
    }

    pub fn pump_heartbeat_once(&mut self) -> bool {
        self.transport.pump_heartbeat_once()
    }

    pub fn pump_heartbeats_until_idle(&mut self) -> usize {
        self.transport.pump_heartbeats_until_idle()
    }
}

impl KernelTransportBackend for LocalSocketTransportBackend {
    type Transport = LocalSocketSession;

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
