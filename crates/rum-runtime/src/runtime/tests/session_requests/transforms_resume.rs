use super::*;

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
fn resume_eval_request_with_input_can_finish_replace_after_suspending_target() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Replace[InputString[\"target: \"], _String -> ok]")
        .expect_err("Replace should suspend when its target requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "target: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue Replace after the target resolves");

    assert_eq!(response.value.to_string(), "ok");
}

#[test]
fn resume_eval_request_with_input_can_finish_replace_after_suspending_levelspec() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Replace[f[1, g[2]], _Integer -> x, If[MatchQ[InputString[\"lvl: \"], \"yes\"], Infinity, 0]]",
        )
        .expect_err("Replace should suspend when its levelspec requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lvl: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Replace after the levelspec resolves");

    assert_eq!(response.value.to_string(), "f[x, g[x]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_replacelist_after_suspending_delayed_rhs() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplaceList[f[1], f[x_Integer] :> InputString[\"rhs: \"]]")
        .expect_err("ReplaceList should suspend when a delayed rhs requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue ReplaceList after the delayed rhs resolves");

    assert_eq!(response.value.to_string(), "List[\"done\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_replacelist_after_multiple_backtracking_matches() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplaceList[f[a, b, c], f[x__, y__] :> InputString[\"rhs: \"]]")
        .expect_err("ReplaceList should suspend on the first delayed rhs match");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("first")
        .expect_err("ReplaceList should continue into the second backtracking match");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("second")
        .expect("resume should finish ReplaceList after all backtracking matches");

    assert_eq!(response.value.to_string(), "List[\"first\", \"second\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_replaceat_after_suspending_position_spec() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "ReplaceAt[f[a, b], x_ -> y, If[MatchQ[InputString[\"pos: \"], \"yes\"], 2, 1]]",
        )
        .expect_err("ReplaceAt should suspend when its position specification requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "pos: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue ReplaceAt after the position resolves");

    assert_eq!(response.value.to_string(), "f[a, y]");
}

#[test]
fn resume_eval_request_with_input_can_finish_replaceat_after_suspending_delayed_rhs() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplaceAt[f[1], x_Integer :> InputString[\"rhs: \"], 1]")
        .expect_err("ReplaceAt should suspend when its delayed rhs requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue ReplaceAt after the delayed rhs resolves");

    assert_eq!(response.value.to_string(), "f[\"done\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_deletecases_after_suspending_target() {
    let mut session = Session::new();

    let failure = session
        .eval_request("DeleteCases[f[InputString[\"target: \"], 2], _String]")
        .expect_err("DeleteCases should suspend when its target requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "target: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue DeleteCases after the target resolves");

    assert_eq!(response.value.to_string(), "f[2]");
}

#[test]
fn resume_eval_request_with_input_can_finish_deletecases_after_suspending_pattern_symbol() {
    let mut session = Session::new();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session
        .eval_request("DeleteCases[f[g[1], 2], p, Infinity]")
        .expect_err("DeleteCases should suspend when a referenced pattern symbol requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue DeleteCases after the pattern resolves");

    assert_eq!(response.value.to_string(), "f[g[]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_deletecases_after_suspending_match_limit() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "DeleteCases[f[g[1], 2, h[3]], _Integer, Infinity, If[MatchQ[InputString[\"n: \"], \"yes\"], 2, 0]]",
        )
        .expect_err("DeleteCases should suspend when its match limit requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "n: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue DeleteCases after the match limit resolves");

    assert_eq!(response.value.to_string(), "f[g[], h[3]]");
}
