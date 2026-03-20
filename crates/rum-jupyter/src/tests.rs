use rum_runtime::{Session, SessionRequestFailure};

use super::{
    InMemoryTransport, KernelAdapter, KernelBootstrapConfig, KernelChannel, KernelCompleteRequest,
    KernelConnectionFile, KernelConnectionFileError, KernelEventPump, KernelExecuteOutcome,
    KernelExecuteRequest, KernelExecutionState, KernelInfoRequest, KernelInputReply,
    KernelInputReplyEnvelope, KernelInputRequest, KernelInspectRequest, KernelInterruptRequest,
    KernelIoPubPayload, KernelIsCompleteRequest, KernelIsCompleteStatus, KernelLifecycleState,
    KernelMessageHeader, KernelOutput, KernelReplyStatus, KernelRequest, KernelRequestEnvelope,
    KernelResponse, KernelShutdownRequest, KernelSignaturePolicy, KernelSignatureScheme,
    KernelStreamName, KernelTransport, KernelTransportBackend, KernelTransportKind,
    KernelWireCodec, KernelWireConfig, KernelWireError, MockSocketTransport, MockTransportBackend,
    decode_input_reply_message, decode_input_request_message, decode_iopub_message,
    decode_response_message, encode_dispatch_result, encode_input_reply_envelope,
    encode_request_envelope,
};

#[test]
fn execute_request_maps_to_session_eval_and_tracks_execution_count() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.execute(KernelExecuteRequest::new("1"));

    assert_eq!(response.status, KernelReplyStatus::Ok);
    assert_eq!(response.execution_count, Some(1));
    assert_eq!(adapter.execution_count(), 1);
    assert_eq!(response.outputs.len(), 1);
    assert!(response.payload.is_empty());
    assert!(response.user_expressions.is_empty());
    assert!(response.error.is_none());

    match response.outcome {
        KernelExecuteOutcome::Success(result) => {
            assert_eq!(result.value.to_string(), "1");
            assert!(result.messages.is_empty());
        }
        other => panic!("expected execute success, got {other:?}"),
    }

    match &response.outputs[0] {
        KernelOutput::ExecuteResult(output) => {
            assert_eq!(output.execution_count, Some(1));
            assert_eq!(output.data.get("text/plain"), Some("1"));
            assert_eq!(output.data.get("text/markdown"), Some("```rum\n1\n```"));
            assert_eq!(output.data.get("application/vnd.rum.expr"), Some("1"));
            assert_eq!(output.metadata.get("output_type"), Some("execute_result"));
            assert_eq!(output.metadata.get("bundle_kind"), Some("expr"));
        }
        other => panic!("expected execute result output, got {other:?}"),
    }

    let request = KernelExecuteRequest::new("1");
    assert!(request.allow_stdin);
}

#[test]
fn connection_file_parser_loads_jupyter_connection_shapes() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "secret",
            "signature_scheme": "hmac-sha256",
            "kernel_name": "rum"
        }"#,
    )
    .expect("connection file should parse");

    assert_eq!(connection.transport, KernelTransportKind::Tcp);
    assert_eq!(connection.ip, "127.0.0.1");
    assert_eq!(connection.shell_port, 57503);
    assert_eq!(
        connection.channel_endpoint(KernelChannel::Shell),
        "tcp://127.0.0.1:57503"
    );
    assert_eq!(connection.heartbeat_endpoint(), "tcp://127.0.0.1:42540");
    assert_eq!(
        connection.signature_scheme,
        KernelSignatureScheme::HmacSha256
    );
    assert_eq!(connection.kernel_name.as_deref(), Some("rum"));
    assert_eq!(
        connection.signature_policy(),
        KernelSignaturePolicy::Required("secret".to_string())
    );
}

#[test]
fn connection_file_parser_reports_missing_required_fields() {
    let error = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1"
        }"#,
    )
    .expect_err("incomplete connection file should fail");

    assert_eq!(
        error,
        KernelConnectionFileError::MissingField {
            field: "shell_port"
        }
    );
}

