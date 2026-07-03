use super::*;

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
fn resume_eval_request_with_input_can_finish_orderless_patternsequence_group_matching() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "MatchQ[a + b + c, Plus[PatternSequence[a, x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]], b]]",
        )
        .expect_err("orderless PatternSequence group matching should suspend when its condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish orderless PatternSequence group matching");

    assert_eq!(response.value.to_string(), "True");
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
fn resume_eval_request_with_input_can_finish_conditioned_cases_over_association_values_only() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Cases[Association[a -> 1, b -> f[2]], x_Symbol /; MatchQ[InputString[\"cond: \"], \"yes\"], Infinity, Heads -> True]",
        )
        .expect_err("association-aware Cases traversal should suspend when a matched head condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("yes")
        .expect_err("association-aware Cases should continue into the next matching value head");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish association-aware Cases after the condition resolves");

    assert_eq!(response.value.to_string(), "List[Association, f]");
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
        .expect_err(
            "Count pattern evaluation should suspend when the referenced symbol requests input",
        );

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
