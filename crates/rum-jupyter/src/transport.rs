use std::collections::BTreeMap;

use crate::json_support::{JsonValue, parse_json_object};
use crate::signature::hmac_sha256_hex;
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
    KernelShutdownRequest, KernelShutdownResponse, KernelStatusOutput, KernelStdinError,
    KernelStreamName, KernelStreamOutput,
};

mod local;
mod event_loop;

pub use event_loop::{KernelEventPump, KernelTransportLoopReport};
pub use local::{
    InMemoryTransport, LocalFrontendConnection, LocalSocketTransport, MockSocketTransport,
};

const DEFAULT_DELIMITER: &str = "<IDS|MSG>";

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelSignaturePolicy {
    Disabled,
    Optional(String),
    Required(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelWireConfig {
    pub session_id: Option<String>,
    pub signature_policy: KernelSignaturePolicy,
    pub delimiter: String,
}

impl Default for KernelWireConfig {
    fn default() -> Self {
        Self {
            session_id: None,
            signature_policy: KernelSignaturePolicy::Disabled,
            delimiter: DEFAULT_DELIMITER.to_string(),
        }
    }
}

impl KernelWireConfig {
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_signature_key(mut self, signature_key: impl Into<String>) -> Self {
        self.signature_policy = KernelSignaturePolicy::Required(signature_key.into());
        self
    }

    pub fn with_optional_signature_key(mut self, signature_key: impl Into<String>) -> Self {
        self.signature_policy = KernelSignaturePolicy::Optional(signature_key.into());
        self
    }

    pub fn with_signature_policy(mut self, signature_policy: KernelSignaturePolicy) -> Self {
        self.signature_policy = signature_policy;
        self
    }

    pub fn with_delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = delimiter.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelWireError {
    InvalidWireFrames {
        channel: KernelChannel,
        message: String,
    },
    InvalidDelimiter {
        expected: String,
        found: String,
    },
    MissingSignature,
    InvalidSignature,
    SessionMismatch {
        expected: String,
        found: String,
    },
    InvalidChannelForMessageType {
        channel: KernelChannel,
        msg_type: String,
    },
    UnsupportedMessageType {
        msg_type: String,
    },
    MissingField {
        context: &'static str,
        field: &'static str,
    },
    InvalidField {
        context: &'static str,
        field: &'static str,
        value: String,
    },
    InvalidStdinState {
        message: String,
    },
    InvalidJson {
        message: String,
    },
}

impl From<KernelStdinError> for KernelWireError {
    fn from(error: KernelStdinError) -> Self {
        Self::InvalidStdinState {
            message: match error {
                KernelStdinError::StdinNotAllowed => {
                    "stdin was requested for an execute request that disallows it".to_string()
                }
                KernelStdinError::AlreadyAwaitingInput => {
                    "kernel is already waiting for an input_reply".to_string()
                }
                KernelStdinError::NoPendingInputRequest => {
                    "received input_reply without a pending input_request".to_string()
                }
                KernelStdinError::ReplyParentMismatch { expected, found } => format!(
                    "input_reply parent header did not match pending input_request ({expected} != {found})"
                ),
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct KernelWireCodec {
    config: KernelWireConfig,
}

impl KernelWireCodec {
    pub fn new(config: KernelWireConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &KernelWireConfig {
        &self.config
    }

    pub fn encode_request_envelope(
        &self,
        envelope: &KernelRequestEnvelope,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_request_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            Vec::new(),
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_request_content(&envelope.request),
        )
    }

    pub fn encode_request_envelope_with_routing(
        &self,
        envelope: &KernelRequestEnvelope,
        routing_ids: Vec<Vec<u8>>,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_request_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            routing_ids,
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_request_content(&envelope.request),
        )
    }

    pub fn encode_response_envelope(
        &self,
        envelope: &KernelResponseEnvelope,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_response_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            Vec::new(),
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_response_content(&envelope.response),
        )
    }

    pub fn encode_response_envelope_with_routing(
        &self,
        envelope: &KernelResponseEnvelope,
        routing_ids: Vec<Vec<u8>>,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_response_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            routing_ids,
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_response_content(&envelope.response),
        )
    }

    pub fn encode_iopub_envelope(
        &self,
        envelope: &KernelIoPubEnvelope,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_iopub_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            Vec::new(),
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_iopub_content(&envelope.payload),
        )
    }

    pub fn encode_dispatch_result(
        &self,
        dispatch: &KernelDispatchResult,
    ) -> Result<KernelWireBatch, KernelWireError> {
        self.encode_dispatch_result_with_routing(dispatch, Vec::new())
    }

    pub fn encode_dispatch_result_with_routing(
        &self,
        dispatch: &KernelDispatchResult,
        reply_routing_ids: Vec<Vec<u8>>,
    ) -> Result<KernelWireBatch, KernelWireError> {
        Ok(KernelWireBatch {
            reply: dispatch
                .reply
                .as_ref()
                .map(|reply| {
                    self.encode_response_envelope_with_routing(reply, reply_routing_ids.clone())
                })
                .transpose()?,
            iopub: dispatch
                .iopub
                .iter()
                .map(|message| self.encode_iopub_envelope(message))
                .collect::<Result<Vec<_>, _>>()?,
            stdin: dispatch
                .stdin
                .iter()
                .map(|message| {
                    self.encode_input_request_envelope_with_routing(
                        message,
                        reply_routing_ids.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn encode_input_request_envelope(
        &self,
        envelope: &KernelInputRequestEnvelope,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.encode_input_request_envelope_with_routing(envelope, Vec::new())
    }

    pub fn encode_input_request_envelope_with_routing(
        &self,
        envelope: &KernelInputRequestEnvelope,
        routing_ids: Vec<Vec<u8>>,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_input_request_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            routing_ids,
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_input_request_content(&envelope.request),
        )
    }

    pub fn encode_input_reply_envelope(
        &self,
        envelope: &KernelInputReplyEnvelope,
    ) -> Result<KernelWireMessage, KernelWireError> {
        self.validate_session_id(&envelope.header.session_id)?;
        self.validate_input_reply_channel(envelope.channel, &envelope.header.msg_type)?;

        self.build_wire_message(
            envelope.channel,
            Vec::new(),
            &envelope.header,
            envelope.parent_header.as_ref(),
            encode_metadata(&envelope.metadata),
            encode_input_reply_content(&envelope.reply),
        )
    }

    pub fn decode_request_message(
        &self,
        message: &KernelWireMessage,
    ) -> Result<KernelRequestEnvelope, KernelWireError> {
        self.validate_delimiter(&message.delimiter)?;
        self.verify_signature(message)?;

        let header = parse_header(&message.header_json)?;
        self.validate_session_id(&header.session_id)?;
        self.validate_request_channel(message.channel, &header.msg_type)?;

        Ok(KernelRequestEnvelope {
            channel: message.channel,
            header: header.clone(),
            parent_header: parse_optional_header(&message.parent_header_json)?,
            metadata: parse_metadata(&message.metadata_json)?,
            request: parse_request_content(&header.msg_type, &message.content_json)?,
        })
    }

    pub fn decode_response_message(
        &self,
        message: &KernelWireMessage,
    ) -> Result<KernelResponseEnvelope, KernelWireError> {
        self.validate_delimiter(&message.delimiter)?;
        self.verify_signature(message)?;

        let header = parse_header(&message.header_json)?;
        self.validate_session_id(&header.session_id)?;
        self.validate_response_channel(message.channel, &header.msg_type)?;
        let metadata = parse_metadata(&message.metadata_json)?;

        Ok(KernelResponseEnvelope {
            channel: message.channel,
            header: header.clone(),
            parent_header: parse_optional_header(&message.parent_header_json)?,
            response: parse_response_content(&header.msg_type, &message.content_json, &metadata)?,
            metadata,
        })
    }

    pub fn decode_iopub_message(
        &self,
        message: &KernelWireMessage,
    ) -> Result<KernelIoPubEnvelope, KernelWireError> {
        self.validate_delimiter(&message.delimiter)?;
        self.verify_signature(message)?;

        let header = parse_header(&message.header_json)?;
        self.validate_session_id(&header.session_id)?;
        self.validate_iopub_channel(message.channel, &header.msg_type)?;
        let metadata = parse_metadata(&message.metadata_json)?;

        Ok(KernelIoPubEnvelope {
            channel: message.channel,
            header: header.clone(),
            parent_header: parse_optional_header(&message.parent_header_json)?,
            payload: parse_iopub_content(&header.msg_type, &message.content_json, &metadata)?,
            metadata,
        })
    }

    pub fn decode_input_request_message(
        &self,
        message: &KernelWireMessage,
    ) -> Result<KernelInputRequestEnvelope, KernelWireError> {
        self.validate_delimiter(&message.delimiter)?;
        self.verify_signature(message)?;

        let header = parse_header(&message.header_json)?;
        self.validate_session_id(&header.session_id)?;
        self.validate_input_request_channel(message.channel, &header.msg_type)?;

        Ok(KernelInputRequestEnvelope {
            channel: message.channel,
            header: header.clone(),
            parent_header: parse_optional_header(&message.parent_header_json)?,
            metadata: parse_metadata(&message.metadata_json)?,
            request: parse_input_request_content(&header.msg_type, &message.content_json)?,
        })
    }

    pub fn decode_input_reply_message(
        &self,
        message: &KernelWireMessage,
    ) -> Result<KernelInputReplyEnvelope, KernelWireError> {
        self.validate_delimiter(&message.delimiter)?;
        self.verify_signature(message)?;

        let header = parse_header(&message.header_json)?;
        self.validate_session_id(&header.session_id)?;
        self.validate_input_reply_channel(message.channel, &header.msg_type)?;

        Ok(KernelInputReplyEnvelope {
            channel: message.channel,
            header: header.clone(),
            parent_header: parse_optional_header(&message.parent_header_json)?,
            metadata: parse_metadata(&message.metadata_json)?,
            reply: parse_input_reply_content(&header.msg_type, &message.content_json)?,
        })
    }

    pub fn send_dispatch<T: KernelTransport>(
        &self,
        transport: &mut T,
        dispatch: &KernelDispatchResult,
    ) -> Result<(), KernelWireError> {
        transport.send_batch(self.encode_dispatch_result(dispatch)?);
        Ok(())
    }

    pub fn send_dispatch_with_routing<T: KernelTransport>(
        &self,
        transport: &mut T,
        dispatch: &KernelDispatchResult,
        reply_routing_ids: Vec<Vec<u8>>,
    ) -> Result<(), KernelWireError> {
        transport
            .send_batch(self.encode_dispatch_result_with_routing(dispatch, reply_routing_ids)?);
        Ok(())
    }

    fn build_wire_message(
        &self,
        channel: KernelChannel,
        routing_ids: Vec<Vec<u8>>,
        header: &KernelMessageHeader,
        parent_header: Option<&KernelMessageHeader>,
        metadata_json: String,
        content_json: String,
    ) -> Result<KernelWireMessage, KernelWireError> {
        let header_json = encode_header(header);
        let parent_header_json = encode_optional_header(parent_header);
        let signature = self.compute_signature(
            &header_json,
            &parent_header_json,
            &metadata_json,
            &content_json,
        );

        Ok(KernelWireMessage {
            channel,
            routing_ids,
            delimiter: self.config.delimiter.clone(),
            signature,
            header_json,
            parent_header_json,
            metadata_json,
            content_json,
            buffers: Vec::new(),
        })
    }

    fn validate_delimiter(&self, delimiter: &str) -> Result<(), KernelWireError> {
        if delimiter == self.config.delimiter {
            Ok(())
        } else {
            Err(KernelWireError::InvalidDelimiter {
                expected: self.config.delimiter.clone(),
                found: delimiter.to_string(),
            })
        }
    }

    fn validate_session_id(&self, session_id: &str) -> Result<(), KernelWireError> {
        if let Some(expected) = &self.config.session_id {
            if expected != session_id {
                return Err(KernelWireError::SessionMismatch {
                    expected: expected.clone(),
                    found: session_id.to_string(),
                });
            }
        }

        Ok(())
    }

    fn verify_signature(&self, message: &KernelWireMessage) -> Result<(), KernelWireError> {
        match (
            self.compute_signature(
                &message.header_json,
                &message.parent_header_json,
                &message.metadata_json,
                &message.content_json,
            ),
            &self.config.signature_policy,
        ) {
            (None, _) => Ok(()),
            (Some(expected), KernelSignaturePolicy::Optional(_)) => match &message.signature {
                Some(found) if found == &expected => Ok(()),
                Some(_) => Err(KernelWireError::InvalidSignature),
                None => Ok(()),
            },
            (Some(expected), KernelSignaturePolicy::Required(_)) => match &message.signature {
                Some(found) if found == &expected => Ok(()),
                Some(_) => Err(KernelWireError::InvalidSignature),
                None => Err(KernelWireError::MissingSignature),
            },
            (Some(_), KernelSignaturePolicy::Disabled) => Ok(()),
        }
    }

    fn compute_signature(
        &self,
        header_json: &str,
        parent_header_json: &str,
        metadata_json: &str,
        content_json: &str,
    ) -> Option<String> {
        match &self.config.signature_policy {
            KernelSignaturePolicy::Disabled => None,
            KernelSignaturePolicy::Optional(key) | KernelSignaturePolicy::Required(key) => {
                Some(hmac_sha256_hex(
                    key,
                    [header_json, parent_header_json, metadata_json, content_json],
                ))
            }
        }
    }

    fn validate_request_channel(
        &self,
        channel: KernelChannel,
        msg_type: &str,
    ) -> Result<(), KernelWireError> {
        let valid = match msg_type {
            "execute_request"
            | "inspect_request"
            | "complete_request"
            | "is_complete_request"
            | "kernel_info_request" => channel == KernelChannel::Shell,
            "interrupt_request" => channel == KernelChannel::Control,
            "shutdown_request" => matches!(channel, KernelChannel::Shell | KernelChannel::Control),
            _ => {
                return Err(KernelWireError::UnsupportedMessageType {
                    msg_type: msg_type.to_string(),
                });
            }
        };

        if valid {
            Ok(())
        } else {
            Err(KernelWireError::InvalidChannelForMessageType {
                channel,
                msg_type: msg_type.to_string(),
            })
        }
    }

    fn validate_response_channel(
        &self,
        channel: KernelChannel,
        msg_type: &str,
    ) -> Result<(), KernelWireError> {
        let valid = match msg_type {
            "execute_reply" | "inspect_reply" | "complete_reply" | "is_complete_reply"
            | "kernel_info_reply" => channel == KernelChannel::Shell,
            "interrupt_reply" => channel == KernelChannel::Control,
            "shutdown_reply" => matches!(channel, KernelChannel::Shell | KernelChannel::Control),
            _ => {
                return Err(KernelWireError::UnsupportedMessageType {
                    msg_type: msg_type.to_string(),
                });
            }
        };

        if valid {
            Ok(())
        } else {
            Err(KernelWireError::InvalidChannelForMessageType {
                channel,
                msg_type: msg_type.to_string(),
            })
        }
    }

    fn validate_iopub_channel(
        &self,
        channel: KernelChannel,
        msg_type: &str,
    ) -> Result<(), KernelWireError> {
        if channel == KernelChannel::IoPub
            && matches!(
                msg_type,
                "status" | "execute_input" | "stream" | "execute_result" | "display_data" | "error"
            )
        {
            Ok(())
        } else {
            Err(KernelWireError::InvalidChannelForMessageType {
                channel,
                msg_type: msg_type.to_string(),
            })
        }
    }

    fn validate_input_request_channel(
        &self,
        channel: KernelChannel,
        msg_type: &str,
    ) -> Result<(), KernelWireError> {
        if channel == KernelChannel::Stdin && msg_type == "input_request" {
            Ok(())
        } else {
            Err(KernelWireError::InvalidChannelForMessageType {
                channel,
                msg_type: msg_type.to_string(),
            })
        }
    }

    fn validate_input_reply_channel(
        &self,
        channel: KernelChannel,
        msg_type: &str,
    ) -> Result<(), KernelWireError> {
        if channel == KernelChannel::Stdin && msg_type == "input_reply" {
            Ok(())
        } else {
            Err(KernelWireError::InvalidChannelForMessageType {
                channel,
                msg_type: msg_type.to_string(),
            })
        }
    }
}

pub fn encode_request_envelope(envelope: &KernelRequestEnvelope) -> KernelWireMessage {
    KernelWireCodec::default()
        .encode_request_envelope(envelope)
        .expect("default transport encoding should succeed for valid request envelopes")
}

pub fn encode_response_envelope(envelope: &KernelResponseEnvelope) -> KernelWireMessage {
    KernelWireCodec::default()
        .encode_response_envelope(envelope)
        .expect("default transport encoding should succeed for valid response envelopes")
}

pub fn encode_iopub_envelope(envelope: &KernelIoPubEnvelope) -> KernelWireMessage {
    KernelWireCodec::default()
        .encode_iopub_envelope(envelope)
        .expect("default transport encoding should succeed for valid iopub envelopes")
}

pub fn encode_dispatch_result(dispatch: &KernelDispatchResult) -> KernelWireBatch {
    KernelWireCodec::default()
        .encode_dispatch_result(dispatch)
        .expect("default transport encoding should succeed for valid dispatch results")
}

pub fn encode_input_request_envelope(envelope: &KernelInputRequestEnvelope) -> KernelWireMessage {
    KernelWireCodec::default()
        .encode_input_request_envelope(envelope)
        .expect("default transport encoding should succeed for valid stdin request envelopes")
}

pub fn encode_input_reply_envelope(envelope: &KernelInputReplyEnvelope) -> KernelWireMessage {
    KernelWireCodec::default()
        .encode_input_reply_envelope(envelope)
        .expect("default transport encoding should succeed for valid stdin reply envelopes")
}

pub fn decode_request_message(
    message: &KernelWireMessage,
) -> Result<KernelRequestEnvelope, KernelWireError> {
    KernelWireCodec::default().decode_request_message(message)
}

pub fn decode_response_message(
    message: &KernelWireMessage,
) -> Result<KernelResponseEnvelope, KernelWireError> {
    KernelWireCodec::default().decode_response_message(message)
}

pub fn decode_iopub_message(
    message: &KernelWireMessage,
) -> Result<KernelIoPubEnvelope, KernelWireError> {
    KernelWireCodec::default().decode_iopub_message(message)
}

pub fn decode_input_request_message(
    message: &KernelWireMessage,
) -> Result<KernelInputRequestEnvelope, KernelWireError> {
    KernelWireCodec::default().decode_input_request_message(message)
}

pub fn decode_input_reply_message(
    message: &KernelWireMessage,
) -> Result<KernelInputReplyEnvelope, KernelWireError> {
    KernelWireCodec::default().decode_input_reply_message(message)
}

fn encode_request_content(request: &KernelRequest) -> String {
    match request {
        KernelRequest::Execute(KernelExecuteRequest {
            code,
            store_history,
            allow_stdin,
        }) => json_object(vec![
            field("code", json_string(code)),
            field("store_history", json_bool(*store_history)),
            field("allow_stdin", json_bool(*allow_stdin)),
        ]),
        KernelRequest::Inspect(KernelInspectRequest {
            code,
            cursor_offset,
        }) => json_object(vec![
            field("code", json_string(code)),
            field("cursor_pos", json_usize(*cursor_offset)),
        ]),
        KernelRequest::Complete(KernelCompleteRequest {
            code,
            cursor_offset,
        }) => json_object(vec![
            field("code", json_string(code)),
            field("cursor_pos", json_usize(*cursor_offset)),
        ]),
        KernelRequest::IsComplete(KernelIsCompleteRequest { code }) => {
            json_object(vec![field("code", json_string(code))])
        }
        KernelRequest::KernelInfo(_) | KernelRequest::Interrupt(_) => json_object(Vec::new()),
        KernelRequest::Shutdown(KernelShutdownRequest { restart }) => {
            json_object(vec![field("restart", json_bool(*restart))])
        }
    }
}

fn encode_input_request_content(request: &KernelInputRequest) -> String {
    json_object(vec![
        field("prompt", json_string(&request.prompt)),
        field("password", json_bool(request.password)),
    ])
}

fn encode_input_reply_content(reply: &KernelInputReply) -> String {
    json_object(vec![field("value", json_string(&reply.value))])
}

fn encode_response_content(response: &KernelResponse) -> String {
    match response {
        KernelResponse::Execute(response) => encode_execute_response(response),
        KernelResponse::Inspect(response) => encode_inspect_response(response),
        KernelResponse::Complete(response) => encode_complete_response(response),
        KernelResponse::IsComplete(response) => encode_is_complete_response(response),
        KernelResponse::KernelInfo(response) => encode_kernel_info_response(response),
        KernelResponse::Interrupt(response) => encode_interrupt_response(response),
        KernelResponse::Shutdown(response) => encode_shutdown_response(response),
    }
}

fn encode_iopub_content(payload: &KernelIoPubPayload) -> String {
    match payload {
        KernelIoPubPayload::Status(status) => encode_status_output(status),
        KernelIoPubPayload::ExecuteInput(input) => encode_execute_input(input),
        KernelIoPubPayload::Stream(stream) => encode_stream_output(stream),
        KernelIoPubPayload::ExecuteResult(result) => encode_execute_result(result),
        KernelIoPubPayload::DisplayData(display) => encode_display_output(display),
        KernelIoPubPayload::Error(error) => encode_error_output(error),
    }
}

fn encode_header(header: &KernelMessageHeader) -> String {
    json_object(vec![
        field("msg_id", json_string(&header.msg_id)),
        field("session", json_string(&header.session_id)),
        field("username", json_string(&header.username)),
        field("msg_type", json_string(&header.msg_type)),
        field("version", json_string(&header.protocol_version)),
        field("date", json_string(&header.date)),
    ])
}

fn encode_optional_header(header: Option<&KernelMessageHeader>) -> String {
    header
        .map(encode_header)
        .unwrap_or_else(|| "{}".to_string())
}

fn encode_execute_response(response: &KernelExecuteResponse) -> String {
    let mut fields = vec![
        field("status", json_string(response.status.as_str())),
        field("execution_count", json_option_u64(response.execution_count)),
        field(
            "payload",
            json_array(
                response
                    .payload
                    .iter()
                    .map(encode_execute_payload)
                    .collect(),
            ),
        ),
        field(
            "user_expressions",
            json_object(
                response
                    .user_expressions
                    .iter()
                    .map(|(name, bundle)| field(name, encode_mime_bundle(bundle)))
                    .collect(),
            ),
        ),
    ];

    if let Some(error) = &response.error {
        fields.push(field("ename", json_string(&error.error_name)));
        fields.push(field("evalue", json_string(&error.error_value)));
        fields.push(field("traceback", encode_traceback(error)));
    }

    json_object(fields)
}

fn encode_inspect_response(response: &KernelInspectResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field("found", json_bool(response.found)),
        field("data", encode_mime_bundle(&response.data)),
        field("metadata", encode_metadata(&response.metadata)),
    ])
}

fn encode_complete_response(response: &KernelCompleteResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field(
            "matches",
            json_array(
                response
                    .matches
                    .iter()
                    .map(|value| json_string(value))
                    .collect(),
            ),
        ),
        field("cursor_start", json_usize(response.replacement_start)),
        field("cursor_end", json_usize(response.replacement_end)),
        field("metadata", encode_metadata(&response.metadata)),
    ])
}

fn encode_is_complete_response(response: &KernelIsCompleteResponse) -> String {
    let mut fields = vec![field(
        "status",
        json_string(response.completion_status.as_str()),
    )];

    if let Some(indent) = &response.indent {
        fields.push(field("indent", json_string(indent)));
    }

    json_object(fields)
}

fn encode_kernel_info_response(response: &KernelInfoResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field("protocol_version", json_string(&response.protocol_version)),
        field("implementation", json_string(&response.implementation)),
        field(
            "implementation_version",
            json_string(&response.implementation_version),
        ),
        field(
            "language_info",
            encode_language_info(&response.language_info),
        ),
        field("banner", json_string(&response.banner)),
        field(
            "help_links",
            json_array(response.help_links.iter().map(encode_help_link).collect()),
        ),
    ])
}

fn encode_interrupt_response(response: &KernelInterruptResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field(
            "interrupt_requested",
            json_bool(response.interrupt_requested),
        ),
        field(
            "lifecycle_state",
            json_string(response.lifecycle_state.as_str()),
        ),
    ])
}

