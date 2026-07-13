use super::*;

#[test]
fn literal_sequence_splicing_is_deferred_until_replacement_result_evaluation() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "SetAttributes[hc, HoldAllComplete]; {a /. (a :> hc[Sequence[1, 2]]), a /. (a :> f[Sequence[1, 2]]), hc[Sequence[1, 2]]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[hc[Sequence[1, 2]], f[1, 2], hc[Sequence[1, 2]]]"
    );
}

#[test]
fn sequence_pattern_bindings_keep_structural_splicing() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "SetAttributes[hc, HoldAllComplete]; {f[1, 2] /. (f[x__] :> hc[x]), Hold[g[f[1, 2]]] /. (f[x__] :> h[x])}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[hc[1, 2], Hold[g[h[1, 2]]]]"
    );
}

#[test]
fn literal_sequence_wrapping_a_sequence_binding_keeps_its_literal_head() {
    let mut session = session_with_test_control_builtins();

    // Pinned rum contract, not a claimed WL result: the bound x splices first
    // into the literal Sequence call, whose head is then preserved for the
    // destination head's evaluation policy.
    assert_eq!(
        session
            .eval_text(
                "SetAttributes[hc, HoldAllComplete]; f[1, 2] /. (f[x__] :> hc[Sequence[x]])",
            )
            .unwrap()
            .value
            .to_string(),
        "hc[Sequence[1, 2]]"
    );
}

#[test]
fn whole_expression_sequence_replacement_under_hold_records_current_boundary() {
    let mut session = session_with_test_control_builtins();

    // Known WL divergence: ReplaceAll's held-parent reconstruction currently
    // keeps a whole-expression Sequence replacement nested. Fixing that is a
    // separate ReplaceAll reconstruction change, outside this contract.
    assert_eq!(
        session
            .eval_text("Hold[g[a]] /. (a -> Sequence[1, 2])")
            .unwrap()
            .value
            .to_string(),
        "Hold[g[Sequence[1, 2]]]"
    );
}

#[test]
fn replacepart_inserts_delayed_rhs_raw_before_result_evaluation() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("c = 0; rp = ReplacePart[Hold[a], {1} :> (c = c + 10; b)]; {rp, c}",)
            .unwrap()
            .value
            .to_string(),
        "List[Hold[CompoundExpression[Set[c, Plus[c, 10]], b]], 0]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[{1, 2, 3}, 2 :> 10 + 10]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 20, 3]"
    );
}

#[test]
fn replacepart_immediate_rhs_still_evaluates_at_rule_construction() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("c = 0; rp = ReplacePart[Hold[a], {1} -> (c = c + 10; b)]; {rp, c}",)
            .unwrap()
            .value
            .to_string(),
        "List[Hold[b], 10]"
    );
}

#[test]
fn replaceat_inserts_delayed_rhs_raw_before_result_evaluation() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("c = 0; rt = ReplaceAt[Hold[a], a :> (c = c + 100; b), {1}]; {rt, c}",)
            .unwrap()
            .value
            .to_string(),
        "List[Hold[CompoundExpression[Set[c, Plus[c, 100]], b]], 0]"
    );
    assert_eq!(
        session
            .eval_text(
                "{ReplaceAt[{1, 2, 3}, 2 :> 10 + 10, {2}], ReplaceAt[2 :> 10 + 10, {2}][{1, 2, 3}], ReplaceAt[{1, 2, 3}, 2 :> 10 + 10, {{2}, {2}}]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 20, 3], List[1, 20, 3], List[1, 20, 3]]"
    );
}

#[test]
fn replaceat_immediate_rhs_still_evaluates_at_rule_construction() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("c = 0; rt = ReplaceAt[Hold[a], a -> (c = c + 100; b), {1}]; {rt, c}",)
            .unwrap()
            .value
            .to_string(),
        "List[Hold[b], 100]"
    );
}

#[test]
fn held_delayed_rhs_firing_is_consumer_independent() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "c = 0; ra = Hold[a] /. (a :> (c = c + 1; b)); rp = ReplacePart[Hold[a], {1} :> (c = c + 10; b)]; rt = ReplaceAt[Hold[a], a :> (c = c + 100; b), {1}]; {ra, rp, rt, c}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Hold[CompoundExpression[Set[c, Plus[c, 1]], b]], Hold[CompoundExpression[Set[c, Plus[c, 10]], b]], Hold[CompoundExpression[Set[c, Plus[c, 100]], b]], 0]"
    );
}
