use super::{
    KernelAdapter, KernelChannel, KernelQueuedTransport, KernelTransport, KernelWireCodec,
    KernelWireError, KernelWireMessage,
};

pub struct KernelEventPump {
    adapter: KernelAdapter,
    codec: KernelWireCodec,
    pending_execute_routing_ids: Option<Vec<Vec<u8>>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelTransportLoopReport {
    pub control_messages: usize,
    pub shell_messages: usize,
    pub stdin_messages: usize,
    pub stopped_for_shutdown: bool,
}

impl KernelTransportLoopReport {
    pub const fn total_messages(&self) -> usize {
        self.control_messages + self.shell_messages + self.stdin_messages
    }
}

impl KernelEventPump {
    pub fn new(adapter: KernelAdapter, codec: KernelWireCodec) -> Self {
        Self {
            adapter,
            codec,
            pending_execute_routing_ids: None,
        }
    }

    pub fn adapter(&self) -> &KernelAdapter {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut KernelAdapter {
        &mut self.adapter
    }

    pub fn codec(&self) -> &KernelWireCodec {
        &self.codec
    }

    pub fn handle_wire_request<T: KernelTransport>(
        &mut self,
        transport: &mut T,
        request: KernelWireMessage,
    ) -> Result<(), KernelWireError> {
        let routing_ids = request.routing_ids.clone();
        let envelope = self.codec.decode_request_message(&request)?;
        let dispatch = self.adapter.dispatch_request(envelope);
        if dispatch.reply.is_none() && !dispatch.stdin.is_empty() {
            self.pending_execute_routing_ids = Some(routing_ids.clone());
        }
        if dispatch.reply.is_some() {
            self.pending_execute_routing_ids = None;
        }
        self.codec
            .send_dispatch_with_routing(transport, &dispatch, routing_ids)
    }

    pub fn handle_wire_input_reply<T: KernelTransport>(
        &mut self,
        transport: &mut T,
        reply: KernelWireMessage,
    ) -> Result<(), KernelWireError> {
        let envelope = self.codec.decode_input_reply_message(&reply)?;
        if let Some(dispatch) = self
            .adapter
            .handle_input_reply(envelope)
            .map_err(KernelWireError::from)?
        {
            let routing_ids = self.pending_execute_routing_ids.clone().unwrap_or_default();
            if dispatch.reply.is_some() {
                self.pending_execute_routing_ids = None;
            }
            self.codec
                .send_dispatch_with_routing(transport, &dispatch, routing_ids)?;
        }
        Ok(())
    }

    pub fn pump_once<T: KernelQueuedTransport>(
        &mut self,
        transport: &mut T,
        channel: KernelChannel,
    ) -> Result<bool, KernelWireError> {
        if let Some(error) = transport.take_pending_error() {
            return Err(error);
        }

        if let Some(request) = transport.pop_incoming(channel) {
            match channel {
                KernelChannel::Shell | KernelChannel::Control => {
                    self.handle_wire_request(transport, request)?;
                }
                KernelChannel::Stdin => {
                    self.handle_wire_input_reply(transport, request)?;
                }
                KernelChannel::IoPub => return Ok(false),
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn pump_next<T: KernelQueuedTransport>(
        &mut self,
        transport: &mut T,
    ) -> Result<Option<KernelChannel>, KernelWireError> {
        if self.adapter.shutdown_requested() {
            return Ok(None);
        }

        let channels = if self.adapter.awaiting_stdin() {
            [
                KernelChannel::Control,
                KernelChannel::Stdin,
                KernelChannel::Shell,
            ]
        } else {
            [
                KernelChannel::Control,
                KernelChannel::Shell,
                KernelChannel::Stdin,
            ]
        };

        for channel in channels {
            if self.pump_once(transport, channel)? {
                return Ok(Some(channel));
            }
        }

        Ok(None)
    }

    pub fn pump_until_idle<T: KernelQueuedTransport>(
        &mut self,
        transport: &mut T,
    ) -> Result<KernelTransportLoopReport, KernelWireError> {
        let mut report = KernelTransportLoopReport::default();

        loop {
            if self.adapter.shutdown_requested() {
                report.stopped_for_shutdown = transport.has_pending_incoming();
                break;
            }

            let Some(channel) = self.pump_next(transport)? else {
                break;
            };

            match channel {
                KernelChannel::Control => report.control_messages += 1,
                KernelChannel::Shell => report.shell_messages += 1,
                KernelChannel::Stdin => report.stdin_messages += 1,
                KernelChannel::IoPub => {}
            }

            if self.adapter.shutdown_requested() {
                report.stopped_for_shutdown = transport.has_pending_incoming();
                break;
            }
        }

        Ok(report)
    }
}