fn encode_shutdown_response(response: &KernelShutdownResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field("restart", json_bool(response.restart)),
        field(
            "lifecycle_state",
            json_string(response.lifecycle_state.as_str()),
        ),
    ])
}

fn encode_status_output(output: &KernelStatusOutput) -> String {
    json_object(vec![
        field(
            "execution_state",
            json_string(output.execution_state.as_str()),
        ),
        field(
            "lifecycle_state",
            json_string(output.lifecycle_state.as_str()),
        ),
    ])
}

fn encode_execute_input(input: &KernelExecuteInput) -> String {
    json_object(vec![
        field("code", json_string(&input.code)),
        field("execution_count", json_option_u64(input.execution_count)),
    ])
}

fn encode_stream_output(stream: &KernelStreamOutput) -> String {
    json_object(vec![
        field(
            "name",
            json_string(match stream.name {
                KernelStreamName::Stdout => "stdout",
                KernelStreamName::Stderr => "stderr",
            }),
        ),
        field("text", json_string(&stream.text)),
    ])
}

fn encode_execute_result(result: &KernelExecuteResult) -> String {
    json_object(vec![
        field("execution_count", json_option_u64(result.execution_count)),
        field("data", encode_mime_bundle(&result.data)),
        field("metadata", encode_metadata(&result.metadata)),
    ])
}

