use super::*;
use crate::{
    KernelExecuteResponse, KernelQueuedTransport, KernelResponseEnvelope, KernelStatusOutput,
    KernelWireMessage, LocalSocketTransport,
};

fn wire_message(channel: KernelChannel, msg_id: &str, msg_type: &str) -> KernelWireMessage {
    KernelWireMessage {
        channel,
        routing_ids: Vec::new(),
        delimiter: "<IDS|MSG>".to_string(),
        signature: None,
        header_json: format!(
            r#"{{"msg_id":"{msg_id}","session":"session-coverage","username":"tester","msg_type":"{msg_type}","version":"5.3"}}"#
        ),
        parent_header_json: "{}".to_string(),
        metadata_json: "{}".to_string(),
        content_json: "{}".to_string(),
        buffers: Vec::new(),
    }
}

#[test]
fn codec_rejects_missing_signature_bad_delimiter_and_session_mismatch() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-coverage")
            .with_signature_key("secret")
            .with_delimiter("<CUSTOM>"),
    );
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-signed",
            "session-coverage",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1+1")),
    };

    let mut missing_signature = codec
        .encode_request_envelope(&envelope)
        .expect("signed request should encode");
    missing_signature.signature = None;
    assert_eq!(
        codec
            .decode_request_message(&missing_signature)
            .expect_err("required signatures must be present"),
        KernelWireError::MissingSignature
    );

    let mut bad_delimiter = codec
        .encode_request_envelope(&envelope)
        .expect("signed request should encode");
    bad_delimiter.delimiter = "<IDS|MSG>".to_string();
    assert_eq!(
        codec
            .decode_request_message(&bad_delimiter)
            .expect_err("wrong delimiter should be rejected before parsing content"),
        KernelWireError::InvalidDelimiter {
            expected: "<CUSTOM>".to_string(),
            found: "<IDS|MSG>".to_string(),
        }
    );

    let wrong_session = KernelRequestEnvelope {
        header: KernelMessageHeader::new(
            "client-msg-wrong-session",
            "other-session",
            "tester",
            "execute_request",
        ),
        ..envelope
    };
    assert_eq!(
        codec
            .encode_request_envelope(&wrong_session)
            .expect_err("configured session ids should be enforced on encode"),
        KernelWireError::SessionMismatch {
            expected: "session-coverage".to_string(),
            found: "other-session".to_string(),
        }
    );
}

#[test]
fn codec_rejects_wrong_channels_for_response_iopub_and_stdin_messages() {
    let codec =
        KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-coverage"));

    let response_on_control = wire_message(KernelChannel::Control, "reply-wrong", "execute_reply");
    assert_eq!(
        codec
            .decode_response_message(&response_on_control)
            .expect_err("execute_reply is a shell reply"),
        KernelWireError::InvalidChannelForMessageType {
            channel: KernelChannel::Control,
            msg_type: "execute_reply".to_string(),
        }
    );

    let unknown_reply = wire_message(KernelChannel::Shell, "reply-unknown", "unknown_reply");
    assert_eq!(
        codec
            .decode_response_message(&unknown_reply)
            .expect_err("unknown replies should be rejected"),
        KernelWireError::UnsupportedMessageType {
            msg_type: "unknown_reply".to_string(),
        }
    );

    let status_on_shell = wire_message(KernelChannel::Shell, "iopub-wrong", "status");
    assert_eq!(
        codec
            .decode_iopub_message(&status_on_shell)
            .expect_err("iopub payloads must arrive on iopub"),
        KernelWireError::InvalidChannelForMessageType {
            channel: KernelChannel::Shell,
            msg_type: "status".to_string(),
        }
    );

    let stdin_on_shell = wire_message(KernelChannel::Shell, "stdin-wrong", "input_request");
    assert_eq!(
        codec
            .decode_input_request_message(&stdin_on_shell)
            .expect_err("stdin requests must arrive on stdin"),
        KernelWireError::InvalidChannelForMessageType {
            channel: KernelChannel::Shell,
            msg_type: "input_request".to_string(),
        }
    );
}

