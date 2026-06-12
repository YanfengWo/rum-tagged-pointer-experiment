use super::*;

#[test]
fn transport_encoding_serializes_reply_and_iopub_messages() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new("client-msg-50", "session-6", "tester", "execute_request"),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
    });

    let batch = encode_dispatch_result(&dispatch);
    let reply = batch
        .reply
        .as_ref()
        .expect("execute batch should include a reply");
    assert_eq!(reply.channel, KernelChannel::Shell);
    assert!(reply.header_json.contains("\"msg_type\":\"execute_reply\""));
    assert!(reply.content_json.contains("\"status\":\"ok\""));
    assert_eq!(batch.iopub.len(), 4);
    assert!(
        batch.iopub[0]
            .content_json
            .contains("\"execution_state\":\"busy\"")
    );
    assert!(
        batch.iopub[1]
            .content_json
            .contains("\"execution_count\":1")
    );

    let request_wire = encode_request_envelope(&KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-51",
            "session-6",
            "tester",
            "complete_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Complete(KernelCompleteRequest::new("loc", 3)),
    });
    assert!(request_wire.content_json.contains("\"cursor_pos\":3"));

    let mut transport = InMemoryTransport::default();
    transport.send_batch(batch);
    assert_eq!(transport.messages.len(), 5);
    assert_eq!(transport.messages[0].channel, KernelChannel::Shell);
    assert_eq!(transport.messages[1].channel, KernelChannel::IoPub);
}

#[test]
fn signed_transport_codec_round_trips_request_messages() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-7")
            .with_signature_key("secret"),
    );
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new("client-msg-60", "session-7", "tester", "execute_request"),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1+1")),
    };

    let wire = codec
        .encode_request_envelope(&envelope)
        .expect("signed request should encode");
    assert!(wire.signature.is_some());

    let decoded = codec
        .decode_request_message(&wire)
        .expect("signed request should decode");
    assert_eq!(decoded.channel, KernelChannel::Shell);
    assert_eq!(decoded.header.msg_id, "client-msg-60");
    assert_eq!(decoded.header.session_id, "session-7");

    match decoded.request {
        KernelRequest::Execute(request) => {
            assert_eq!(request.code, "1+1");
            assert!(request.store_history);
            assert!(request.allow_stdin);
        }
        other => panic!("expected execute request, got {other:?}"),
    }
}

#[test]
fn transport_codec_decodes_escaped_and_utf8_unicode_request_payloads() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-7u")
            .with_signature_key("secret"),
    );
    let mut wire = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-60u",
                "session-7u",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("Print[\"ok\"]")),
        })
        .expect("request should encode");

    wire.content_json =
        "{\"code\":\"\\u4f60\\u597d 🚀\",\"store_history\":true,\"allow_stdin\":true}".to_string();
    wire.signature = Some(crate::signature::hmac_sha256_hex(
        "secret",
        [
            &wire.header_json,
            &wire.parent_header_json,
            &wire.metadata_json,
            &wire.content_json,
        ],
    ));

    let decoded = codec
        .decode_request_message(&wire)
        .expect("unicode request should decode");
    match decoded.request {
        KernelRequest::Execute(request) => assert_eq!(request.code, "你好 🚀"),
        other => panic!("expected execute request, got {other:?}"),
    }
}

#[test]
fn transport_codec_accepts_structured_frontend_metadata() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-7m"));
    let mut wire = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-60m",
                "session-7m",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1+1")),
        })
        .expect("request should encode");

    wire.metadata_json =
        r#"{"trusted":true,"tags":["parameters"],"frontend":{"name":"jupyterlab"}}"#.to_string();

    let decoded = codec
        .decode_request_message(&wire)
        .expect("structured metadata should not reject the request");

    assert_eq!(decoded.metadata.get("trusted"), Some("true"));
    assert_eq!(decoded.metadata.get("tags"), Some(r#"["parameters"]"#));
    assert_eq!(
        decoded.metadata.get("frontend"),
        Some(r#"{"name":"jupyterlab"}"#)
    );
}

#[test]
fn execute_request_round_trips_allow_stdin_flag() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-7b"));
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-60b",
            "session-7b",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1+1").without_stdin()),
    };

    let wire = codec
        .encode_request_envelope(&envelope)
        .expect("request should encode");
    assert!(wire.content_json.contains("\"allow_stdin\":false"));

    let decoded = codec
        .decode_request_message(&wire)
        .expect("request should decode");
    match decoded.request {
        KernelRequest::Execute(request) => {
            assert!(!request.allow_stdin);
        }
        other => panic!("expected execute request, got {other:?}"),
    }
}