fn encode_display_output(display: &KernelDisplayOutput) -> String {
    json_object(vec![
        field("data", encode_mime_bundle(&display.data)),
        field("metadata", encode_metadata(&display.metadata)),
    ])
}

fn encode_error_output(error: &KernelErrorOutput) -> String {
    json_object(vec![
        field("ename", json_string(&error.error_name)),
        field("evalue", json_string(&error.error_value)),
        field("traceback", encode_traceback(error)),
    ])
}

fn encode_execute_payload(payload: &KernelExecutePayload) -> String {
    json_object(vec![
        field("source", json_string(&payload.source)),
        field("data", encode_mime_bundle(&payload.data)),
        field("metadata", encode_metadata(&payload.metadata)),
    ])
}

fn encode_language_info(info: &KernelLanguageInfo) -> String {
    json_object(vec![
        field("name", json_string(&info.name)),
        field("version", json_string(&info.version)),
        field("mimetype", json_string(&info.mimetype)),
        field("file_extension", json_string(&info.file_extension)),
        field("pygments_lexer", json_string(&info.pygments_lexer)),
        field("codemirror_mode", json_string(&info.codemirror_mode)),
    ])
}

fn encode_help_link(link: &KernelHelpLink) -> String {
    json_object(vec![
        field("text", json_string(&link.text)),
        field("url", json_string(&link.url)),
    ])
}

