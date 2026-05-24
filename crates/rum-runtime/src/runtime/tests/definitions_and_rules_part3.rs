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
fn replace_defaults_to_the_root_level() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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
    let mut session = Session::new();

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
fn replacelist_collects_multiple_backtracking_matches_for_sequence_patterns() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
        "List[2]"
    );
}

#[test]
fn cases_can_match_the_root_expression() {
    let mut session = Session::new();

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
fn cases_supports_standard_integer_level_specs() {
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
        "List[1, 2, 3]"
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
fn cases_supports_exact_range_negative_and_infinity_level_specs() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, {1}]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, {2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, {-1}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[], _, {-1}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[], _, {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[]]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[]], _, {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[g[]]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[]], _, {-3}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[]]]"
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
            .eval_text("Cases[x, _Symbol, All, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[x]"
    );
    assert_eq!(
        session
            .eval_text("Cases[x, _Symbol, All, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[x]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[1, g[2]], x_Symbol, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f]"
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
            .eval_text("Cases[f[1, g[2]], x_Symbol, {2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[g]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1]], _Symbol, {-1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, g]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g][a], _, {-2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g], f[g][a]]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g][a], _, {-3}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    let unknown_option = session
        .eval_text("Cases[f[a], _, Infinity, Bad -> 1]")
        .unwrap();
    assert_eq!(
        unknown_option.value.to_string(),
        "Cases[f[a], Blank[], Infinity, Rule[Bad, 1]]"
    );
    assert!(
        unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "Cases" && message.tag == "optx")
    );
    let invalid_heads_value = session
        .eval_text("Cases[f[a], _, Infinity, Heads -> x]")
        .unwrap();
    assert_eq!(
        invalid_heads_value.value.to_string(),
        "Cases[f[a], Blank[], Infinity, Rule[Heads, x]]"
    );
    assert!(
        invalid_heads_value
            .messages
            .iter()
            .any(|message| message.symbol == "Cases" && message.tag == "opttf")
    );
    let non_option = session
        .eval_text("Cases[f[a], _, Infinity, 1, {1}]")
        .unwrap();
    assert_eq!(
        non_option.value.to_string(),
        "Cases[f[a], Blank[], Infinity, 1, List[1]]"
    );
    assert!(
        non_option
            .messages
            .iter()
            .any(|message| message.symbol == "Cases" && message.tag == "nonopt")
    );
}

#[test]
fn cases_traverses_depth_first_with_leaves_before_roots_and_respects_match_limits() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[f[g[1], h[2]], x_, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[1, g[1], 2, h[2]]"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[g[1], 2, h[3]], x_Integer, Infinity, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
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
    assert_eq!(
        session
            .eval_text("Cases[Association[a -> 1, b -> f[2]], x_Integer :> g[x], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[g[1], g[2]]"
    );
}

#[test]
fn cases_and_deletecases_use_value_only_association_traversal() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Cases[Association[a -> 1, b -> f[2]], _Symbol, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Cases[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Association, f]"
    );
    assert_eq!(
        session
            .eval_text(
                "DeleteCases[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[Association[a -> 1, b -> f[2]], f, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text(
                "DeleteCases[Association[a -> 1, b -> f[2]], Association, Infinity, Heads -> True]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[]"
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
fn patternsequence_flattens_argument_patterns_in_matching_rules_and_definitions() {
    let mut session = Session::new();
    session
        .eval_text("pair[PatternSequence[x_, y_]] := List[x, y]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("MatchQ[f[a, b], f[PatternSequence[x_, y_]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[PatternSequence[]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("pair[a, b]").unwrap().value.to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("f[a, b] /. f[PatternSequence[x_, y_]] :> List[x, y]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
}

#[test]
fn orderless_heads_backtrack_across_exact_and_patternsequence_groups() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("a + b + c /. Plus[b, x_, y_] :> List[x, y]")
            .unwrap()
            .value
            .to_string(),
        "List[a, c]"
    );
    assert_eq!(
        session
            .eval_text("a + b + c /. Plus[PatternSequence[a, x_], b] :> x")
            .unwrap()
            .value
            .to_string(),
        "c"
    );
    assert_eq!(
        session
            .eval_text("a + b + c /. Plus[PatternSequence[x_, y_], z_] :> List[x, y, z]")
            .unwrap()
            .value
            .to_string(),
        "List[b, c, a]"
    );
}

#[test]
fn orderlesspatternsequence_uses_orderless_group_matching_inside_orderless_heads() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[f[a, b], f[OrderlessPatternSequence[x_, y_]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("a + b + c /. Plus[OrderlessPatternSequence[a, x_], b] :> x")
            .unwrap()
            .value
            .to_string(),
        "c"
    );
    assert_eq!(
        session
            .eval_text("a + b + c /. Plus[OrderlessPatternSequence[x_, y_], z_] :> List[x, y, z]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
}

#[test]
fn verbatim_matches_literal_pattern_language_forms() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[Blank[], Verbatim[Blank[]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[7, Verbatim[Blank[]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Cases[List[Blank[], a, BlankSequence[]], Verbatim[Blank[]]]")
            .unwrap()
            .value
            .to_string(),
        "List[Blank[]]"
    );
    assert_eq!(
        session
            .eval_text("f[Blank[]] /. f[Verbatim[Blank[]]] -> ok")
            .unwrap()
            .value
            .to_string(),
        "ok"
    );
}

#[test]
fn longest_and_shortest_control_sequence_pattern_greediness() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("f[a, b, c] /. f[x__, y__] :> List[List[x], List[y]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("f[a, b, c] /. f[Longest[x__], y__] :> List[List[x], List[y]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[c]]"
    );
    assert_eq!(
        session
            .eval_text("f[a, b, c] /. f[Shortest[x__], y__] :> List[List[x], List[y]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b, c]]"
    );
    assert_eq!(
        session
            .eval_text(
                "f[a, b, c, d] /. f[x__, Longest[y__], z__] :> List[List[x], List[y], List[z]]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b, c], List[d]]"
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
fn deletecases_deletes_top_level_matches_by_default() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("DeleteCases[f[g[1], 2, \"hi\"], x_Integer]")
            .unwrap()
            .value
            .to_string(),
        "f[g[1], \"hi\"]"
    );
}

#[test]
fn deletecases_supports_standard_levelspecs_and_limits() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("DeleteCases[f[g[1], 2, h[3]], x_Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "f[g[], h[]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[f[g[1], 2, h[3]], x_Integer, Infinity, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[g[], h[3]]"
    );
}

#[test]
fn deletecases_can_delete_heads_and_association_values() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("DeleteCases[f[g[1]], g, All, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "f[1]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[f[g[1]], g, All, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "f[g[1]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[f[1, g[2]], x_Symbol, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "Sequence[1, g[2]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[f[g][a], f[g][a], {-3}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "f[g][a]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[<|a -> 1, b -> x, c -> 2|>, _Integer]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, x]]"
    );
    let unknown_option = session
        .eval_text("DeleteCases[f[a], _, Infinity, Bad -> 1]")
        .unwrap();
    assert_eq!(
        unknown_option.value.to_string(),
        "DeleteCases[f[a], Blank[], Infinity, Rule[Bad, 1]]"
    );
    assert!(
        unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "DeleteCases" && message.tag == "optx")
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