#[test]
fn signed_transport_codec_rejects_signature_mismatch() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-8")
            .with_signature_key("secret"),
    );
    let mut wire = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-61",
                "session-8",
                "tester",
                "complete_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Complete(KernelCompleteRequest::new("loc", 3)),
        })
        .expect("request should encode");

    wire.content_json = "{\"code\":\"tampered\",\"cursor_pos\":3}".to_string();

    let error = codec
        .decode_request_message(&wire)
        .expect_err("tampered request should fail signature validation");
    assert_eq!(error, KernelWireError::InvalidSignature);
}

#[test]
fn transport_codec_rejects_invalid_channel_for_request_type() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-9"));
    let mut wire = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-62",
                "session-9",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
        })
        .expect("request should encode");

    wire.channel = KernelChannel::Control;

    let error = codec
        .decode_request_message(&wire)
        .expect_err("execute requests on control should be rejected");
    assert_eq!(
        error,
        KernelWireError::InvalidChannelForMessageType {
            channel: KernelChannel::Control,
            msg_type: "execute_request".to_string(),
        }
    );
}

#[test]
fn mock_socket_transport_routes_reply_and_iopub_messages_by_channel() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-10"));
    let mut transport = MockSocketTransport::default();

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-63",
            "session-10",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
    });

    codec
        .send_dispatch(&mut transport, &dispatch)
        .expect("dispatch should encode and route");

    assert_eq!(transport.shell.len(), 1);
    assert_eq!(transport.iopub.len(), 4);
    assert!(transport.control.is_empty());
    assert_eq!(transport.shell[0].channel, KernelChannel::Shell);
    assert_eq!(transport.iopub[0].channel, KernelChannel::IoPub);
}

#[test]
fn transport_codec_decodes_execute_reply_and_iopub_messages() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-64",
            "session-11",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
    });

    let batch = encode_dispatch_result(&dispatch);
    let reply = decode_response_message(
        batch
            .reply
            .as_ref()
            .expect("execute batch should include a reply"),
    )
    .expect("reply should decode");
    assert_eq!(reply.channel, KernelChannel::Shell);
    assert_eq!(reply.header.msg_type, "execute_reply");
    assert!(reply.header.date.contains('T'));
    assert!(reply.header.date.ends_with('Z'));

    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert!(response.outputs.is_empty());
            assert!(matches!(
                response.outcome,
                super::KernelExecuteOutcome::WireDecoded
            ));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let execute_result = decode_iopub_message(&batch.iopub[2]).expect("iopub should decode");
    match execute_result.payload {
        KernelIoPubPayload::ExecuteResult(result) => {
            assert_eq!(result.execution_count, Some(1));
            assert_eq!(result.data.get("text/plain"), Some("1"));
        }
        other => panic!("expected execute_result payload, got {other:?}"),
    }

    let busy_status = decode_iopub_message(&batch.iopub[0]).expect("status should decode");
    match busy_status.payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected status payload, got {other:?}"),
    }
}

#[test]
fn optional_signature_policy_accepts_unsigned_messages_but_still_checks_signed_ones() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-12")
            .with_signature_policy(KernelSignaturePolicy::Optional("secret".to_string())),
    );
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-65",
            "session-12",
            "tester",
            "is_complete_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::IsComplete(KernelIsCompleteRequest::new("f[1]")),
    };

    let mut wire = codec
        .encode_request_envelope(&envelope)
        .expect("request should encode");
    assert!(wire.signature.is_some());
    wire.signature = None;

    let decoded = codec
        .decode_request_message(&wire)
        .expect("optional signatures should allow unsigned messages");
    assert_eq!(decoded.header.msg_type, "is_complete_request");

    let mut signed = codec
        .encode_request_envelope(&envelope)
        .expect("signed request should encode");
    signed.signature = Some("bad-signature".to_string());
    let error = codec
        .decode_request_message(&signed)
        .expect_err("bad signed message should still be rejected");
    assert_eq!(error, KernelWireError::InvalidSignature);
}