fn encode_metadata(metadata: &KernelMetadata) -> String {
    json_object(
        metadata
            .items
            .iter()
            .map(|(key, value)| field(key, json_string(value)))
            .collect(),
    )
}

fn encode_mime_bundle(bundle: &KernelMimeBundle) -> String {
    json_object(
        bundle
            .items
            .iter()
            .map(|(key, value)| field(key, json_string(value)))
            .collect(),
    )
}

fn encode_traceback(error: &KernelErrorOutput) -> String {
    let traceback = error
        .rendered
        .as_deref()
        .unwrap_or(error.error_value.as_str());
    json_array(vec![json_string(traceback)])
}

fn parse_header(input: &str) -> Result<KernelMessageHeader, KernelWireError> {
    let object = parse_object(input, "header")?;

    Ok(KernelMessageHeader {
        msg_id: string_field(&object, "header", "msg_id")?,
        session_id: string_field(&object, "header", "session")?,
        username: string_field(&object, "header", "username")?,
        msg_type: string_field(&object, "header", "msg_type")?,
        protocol_version: string_field(&object, "header", "version")?,
        date: optional_string_field(&object, "header", "date")?
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string()),
    })
}

fn parse_optional_header(input: &str) -> Result<Option<KernelMessageHeader>, KernelWireError> {
    let object = parse_object(input, "parent_header")?;
    if object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(KernelMessageHeader {
            msg_id: string_field(&object, "parent_header", "msg_id")?,
            session_id: string_field(&object, "parent_header", "session")?,
            username: string_field(&object, "parent_header", "username")?,
            msg_type: string_field(&object, "parent_header", "msg_type")?,
            protocol_version: string_field(&object, "parent_header", "version")?,
            date: optional_string_field(&object, "parent_header", "date")?
                .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string()),
        }))
    }
}

