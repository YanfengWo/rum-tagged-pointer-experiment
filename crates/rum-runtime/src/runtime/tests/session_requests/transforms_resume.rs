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
fn resume_eval_request_with_input_can_finish_replacepart_pattern_bindings_in_delayed_rhs() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "ReplacePart[f[g[a]], {1, i_} :> StringJoin[InputString[\"rhs: \"], ToString[i]]]",
        )
        .expect_err(
            "ReplacePart should suspend when a delayed rhs with pattern bindings requests input",
        );

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
fn resume_eval_request_with_input_can_finish_deletecases_after_deleting_an_association_head() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "DeleteCases[Association[a -> 1, b -> f[2]], x_Symbol /; MatchQ[InputString[\"cond: \"], \"yes\"], Infinity, Heads -> True]",
        )
        .expect_err("association-aware DeleteCases should suspend when a matched head condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish association-aware DeleteCases after the condition resolves");

    assert_eq!(response.value.to_string(), "Association[]");
}

#[test]
fn associateto_requires_an_immediate_symbol_value_instead_of_suspending_delayed_values() {
    let mut session = Session::new();
    session
        .eval_text("assoc := Association[a -> InputString[\"value: \"]]")
        .unwrap();

    let response = session
        .eval_request("AssociateTo[assoc, b -> 2]")
        .expect("AssociateTo should reject delayed values without trying to evaluate them");

    assert_eq!(response.value.to_string(), "AssociateTo[assoc, Rule[b, 2]]");
    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].symbol, "AssociateTo");
    assert_eq!(response.messages[0].tag, "blnoval");
    match session
        .eval_request("assoc")
        .expect_err("delayed assoc should still request input")
        .error
    {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required error, got {other:?}"),
    }
}

#[test]
fn resume_eval_request_with_input_can_finish_keysortby_after_suspending_sort_key() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "KeySortBy[{bb -> 2, a -> 1}, Function[If[SameQ[Slot[1], bb], StringLength[InputString[\"key: \"]], StringLength[ToString[Slot[1]]]]]]",
        )
        .expect_err("KeySortBy should suspend when a transformed sort key requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "key: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("xxxx")
        .expect("resume should continue KeySortBy after the transformed sort key resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_keysort_after_suspending_comparer() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "KeySort[{bb -> 2, a -> 1}, Function[If[SameQ[Slot[1], a], MatchQ[InputString[\"cmp: \"], \"yes\"], True]]]",
        )
        .expect_err("KeySort should suspend when its custom comparer requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cmp: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue KeySort after the custom comparer resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_keyunion_after_suspending_fill_function() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "KeyUnion[{Association[a -> 1], Association[]}, Function[k, InputString[\"fill: \"]]]",
        )
        .expect_err("KeyUnion should suspend when the fill function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "fill: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue KeyUnion after the fill function resolves");

    assert_eq!(
        response.value.to_string(),
        "List[Association[Rule[a, 1]], Association[Rule[a, \"done\"]]]"
    );
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

#[test]
fn eval_request_reports_prints_on_failure_and_across_input_resume() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Print[\"before\"]; Throw[1]")
        .expect_err("uncaught Throw should fail the request");
    assert_eq!(failure.prints, vec!["before"]);

    let suspended = session
        .eval_request("Print[\"pre\"]; Input[\"expr: \"]; Print[\"post\"]")
        .expect_err("Input should suspend the request");
    match &suspended.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(suspended.prints, vec!["pre"]);

    let resumed = session
        .resume_eval_request_with_input("2 + 3")
        .expect("resume should keep both pre- and post-input prints");
    assert_eq!(resumed.value.to_string(), "Null");
    assert_eq!(resumed.prints, vec!["pre", "post"]);
}
