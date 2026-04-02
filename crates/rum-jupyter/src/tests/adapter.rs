use super::*;

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
fn dispatch_request_emits_busy_and_idle_status_for_kernel_info() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-kinfo-1",
            "session-kinfo",
            "tester",
            "kernel_info_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::KernelInfo(KernelInfoRequest),
    });

    assert_eq!(dispatch.iopub.len(), 2);
    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }
    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected idle status payload, got {other:?}"),
    }
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
    assert_eq!(dispatch.iopub.len(), 2);
    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(
                status.lifecycle_state,
                KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }
    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
            assert_eq!(
                status.lifecycle_state,
                KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected idle status payload, got {other:?}"),
    }
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
