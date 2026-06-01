use crate::KernelStdinError;

use super::json::*;
use super::*;

mod validation;

const DEFAULT_DELIMITER: &str = "<IDS|MSG>";

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