fn parse_metadata(input: &str) -> Result<KernelMetadata, KernelWireError> {
    let object = parse_object(input, "metadata")?;
    let mut metadata = KernelMetadata::new();

    for (key, value) in object {
        metadata.insert(key, scalar_to_string("metadata", value)?);
    }

    Ok(metadata)
}

fn parse_request_content(msg_type: &str, input: &str) -> Result<KernelRequest, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "execute_request" => Ok(KernelRequest::Execute(KernelExecuteRequest {
            code: string_field(&object, "content", "code")?,
            store_history: bool_field_with_default(&object, "content", "store_history", true)?,
            allow_stdin: bool_field_with_default(&object, "content", "allow_stdin", true)?,
        })),
        "inspect_request" => Ok(KernelRequest::Inspect(KernelInspectRequest {
            code: string_field(&object, "content", "code")?,
            cursor_offset: usize_field(&object, "content", "cursor_pos")?,
        })),
        "complete_request" => Ok(KernelRequest::Complete(KernelCompleteRequest {
            code: string_field(&object, "content", "code")?,
            cursor_offset: usize_field(&object, "content", "cursor_pos")?,
        })),
        "is_complete_request" => Ok(KernelRequest::IsComplete(KernelIsCompleteRequest {
            code: string_field(&object, "content", "code")?,
        })),
        "kernel_info_request" => Ok(KernelRequest::KernelInfo(crate::KernelInfoRequest)),
        "interrupt_request" => Ok(KernelRequest::Interrupt(KernelInterruptRequest)),
        "shutdown_request" => Ok(KernelRequest::Shutdown(KernelShutdownRequest {
            restart: bool_field_with_default(&object, "content", "restart", false)?,
        })),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

