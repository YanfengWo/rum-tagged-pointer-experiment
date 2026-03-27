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
fn resume_eval_request_with_input_formats_delayed_message_templates_after_resume() {
    let mut session = Session::new();
    session
        .eval_text(
            "f::argx := If[MatchQ[InputString[\"msg: \"], \"Alice\"], \"bad arg `1` from Alice\", \"bad arg `1` from stranger\"]",
        )
        .unwrap();

    let failure = session
        .eval_request("Message[f::argx, 42]")
        .expect_err("delayed formatted message template should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish delayed message formatting after the template resolves");

    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "bad arg 42 from Alice"
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

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request("MatchQ[1, x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]]")
        .expect_err("matcher condition should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned MatchQ");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_sequence_conditioned_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request("MatchQ[f[1, 2], f[x__ /; InputString[\"cond: \"], y_]]")
        .expect_err("sequence-pattern condition should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish sequence-pattern MatchQ");

    assert_eq!(response.value.to_string(), "False");
}

#[test]
fn resume_eval_request_with_input_can_finish_nested_matcher_inside_sequence_conditioned_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request("MatchQ[f[1, 2], f[x__ /; MatchQ[InputString[\"cond: \"], \"yes\"], y_]]")
        .expect_err("nested matcher inside a sequence-pattern condition should suspend");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish sequence-pattern matching with the nested matcher");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_multi_sequence_backtracking_conditions() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "MatchQ[f[1, 2, 3], f[x__ /; MatchQ[InputString[\"first: \"], \"yes\"], y__ /; MatchQ[InputString[\"second: \"], \"ok\"]]]",
        )
        .expect_err("multi-sequence matcher conditions should suspend on the first nested matcher");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "first: ");
        }
        other => panic!("expected first input-required failure, got {other:?}"),
    }

    let second = session.resume_eval_request_with_input("yes").expect_err(
        "after the first sequence condition resumes, the second should still be able to suspend",
    );

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let third = session
        .resume_eval_request_with_input("ok")
        .expect_err("backtracking into a new sequence split should be able to suspend again");

    match &third.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected repeated second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resume should finish the original multi-sequence MatchQ after backtracking");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_downvalue_matching() {
    let mut session = Session::new();
    session
        .eval_text("f[x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]] := x")
        .unwrap();

    let failure = session
        .eval_request("f[1]")
        .expect_err("conditioned downvalue match should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned downvalue matching");

    assert_eq!(response.value.to_string(), "1");
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_replace_all() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplaceAll[1, x_ /; MatchQ[InputString[\"cond: \"], \"yes\"] :> List[x, x]]")
        .expect_err("conditioned ReplaceAll rule should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned ReplaceAll");

    assert_eq!(response.value.to_string(), "List[1, 1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_cases() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Cases[f[1], x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]]")
        .expect_err("conditioned Cases pattern should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned Cases");

    assert_eq!(response.value.to_string(), "List[1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_symbol_spec() {
    let mut session = Session::new();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session.eval_request("Cases[f[1], p]").expect_err(
        "Cases spec evaluation should suspend when the referenced symbol requests input",
    );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the spec symbol resolves");

    assert_eq!(response.value.to_string(), "List[1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_level_spec() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Cases[f[1], _Integer, If[MatchQ[InputString[\"lvl: \"], \"yes\"], 1, 0]]")
        .expect_err("Cases level-spec evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lvl: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the level spec resolves");

    assert_eq!(response.value.to_string(), "List[1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_third_arg() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Cases[f[g[1]], x_Symbol, If[MatchQ[InputString[\"heads: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("Cases third-argument traversal config should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the third argument resolves");

    assert_eq!(response.value.to_string(), "List[f]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fourth_arg() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Cases[f[1, g[2]], x_Symbol, 2, If[MatchQ[InputString[\"heads: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("Cases fourth-argument traversal config should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the fourth argument resolves");

    assert_eq!(response.value.to_string(), "List[f, g]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_match_limit() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Cases[f[g[1], 2, h[3]], _Integer, Infinity, If[MatchQ[InputString[\"n: \"], \"yes\"], 2, 0]]",
        )
        .expect_err("Cases match-limit evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "n: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the match limit resolves");

    assert_eq!(response.value.to_string(), "List[1, 2]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fifth_arg() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Cases[f[1, g[2]], x_Symbol, {2}, 1, If[MatchQ[InputString[\"heads: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("Cases fifth-argument traversal config should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the fifth argument resolves");

    assert_eq!(response.value.to_string(), "List[g]");
}

#[test]
fn resume_eval_request_with_input_can_finish_position_with_suspending_symbol_spec() {
    let mut session = Session::new();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session.eval_request("Position[f[1], p]").expect_err(
        "Position pattern evaluation should suspend when the referenced symbol requests input",
    );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Position after the pattern resolves");

    assert_eq!(response.value.to_string(), "List[List[1]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_position_with_suspending_match_limit() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Position[f[g[1], 2, h[3]], _Integer, Infinity, If[MatchQ[InputString[\"n: \"], \"yes\"], 2, 0]]",
        )
        .expect_err("Position match-limit evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "n: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Position after the match limit resolves");

    assert_eq!(response.value.to_string(), "List[List[1, 1], List[2]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_count_with_suspending_symbol_spec() {
    let mut session = Session::new();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session
        .eval_request("Count[f[g[1], 2], p, Infinity]")
        .expect_err("Count pattern evaluation should suspend when the referenced symbol requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Count after the pattern resolves");

    assert_eq!(response.value.to_string(), "2");
}

#[test]
fn resume_eval_request_with_input_can_finish_freeq_with_suspending_heads_option() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "FreeQ[f[1], x_Symbol, If[MatchQ[InputString[\"heads: \"], \"yes\"], {0, Infinity}, {1}], If[MatchQ[InputString[\"heads2: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("FreeQ traversal option evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second_failure = session
        .resume_eval_request_with_input("yes")
        .expect_err("FreeQ should continue until the heads option also resolves");

    match &second_failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads2: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue FreeQ after traversal options resolve");

    assert_eq!(response.value.to_string(), "False");
}

#[test]
fn logic_short_circuit_can_avoid_or_resume_input_requests() {
    let mut session = Session::new();

    let response = session
        .eval_request("And[False, InputString[\"rhs: \"]]")
        .expect("And should short-circuit before requesting input");
    assert_eq!(response.value.to_string(), "False");
    assert!(session.pending_input_request().is_none());

    let failure = session
        .eval_request("And[InputString[\"lhs: \"], False]")
        .expect_err("leading InputString should still suspend the request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("text")
        .expect("resume should continue And evaluation after the suspended input");

    assert_eq!(response.value.to_string(), "False");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn truth_aggregates_short_circuit_and_resume_input_requests() {
    let mut session = Session::new();

    let response = session
        .eval_request(
            "AnyTrue[{1, 2}, Function[x, If[SameQ[x, 1], True, MatchQ[InputString[\"rhs: \"], \"yes\"]]]]",
        )
        .expect("AnyTrue should short-circuit before requesting later inputs");
    assert_eq!(response.value.to_string(), "True");
    assert!(session.pending_input_request().is_none());

    let failure = session
        .eval_request(
            "AllTrue[{1, 2}, Function[x, If[SameQ[x, 1], True, MatchQ[InputString[\"rhs: \"], \"yes\"]]]]",
        )
        .expect_err("AllTrue should suspend when a later criterion application requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish AllTrue after the suspended criterion resolves");

    assert_eq!(response.value.to_string(), "True");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_can_finish_replacepart_with_suspending_delayed_rhs() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplacePart[f[1, 2], 2 :> InputString[\"rhs: \"]]")
        .expect_err("ReplacePart delayed rhs should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue ReplacePart after the delayed rhs resolves");

    assert_eq!(response.value.to_string(), "f[1, \"done\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_replacepart_after_suspending_pattern_position_spec() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "ReplacePart[f[a], Condition[List[Pattern[i, Blank[]]], MatchQ[InputString[\"pos: \"], \"yes\"]] -> x]",
        )
        .expect_err("ReplacePart should suspend when a pattern position condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "pos: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue ReplacePart after the pattern position resolves");

    assert_eq!(response.value.to_string(), "f[x]");
}

#[test]
fn resume_eval_request_with_input_can_finish_replacepart_pattern_bindings_in_delayed_rhs() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "ReplacePart[f[g[a]], {1, i_} :> StringJoin[InputString[\"rhs: \"], ToString[i]]]",
        )
        .expect_err("ReplacePart should suspend when a delayed rhs with pattern bindings requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue ReplacePart after the delayed rhs resolves");

    assert_eq!(response.value.to_string(), "f[g[\"done1\"]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_normalize_rules_after_suspending_rule_source() {
    let mut session = Session::new();
    session
        .eval_text(
            "rules := If[MatchQ[InputString[\"rules: \"], \"yes\"], x_Integer -> 1, y_String :> y]",
        )
        .unwrap();

    let failure = session
        .eval_request("NormalizeRules[rules]")
        .expect_err("NormalizeRules should suspend when its rule source requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rules: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue NormalizeRules after the rule source resolves");

    assert_eq!(
        response.value.to_string(),
        "List[Rule[Pattern[x, Blank[Integer]], 1]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_replaceall_after_suspending_rule_source() {
    let mut session = Session::new();
    session
        .eval_text(
            "rules := If[MatchQ[InputString[\"rules: \"], \"yes\"], x_Integer -> 1, y_String :> y]",
        )
        .unwrap();

    let failure = session
        .eval_request("1 /. rules")
        .expect_err("ReplaceAll should suspend when its rule source requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rules: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue ReplaceAll after the rule source resolves");

    assert_eq!(response.value.to_string(), "1");
}

#[test]
fn resume_eval_request_with_input_can_finish_patterntest_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "MatchQ[1, PatternTest[x_Integer, Function[x, MatchQ[InputString[\"cond: \"], \"yes\"]]]]",
        )
        .expect_err("PatternTest should suspend when its test function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish PatternTest-based MatchQ");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_sequence_patterntest_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "MatchQ[f[1, 2], f[PatternTest[x__, Function[x, MatchQ[InputString[\"cond: \"], \"yes\"]]], y_]]",
        )
        .expect_err("sequence PatternTest should suspend when its test function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish sequence PatternTest matching");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_except_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request("MatchQ[a, Except[x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]]]")
        .expect_err("Except should suspend when its excluded branch requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Except-based MatchQ");

    assert_eq!(response.value.to_string(), "False");
}

#[test]
fn resume_eval_request_with_input_can_finish_repeated_patterntest_matchq() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "MatchQ[f[1], f[Repeated[PatternTest[x_Integer, Function[x, MatchQ[InputString[\"cond: \"], \"yes\"]]]]]]",
        )
        .expect_err("Repeated should suspend when its element matcher requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Repeated-based MatchQ");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_thread_with_suspending_arg_spec() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Thread[f[{a, b}, {c, d}, {e, f}], List, If[MatchQ[InputString[\"spec: \"], \"yes\"], {2}, All]]",
        )
        .expect_err("Thread should suspend while its argument-spec expression requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "spec: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Thread after the argument spec resolves");

    assert_eq!(
        response.value.to_string(),
        "List[f[List[a, b], c, List[e, f]], f[List[a, b], d, List[e, f]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_setattributes_with_suspending_attribute_spec() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "SetAttributes[f, If[MatchQ[InputString[\"attr: \"], \"yes\"], HoldFirst, HoldRest]]",
        )
        .expect_err("SetAttributes should suspend while its attribute spec requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "attr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish SetAttributes after the attribute spec resolves");

    assert_eq!(response.value.to_string(), "Null");
    assert_eq!(
        session
            .eval_text("f[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "f[Identity[1], 2]"
    );
}