#[test]
fn codec_preserves_reply_routing_ids_for_replies_and_stdin_requests() {
    let codec =
        KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-coverage"));
    let parent = KernelMessageHeader::new(
        "client-msg-parent",
        "session-coverage",
        "tester",
        "execute_request",
    );
    let dispatch = KernelDispatchResult {
        reply: Some(KernelResponseEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "kernel-reply",
                "session-coverage",
                "tester",
                "execute_reply",
            ),
            parent_header: Some(parent.clone()),
            metadata: KernelMetadata::new(),
            response: KernelResponse::Execute(KernelExecuteResponse {
                status: KernelReplyStatus::Ok,
                execution_count: Some(1),
                payload: Vec::new(),
                user_expressions: Default::default(),
                error: None,
                outputs: Vec::new(),
                outcome: KernelExecuteOutcome::WireDecoded,
            }),
        }),
        iopub: vec![crate::KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: KernelMessageHeader::new(
                "kernel-status",
                "session-coverage",
                "tester",
                "status",
            ),
            parent_header: Some(parent.clone()),
            metadata: KernelMetadata::new(),
            payload: KernelIoPubPayload::Status(KernelStatusOutput {
                execution_state: KernelExecutionState::Busy,
                lifecycle_state: KernelLifecycleState::Running,
            }),
        }],
        stdin: vec![crate::KernelInputRequestEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "kernel-stdin",
                "session-coverage",
                "tester",
                "input_request",
            ),
            parent_header: Some(parent),
            metadata: KernelMetadata::new(),
            request: KernelInputRequest::new("value: "),
        }],
    };

    let routing = vec![b"frontend".to_vec(), b"worker".to_vec()];
    let batch = codec
        .encode_dispatch_result_with_routing(&dispatch, routing.clone())
        .expect("dispatch should encode");

    assert_eq!(batch.reply.as_ref().unwrap().routing_ids, routing);
    assert_eq!(batch.stdin[0].routing_ids, routing);
    assert!(batch.iopub[0].routing_ids.is_empty());
}

#[test]
fn local_socket_transport_pair_routes_messages_and_heartbeats_in_memory() {
    let (mut kernel, frontend) = LocalSocketTransport::pair();
    let shell = wire_message(KernelChannel::Shell, "shell-in", "kernel_info_request");
    let control = wire_message(KernelChannel::Control, "control-in", "interrupt_request");
    let stdin = wire_message(KernelChannel::Stdin, "stdin-in", "input_reply");
    let iopub = wire_message(KernelChannel::IoPub, "iopub-in", "status");

    frontend.queue_incoming(shell.clone());
    frontend.queue_incoming(control.clone());
    frontend.queue_incoming(stdin.clone());
    frontend.queue_incoming(iopub);

    assert!(kernel.has_pending_incoming());
    assert!(kernel.has_pending_incoming_channel(KernelChannel::Shell));
    assert!(kernel.has_pending_incoming_channel(KernelChannel::Control));
    assert!(kernel.has_pending_incoming_channel(KernelChannel::Stdin));
    assert!(!kernel.has_pending_incoming_channel(KernelChannel::IoPub));
    assert_eq!(kernel.pop_incoming(KernelChannel::Shell), Some(shell));
    assert_eq!(kernel.pop_incoming(KernelChannel::Control), Some(control));
    assert_eq!(kernel.pop_incoming(KernelChannel::Stdin), Some(stdin));
    assert_eq!(kernel.pop_incoming(KernelChannel::IoPub), None);
    assert!(!kernel.has_pending_incoming());

    let reply = wire_message(KernelChannel::Shell, "shell-out", "kernel_info_reply");
    let status = wire_message(KernelChannel::IoPub, "status-out", "status");
    kernel.send(reply.clone());
    kernel.send(status.clone());
    assert!(frontend.has_pending_outgoing());
    assert_eq!(frontend.outgoing_len(KernelChannel::Shell), 1);
    assert_eq!(frontend.drain_outgoing(KernelChannel::Shell), vec![reply]);
    assert_eq!(frontend.pop_outgoing(KernelChannel::IoPub), Some(status));
    assert!(!frontend.has_pending_outgoing());

    frontend.send_heartbeat(b"ping".to_vec());
    assert!(kernel.has_pending_heartbeat());
    assert_eq!(kernel.pump_heartbeats_until_idle(), 1);
    assert_eq!(frontend.pop_heartbeat_reply(), Some(b"ping".to_vec()));
    assert!(!kernel.has_pending_heartbeat());
}
