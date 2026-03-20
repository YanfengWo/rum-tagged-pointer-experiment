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
    assert!(
        incomplete
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| diagnostic.is_incomplete)
    );

    let invalid = session.is_complete_request("f[1 2]");
    assert_eq!(invalid.status, SessionCompletenessStatus::Invalid);
    assert!(invalid.indent.is_none());
    assert!(
        invalid
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| !diagnostic.is_incomplete)
    );
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
    assert!(
        failure
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "cycle")
    );
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

#[test]
fn queued_input_responses_feed_inputstring_without_prompting_again() {
    let mut session = Session::new();
    session.queue_input_response("Alice");

    let response = session
        .eval_request("InputString[\"name: \"]")
        .expect("queued host input should satisfy InputString");

    assert_eq!(response.value.to_string(), "\"Alice\"");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments() {
    let mut session = Session::new();
    session.eval_text("x = seed").unwrap();

    let failure = session
        .eval_request("List[x = List[x], InputString[\"name: \"], x = List[x]]")
        .expect_err("InputString should suspend the request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue from the suspended argument list");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed], \"Alice\", List[List[seed]]]"
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[seed]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_can_suspend_multiple_times_without_restarting() {
    let mut session = Session::new();
    session.eval_text("x = seed").unwrap();

    let first = session
        .eval_request(
            "List[x = List[x], InputString[\"first: \"], x = List[x], InputString[\"second: \"], x = List[x]]",
        )
        .expect_err("first InputString should suspend the request");

    match &first.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "first: ");
        }
        other => panic!("expected first input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("Alice")
        .expect_err("second InputString should suspend the resumed request");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Bob")
        .expect("second resume should finish the original request");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed], \"Alice\", List[List[seed]], \"Bob\", List[List[List[seed]]]]"
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[List[seed]]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend() {
    let mut session = Session::new();
    session
        .eval_text("name := InputString[\"name: \"]")
        .unwrap();
    session.eval_text("g[\"Alice\"] := done").unwrap();

    let failure = session
        .eval_request("g[name]")
        .expect_err("symbol evaluation should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue the suspended fixed-point evaluation");

    assert_eq!(response.value.to_string(), "done");
}

#[test]
fn resume_eval_request_with_input_finishes_with_bindings_without_reprompting() {
    let mut session = Session::new();

    let failure = session
        .eval_request("With[{x = InputString[\"name: \"]}, List[x, x]]")
        .expect_err("With binding should suspend when its initializer requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should reuse the suspended With binding");

    assert_eq!(response.value.to_string(), "List[\"Alice\", \"Alice\"]");
}

#[test]
fn resume_eval_request_with_input_preserves_block_scope_until_completion() {
    let mut session = Session::new();
    session.eval_text("x = global").unwrap();

    let failure = session
        .eval_request("Block[{x = local}, List[x, InputString[\"name: \"], x = List[x], x]]")
        .expect_err("Block body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue inside the active Block scope");

    assert_eq!(
        response.value.to_string(),
        "List[local, \"Alice\", List[local], List[local]]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "global");
}

#[test]
fn resume_eval_request_with_input_can_finish_delayed_message_templates() {
    let mut session = Session::new();
    session
        .eval_text("f::argx := InputString[\"msg: \"]")
        .unwrap();

    let failure = session
        .eval_request("Message[f::argx]")
        .expect_err("delayed message template should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish delayed message rendering");

    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "Alice"
    }));
}

#[test]
fn resume_eval_request_with_input_continues_replace_all_without_restarting_target() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplaceAll[InputString[\"name: \"], x_String :> List[x, x]]")
        .expect_err("ReplaceAll target should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue ReplaceAll from the suspended target");

    assert_eq!(response.value.to_string(), "List[\"Alice\", \"Alice\"]");
}
