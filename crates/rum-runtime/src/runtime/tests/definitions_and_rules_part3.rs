use super::*;

#[test]
fn replaceall_applies_single_rule() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("7 /. x_Integer -> x")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn replaceall_applies_rule_list_in_order() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("7 /. {x_String -> 0, x_Integer -> 1}")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn replaceall_traverses_nested_expressions() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("f[g[1]] /. g[x_Integer] -> h[x]")
            .unwrap()
            .value
            .to_string(),
        "f[h[1]]"
    );
}

#[test]
fn replaceall_splices_named_blanksequence_into_rhs_calls() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("f[1, 2] /. f[x__] -> g[x]")
            .unwrap()
            .value
            .to_string(),
        "g[1, 2]"
    );
}

#[test]
fn replace_defaults_to_the_root_level() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Replace[f[1], f[x_Integer] :> x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Replace[f[1], x_Integer :> x + 1]")
            .unwrap()
            .value
            .to_string(),
        "f[1]"
    );
}

#[test]
fn replace_supports_levelspecs_and_rule_lists() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Replace[f[g[1], 2], x_Integer :> h[x], {1, Infinity}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[h[1]], h[2]]"
    );
    assert_eq!(
        session
            .eval_text("Replace[f[a], {x_String :> 0, f[x_] :> g[x]}, {0}]")
            .unwrap()
            .value
            .to_string(),
        "g[a]"
    );
    assert_eq!(
        session
            .eval_text("Replace[f[], _ -> x, {-1}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Replace[f[], _ -> x, {-2}]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Replace[f[g[]], _ -> x, {-2}]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("Replace[f[g[]], _ -> x, {-3}]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
}

#[test]
fn replace_rewrites_association_values_without_touching_keys() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "Replace[Association[a -> 1, b -> f[2]], x_Integer :> x + 10, {1, Infinity}]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 11], Rule[b, f[12]]]"
    );
}

#[test]
fn replacelist_supports_operator_form_rule_order_and_max_results() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("ReplaceList[f[x_] :> x][f[a]]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceList[f[a, b], {f[x_, y_] :> x, f[x_, y_] :> y}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceList[f[a, b], {f[x_, y_] :> x, f[x_, y_] :> y}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
}

#[test]
fn rewrite_family_operator_forms_are_inert_and_apply_to_data() {
    let mut session = session_with_test_control_builtins();

    for (source, expected) in [
        ("Cases[_Integer]", "Cases[Blank[Integer]]"),
        ("DeleteCases[_Integer]", "DeleteCases[Blank[Integer]]"),
        (
            "Replace[x_ :> f[x]]",
            "Replace[RuleDelayed[Pattern[x, Blank[]], f[x]]]",
        ),
        (
            "ReplaceAll[x_ :> f[x]]",
            "ReplaceAll[RuleDelayed[Pattern[x, Blank[]], f[x]]]",
        ),
        (
            "ReplaceRepeated[x_ :> f[x]]",
            "ReplaceRepeated[RuleDelayed[Pattern[x, Blank[]], f[x]]]",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert!(outcome.messages.is_empty(), "{source}");
        assert_eq!(outcome.value.to_string(), expected, "{source}");
    }

    for (source, expected) in [
        ("Cases[_Integer][{1, a, 2}]", "List[1, 2]"),
        ("DeleteCases[_Integer][{1, a, 2}]", "List[a]"),
        ("Replace[x_ :> f[x]][a]", "f[a]"),
        ("ReplaceAll[x_ :> f[x]][a]", "f[a]"),
        ("ReplaceRepeated[a -> b][a]", "b"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert!(outcome.messages.is_empty(), "{source}");
        assert_eq!(outcome.value.to_string(), expected, "{source}");
    }

    let invalid = session.eval_text("Cases[_Integer][1, 2]").unwrap();
    assert!(
        invalid
            .messages
            .iter()
            .any(|message| message.symbol == "Cases" && message.tag == "argx")
    );
}

#[test]
fn replacelist_collects_multiple_backtracking_matches_for_sequence_patterns() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("ReplaceList[f[a, b, c], f[x__, y__] :> {List[x], List[y]}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[a], List[b, c]], List[List[a, b], List[c]]]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceList[a + b + c, Plus[PatternSequence[a, x_], b] :> x]")
            .unwrap()
            .value
            .to_string(),
        "List[c]"
    );
}