#[test]
fn request_headers_round_trip_date_field_through_wire_codec() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-14"));
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-67",
            "session-14",
            "tester",
            "kernel_info_request",
        )
        .with_date("2026-03-20T12:34:56Z"),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::KernelInfo(KernelInfoRequest),
    };

    let wire = codec
        .encode_request_envelope(&envelope)
        .expect("request should encode");
    let decoded = codec
        .decode_request_message(&wire)
        .expect("request should decode");

    assert_eq!(decoded.header.date, "2026-03-20T12:34:56Z");
}

#[test]
fn kernel_info_round_trips_on_control_channel() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-14c"));
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Control,
        header: KernelMessageHeader::new(
            "client-msg-67c",
            "session-14c",
            "tester",
            "kernel_info_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::KernelInfo(KernelInfoRequest),
    };

    let wire = codec
        .encode_request_envelope(&envelope)
        .expect("control kernel_info request should encode");
    let decoded = codec
        .decode_request_message(&wire)
        .expect("control kernel_info request should decode");
    assert_eq!(decoded.channel, KernelChannel::Control);

    let mut adapter = KernelAdapter::new(Session::new());
    let dispatch = adapter.dispatch_request(decoded);
    let reply = dispatch
        .reply
        .expect("kernel_info should reply on the request channel");
    assert_eq!(reply.channel, KernelChannel::Control);

    let encoded_reply = codec
        .encode_response_envelope(&reply)
        .expect("control kernel_info reply should encode");
    let decoded_reply = codec
        .decode_response_message(&encoded_reply)
        .expect("control kernel_info reply should decode");
    assert_eq!(decoded_reply.channel, KernelChannel::Control);
    assert_eq!(decoded_reply.header.msg_type, "kernel_info_reply");
}

#[test]
fn history_request_round_trips_with_empty_reply() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-14h"));
    let envelope = KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-67h",
            "session-14h",
            "tester",
            "history_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::History(KernelHistoryRequest),
    };

    let wire = codec
        .encode_request_envelope(&envelope)
        .expect("history request should encode");
    let decoded = codec
        .decode_request_message(&wire)
        .expect("history request should decode");
    assert_eq!(decoded.channel, KernelChannel::Shell);

    let mut adapter = KernelAdapter::new(Session::new());
    let dispatch = adapter.dispatch_request(decoded);
    let reply = dispatch
        .reply
        .expect("history request should receive an immediate reply");
    assert_eq!(reply.channel, KernelChannel::Shell);

    let encoded_reply = codec
        .encode_response_envelope(&reply)
        .expect("history reply should encode");
    assert!(encoded_reply.content_json.contains("\"history\":[]"));

    let decoded_reply = codec
        .decode_response_message(&encoded_reply)
        .expect("history reply should decode");
    assert_eq!(decoded_reply.header.msg_type, "history_reply");
    match decoded_reply.response {
        KernelResponse::History(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.history.is_empty());
        }
        other => panic!("expected history reply, got {other:?}"),
    }
}

#[test]
fn transport_codec_round_trips_stdin_input_messages() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-14d")
            .with_signature_key("secret"),
    );
    let input_request = crate::KernelInputRequestEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new("kernel-msg-1", "session-14d", "tester", "input_request"),
        parent_header: Some(KernelMessageHeader::new(
            "client-msg-parent",
            "session-14d",
            "tester",
            "execute_request",
        )),
        metadata: super::KernelMetadata::new(),
        request: KernelInputRequest::new("password: ").password(),
    };

    let request_wire = codec
        .encode_input_request_envelope(&input_request)
        .expect("stdin request should encode");
    let decoded_request =
        decode_input_request_message(&request_wire).expect("request should decode");
    assert_eq!(decoded_request.request.prompt, "password: ");
    assert!(decoded_request.request.password);

    let reply_wire = encode_input_reply_envelope(&KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new(
            "client-msg-stdin",
            "session-14d",
            "tester",
            "input_reply",
        ),
        parent_header: Some(decoded_request.header.clone()),
        metadata: super::KernelMetadata::new(),
        reply: KernelInputReply::new("secret"),
    });
    let decoded_reply = decode_input_reply_message(&reply_wire).expect("reply should decode");
    assert_eq!(decoded_reply.reply.value, "secret");
}
