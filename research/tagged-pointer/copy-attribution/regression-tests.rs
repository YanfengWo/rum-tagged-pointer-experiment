use super::*;

// This module is copied only into isolated baseline/candidate overlays.
// The equivalent language acceptance probe passed on the pinned baseline.
// The Rust module still requires execution in each prepared test overlay.

#[test]
fn replace_shared_children_preserve_input_aliases_and_reusable_rules() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "copySource = {f[1, 2], f[3, 4]}; \
             copyAlias = copySource; \
             copyRule = f[x_Integer, y_Integer] :> g[y, x]; \
             copyFirst = Replace[copySource, copyRule, {1}]; \
             copySecond = Replace[copyAlias, copyRule, {1}]; \
             {copySource, copyAlias, copyFirst, copySecond}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[f[1, 2], f[3, 4]], List[f[1, 2], f[3, 4]], List[g[2, 1], g[4, 3]], List[g[2, 1], g[4, 3]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn replace_repeated_compound_binding_isolates_later_branch_mutation() {
    let mut session = session_with_test_control_builtins();
    // Part assignment requires a plain List at every level of its path.
    let outcome = session
        .eval_text(
            "copySource = {{1, 2}, {3, 4}}; \
             copyAlias = copySource; \
             copyRule = x_List :> {x, x}; \
             copyResult = Replace[copySource, copyRule, {1}]; \
             copyResult[[1, 1, 1]] = 99; \
             {copySource, copyAlias, copyResult, Replace[copySource, copyRule, {1}]}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[1, 2], List[3, 4]], List[List[1, 2], List[3, 4]], List[List[List[99, 2], List[1, 2]], List[List[3, 4], List[3, 4]]], List[List[List[1, 2], List[1, 2]], List[List[3, 4], List[3, 4]]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn replace_level_one_finishes_guard_scan_before_result_body_evaluation() {
    let mut session = session_with_test_control_builtins();
    // Confirmed by the baseline language acceptance probe:
    // both guards observe c == 1 before either materialized body increments it.
    // Evaluating each replacement eagerly would instead change guardOrder.
    let outcome = session
        .eval_text(
            "c = 1; guardOrder = 0; \
             copyResult = Replace[{1, 2}, \
               x_Integer :> ((c = c + 1; x + c) /; \
                 (guardOrder = guardOrder * 10 + c; True)), {1}]; \
             {copyResult, c, guardOrder}",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[List[3, 5], 3, 11]");
    assert!(outcome.messages.is_empty());
}