#[test]
fn flat_and_orderless_flat_heads_regroup_for_match_replace_and_replacelist() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("SetAttributes[f, Flat]").unwrap();

    assert_eq!(
        session
            .eval_text("MatchQ[f[a, b, c], f[x_, y_]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("f[a, b, c] /. f[x_, y_] :> List[x, y]")
            .unwrap()
            .value
            .to_string(),
        "List[a, f[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceList[f[a, b, c], f[x_, y_] :> List[x, y]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, f[b, c]], List[f[a, b], c]]"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[a + b + c, Plus[x_, y_]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("a + b + c /. Plus[x_, y_] :> List[x, y]")
            .unwrap()
            .value
            .to_string(),
        "List[a, Plus[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceList[a + b + c, Plus[x_, y_] :> List[x, y]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, Plus[b, c]], List[b, Plus[a, c]], List[c, Plus[a, b]], List[Plus[a, b], c], List[Plus[a, c], b], List[Plus[b, c], a]]"
    );
    assert_eq!(
        session
            .eval_text("Cases[List[a + b + c], Plus[x_, y_] :> List[x, y], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, Plus[b, c]]]"
    );
}

#[test]
fn replacerepeated_reaches_a_fixed_point() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("f[1] //. {f[x_Integer] -> g[x], g[x_Integer] -> x}")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn replacerepeated_limit_returns_terminated_value_for_non_terminating_rewrite_cycle() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("1 //. {1 -> 2, 2 -> 1}")
        .expect("iteration limit should return a termination value");

    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "iterlim")
    );
}

#[test]
fn replacerepeated_delayed_rule_limit_returns_terminated_value() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_request("1 //. x_ :> x + 1")
        .expect("iteration limit should return a request success");

    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "iterlim")
    );
}

#[test]
fn immediate_rule_captures_current_rhs_value() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("n = 1").unwrap();
    session.eval_text("rule = x_Integer -> n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(
        session.eval_text("7 /. rule").unwrap().value.to_string(),
        "1"
    );
}

#[test]
fn delayed_rule_uses_current_rhs_value_at_application_time() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("n = 1").unwrap();
    session.eval_text("rule = x_Integer :> n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(
        session.eval_text("7 /. rule").unwrap().value.to_string(),
        "2"
    );
}

#[test]
fn delayed_rhs_substitution_does_not_capture_inserted_values_in_lexical_scopes() {
    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x]; SetAttributes[f, HoldAll]; f[body_] := Module[{x}, Hold[body, x]]; f[x]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x, x$1]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x]; f[body_] := With[{x = 1}, Hold[body, x]]; f[x]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x, 1]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x, y]; SetAttributes[f, HoldAll]; f[body_] := Function[x, Hold[body, x], HoldAll][y]; f[x]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x, y]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[x]; Replace[x, body_ :> Module[{x}, Hold[body, x]]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x, x$1]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x]; SetAttributes[f, HoldAll]; f[body_] := Module[{x}, Module[{x}, Hold[body, x]]]; f[x]")
            .unwrap()
            .value
            .to_string(),
        "Hold[x, x$2]"
    );
}

#[test]
fn delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals() {
    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x]; SetAttributes[f, HoldAll]; f[body_] := Module[{x}, Hold[body, x]]; f[Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Hold[x], x$1]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x]; SetAttributes[f, HoldAll]; f[body_] := With[{x = 1}, Hold[body, x]]; f[Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Hold[x], 1]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[f, x, y]; SetAttributes[f, HoldAll]; f[body_] := Function[x, Hold[body, x], HoldAll][y]; f[Hold[x]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Hold[x], y]"
    );
}

#[test]
fn delayed_rhs_substitution_preserves_same_name_local_binding_behavior() {
    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[x, y]; Replace[y, x_ :> Module[{x}, Hold[x]]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[y$1]"
    );

    let mut session = session_with_test_control_builtins();
    assert_eq!(
        session
            .eval_text("ClearAll[x, y]; Replace[y, x_ :> With[{x = 1}, Hold[x]]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1]"
    );
}

#[test]
fn replaceall_honors_condition_patterns() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("7 /. (x_Integer /; MatchQ[x, 7]) -> y")
            .unwrap()
            .value
            .to_string(),
        "y"
    );
    assert_eq!(
        session
            .eval_text("8 /. (x_Integer /; MatchQ[x, 7]) -> y")
            .unwrap()
            .value
            .to_string(),
        "8"
    );
}

#[test]
fn replaceall_honors_patterntest_patterns() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("goodQ[x_] := MatchQ[x, 7]").unwrap();

    assert_eq!(
        session
            .eval_text("7 /. PatternTest[x_Integer, goodQ] -> y")
            .unwrap()
            .value
            .to_string(),
        "y"
    );
    assert_eq!(
        session
            .eval_text("8 /. PatternTest[x_Integer, goodQ] -> y")
            .unwrap()
            .value
            .to_string(),
        "8"
    );
}

#[test]
fn replaceall_honors_alternatives_patterns() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("7 /. Alternatives[x_String, x_Integer] -> x")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("a /. Alternatives[x_String, x_Integer] -> x")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
}

#[test]
fn replaceall_honors_except_and_repeated_patterns() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("a /. Except[x_Integer] -> y")
            .unwrap()
            .value
            .to_string(),
        "y"
    );
    assert_eq!(
        session
            .eval_text("7 /. Except[x_Integer] -> y")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("f[1, 2] /. f[Repeated[_Integer]] -> ok")
            .unwrap()
            .value
            .to_string(),
        "ok"
    );
}