#[test]
fn bootstrap_config_derives_endpoints_and_wire_policy_from_connection_file() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "secret",
            "signature_scheme": "hmac-sha256"
        }"#,
    )
    .expect("connection file should parse");

    let bootstrap = KernelBootstrapConfig::from_connection_file(connection)
        .with_username("rum-kernel")
        .with_session_validation("frontend-session");
    let layout = bootstrap.transport_layout();
    let wire = bootstrap.wire_config();

    assert_eq!(bootstrap.username, "rum-kernel");
    assert_eq!(layout.shell.uri(), "tcp://127.0.0.1:57503");
    assert_eq!(layout.control.uri(), "tcp://127.0.0.1:50160");
    assert_eq!(layout.stdin.uri(), "tcp://127.0.0.1:52597");
    assert_eq!(layout.iopub.uri(), "tcp://127.0.0.1:40885");
    assert_eq!(layout.heartbeat.uri(), "tcp://127.0.0.1:42540");
    assert_eq!(wire.session_id, Some("frontend-session".to_string()));
    assert_eq!(
        wire.signature_policy,
        KernelSignaturePolicy::Required("secret".to_string())
    );
}

#[test]
fn mock_transport_backend_exposes_bootstrap_boundary_for_future_real_transports() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "",
            "signature_scheme": ""
        }"#,
    )
    .expect("connection file should parse");

    let bootstrap = KernelBootstrapConfig::from_connection_file(connection);
    let backend = MockTransportBackend::new(bootstrap.clone());

    assert_eq!(backend.bootstrap_config(), &bootstrap);
    assert_eq!(
        backend.transport_layout().shell.uri(),
        "tcp://127.0.0.1:57503"
    );
    assert!(backend.transport().shell.is_empty());

    let pump = backend.event_pump(Session::new());
    assert_eq!(pump.codec().config().session_id, None);
    assert_eq!(
        pump.codec().config().signature_policy,
        KernelSignaturePolicy::Disabled
    );
}

#[test]
fn execute_request_failure_emits_stderr_streams_and_error_payload() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := g[x]").unwrap();
    session.eval_text("g[x_Integer] := f[x]").unwrap();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.execute(KernelExecuteRequest::new("f[1]"));

    assert_eq!(response.status, KernelReplyStatus::Error);
    assert_eq!(response.execution_count, Some(1));
    assert_eq!(response.outputs.len(), 3);
    assert_eq!(
        response
            .error
            .as_ref()
            .map(|error| error.error_name.as_str()),
        Some("EvaluationCycleDetected")
    );

    match &response.outputs[0] {
        KernelOutput::Stream(output) => {
            assert_eq!(output.name, KernelStreamName::Stderr);
            assert!(output.text.contains("General::cycle:"));
        }
        other => panic!("expected stderr stream output, got {other:?}"),
    }

    match &response.outputs[1] {
        KernelOutput::DisplayData(output) => {
            assert!(output.data.get("text/plain").is_some());
            assert!(output.data.get("text/markdown").is_some());
            assert!(output.data.get("application/vnd.rum.trace").is_some());
            assert_eq!(output.metadata.get("output_type"), Some("display_data"));
            assert_eq!(output.metadata.get("bundle_kind"), Some("trace"));
            assert!(
                output
                    .data
                    .get("text/plain")
                    .is_some_and(|text| text.contains("rule-applied: downvalue"))
            );
        }
        other => panic!("expected display data output, got {other:?}"),
    }

    match &response.outputs[2] {
        KernelOutput::Error(output) => {
            assert_eq!(output.error_name, "EvaluationCycleDetected");
            assert!(output.error_value.contains("evaluation cycle detected"));
            assert!(output.rendered.is_none());
        }
        other => panic!("expected error output, got {other:?}"),
    }

    match response.outcome {
        KernelExecuteOutcome::Failure(failure) => {
            assert!(!failure.messages.is_empty());
        }
        other => panic!("expected execute failure, got {other:?}"),
    }
}

