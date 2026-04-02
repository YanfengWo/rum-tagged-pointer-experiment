use std::collections::BTreeMap;

use crate::json_support::{JsonValue, parse_json_object};
use crate::{
    KernelAdapter, KernelChannel, KernelCompleteRequest, KernelCompleteResponse,
    KernelDispatchResult, KernelDisplayOutput, KernelErrorOutput, KernelExecuteInput,
    KernelExecuteOutcome, KernelExecutePayload, KernelExecuteRequest, KernelExecuteResponse,
    KernelExecuteResult, KernelHelpLink, KernelInfoResponse, KernelInputReply,
    KernelInputReplyEnvelope, KernelInputRequest, KernelInputRequestEnvelope, KernelInspectRequest,
    KernelInspectResponse, KernelInterruptRequest, KernelInterruptResponse, KernelIoPubEnvelope,
    KernelIoPubPayload, KernelIsCompleteRequest, KernelIsCompleteResponse, KernelIsCompleteStatus,
    KernelLanguageInfo, KernelMessageHeader, KernelMetadata, KernelMimeBundle, KernelReplyStatus,
    KernelRequest, KernelRequestEnvelope, KernelResponse, KernelResponseEnvelope,
    KernelShutdownRequest, KernelShutdownResponse, KernelStatusOutput, KernelStreamName,
    KernelStreamOutput,
};

mod codec;
mod event_loop;
mod json;
mod local;

pub use codec::{
    KernelSignaturePolicy, KernelWireCodec, KernelWireConfig, KernelWireError,
    decode_input_reply_message, decode_input_request_message, decode_iopub_message,
    decode_request_message, decode_response_message, encode_dispatch_result,
    encode_input_reply_envelope, encode_input_request_envelope, encode_iopub_envelope,
    encode_request_envelope, encode_response_envelope,
};
pub use event_loop::{KernelEventPump, KernelTransportLoopReport};
pub use local::{
    InMemoryTransport, LocalFrontendConnection, LocalSocketTransport, MockSocketTransport,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelWireMessage {
    pub channel: KernelChannel,
    pub routing_ids: Vec<Vec<u8>>,
    pub delimiter: String,
    pub signature: Option<String>,
    pub header_json: String,
    pub parent_header_json: String,
    pub metadata_json: String,
    pub content_json: String,
    pub buffers: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelWireBatch {
    pub reply: Option<KernelWireMessage>,
    pub iopub: Vec<KernelWireMessage>,
    pub stdin: Vec<KernelWireMessage>,
}

impl KernelWireBatch {
    pub fn messages(&self) -> Vec<&KernelWireMessage> {
        let mut messages =
            Vec::with_capacity(self.reply.iter().count() + self.iopub.len() + self.stdin.len());
        if let Some(reply) = &self.reply {
            messages.push(reply);
        }
        messages.extend(self.iopub.iter());
        messages.extend(self.stdin.iter());
        messages
    }
}

pub trait KernelTransport {
    fn send(&mut self, message: KernelWireMessage);

    fn send_batch(&mut self, batch: KernelWireBatch) {
        if let Some(reply) = batch.reply {
            self.send(reply);
        }
        for message in batch.iopub {
            self.send(message);
        }
        for message in batch.stdin {
            self.send(message);
        }
    }
}

pub trait KernelQueuedTransport: KernelTransport {
    fn pop_incoming(&mut self, channel: KernelChannel) -> Option<KernelWireMessage>;
    fn has_pending_incoming(&self) -> bool;

    fn take_pending_error(&mut self) -> Option<KernelWireError> {
        None
    }
}
