use crate::signature::hmac_sha256_hex;

use super::*;

impl KernelWireCodec {
    pub(super) fn validate_delimiter(&self, delimiter: &str) -> Result<(), KernelWireError> {
        if delimiter == self.config.delimiter {
            Ok(())
        } else {
            Err(KernelWireError::InvalidDelimiter {
                expected: self.config.delimiter.clone(),
                found: delimiter.to_string(),
            })
        }
    }

    pub(super) fn validate_session_id(&self, session_id: &str) -> Result<(), KernelWireError> {
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

    pub(super) fn verify_signature(
        &self,
        message: &KernelWireMessage,
    ) -> Result<(), KernelWireError> {
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

    pub(super) fn compute_signature(
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

    pub(super) fn validate_request_channel(
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

    pub(super) fn validate_response_channel(
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

    pub(super) fn validate_iopub_channel(
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

    pub(super) fn validate_input_request_channel(
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

    pub(super) fn validate_input_reply_channel(
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