#[test]
fn inspect_request_maps_symbol_at_cursor_to_session_inspect_response() {
    let mut session = Session::new();
    session.eval_text("f = 1").unwrap();
    let adapter = KernelAdapter::new(session);

    let response = adapter.inspect(KernelInspectRequest::new("f", 1));

    assert_eq!(response.status, KernelReplyStatus::Ok);
    assert_eq!(response.target.as_deref(), Some("f"));
    assert!(response.found);
    assert_eq!(response.metadata.get("bundle_kind"), Some("inspect"));
    assert_eq!(response.metadata.get("found"), Some("true"));
    let data = &response.data;
    assert!(data.get("text/plain").is_some());
    assert!(data.get("text/markdown").is_some());
    assert!(data.get("application/vnd.rum.inspect").is_some());
    let result = response
        .result
        .expect("known symbol should map to an inspect response");
    assert_eq!(result.info.qualified_name, "Global`f");
    assert!(result.info.has_own_value);
}

#[test]
fn complete_request_maps_cursor_prefix_to_session_completion_response() {
    let mut session = Session::new();
    session.eval_text("localName = 1").unwrap();
    let adapter = KernelAdapter::new(session);

    let response = adapter.complete(KernelCompleteRequest::new("loc", 3));

    assert_eq!(response.status, KernelReplyStatus::Ok);
    assert_eq!(response.replacement_start, 0);
    assert_eq!(response.replacement_end, 3);
    assert_eq!(response.prefix, "loc");
    assert_eq!(response.matches, vec!["localName"]);
    assert_eq!(response.metadata.get("cursor_start"), Some("0"));
    assert_eq!(response.metadata.get("cursor_end"), Some("3"));
}

#[test]
fn complete_request_replaces_the_full_symbol_when_cursor_is_inside_it() {
    let mut session = Session::new();
    session.eval_text("localName = 1").unwrap();
    let adapter = KernelAdapter::new(session);

    let response = adapter.complete(KernelCompleteRequest::new("localName", 3));

    assert_eq!(response.replacement_start, 0);
    assert_eq!(response.replacement_end, "localName".len());
    assert_eq!(response.prefix, "loc");
    assert_eq!(response.matches, vec!["localName"]);
}

#[test]
fn execute_request_with_tracing_emits_display_data_separately_from_execute_result() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := x").unwrap();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.execute(KernelExecuteRequest::new("f[1]"));

    assert_eq!(response.outputs.len(), 2);

    match &response.outputs[0] {
        KernelOutput::DisplayData(output) => {
            assert!(
                output
                    .data
                    .get("text/plain")
                    .is_some_and(|text| text.contains("rule-applied: downvalue"))
            );
            assert!(output.data.get("application/vnd.rum.trace").is_some());
            assert_eq!(output.metadata.get("bundle_kind"), Some("trace"));
        }
        other => panic!("expected display data output, got {other:?}"),
    }

    match &response.outputs[1] {
        KernelOutput::ExecuteResult(output) => {
            assert_eq!(output.data.get("text/plain"), Some("1"));
            assert_eq!(output.data.get("text/markdown"), Some("```rum\n1\n```"));
            assert_eq!(output.data.get("application/vnd.rum.expr"), Some("1"));
        }
        other => panic!("expected execute result output, got {other:?}"),
    }
}

#[test]
fn is_complete_request_maps_runtime_completeness_to_kernel_shape() {
    let session = Session::new();
    let adapter = KernelAdapter::new(session);

    let incomplete = adapter.is_complete(KernelIsCompleteRequest::new("f[\n  1"));
    assert_eq!(incomplete.reply_status, KernelReplyStatus::Ok);
    assert_eq!(
        incomplete.completion_status,
        KernelIsCompleteStatus::Incomplete
    );
    assert_eq!(incomplete.indent.as_deref(), Some("    "));

    let invalid = adapter.is_complete(KernelIsCompleteRequest::new("f[1 2]"));
    assert_eq!(invalid.completion_status, KernelIsCompleteStatus::Invalid);
    assert!(invalid.indent.is_none());
}

