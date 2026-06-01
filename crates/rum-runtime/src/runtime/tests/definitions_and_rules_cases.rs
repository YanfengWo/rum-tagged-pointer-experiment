use super::*;

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
