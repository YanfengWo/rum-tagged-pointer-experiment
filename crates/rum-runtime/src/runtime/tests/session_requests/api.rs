use super::*;

#[test]
fn parse_request_returns_structured_syntax_diagnostics() {
    let session = Session::new();
    let failure = session
        .parse_request("f[1 2]")
        .expect_err("malformed input should fail with a syntax diagnostic");

    match failure.error {
        SessionRequestError::Syntax(diagnostic) => {
            assert!(diagnostic.message.starts_with("expected ,"));
            assert!(!diagnostic.is_incomplete);
            assert!(diagnostic.rendered.contains("--> 1:5"));
            assert_eq!(diagnostic.spans.len(), 1);
            assert_eq!(diagnostic.spans[0].start.line, 1);
            assert_eq!(diagnostic.spans[0].start.column, 5);
            assert_eq!(diagnostic.spans[0].text, "2");
        }
        other => panic!("expected syntax diagnostic, got {other:?}"),
    }

    assert!(failure.messages.is_empty());
    assert!(failure.trace.is_empty());
}

#[test]
fn complete_request_returns_transport_friendly_matches() {
    let mut session = Session::new();
    session.eval_text("localName = 1").unwrap();

    let response = session.complete_request("loc");

    assert_eq!(response.prefix, "loc");
    assert_eq!(response.matches, vec!["localName"]);
}

#[test]
fn is_complete_request_distinguishes_complete_incomplete_and_invalid_input() {
    let session = Session::new();

    let complete = session.is_complete_request("f[1]");
    assert_eq!(complete.status, SessionCompletenessStatus::Complete);
    assert!(complete.indent.is_none());
    assert!(complete.diagnostic.is_none());

    let incomplete = session.is_complete_request("f[\n  1");
    assert_eq!(incomplete.status, SessionCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));
    assert!(incomplete
        .diagnostic
        .as_ref()
        .is_some_and(|diagnostic| diagnostic.is_incomplete));

    let invalid = session.is_complete_request("f[1 2]");
    assert_eq!(invalid.status, SessionCompletenessStatus::Invalid);
    assert!(invalid.indent.is_none());
    assert!(invalid
        .diagnostic
        .as_ref()
        .is_some_and(|diagnostic| !diagnostic.is_incomplete));
}

#[test]
fn eval_request_returns_transport_friendly_success_data() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := x").unwrap();

    let response = session.eval_request("f[1]").unwrap();

    assert_eq!(response.value.to_string(), "1");
    assert!(response.messages.is_empty());
    assert!(response.trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input == "f[1]"
    )));
}

#[test]
fn eval_request_returns_transport_friendly_failure_data() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := g[x]").unwrap();
    session.eval_text("g[x_Integer] := f[x]").unwrap();

    let failure = session
        .eval_request("f[1]")
        .expect_err("recursive evaluation should surface a request failure");

    assert!(matches!(
        failure.error,
        SessionRequestError::EvaluationCycleDetected { ref expr } if expr == "f[1]"
    ));
    assert!(failure.trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input == "f[1]" || input == "g[1]"
    )));
    assert!(failure
        .messages
        .iter()
        .any(|message| message.symbol == "General" && message.tag == "cycle"));
}

#[test]
fn inspect_request_aggregates_symbol_definitions_and_messages() {
    let mut session = Session::new();
    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f::usage = \"desc\"").unwrap();

    let response = session
        .inspect_request("f")
        .expect("known symbol should produce an inspect response");

    assert_eq!(response.info.qualified_name, "Global`f");
    assert!(response.info.has_down_values);
    assert_eq!(response.definitions.len(), 1);
    assert_eq!(response.message_catalog.len(), 1);
    assert_eq!(response.message_catalog[0].message_name, "f::usage");
}

#[test]
fn eval_request_surfaces_structured_input_requests() {
    let mut session = Session::new();

    let failure = session
        .eval_request("InputString[\"name: \"]")
        .expect_err("missing host input should become a structured request failure");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
            assert!(!request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    assert_eq!(
        session.pending_input_request(),
        Some(&RuntimeInputRequest::new("name: "))
    );
}