#[test]
fn dispatch_request_emits_iopub_status_execute_input_and_execute_result() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new("client-msg-20", "session-3", "tester", "execute_request"),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("execute should reply immediately");

    assert_eq!(reply.channel, KernelChannel::Shell);
    assert_eq!(reply.header.msg_type, "execute_reply");
    assert_eq!(reply.metadata.get("status"), Some("ok"));
    assert_eq!(adapter.execution_state(), KernelExecutionState::Idle);
    assert_eq!(dispatch.iopub.len(), 4);

    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }

    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::ExecuteInput(input) => {
            assert_eq!(input.execution_count, Some(1));
            assert_eq!(input.code, "1");
        }
        other => panic!("expected execute_input payload, got {other:?}"),
    }

    match &dispatch.iopub[2].payload {
        KernelIoPubPayload::ExecuteResult(result) => {
            assert_eq!(result.data.get("text/plain"), Some("1"));
        }
        other => panic!("expected execute_result payload, got {other:?}"),
    }

    match &dispatch.iopub[3].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected idle status payload, got {other:?}"),
    }

    assert_eq!(reply.metadata.get("channel"), Some("shell"));
    assert_eq!(
        reply.metadata.get("parent_msg_type"),
        Some("execute_request")
    );
    assert_eq!(dispatch.iopub[0].metadata.get("channel"), Some("iopub"));
    assert_eq!(
        dispatch.iopub[0].metadata.get("parent_msg_type"),
        Some("execute_request")
    );
    assert_eq!(
        dispatch.iopub[0].metadata.get("lifecycle_state"),
        Some("running")
    );
}

#[test]
fn shell_request_envelope_dispatches_kernel_info_and_shutdown() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let info = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-7",
                "session-1",
                "tester",
                "kernel_info_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::KernelInfo(KernelInfoRequest),
        })
        .expect("kernel_info should reply immediately");

    assert_eq!(info.channel, KernelChannel::Shell);
    assert_eq!(info.header.msg_type, "kernel_info_reply");
    assert_eq!(info.header.session_id, "session-1");
    assert_eq!(
        info.parent_header
            .as_ref()
            .map(|header| header.msg_id.as_str()),
        Some("client-msg-7")
    );
    assert_eq!(info.metadata.get("status"), Some("ok"));
    match info.response {
        KernelResponse::KernelInfo(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.implementation, "rum");
            assert_eq!(response.protocol_version, "5.3");
            assert_eq!(response.language_info.name, "rum");
        }
        other => panic!("expected kernel-info response, got {other:?}"),
    }

    let shutdown = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-8",
                "session-1",
                "tester",
                "shutdown_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Shutdown(KernelShutdownRequest { restart: false }),
        })
        .expect("shutdown should reply immediately");

    assert_eq!(shutdown.channel, KernelChannel::Shell);
    assert_eq!(shutdown.header.msg_type, "shutdown_reply");
    assert!(adapter.shutdown_requested());
    match shutdown.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(!response.restart);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown response, got {other:?}"),
    }
}

#[test]
fn shell_request_envelope_dispatches_execute_requests() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-12",
                "session-2",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
        })
        .expect("simple execute should reply immediately");

    assert_eq!(response.channel, KernelChannel::Shell);
    assert_eq!(response.header.msg_type, "execute_reply");
    assert_eq!(
        response
            .parent_header
            .as_ref()
            .map(|header| header.msg_id.as_str()),
        Some("client-msg-12")
    );
    assert_eq!(response.metadata.get("status"), Some("ok"));
    match response.response {
        KernelResponse::Execute(execute) => {
            assert_eq!(execute.execution_count, Some(1));
            assert_eq!(execute.outputs.len(), 1);
        }
        other => panic!("expected execute response, got {other:?}"),
    }
}

#[test]
fn control_request_interrupt_updates_lifecycle_state() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Control,
        header: KernelMessageHeader::new(
            "client-msg-30",
            "session-4",
            "tester",
            "interrupt_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Interrupt(KernelInterruptRequest),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("interrupt should reply immediately");

    assert_eq!(reply.channel, KernelChannel::Control);
    assert!(dispatch.iopub.is_empty());
    assert_eq!(reply.header.msg_type, "interrupt_reply");
    assert_eq!(
        reply.metadata.get("lifecycle_state"),
        Some("interrupt_requested")
    );
    assert_eq!(
        adapter.lifecycle_state(),
        KernelLifecycleState::InterruptRequested
    );
    assert!(adapter.interrupt_requested());

    match &reply.response {
        KernelResponse::Interrupt(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.interrupt_requested);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected interrupt response, got {other:?}"),
    }
}

