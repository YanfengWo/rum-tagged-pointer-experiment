use super::*;

#[test]
fn host_eval_reports_value_messages_prints_and_structured_outputs() {
    let mut session = HostSession::new();

    let outcome = session.eval("Print[1]; Message[General::stop]; 2");

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "2");
            assert_eq!(success.prints, vec!["1"]);
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "stop")
            );
            assert!(
                success
                    .outputs
                    .iter()
                    .any(|output| output.kind == HostStructuredOutputKind::Result)
            );
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn host_suspend_resume_flow_wraps_input_requests() {
    let mut session = HostSession::new();

    let suspended = session.eval("StringJoin[InputString[\"name: \"], \"!\"]");
    match suspended {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, "name: ");
            assert!(!suspension.request.password);
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    let resumed = session.resume_with_input("rum");
    match resumed {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"rum!\""),
        other => panic!("expected resumed success, got {other:?}"),
    }
}

#[test]
fn host_parse_inspect_complete_and_is_complete_cover_editor_flows() {
    let mut session = HostSession::new();

    let parsed = session.parse("1 + 2").expect("parse should succeed");
    assert_eq!(parsed.syntax, "1 + 2");

    assert!(matches!(
        session.eval("localName = 1"),
        HostEvalOutcome::Success(_)
    ));

    let inspect = session.inspect(HostInspectRequest::new("localName", 3));
    assert!(inspect.found);
    assert_eq!(
        inspect
            .info
            .as_ref()
            .map(|info| info.qualified_name.as_str()),
        Some("Global`localName")
    );

    let completion = session.complete(HostCompletionRequest::new("localName", 3));
    assert_eq!(completion.replacement_start, 0);
    assert_eq!(completion.replacement_end, "localName".len());
    assert_eq!(completion.prefix, "loc");
    assert_eq!(completion.matches, vec!["localName"]);

    let incomplete = session.is_complete("f[\n  1");
    assert_eq!(incomplete.status, HostCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));
}

#[test]
fn host_failures_use_stable_failure_codes() {
    let mut session = HostSession::new();

    // `1 2` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let outcome = session.eval("f[1,]");

    match outcome {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Syntax);
            assert!(!failure.message.is_empty());
        }
        other => panic!("expected syntax failure, got {other:?}"),
    }
}
