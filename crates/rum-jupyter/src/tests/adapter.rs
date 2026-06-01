use super::*;

mod dispatch;

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
