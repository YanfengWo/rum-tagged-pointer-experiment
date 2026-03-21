use super::*;

#[test]
fn replaceall_applies_single_rule() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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
fn replacerepeated_reaches_a_fixed_point() {
    let mut session = Session::new();

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
fn replacerepeated_errors_on_non_terminating_rewrite_cycle() {
    let mut session = Session::new();

    let error = session
        .eval_text("1 //. {1 -> 2, 2 -> 1}")
        .expect_err("expected iteration limit error");

    assert!(matches!(
        error,
        EvalError::IterationLimitExceeded {
            limit: DEFAULT_REPLACE_REPEATED_LIMIT
        }
    ));
}

#[test]
fn immediate_rule_captures_current_rhs_value() {
    let mut session = Session::new();

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
    let mut session = Session::new();

    session.eval_text("n = 1").unwrap();
    session.eval_text("rule = x_Integer :> n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(
        session.eval_text("7 /. rule").unwrap().value.to_string(),
        "2"
    );
}

#[test]
fn matchq_honors_condition_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[7, x_Integer /; MatchQ[x, 7]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[8, x_Integer /; MatchQ[x, 7]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_honors_patterntest_patterns() {
    let mut session = Session::new();

    session.eval_text("goodQ[x_] := MatchQ[x, 7]").unwrap();

    assert_eq!(
        session
            .eval_text("MatchQ[7, PatternTest[x_Integer, goodQ]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[8, PatternTest[x_Integer, goodQ]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_honors_alternatives_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[7, Alternatives[x_String, x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[a, Alternatives[x_String, x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_honors_except_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[7, Except[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[a, Except[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_honors_optional_and_repeated_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[Optional[x_Integer, 7]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[Repeated[x_Integer]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1], f[Repeated[x_Integer, {2}]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn cases_collects_nested_matches() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, \"hi\"], x_Integer]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
}

#[test]
fn cases_can_match_the_root_expression() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[1], f[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[1]]"
    );
}

#[test]
fn cases_does_not_treat_heads_as_separate_candidates() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[1, g[2]], x_Symbol]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn cases_supports_exact_level_specs() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[1], f[x_Integer], 0]")
            .unwrap()
            .value
            .to_string(),
        "List[f[1]]"
    );
}

#[test]
fn cases_supports_range_and_infinity_level_specs() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, {1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, {2, Infinity}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
}

#[test]
fn cases_supports_explicit_head_traversal_control() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[1, g[2]], x_Symbol, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, g]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[1, g[2]], x_Symbol, 1, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[1, g[2]], x_Symbol, 2, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[g]"
    );
}

#[test]
fn cases_applies_rule_transformations() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2], g[x_Integer] :> h[x]]")
            .unwrap()
            .value
            .to_string(),
        "List[h[1]]"
    );
}

#[test]
fn cases_preserves_immediate_and_delayed_rule_behavior() {
    let mut session = Session::new();

    session.eval_text("n = 1").unwrap();
    session.eval_text("immediate = x_Integer -> n").unwrap();
    session.eval_text("delayed = x_Integer :> n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(
        session
            .eval_text("Cases[{7}, immediate]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    assert_eq!(
        session
            .eval_text("Cases[{7}, delayed]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
}

#[test]
fn cases_honors_condition_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[7, 8, 9], x_Integer /; MatchQ[x, 8]]")
            .unwrap()
            .value
            .to_string(),
        "List[8]"
    );
}

#[test]
fn cases_honors_patterntest_patterns() {
    let mut session = Session::new();

    session.eval_text("goodQ[x_] := MatchQ[x, 8]").unwrap();

    assert_eq!(
        session
            .eval_text("Cases[f[7, 8, 9], PatternTest[x_Integer, goodQ]]")
            .unwrap()
            .value
            .to_string(),
        "List[8]"
    );
}

#[test]
fn cases_honors_alternatives_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[7, \"hi\", a], Alternatives[x_Integer, x_String]]")
            .unwrap()
            .value
            .to_string(),
        "List[7, \"hi\"]"
    );
}

#[test]
fn cases_honors_except_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[7, a, \"hi\"], Except[x_Integer], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[a, \"hi\"]"
    );
}

#[test]
fn cases_honors_optional_and_repeated_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[], g[1], g[1, 2]], g[Optional[x_Integer, 0]] :> x]")
            .unwrap()
            .value
            .to_string(),
        "List[0, 1]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], g[1, 2], g[1, a]], g[Repeated[x_Integer]]]")
            .unwrap()
            .value
            .to_string(),
        "List[g[1], g[1, 2]]"
    );
}

#[test]
fn cases_rule_rhs_splices_named_blanksequence_matches() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1, 2], g[3]], g[x__] :> {x}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], List[3]]"
    );
}

#[test]
fn replaceall_honors_condition_patterns() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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

#[test]
fn conditioned_downvalue_matches_only_when_test_passes() {
    let mut session = Session::new();

    session
        .eval_text("f[x_Integer /; MatchQ[x, 7]] := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
}

#[test]
fn patterntest_downvalue_matches_only_when_test_passes() {
    let mut session = Session::new();

    session.eval_text("goodQ[x_] := MatchQ[x, 7]").unwrap();
    session
        .eval_text("f[PatternTest[x_Integer, goodQ]] := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
}

#[test]
fn alternatives_downvalue_matches_each_branch() {
    let mut session = Session::new();

    session
        .eval_text("f[Alternatives[x_String, x_Integer]] := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(
        session.eval_text("f[\"hi\"]").unwrap().value.to_string(),
        "\"hi\""
    );
    assert_eq!(session.eval_text("f[a]").unwrap().value.to_string(), "f[a]");
}

#[test]
fn except_optional_and_repeated_downvalues_match_expected_shapes() {
    let mut session = Session::new();

    session.eval_text("f[Except[x_Integer]] := ok").unwrap();
    session.eval_text("g[Optional[x_Integer, 7]] := x").unwrap();
    session.eval_text("Default[h, 1] = 9").unwrap();
    session.eval_text("h[Optional[x_Integer]] := x").unwrap();
    session.eval_text("r[Repeated[_Integer]] := ok").unwrap();

    assert_eq!(session.eval_text("f[a]").unwrap().value.to_string(), "ok");
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("g[3]").unwrap().value.to_string(), "3");
    assert_eq!(session.eval_text("h[]").unwrap().value.to_string(), "9");
    assert_eq!(
        session.eval_text("r[1, 2]").unwrap().value.to_string(),
        "ok"
    );
    assert_eq!(session.eval_text("r[]").unwrap().value.to_string(), "r[]");
}

#[test]
fn outer_conditioned_downvalue_is_stored_and_applied() {
    let mut session = Session::new();

    session
        .eval_text("(f[x_Integer] /; MatchQ[x, 7]) := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
}