fn parse_input_request_content(
    msg_type: &str,
    input: &str,
) -> Result<KernelInputRequest, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "input_request" => Ok(KernelInputRequest {
            prompt: string_field(&object, "content", "prompt")?,
            password: bool_field_with_default(&object, "content", "password", false)?,
        }),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

fn parse_input_reply_content(
    msg_type: &str,
    input: &str,
) -> Result<KernelInputReply, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "input_reply" => Ok(KernelInputReply {
            value: string_field(&object, "content", "value")?,
        }),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

fn parse_response_content(
    msg_type: &str,
    input: &str,
    metadata: &KernelMetadata,
) -> Result<KernelResponse, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "execute_reply" => {
            let status = parse_reply_status(&string_field(&object, "content", "status")?)?;
            let execution_count = optional_u64_field(&object, "content", "execution_count")?;
            let payload = execute_payloads_field(&object, "content", "payload")?;
            let user_expressions = mime_bundle_map_field(&object, "content", "user_expressions")?;
            let error = if object.contains_key("ename") {
                Some(parse_error_fields(&object, "content")?)
            } else {
                None
            };

            Ok(KernelResponse::Execute(KernelExecuteResponse {
                status,
                execution_count,
                payload,
                user_expressions,
                error,
                outputs: Vec::new(),
                outcome: KernelExecuteOutcome::WireDecoded,
            }))
        }
        "inspect_reply" => Ok(KernelResponse::Inspect(KernelInspectResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            target: metadata.get("target").map(ToOwned::to_owned),
            found: bool_field_with_default(&object, "content", "found", false)?,
            data: mime_bundle_field(&object, "content", "data")?,
            metadata: metadata_field(&object, "content", "metadata")?,
            result: None,
        })),
        "complete_reply" => Ok(KernelResponse::Complete(KernelCompleteResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            replacement_start: usize_field(&object, "content", "cursor_start")?,
            replacement_end: usize_field(&object, "content", "cursor_end")?,
            matches: string_array_field(&object, "content", "matches")?,
            metadata: metadata_field(&object, "content", "metadata")?,
            prefix: metadata.get("prefix").unwrap_or_default().to_string(),
        })),
        "is_complete_reply" => Ok(KernelResponse::IsComplete(KernelIsCompleteResponse {
            reply_status: parse_reply_status(
                metadata
                    .get("status")
                    .unwrap_or(KernelReplyStatus::Ok.as_str()),
            )?,
            completion_status: parse_is_complete_status(&string_field(
                &object, "content", "status",
            )?)?,
            indent: optional_string_field(&object, "content", "indent")?,
        })),
        "kernel_info_reply" => Ok(KernelResponse::KernelInfo(KernelInfoResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            protocol_version: string_field(&object, "content", "protocol_version")?,
            implementation: string_field(&object, "content", "implementation")?,
            implementation_version: string_field(&object, "content", "implementation_version")?,
            banner: string_field(&object, "content", "banner")?,
            language_info: language_info_field(&object, "content", "language_info")?,
            help_links: help_links_field(&object, "content", "help_links")?,
        })),
        "interrupt_reply" => Ok(KernelResponse::Interrupt(KernelInterruptResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            interrupt_requested: bool_field_with_default(
                &object,
                "content",
                "interrupt_requested",
                false,
            )?,
            lifecycle_state: parse_lifecycle_state(&string_field(
                &object,
                "content",
                "lifecycle_state",
            )?)?,
        })),
        "shutdown_reply" => Ok(KernelResponse::Shutdown(KernelShutdownResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            restart: bool_field_with_default(&object, "content", "restart", false)?,
            lifecycle_state: parse_lifecycle_state(&string_field(
                &object,
                "content",
                "lifecycle_state",
            )?)?,
        })),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

