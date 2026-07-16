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
            assert_eq!(result.value.text, "1");
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
fn execute_literal_cancelled_termination_value_remains_successful() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.execute(KernelExecuteRequest::new(
        "TerminatedEvaluation[\"Cancelled\"]",
    ));

    assert_eq!(response.status, KernelReplyStatus::Ok);
    assert!(response.error.is_none());
    match response.outcome {
        KernelExecuteOutcome::Success(result) => {
            assert_eq!(result.value.text, "TerminatedEvaluation[\"Cancelled\"]");
            assert_eq!(result.termination, None);
        }
        other => panic!("expected literal value to remain a success, got {other:?}"),
    }
}

#[test]
fn execute_request_cycle_termination_emits_stderr_trace_and_result() {
    let mut session = tracing_session();
    eval_text(&mut session, "f[x_Integer] := g[x]");
    eval_text(&mut session, "g[x_Integer] := f[x]");
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.execute(KernelExecuteRequest::new("f[1]"));

    assert_eq!(response.status, KernelReplyStatus::Ok);
    assert_eq!(response.execution_count, Some(1));
    assert_eq!(response.outputs.len(), 3);
    assert!(response.error.is_none());

    match &response.outputs[0] {
        KernelOutput::Stream(output) => {
            assert_eq!(output.name, KernelStreamName::Stderr);
            assert!(output.text.contains("General::passlim:"));
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
        KernelOutput::ExecuteResult(output) => {
            assert_eq!(
                output.data.get("text/plain"),
                Some("TerminatedEvaluation[\"IterationLimit\"]")
            );
            assert_eq!(
                output.data.get("application/vnd.rum.expr"),
                Some("TerminatedEvaluation[\"IterationLimit\"]")
            );
        }
        other => panic!("expected execute result output, got {other:?}"),
    }

    match response.outcome {
        KernelExecuteOutcome::Success(result) => {
            assert_eq!(
                result.value.text,
                "TerminatedEvaluation[\"IterationLimit\"]"
            );
            assert!(!result.messages.is_empty());
        }
        other => panic!("expected execute success, got {other:?}"),
    }
}

#[test]
fn inspect_request_maps_symbol_at_cursor_to_session_inspect_response() {
    let mut session = Session::new();
    eval_text(&mut session, "f = 1");
    let mut adapter = KernelAdapter::new(session);

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
    let info = result
        .info
        .expect("known symbol should include symbol info");
    assert_eq!(info.qualified_name, "Global`f");
    assert!(info.has_own_value);
}

#[test]
fn complete_request_maps_cursor_prefix_to_session_completion_response() {
    let mut session = Session::new();
    eval_text(&mut session, "localName = 1");
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.complete(KernelCompleteRequest::new("loc", 3));

    assert_eq!(response.status, KernelReplyStatus::Ok);
    assert_eq!(response.replacement_start, 0);
    assert_eq!(response.replacement_end, 3);
    assert_eq!(response.prefix, "loc");
    assert!(response.matches.iter().any(|item| item == "localName"));
    assert_eq!(response.metadata.get("cursor_start"), Some("0"));
    assert_eq!(response.metadata.get("cursor_end"), Some("3"));
}

#[test]
fn complete_request_replaces_the_full_symbol_when_cursor_is_inside_it() {
    let mut session = Session::new();
    eval_text(&mut session, "localName = 1");
    let mut adapter = KernelAdapter::new(session);

    let response = adapter.complete(KernelCompleteRequest::new("localName", 3));

    assert_eq!(response.replacement_start, 0);
    assert_eq!(response.replacement_end, "localName".len());
    assert_eq!(response.prefix, "loc");
    assert!(response.matches.iter().any(|item| item == "localName"));
}

#[test]
fn execute_request_with_tracing_emits_display_data_separately_from_execute_result() {
    let mut session = tracing_session();
    eval_text(&mut session, "f[x_Integer] := x");
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
    let mut adapter = KernelAdapter::new(session);

    let incomplete = adapter.is_complete(KernelIsCompleteRequest::new("f[\n  1"));
    assert_eq!(incomplete.reply_status, KernelReplyStatus::Ok);
    assert_eq!(
        incomplete.completion_status,
        KernelIsCompleteStatus::Incomplete
    );
    assert_eq!(incomplete.indent.as_deref(), Some("    "));

    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let invalid = adapter.is_complete(KernelIsCompleteRequest::new("f[1,]"));
    assert_eq!(invalid.completion_status, KernelIsCompleteStatus::Invalid);
    assert!(invalid.indent.is_none());
}