#[test]
fn control_request_shutdown_transitions_kernel_to_shutdown() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Control,
        header: KernelMessageHeader::new(
            "client-msg-31",
            "session-4",
            "tester",
            "shutdown_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Shutdown(KernelShutdownRequest { restart: false }),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("shutdown should reply immediately");

    assert_eq!(reply.channel, KernelChannel::Control);
    assert_eq!(reply.header.msg_type, "shutdown_reply");
    assert_eq!(
        reply.metadata.get("lifecycle_state"),
        Some("shutdown_requested")
    );
    assert_eq!(adapter.lifecycle_state(), KernelLifecycleState::Shutdown);
    assert!(adapter.shutdown_requested());
}

#[test]
fn shell_request_envelope_dispatches_is_complete_requests() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-40",
                "session-5",
                "tester",
                "is_complete_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::IsComplete(KernelIsCompleteRequest::new("f[\n  1")),
        })
        .expect("is_complete should reply immediately");

    assert_eq!(response.header.msg_type, "is_complete_reply");
    assert_eq!(response.metadata.get("status"), Some("ok"));
    assert_eq!(
        response.metadata.get("completion_status"),
        Some("incomplete")
    );

    match response.response {
        KernelResponse::IsComplete(result) => {
            assert_eq!(result.completion_status, KernelIsCompleteStatus::Incomplete);
            assert_eq!(result.indent.as_deref(), Some("    "));
        }
        other => panic!("expected is-complete response, got {other:?}"),
    }
}

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
fn event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-13")
            .with_signature_key("secret"),
    );
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let mut request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-66",
                "session-13",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
        })
        .expect("request should encode");
    request.routing_ids = vec![b"frontend".to_vec(), b"shell".to_vec()];
    transport.queue_incoming(request);

    assert!(
        pump.pump_once(&mut transport, KernelChannel::Shell)
            .expect("pump should succeed")
    );
    assert_eq!(transport.shell.len(), 1);
    assert_eq!(transport.iopub.len(), 4);
    assert_eq!(
        transport.shell[0].routing_ids,
        vec![b"frontend".to_vec(), b"shell".to_vec()]
    );
    assert!(
        transport
            .iopub
            .iter()
            .all(|message| message.routing_ids.is_empty())
    );
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
fn adapter_can_stage_stdin_requests_for_future_execute_integration() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);
    let parent_header =
        KernelMessageHeader::new("client-msg-67b", "session-14b", "tester", "execute_request");
    let execute = KernelExecuteRequest::new("futureInput[]");

    let request = adapter
        .request_stdin_for_execute(&parent_header, &execute, KernelInputRequest::new("name: "))
        .expect("stdin request should be allowed");

    assert!(adapter.awaiting_stdin());
    assert_eq!(request.channel, KernelChannel::Stdin);
    assert_eq!(request.header.msg_type, "input_request");
    assert_eq!(request.request.prompt, "name: ");
    assert!(!request.request.password);
    assert_eq!(request.metadata.get("allow_stdin"), Some("true"));
    assert_eq!(request.metadata.get("channel"), Some("stdin"));
}

#[test]
fn adapter_rejects_stdin_requests_when_execute_disallows_them() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);
    let parent_header =
        KernelMessageHeader::new("client-msg-67c", "session-14c", "tester", "execute_request");
    let execute = KernelExecuteRequest::new("futureInput[]").without_stdin();

    let error = adapter
        .request_stdin_for_execute(&parent_header, &execute, KernelInputRequest::new("name: "))
        .expect_err("stdin request should be rejected");
    assert_eq!(error, super::KernelStdinError::StdinNotAllowed);
    assert!(!adapter.awaiting_stdin());
}