fn parse_iopub_content(
    msg_type: &str,
    input: &str,
    _metadata: &KernelMetadata,
) -> Result<KernelIoPubPayload, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "status" => Ok(KernelIoPubPayload::Status(KernelStatusOutput {
            execution_state: parse_execution_state(&string_field(
                &object,
                "content",
                "execution_state",
            )?)?,
            lifecycle_state: parse_lifecycle_state(&string_field(
                &object,
                "content",
                "lifecycle_state",
            )?)?,
        })),
        "execute_input" => Ok(KernelIoPubPayload::ExecuteInput(KernelExecuteInput {
            execution_count: optional_u64_field(&object, "content", "execution_count")?,
            code: string_field(&object, "content", "code")?,
        })),
        "stream" => Ok(KernelIoPubPayload::Stream(KernelStreamOutput {
            name: parse_stream_name(&string_field(&object, "content", "name")?)?,
            text: string_field(&object, "content", "text")?,
        })),
        "execute_result" => Ok(KernelIoPubPayload::ExecuteResult(KernelExecuteResult {
            execution_count: optional_u64_field(&object, "content", "execution_count")?,
            data: mime_bundle_field(&object, "content", "data")?,
            metadata: metadata_field(&object, "content", "metadata")?,
        })),
        "display_data" => Ok(KernelIoPubPayload::DisplayData(KernelDisplayOutput {
            data: mime_bundle_field(&object, "content", "data")?,
            metadata: metadata_field(&object, "content", "metadata")?,
        })),
        "error" => Ok(KernelIoPubPayload::Error(parse_error_fields(
            &object, "content",
        )?)),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

fn parse_object(
    input: &str,
    context: &'static str,
) -> Result<BTreeMap<String, JsonValue>, KernelWireError> {
    parse_json_object(input).map_err(|message| {
        if let Some(found) = message.strip_prefix("expected top-level object, found ") {
            KernelWireError::InvalidField {
                context,
                field: "<root>",
                value: found.to_string(),
            }
        } else {
            KernelWireError::InvalidJson { message }
        }
    })
}

fn string_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<String, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

fn usize_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<usize, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<usize>()
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

fn bool_field_with_default(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
    default: bool,
) -> Result<bool, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Bool(value)) => Ok(*value),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(default),
    }
}

fn scalar_to_string(context: &'static str, value: JsonValue) -> Result<String, KernelWireError> {
    match value {
        JsonValue::String(value) => Ok(value),
        JsonValue::Number(value) => Ok(value),
        JsonValue::Bool(value) => Ok(if value { "true" } else { "false" }.to_string()),
        JsonValue::Null => Ok("null".to_string()),
        JsonValue::Array(_) | JsonValue::Object(_) => Err(KernelWireError::InvalidField {
            context,
            field: "<scalar>",
            value: value.kind_name().to_string(),
        }),
    }
}

