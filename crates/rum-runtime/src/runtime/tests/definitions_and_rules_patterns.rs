use super::*;

#[test]
fn matchq_honors_condition_patterns() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn rule_lhs_evaluates_values_without_evaluating_pattern_names() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("x = 1; x -> y")
            .unwrap()
            .value
            .to_string(),
        "Rule[1, y]"
    );
    assert_eq!(
        session.eval_text("x_ -> y").unwrap().value.to_string(),
        "Rule[Pattern[x, Blank[]], y]"
    );
    assert_eq!(
        session
            .eval_text("p = a; f[p] :> y")
            .unwrap()
            .value
            .to_string(),
        "RuleDelayed[f[a], y]"
    );
    assert_eq!(
        session
            .eval_text("h = Integer; Blank[h] -> y")
            .unwrap()
            .value
            .to_string(),
        "Rule[Blank[Integer], y]"
    );
}

#[test]
fn matchq_honors_except_patterns() {
    let mut session = session_with_test_control_builtins();

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
fn except_downvalue_prefilter_stays_conservative() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("g[Except[x_Integer]] := ok").unwrap();

    assert_eq!(session.eval_text("g[a]").unwrap().value.to_string(), "ok");
    assert_eq!(session.eval_text("g[7]").unwrap().value.to_string(), "g[7]");
}

#[test]
fn compiled_downvalue_matcher_handles_named_typed_blank_hit() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f[x_Integer] := x + 1").unwrap();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "2");
}

#[test]
fn compiled_downvalue_matcher_keeps_typed_blank_miss_unevaluated() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f[x_Integer] := x + 1").unwrap();

    assert_eq!(
        session.eval_text("f[\"a\"]").unwrap().value.to_string(),
        "f[\"a\"]"
    );
}

#[test]
fn compiled_downvalue_matcher_binds_multiple_named_args() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("g[x_, y_] := x - y").unwrap();

    assert_eq!(session.eval_text("g[5, 2]").unwrap().value.to_string(), "3");
}

#[test]
fn compiled_downvalue_matcher_preserves_exact_vs_pattern_precedence() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("h[x_] := x + 1").unwrap();
    session.eval_text("h[1] = 10").unwrap();

    assert_eq!(session.eval_text("h[1]").unwrap().value.to_string(), "10");
    assert_eq!(session.eval_text("h[2]").unwrap().value.to_string(), "3");
}

#[test]
fn wide_ground_downvalue_index_preserves_pattern_priority_fallback_and_invalidation() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "ClearAll[f, oneQ]; \
             oneQ[x_] := MatchQ[x, 1]; \
             f[x_Integer?oneQ?oneQ?oneQ?oneQ] := pattern; \
             Do[With[{k = i}, f[k] = k], {i, 1, 32}]; \
             f[x_Integer] := fallback",
        )
        .unwrap();

    assert_eq!(
        session.eval_text("f[1]").unwrap().value.to_string(),
        "pattern"
    );
    assert_eq!(session.eval_text("f[2]").unwrap().value.to_string(), "2");
    assert_eq!(
        session.eval_text("f[33]").unwrap().value.to_string(),
        "fallback"
    );

    session.eval_text("f[2] = updated").unwrap();
    assert_eq!(
        session.eval_text("f[2]").unwrap().value.to_string(),
        "updated"
    );
    session.eval_text("Unset[f[2]]").unwrap();
    assert_eq!(
        session.eval_text("f[2]").unwrap().value.to_string(),
        "fallback"
    );

    let snapshot = session.capture_loader_snapshot();
    session.eval_text("ClearAll[f]").unwrap();
    session.restore_loader_snapshot(snapshot);
    assert_eq!(
        session.eval_text("f[2]").unwrap().value.to_string(),
        "fallback"
    );
}

#[test]
fn wide_ground_downvalue_scan_falls_back_to_live_order_after_matcher_mutation() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "ClearAll[f, mutatingQ]; \
             mutatingQ[x_] := (Unset[f[1]]; False); \
             f[x_Integer?mutatingQ?mutatingQ?mutatingQ?mutatingQ] := pattern; \
             Do[With[{k = i}, f[k] = k], {i, 1, 32}]",
        )
        .unwrap();

    assert_eq!(session.eval_text("f[2]").unwrap().value.to_string(), "2");
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
}

#[test]
fn compiled_downvalue_matcher_falls_back_for_sequence_patterns() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("q[x__] := {x}").unwrap();

    assert_eq!(
        session.eval_text("q[1, 2, 3]").unwrap().value.to_string(),
        "List[1, 2, 3]"
    );
}

#[test]
fn compiled_downvalue_matcher_falls_back_for_alternatives_patterns() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("r[1 | 2] := ok").unwrap();

    assert_eq!(session.eval_text("r[2]").unwrap().value.to_string(), "ok");
}

#[test]
fn matchq_honors_optional_and_repeated_patterns() {
    let mut session = session_with_test_control_builtins();

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
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 1], f[Repeated[1, 2]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 1], f[RepeatedNull[1, 2]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1], f[Repeated[1, {0, 2}]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[RepeatedNull[1, {0}]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Repeated[1, -1], Verbatim[Repeated[1, -1]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_rejects_malformed_repetition_bounds_with_head_specific_messages() {
    let mut session = session_with_test_control_builtins();

    for (input, symbol, expected) in [
        (
            "MatchQ[f[1], f[Repeated[1, -1]]]",
            "Repeated",
            "MatchQ[f[1], f[Repeated[1, -1]]]",
        ),
        (
            "MatchQ[f[1], f[RepeatedNull[1, {2, 1}]]]",
            "RepeatedNull",
            "MatchQ[f[1], f[RepeatedNull[1, List[2, 1]]]]",
        ),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{input}");
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "range"
                && message.text.contains("0 <= min <= max")
        }));
    }
}

#[test]
fn patternsequence_flattens_argument_patterns_in_matching_rules_and_definitions() {
    let mut session = session_with_test_control_builtins();
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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn orderless_sequence_matching_still_accepts_small_normal_cases() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("SetAttributes[f, Orderless]; MatchQ[f[3, 1, 2], f[a_, b__]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[g[1, 2, 3], g[a_, b__]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SetAttributes[h, Orderless]; MatchQ[h[a, c, b], h[a, x__, c]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn verbatim_matches_literal_pattern_language_forms() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn conditioned_downvalue_matches_only_when_test_passes() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("f[x_Integer /; MatchQ[x, 7]] := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
}

#[test]
fn patterntest_downvalue_matches_only_when_test_passes() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("goodQ[x_] := MatchQ[x, 7]").unwrap();
    session
        .eval_text("f[PatternTest[x_Integer, goodQ]] := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
}

#[test]
fn alternatives_downvalue_matches_each_branch() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("(f[x_Integer] /; MatchQ[x, 7]) := x")
        .unwrap();

    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
}
