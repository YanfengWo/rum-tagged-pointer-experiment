use std::{thread, time::Duration};

use rum_host::HostSession;

use crate::{
    KernelBootstrapConfig, KernelChannel, KernelEventPump, KernelQueuedTransport, KernelTransport,
    KernelTransportEndpoint, KernelWireError, KernelWireMessage, LocalFrontendConnection,
    LocalSocketTransport,
};

const RUNNING_IDLE_SLEEP: Duration = Duration::from_millis(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelSocketKind {
    Router,
    Pub,
    Rep,
}

impl KernelSocketKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Router => "router",
            Self::Pub => "pub",
            Self::Rep => "rep",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelSocketBindMode {
    Bind,
    Connect,
}

impl KernelSocketBindMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bind => "bind",
            Self::Connect => "connect",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelSocketSpec {
    pub endpoint: KernelTransportEndpoint,
    pub kind: KernelSocketKind,
    pub bind_mode: KernelSocketBindMode,
}

impl KernelSocketSpec {
    pub fn uri(&self) -> String {
        self.endpoint.uri()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelSocketSessionConfig {
    pub shell: KernelSocketSpec,
    pub control: KernelSocketSpec,
    pub stdin: KernelSocketSpec,
    pub iopub: KernelSocketSpec,
    pub heartbeat: KernelSocketSpec,
}

impl KernelSocketSessionConfig {
    pub fn socket(&self, channel: KernelChannel) -> &KernelSocketSpec {
        match channel {
            KernelChannel::Shell => &self.shell,
            KernelChannel::Control => &self.control,
            KernelChannel::Stdin => &self.stdin,
            KernelChannel::IoPub => &self.iopub,
        }
    }
}

impl KernelBootstrapConfig {
    pub fn socket_session_config(&self) -> KernelSocketSessionConfig {
        let layout = self.transport_layout();
        let socket = |endpoint: KernelTransportEndpoint, kind| KernelSocketSpec {
            endpoint,
            kind,
            bind_mode: KernelSocketBindMode::Bind,
        };

        KernelSocketSessionConfig {
            shell: socket(layout.shell, KernelSocketKind::Router),
            control: socket(layout.control, KernelSocketKind::Router),
            stdin: socket(layout.stdin, KernelSocketKind::Router),
            iopub: socket(layout.iopub, KernelSocketKind::Pub),
            heartbeat: socket(layout.heartbeat, KernelSocketKind::Rep),
        }
    }
}

pub trait KernelSocketSession: KernelQueuedTransport {
    fn session_config(&self) -> &KernelSocketSessionConfig;
    fn has_pending_channel(&self, channel: KernelChannel) -> bool;
    fn has_pending_heartbeat(&self) -> bool;
    fn pop_heartbeat(&mut self) -> Option<Vec<u8>>;
    fn send_heartbeat(&mut self, payload: Vec<u8>);
}

#[derive(Debug)]
pub struct LocalSocketSession {
    session_config: KernelSocketSessionConfig,
    transport: LocalSocketTransport,
}

impl LocalSocketSession {
    pub fn new(session_config: KernelSocketSessionConfig) -> (Self, LocalFrontendConnection) {
        let (transport, frontend) = LocalSocketTransport::pair();
        (
            Self {
                session_config,
                transport,
            },
            frontend,
        )
    }

    pub fn pump_heartbeat_once(&mut self) -> bool {
        self.transport.pump_heartbeat_once()
    }

    pub fn pump_heartbeats_until_idle(&mut self) -> usize {
        self.transport.pump_heartbeats_until_idle()
    }
}

impl KernelTransport for LocalSocketSession {
    fn send(&mut self, message: KernelWireMessage) {
        self.transport.send(message);
    }
}

impl KernelQueuedTransport for LocalSocketSession {
    fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage> {
        self.transport.pop_incoming(channel)
    }

    fn has_pending_incoming(&self) -> bool {
        self.transport.has_pending_incoming()
    }
}

impl KernelSocketSession for LocalSocketSession {
    fn session_config(&self) -> &KernelSocketSessionConfig {
        &self.session_config
    }

    fn has_pending_channel(&self, channel: KernelChannel) -> bool {
        self.transport.has_pending_incoming_channel(channel)
    }

    fn has_pending_heartbeat(&self) -> bool {
        self.transport.has_pending_heartbeat()
    }

    fn pop_heartbeat(&mut self) -> Option<Vec<u8>> {
        self.transport.pop_heartbeat()
    }

    fn send_heartbeat(&mut self, payload: Vec<u8>) {
        self.transport.send_heartbeat_reply(payload);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelSocketPollEvent {
    Channel(KernelChannel),
    Heartbeat,
    ExecuteCompletion,
}

pub trait KernelSocketPoller {
    fn next_event<S: KernelSocketSession>(
        &mut self,
        session: &S,
        awaiting_stdin: bool,
        execute_running: bool,
        shutdown_requested: bool,
    ) -> Option<KernelSocketPollEvent>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PriorityKernelSocketPoller;

impl KernelSocketPoller for PriorityKernelSocketPoller {
    fn next_event<S: KernelSocketSession>(
        &mut self,
        session: &S,
        awaiting_stdin: bool,
        execute_running: bool,
        shutdown_requested: bool,
    ) -> Option<KernelSocketPollEvent> {
        if session.has_pending_heartbeat() {
            return Some(KernelSocketPollEvent::Heartbeat);
        }

        if shutdown_requested {
            return None;
        }

        let channels: &[KernelChannel] = if execute_running {
            &[KernelChannel::Control]
        } else if awaiting_stdin {
            &[
                KernelChannel::Control,
                KernelChannel::Stdin,
                KernelChannel::Shell,
            ]
        } else {
            &[
                KernelChannel::Control,
                KernelChannel::Shell,
                KernelChannel::Stdin,
            ]
        };

        for &channel in channels {
            if session.has_pending_channel(channel) {
                return Some(KernelSocketPollEvent::Channel(channel));
            }
        }

        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelSocketLoopReport {
    pub control_messages: usize,
    pub shell_messages: usize,
    pub stdin_messages: usize,
    pub heartbeat_messages: usize,
    pub execute_completions: usize,
    pub stopped_for_shutdown: bool,
}

impl KernelSocketLoopReport {
    pub const fn total_messages(&self) -> usize {
        self.control_messages
            + self.shell_messages
            + self.stdin_messages
            + self.heartbeat_messages
            + self.execute_completions
    }
}

pub struct KernelSocketDriver<P = PriorityKernelSocketPoller> {
    event_pump: KernelEventPump,
    poller: P,
}

impl<P> KernelSocketDriver<P> {
    pub fn new(event_pump: KernelEventPump, poller: P) -> Self {
        Self { event_pump, poller }
    }

    pub fn event_pump(&self) -> &KernelEventPump {
        &self.event_pump
    }

    pub fn event_pump_mut(&mut self) -> &mut KernelEventPump {
        &mut self.event_pump
    }

    pub fn poller(&self) -> &P {
        &self.poller
    }

    pub fn poller_mut(&mut self) -> &mut P {
        &mut self.poller
    }
}

impl<P: KernelSocketPoller> KernelSocketDriver<P> {
    pub fn pump_once<S: KernelSocketSession>(
        &mut self,
        session: &mut S,
    ) -> Result<Option<KernelSocketPollEvent>, KernelWireError> {
        if self.event_pump.pump_execute_completion(session)? {
            return Ok(Some(KernelSocketPollEvent::ExecuteCompletion));
        }

        let awaiting_stdin = self.event_pump.adapter().awaiting_stdin();
        let execute_running = self.event_pump.adapter().execute_running();
        let shutdown_requested = self.event_pump.adapter().shutdown_requested();
        let Some(event) =
            self.poller
                .next_event(session, awaiting_stdin, execute_running, shutdown_requested)
        else {
            return Ok(None);
        };

        match event {
            KernelSocketPollEvent::Channel(channel) => {
                if self.event_pump.pump_once(session, channel)? {
                    Ok(Some(KernelSocketPollEvent::Channel(channel)))
                } else {
                    Ok(None)
                }
            }
            KernelSocketPollEvent::Heartbeat => {
                if let Some(payload) = session.pop_heartbeat() {
                    session.send_heartbeat(payload);
                    Ok(Some(KernelSocketPollEvent::Heartbeat))
                } else {
                    Ok(None)
                }
            }
            KernelSocketPollEvent::ExecuteCompletion => {
                Ok(Some(KernelSocketPollEvent::ExecuteCompletion))
            }
        }
    }

    pub fn pump_until_idle<S: KernelSocketSession>(
        &mut self,
        session: &mut S,
    ) -> Result<KernelSocketLoopReport, KernelWireError> {
        let mut report = KernelSocketLoopReport::default();

        loop {
            let shutdown_requested = self.event_pump.adapter().shutdown_requested();
            if shutdown_requested && !session.has_pending_heartbeat() {
                report.stopped_for_shutdown = session.has_pending_incoming();
                break;
            }

            let Some(event) = self.pump_once(session)? else {
                if self.event_pump.adapter().execute_running() {
                    thread::sleep(RUNNING_IDLE_SLEEP);
                    continue;
                }
                break;
            };

            match event {
                KernelSocketPollEvent::Channel(KernelChannel::Control) => {
                    report.control_messages += 1;
                }
                KernelSocketPollEvent::Channel(KernelChannel::Shell) => {
                    report.shell_messages += 1;
                }
                KernelSocketPollEvent::Channel(KernelChannel::Stdin) => {
                    report.stdin_messages += 1;
                }
                KernelSocketPollEvent::Channel(KernelChannel::IoPub) => {}
                KernelSocketPollEvent::Heartbeat => {
                    report.heartbeat_messages += 1;
                }
                KernelSocketPollEvent::ExecuteCompletion => {
                    report.execute_completions += 1;
                }
            }
        }

        Ok(report)
    }
}

impl KernelBootstrapConfig {
    pub fn socket_driver(&self, session: HostSession) -> KernelSocketDriver {
        KernelSocketDriver::new(self.event_pump(session), PriorityKernelSocketPoller)
    }
}