fn optional_string_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Option<String>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(None),
    }
}

fn optional_u64_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Option<u64>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        Some(JsonValue::Null) => Ok(None),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(None),
    }
}

fn string_array_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<String>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                JsonValue::String(value) => Ok(value.clone()),
                other => Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: other.kind_name().to_string(),
                }),
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

fn metadata_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelMetadata, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut metadata = KernelMetadata::new();
    for (key, value) in nested {
        metadata.insert(key, scalar_to_string(context, value)?);
    }
    Ok(metadata)
}

fn mime_bundle_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelMimeBundle, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut bundle = KernelMimeBundle::new();
    for (key, value) in nested {
        bundle.insert(key, scalar_to_string(context, value)?);
    }
    Ok(bundle)
}

fn mime_bundle_map_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<BTreeMap<String, KernelMimeBundle>, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut result = BTreeMap::new();
    for (key, value) in nested {
        let inner = match value {
            JsonValue::Object(map) => map,
            other => {
                return Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: other.kind_name().to_string(),
                });
            }
        };
        let mut bundle = KernelMimeBundle::new();
        for (mime, value) in inner {
            bundle.insert(mime, scalar_to_string(context, value)?);
        }
        result.insert(key, bundle);
    }
    Ok(result)
}

fn execute_payloads_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelExecutePayload>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Object(map) => map,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                Ok(KernelExecutePayload {
                    source: string_field(item, context, "source")?,
                    data: mime_bundle_field(item, context, "data")?,
                    metadata: metadata_field(item, context, "metadata")?,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

fn language_info_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelLanguageInfo, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    Ok(KernelLanguageInfo {
        name: string_field(&nested, context, "name")?,
        version: string_field(&nested, context, "version")?,
        mimetype: string_field(&nested, context, "mimetype")?,
        file_extension: string_field(&nested, context, "file_extension")?,
        pygments_lexer: string_field(&nested, context, "pygments_lexer")?,
        codemirror_mode: string_field(&nested, context, "codemirror_mode")?,
    })
}

fn help_links_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelHelpLink>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Object(map) => map,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                Ok(KernelHelpLink {
                    text: string_field(item, context, "text")?,
                    url: string_field(item, context, "url")?,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

fn parse_error_fields(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
) -> Result<KernelErrorOutput, KernelWireError> {
    let rendered = match object.get("traceback") {
        Some(JsonValue::Array(values)) => values.first().and_then(|value| match value {
            JsonValue::String(value) => Some(value.clone()),
            _ => None,
        }),
        _ => None,
    };

    Ok(KernelErrorOutput {
        error_name: string_field(object, context, "ename")?,
        error_value: string_field(object, context, "evalue")?,
        rendered,
    })
}

fn nested_object_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<BTreeMap<String, JsonValue>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Object(value)) => Ok(value.clone()),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

fn parse_reply_status(value: &str) -> Result<KernelReplyStatus, KernelWireError> {
    match value {
        "ok" => Ok(KernelReplyStatus::Ok),
        "error" => Ok(KernelReplyStatus::Error),
        "abort" => Ok(KernelReplyStatus::Abort),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "status",
            value: other.to_string(),
        }),
    }
}

fn parse_is_complete_status(value: &str) -> Result<KernelIsCompleteStatus, KernelWireError> {
    match value {
        "complete" => Ok(KernelIsCompleteStatus::Complete),
        "incomplete" => Ok(KernelIsCompleteStatus::Incomplete),
        "invalid" => Ok(KernelIsCompleteStatus::Invalid),
        "unknown" => Ok(KernelIsCompleteStatus::Unknown),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "status",
            value: other.to_string(),
        }),
    }
}

fn parse_execution_state(value: &str) -> Result<crate::KernelExecutionState, KernelWireError> {
    match value {
        "starting" => Ok(crate::KernelExecutionState::Starting),
        "busy" => Ok(crate::KernelExecutionState::Busy),
        "idle" => Ok(crate::KernelExecutionState::Idle),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "execution_state",
            value: other.to_string(),
        }),
    }
}

fn parse_lifecycle_state(value: &str) -> Result<crate::KernelLifecycleState, KernelWireError> {
    match value {
        "running" => Ok(crate::KernelLifecycleState::Running),
        "interrupt_requested" => Ok(crate::KernelLifecycleState::InterruptRequested),
        "shutdown_requested" => Ok(crate::KernelLifecycleState::ShutdownRequested),
        "shutdown" => Ok(crate::KernelLifecycleState::Shutdown),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "lifecycle_state",
            value: other.to_string(),
        }),
    }
}

fn parse_stream_name(value: &str) -> Result<KernelStreamName, KernelWireError> {
    match value {
        "stdout" => Ok(KernelStreamName::Stdout),
        "stderr" => Ok(KernelStreamName::Stderr),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "name",
            value: other.to_string(),
        }),
    }
}

fn field(name: &str, value: String) -> (String, String) {
    (name.to_string(), value)
}

fn json_object(fields: Vec<(String, String)>) -> String {
    let parts = fields
        .into_iter()
        .map(|(key, value)| format!("{}:{}", json_string(&key), value))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{parts}}}")
}

fn json_array(items: Vec<String>) -> String {
    format!("[{}]", items.join(","))
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }

    escaped.push('"');
    escaped
}

fn json_bool(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn json_usize(value: usize) -> String {
    value.to_string()
}

fn json_option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}
