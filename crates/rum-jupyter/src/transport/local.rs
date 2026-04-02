use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use super::{KernelChannel, KernelQueuedTransport, KernelTransport, KernelWireMessage};

#[derive(Debug, Default)]
pub struct InMemoryTransport {
    pub messages: Vec<KernelWireMessage>,
}

impl KernelTransport for InMemoryTransport {
    fn send(&mut self, message: KernelWireMessage) {
        self.messages.push(message);
    }
}

#[derive(Debug, Default)]
pub struct MockSocketTransport {
    pub incoming_shell: Vec<KernelWireMessage>,
    pub incoming_control: Vec<KernelWireMessage>,
    pub incoming_stdin: Vec<KernelWireMessage>,
    pub shell: Vec<KernelWireMessage>,
    pub control: Vec<KernelWireMessage>,
    pub iopub: Vec<KernelWireMessage>,
    pub stdin: Vec<KernelWireMessage>,
}

impl MockSocketTransport {
    pub fn channel_messages(&self, channel: KernelChannel) -> &[KernelWireMessage] {
        match channel {
            KernelChannel::Shell => &self.shell,
            KernelChannel::Control => &self.control,
            KernelChannel::IoPub => &self.iopub,
            KernelChannel::Stdin => &self.stdin,
        }
    }

    pub fn queue_incoming(&mut self, message: KernelWireMessage) {
        match message.channel {
            KernelChannel::Shell => self.incoming_shell.push(message),
            KernelChannel::Control => self.incoming_control.push(message),
            KernelChannel::IoPub => self.iopub.push(message),
            KernelChannel::Stdin => self.incoming_stdin.push(message),
        }
    }

    pub fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage> {
        match channel {
            KernelChannel::Shell => pop_front(&mut self.incoming_shell),
            KernelChannel::Control => pop_front(&mut self.incoming_control),
            KernelChannel::IoPub => None,
            KernelChannel::Stdin => pop_front(&mut self.incoming_stdin),
        }
    }

    pub fn incoming_len(&self, channel: KernelChannel) -> usize {
        match channel {
            KernelChannel::Shell => self.incoming_shell.len(),
            KernelChannel::Control => self.incoming_control.len(),
            KernelChannel::IoPub => 0,
            KernelChannel::Stdin => self.incoming_stdin.len(),
        }
    }

    pub fn has_pending_incoming(&self) -> bool {
        !self.incoming_shell.is_empty()
            || !self.incoming_control.is_empty()
            || !self.incoming_stdin.is_empty()
    }
}

impl KernelTransport for MockSocketTransport {
    fn send(&mut self, message: KernelWireMessage) {
        match message.channel {
            KernelChannel::Shell => self.shell.push(message),
            KernelChannel::Control => self.control.push(message),
            KernelChannel::IoPub => self.iopub.push(message),
            KernelChannel::Stdin => self.stdin.push(message),
        }
    }
}

impl KernelQueuedTransport for MockSocketTransport {
    fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage> {
        MockSocketTransport::pop_incoming(self, channel)
    }

    fn has_pending_incoming(&self) -> bool {
        MockSocketTransport::has_pending_incoming(self)
    }
}

#[derive(Debug, Default)]
struct LocalSocketState {
    incoming_shell: VecDeque<KernelWireMessage>,
    incoming_control: VecDeque<KernelWireMessage>,
    incoming_stdin: VecDeque<KernelWireMessage>,
    shell: VecDeque<KernelWireMessage>,
    control: VecDeque<KernelWireMessage>,
    iopub: VecDeque<KernelWireMessage>,
    stdin: VecDeque<KernelWireMessage>,
    heartbeat_incoming: VecDeque<Vec<u8>>,
    heartbeat_outgoing: VecDeque<Vec<u8>>,
}

impl LocalSocketState {
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

    fn outgoing_queue_mut(&mut self, channel: KernelChannel) -> &mut VecDeque<KernelWireMessage> {
        match channel {
            KernelChannel::Shell => &mut self.shell,
            KernelChannel::Control => &mut self.control,
            KernelChannel::IoPub => &mut self.iopub,
            KernelChannel::Stdin => &mut self.stdin,
        }
    }

    fn outgoing_queue(&self, channel: KernelChannel) -> &VecDeque<KernelWireMessage> {
        match channel {
            KernelChannel::Shell => &self.shell,
            KernelChannel::Control => &self.control,
            KernelChannel::IoPub => &self.iopub,
            KernelChannel::Stdin => &self.stdin,
        }
    }

    fn has_pending_incoming(&self) -> bool {
        !self.incoming_shell.is_empty()
            || !self.incoming_control.is_empty()
            || !self.incoming_stdin.is_empty()
    }