#[test]
fn execute_request_with_inputstring_stages_stdin_and_defers_execute_reply() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-1",
            "session-input-1",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[\"name: \"]")),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);
    assert_eq!(dispatch.stdin[0].request.prompt, "name: ");
    assert_eq!(dispatch.iopub.len(), 2);
    assert!(adapter.awaiting_stdin());
    assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);

    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }

    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::ExecuteInput(input) => {
            assert_eq!(input.code, "InputString[\"name: \"]");
        }
        other => panic!("expected execute_input payload, got {other:?}"),
    }
}

#[test]
fn execute_request_with_disallowed_stdin_returns_input_error_reply() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-2",
            "session-input-2",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[]").without_stdin()),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("disallowed stdin should fail immediately");

    assert!(dispatch.stdin.is_empty());
    assert_eq!(dispatch.iopub.len(), 4);
    assert_eq!(reply.metadata.get("status"), Some("error"));

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Error);
            assert!(matches!(
                response.outcome,
                KernelExecuteOutcome::Failure(SessionRequestFailure {
                    error: rum_runtime::SessionRequestError::InputRequired { .. },
                    ..
                })
            ));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}

#[test]
fn transport_codec_round_trips_stdin_input_messages() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-14d")
            .with_signature_key("secret"),
    );
    let input_request = super::KernelInputRequestEnvelope {
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

#[test]
fn pump_next_prioritizes_control_messages_before_shell_messages() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-68",
                    "session-15",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-69",
                    "session-15",
                    "tester",
                    "interrupt_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Interrupt(KernelInterruptRequest),
            })
            .expect("control request should encode"),
    );

    let channel = pump
        .pump_next(&mut transport)
        .expect("pump should succeed")
        .expect("one request should be processed");

    assert_eq!(channel, KernelChannel::Control);
    assert_eq!(transport.control.len(), 1);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
}

#[test]
fn pump_next_prioritizes_stdin_replies_while_kernel_is_waiting_for_input() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15b"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let parent_header = KernelMessageHeader::new(
        "client-msg-stdin-parent",
        "session-15b",
        "tester",
        "execute_request",
    );
    let pending = pump
        .adapter_mut()
        .request_stdin_for_execute(
            &parent_header,
            &KernelExecuteRequest::new("futureInput[]"),
            KernelInputRequest::new("name: "),
        )
        .expect("stdin request should stage");

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-shell",
                    "session-15b",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(encode_input_reply_envelope(&KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new(
            "client-msg-stdin-reply",
            "session-15b",
            "tester",
            "input_reply",
        ),
        parent_header: Some(pending.header.clone()),
        metadata: super::KernelMetadata::new(),
        reply: KernelInputReply::new("Alice"),
    }));

    let channel = pump
        .pump_next(&mut transport)
        .expect("pump should succeed")
        .expect("one request should be processed");

    assert_eq!(channel, KernelChannel::Stdin);
    assert!(!pump.adapter().awaiting_stdin());
    assert_eq!(
        pump.adapter_mut()
            .take_last_input_reply()
            .map(|reply| reply.value),
        Some("Alice".to_string())
    );
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
}

#[test]
fn input_reply_resumes_execute_and_preserves_shell_routing_ids() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15c"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let mut request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-input-3",
                "session-15c",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[\"name: \"]")),
        })
        .expect("execute request should encode");
    request.routing_ids = vec![b"frontend".to_vec(), b"shell".to_vec()];
    transport.queue_incoming(request);

    let first_channel = pump
        .pump_next(&mut transport)
        .expect("first pump should succeed")
        .expect("one request should be processed");
    assert_eq!(first_channel, KernelChannel::Shell);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.stdin.len(), 1);
    assert_eq!(transport.iopub.len(), 2);

    let input_request =
        decode_input_request_message(&transport.stdin[0]).expect("stdin request should decode");
    transport.queue_incoming(encode_input_reply_envelope(&KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new(
            "client-msg-input-reply",
            "session-15c",
            "tester",
            "input_reply",
        ),
        parent_header: Some(input_request.header.clone()),
        metadata: super::KernelMetadata::new(),
        reply: KernelInputReply::new("Alice"),
    }));

    let second_channel = pump
        .pump_next(&mut transport)
        .expect("second pump should succeed")
        .expect("one request should be processed");
    assert_eq!(second_channel, KernelChannel::Stdin);
    assert_eq!(
        pump.adapter_mut()
            .take_last_input_reply()
            .map(|reply| reply.value),
        Some("Alice".to_string())
    );

    assert_eq!(transport.shell.len(), 1);
    assert_eq!(
        transport.shell[0].routing_ids,
        vec![b"frontend".to_vec(), b"shell".to_vec()]
    );
    assert_eq!(transport.iopub.len(), 4);

    let reply = decode_response_message(&transport.shell[0]).expect("reply should decode");
    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let execute_result =
        decode_iopub_message(&transport.iopub[2]).expect("execute_result should decode");
    match execute_result.payload {
        KernelIoPubPayload::ExecuteResult(result) => {
            assert_eq!(result.data.get("text/plain"), Some("\"Alice\""));
        }
        other => panic!("expected execute_result payload, got {other:?}"),
    }

    let idle_status = decode_iopub_message(&transport.iopub[3]).expect("idle status should decode");
    match idle_status.payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
        }
        other => panic!("expected status payload, got {other:?}"),
    }
}

#[test]
fn input_reply_resumes_execute_without_rerunning_prefix_side_effects() {
    let mut session = Session::new();
    session.eval_text("x = seed").unwrap();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-4",
            "session-input-4",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new(
            "List[x = List[x], InputString[\"name: \"], x = List[x]]",
        )),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);

    let resumed = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-2",
                "session-input-4",
                "tester",
                "input_reply",
            ),
            parent_header: Some(dispatch.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("Alice"),
        })
        .expect("stdin reply should be accepted")
        .expect("stdin reply should resume the suspended execute");

    let reply = resumed
        .reply
        .as_ref()
        .expect("resumed execute should now have a reply");

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert_eq!(response.outputs.len(), 1);
            match &response.outputs[0] {
                KernelOutput::ExecuteResult(result) => {
                    assert_eq!(
                        result.data.get("text/plain"),
                        Some("List[List[seed], \"Alice\", List[List[seed]]]")
                    );
                }
                other => panic!("expected execute result output, got {other:?}"),
            }
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    assert_eq!(
        adapter
            .session_mut()
            .eval_request("x")
            .unwrap()
            .value
            .to_string(),
        "List[List[seed]]"
    );
}

#[test]
fn input_reply_resumes_delayed_message_templates_through_internal_string_driver() {
    let mut session = Session::new();
    session
        .eval_text("f::argx := InputString[\"msg: \"]")
        .unwrap();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-5",
            "session-input-5",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("Message[f::argx]")),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);
    assert_eq!(dispatch.stdin[0].request.prompt, "msg: ");

    let resumed = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-3",
                "session-input-5",
                "tester",
                "input_reply",
            ),
            parent_header: Some(dispatch.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("Alice"),
        })
        .expect("stdin reply should be accepted")
        .expect("stdin reply should resume the suspended execute");

    let reply = resumed
        .reply
        .as_ref()
        .expect("resumed execute should now have a reply");

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert!(response.outputs.iter().any(|output| matches!(
                output,
                KernelOutput::Stream(stream)
                    if stream.text.contains("f::argx: Alice")
            )));
            assert!(response.outputs.iter().any(|output| matches!(
                output,
                KernelOutput::ExecuteResult(result)
                    if result.data.get("text/plain") == Some("Null")
            )));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}

#[test]
fn pump_until_idle_stops_after_shutdown_and_leaves_later_shell_work_queued() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-16"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-70",
                    "session-16",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-71",
                    "session-16",
                    "tester",
                    "shutdown_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Shutdown(KernelShutdownRequest { restart: true }),
            })
            .expect("shutdown request should encode"),
    );

    let report = pump
        .pump_until_idle(&mut transport)
        .expect("loop should succeed");

    assert_eq!(report.control_messages, 1);
    assert_eq!(report.shell_messages, 0);
    assert_eq!(report.total_messages(), 1);
    assert!(report.stopped_for_shutdown);
    assert_eq!(transport.control.len(), 1);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
    assert!(pump.adapter().shutdown_requested());

    let reply = decode_response_message(&transport.control[0]).expect("reply should decode");
    match reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.restart);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }
}