    fn has_pending_channel(&self, channel: KernelChannel) -> bool {
        match channel {
            KernelChannel::Shell => !self.incoming_shell.is_empty(),
            KernelChannel::Control => !self.incoming_control.is_empty(),
            KernelChannel::IoPub => false,
            KernelChannel::Stdin => !self.incoming_stdin.is_empty(),
        }
    }

    fn has_pending_outgoing(&self) -> bool {
        !self.shell.is_empty()
            || !self.control.is_empty()
            || !self.iopub.is_empty()
            || !self.stdin.is_empty()
            || !self.heartbeat_outgoing.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct LocalFrontendConnection {
    state: Arc<Mutex<LocalSocketState>>,
}

impl LocalFrontendConnection {
    pub fn queue_incoming(&self, message: KernelWireMessage) {
        let mut state = self
            .state
            .lock()
            .expect("local frontend connection lock should not be poisoned");
        if let Some(queue) = state.incoming_queue_mut(message.channel) {
            queue.push_back(message);
        }
    }

    pub fn pop_outgoing(&self, channel: KernelChannel) -> Option<KernelWireMessage> {
        let mut state = self
            .state
            .lock()
            .expect("local frontend connection lock should not be poisoned");
        state.outgoing_queue_mut(channel).pop_front()
    }

    pub fn drain_outgoing(&self, channel: KernelChannel) -> Vec<KernelWireMessage> {
        let mut drained = Vec::new();
        while let Some(message) = self.pop_outgoing(channel) {
            drained.push(message);
        }
        drained
    }

    pub fn outgoing_len(&self, channel: KernelChannel) -> usize {
        let state = self
            .state
            .lock()
            .expect("local frontend connection lock should not be poisoned");
        state.outgoing_queue(channel).len()
    }

    pub fn has_pending_outgoing(&self) -> bool {
        let state = self
            .state
            .lock()
            .expect("local frontend connection lock should not be poisoned");
        state.has_pending_outgoing()
    }

    pub fn send_heartbeat(&self, payload: impl Into<Vec<u8>>) {
        let mut state = self
            .state
            .lock()
            .expect("local frontend connection lock should not be poisoned");
        state.heartbeat_incoming.push_back(payload.into());
    }

    pub fn pop_heartbeat_reply(&self) -> Option<Vec<u8>> {
        let mut state = self
            .state
            .lock()
            .expect("local frontend connection lock should not be poisoned");
        state.heartbeat_outgoing.pop_front()
    }
}

#[derive(Debug)]
pub struct LocalSocketTransport {
    state: Arc<Mutex<LocalSocketState>>,
}

impl LocalSocketTransport {
    pub fn pair() -> (Self, LocalFrontendConnection) {
        let state = Arc::new(Mutex::new(LocalSocketState::default()));
        (
            Self {
                state: Arc::clone(&state),
            },
            LocalFrontendConnection { state },
        )
    }

    pub fn pump_heartbeat_once(&mut self) -> bool {
        if let Some(payload) = self.pop_heartbeat() {
            self.send_heartbeat_reply(payload);
            true
        } else {
            false
        }
    }

    pub fn pump_heartbeats_until_idle(&mut self) -> usize {
        let mut pumped = 0;
        while self.pump_heartbeat_once() {
            pumped += 1;
        }
        pumped
    }

    pub fn has_pending_incoming_channel(&self, channel: KernelChannel) -> bool {
        let state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        state.has_pending_channel(channel)
    }

    pub fn has_pending_heartbeat(&self) -> bool {
        let state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        !state.heartbeat_incoming.is_empty()
    }

    pub fn pop_heartbeat(&mut self) -> Option<Vec<u8>> {
        let mut state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        state.heartbeat_incoming.pop_front()
    }

    pub fn send_heartbeat_reply(&mut self, payload: Vec<u8>) {
        let mut state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        state.heartbeat_outgoing.push_back(payload);
    }
}

impl KernelTransport for LocalSocketTransport {
    fn send(&mut self, message: KernelWireMessage) {
        let mut state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        state.outgoing_queue_mut(message.channel).push_back(message);
    }
}

impl KernelQueuedTransport for LocalSocketTransport {
    fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage> {
        let mut state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        state
            .incoming_queue_mut(channel)
            .and_then(VecDeque::pop_front)
    }

    fn has_pending_incoming(&self) -> bool {
        let state = self
            .state
            .lock()
            .expect("local socket transport lock should not be poisoned");
        state.has_pending_incoming()
    }
}

fn pop_front<T>(items: &mut Vec<T>) -> Option<T> {
    if items.is_empty() {
        None
    } else {
        Some(items.remove(0))
    }
}
