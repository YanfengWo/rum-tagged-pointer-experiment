use super::*;

#[test]
fn upvalues_lists_and_applies_tagged_definitions() {
    let mut session = Session::new();

    session.eval_text("g[x_] := x").unwrap();
    session.eval_text("f /: g[f[x_Integer]] := h[x]").unwrap();

    assert_eq!(
        session.eval_text("g[f[7]]").unwrap().value.to_string(),
        "h[7]"
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[g[f[Pattern[x, Blank[Integer]]]]], h[x]]]"
    );
}

#[test]
fn tagset_immediate_captures_rhs_before_later_changes() {
    let mut session = Session::new();

    session.eval_text("n = 1").unwrap();
    session.eval_text("f /: g[f[x_Integer]] = n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(session.eval_text("g[f[7]]").unwrap().value.to_string(), "1");
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[g[f[Pattern[x, Blank[Integer]]]]], 1]]"
    );
}

#[test]
fn downvalues_lists_stored_pattern_definitions_in_order() {
    let mut session = Session::new();

    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f[y_String] := y").unwrap();

    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[Integer]]]], x], RuleDelayed[HoldPattern[f[Pattern[y, Blank[String]]]], y]]"
    );
}

#[test]
fn redefining_same_downvalue_replaces_the_older_rule_in_place() {
    let mut session = Session::new();

    session.eval_text("f[x_] := 1").unwrap();
    session.eval_text("f[y_] := 2").unwrap();

    assert_eq!(session.eval_text("f[a]").unwrap().value.to_string(), "2");
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[y, Blank[]]]], 2]]"
    );
}

#[test]
fn literal_downvalue_is_prioritized_over_existing_typed_downvalue() {
    let mut session = Session::new();

    session.eval_text("d[x_Integer] := \"integer\"").unwrap();
    session.eval_text("d[x_] := \"generic\"").unwrap();
    session.eval_text("d[1] := \"one\"").unwrap();

    assert_eq!(
        session.eval_text("d[1]").unwrap().value.to_string(),
        "\"one\""
    );
    assert_eq!(
        session.eval_text("d[3]").unwrap().value.to_string(),
        "\"integer\""
    );
    assert_eq!(
        session.eval_text("d[a]").unwrap().value.to_string(),
        "\"generic\""
    );
    assert_eq!(
        session
            .eval_text("DownValues[d]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[d[1]], \"one\"], RuleDelayed[HoldPattern[d[Pattern[x, Blank[Integer]]]], \"integer\"], RuleDelayed[HoldPattern[d[Pattern[x, Blank[]]]], \"generic\"]]"
    );
}

#[test]
fn exact_downvalue_dispatch_preserves_ground_and_pattern_semantics() {
    let mut exact_session = Session::new();

    exact_session.eval_text("f[1] = 10").unwrap();

    assert_eq!(
        exact_session.eval_text("f[1]").unwrap().value.to_string(),
        "10"
    );
    assert_eq!(
        exact_session.eval_text("f[2]").unwrap().value.to_string(),
        "f[2]"
    );

    let mut precedence_session = Session::new();

    precedence_session.eval_text("f[x_] := x + 1").unwrap();
    precedence_session.eval_text("f[1] = 10").unwrap();

    assert_eq!(
        precedence_session
            .eval_text("f[1]")
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(
        precedence_session
            .eval_text("f[2]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );

    let mut delayed_session = Session::new();

    delayed_session.eval_text("g[1] := 10").unwrap();

    assert_eq!(
        delayed_session.eval_text("g[1]").unwrap().value.to_string(),
        "10"
    );

    let mut conditioned_session = Session::new();

    conditioned_session.eval_text("h[1] /; True := 5").unwrap();

    assert_eq!(
        conditioned_session
            .eval_text("h[1]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}

#[test]
fn redefining_same_upvalue_replaces_the_older_rule_in_place() {
    let mut session = Session::new();

    session.eval_text("f /: g[f[x_]] := 1").unwrap();
    session.eval_text("f /: g[f[y_]] := 2").unwrap();

    assert_eq!(session.eval_text("g[f[a]]").unwrap().value.to_string(), "2");
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[g[f[Pattern[y, Blank[]]]]], 2]]"
    );
}

#[test]
fn redefining_same_subvalue_replaces_the_older_rule_in_place() {
    let mut session = Session::new();

    session.eval_text("f[x_][y_] := pair[x, y]").unwrap();
    session.eval_text("f[a_][b_] := List[a, b]").unwrap();

    assert_eq!(
        session.eval_text("f[1][2]").unwrap().value.to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session.eval_text("SubValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[a, Blank[]]][Pattern[b, Blank[]]]], List[a, b]]]"
    );
}

#[test]
fn typed_upvalues_are_prioritized_over_earlier_generic_upvalues() {
    let mut session = Session::new();

    session.eval_text("u /: h[u[x_]] := any[x]").unwrap();
    session
        .eval_text("u /: h[u[x_Integer]] := integer[x]")
        .unwrap();

    assert_eq!(
        session.eval_text("h[u[7]]").unwrap().value.to_string(),
        "integer[7]"
    );
    assert_eq!(
        session.eval_text("h[u[a]]").unwrap().value.to_string(),
        "any[a]"
    );
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[h[u[Pattern[x, Blank[Integer]]]]], integer[x]], RuleDelayed[HoldPattern[h[u[Pattern[x, Blank[]]]]], any[x]]]"
    );
}

#[test]
fn typed_subvalues_are_prioritized_over_earlier_generic_subvalues() {
    let mut session = Session::new();

    session.eval_text("sv[x_][y_] := any[x, y]").unwrap();
    session
        .eval_text("sv[x_][y_Integer] := integer[x, y]")
        .unwrap();

    assert_eq!(
        session.eval_text("sv[a][7]").unwrap().value.to_string(),
        "integer[a, 7]"
    );
    assert_eq!(
        session.eval_text("sv[a][b]").unwrap().value.to_string(),
        "any[a, b]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[sv]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[sv[Pattern[x, Blank[]]][Pattern[y, Blank[Integer]]]], integer[x, y]], RuleDelayed[HoldPattern[sv[Pattern[x, Blank[]]][Pattern[y, Blank[]]]], any[x, y]]]"
    );
}

#[test]
fn wide_upvalue_store_index_preserves_specificity_replacement_and_invalidation() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "ClearAll[f, g]; \
             Do[With[{k = i}, g /: f[g[k], x_] := k], {i, 1, 32}]; \
             g /: f[g[z_], x_] := fallback[z, x]; \
             g /: f[g[33], x_] := 33; \
             g /: f[g[17], y_] := updated[y]; \
             TagUnset[g, f[g[5], q_]]; \
             g /: f[g[34], x_] := 34",
        )
        .unwrap();

    assert_eq!(
        session.eval_text("f[g[17], a]").unwrap().value.to_string(),
        "updated[a]"
    );
    assert_eq!(
        session.eval_text("f[g[5], a]").unwrap().value.to_string(),
        "fallback[5, a]"
    );
    assert_eq!(
        session.eval_text("f[g[34], a]").unwrap().value.to_string(),
        "34"
    );
    assert_eq!(
        session.eval_text("f[g[99], a]").unwrap().value.to_string(),
        "fallback[99, a]"
    );
    let up_values = session.eval_text("UpValues[g]").unwrap().value;
    let Expr::Call { args, .. } = up_values else {
        panic!("UpValues[g] should return a List");
    };
    assert_eq!(args.len(), 34);

    session
        .eval_text("Do[With[{k = i}, g[h[k]] = k], {i, 1, 32}]")
        .unwrap();
    assert_eq!(
        session.eval_text("g[h[17]]").unwrap().value.to_string(),
        "17"
    );
    session.eval_text("Unset[g[h[5]]]").unwrap();
    assert_eq!(
        session.eval_text("g[h[5]]").unwrap().value.to_string(),
        "g[h[5]]"
    );
    session.eval_text("g /: f[g[35], x_] := 35").unwrap();
    assert_eq!(
        session.eval_text("f[g[35], a]").unwrap().value.to_string(),
        "35"
    );
    session.eval_text("DownValues[g] = {}").unwrap();
    assert_eq!(
        session.eval_text("f[g[17], a]").unwrap().value.to_string(),
        "updated[a]"
    );
}

#[test]
fn wide_subvalue_store_index_preserves_specificity_replacement_and_invalidation() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "ClearAll[f]; \
             Do[With[{k = i}, f[k][x_] := k], {i, 1, 32}]; \
             f[z_][x_] := fallback[z, x]; \
             f[33][x_] := 33; \
             f[17][y_] := updated[y]; \
             Unset[f[5][q_]]; \
             f[34][x_] := 34",
        )
        .unwrap();

    assert_eq!(
        session.eval_text("f[17][a]").unwrap().value.to_string(),
        "updated[a]"
    );
    assert_eq!(
        session.eval_text("f[5][a]").unwrap().value.to_string(),
        "fallback[5, a]"
    );
    assert_eq!(
        session.eval_text("f[34][a]").unwrap().value.to_string(),
        "34"
    );
    assert_eq!(
        session.eval_text("f[99][a]").unwrap().value.to_string(),
        "fallback[99, a]"
    );
    let sub_values = session.eval_text("SubValues[f]").unwrap().value;
    let Expr::Call { args, .. } = sub_values else {
        panic!("SubValues[f] should return a List");
    };
    assert_eq!(args.len(), 34);
}

#[test]
fn subvalues_list_and_apply_nested_head_definitions() {
    let mut session = Session::new();

    session.eval_text("f[x_][y_Integer] := pair[x, y]").unwrap();

    assert_eq!(
        session.eval_text("f[1][2]").unwrap().value.to_string(),
        "pair[1, 2]"
    );
    assert_eq!(
        session.eval_text("SubValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]][Pattern[y, Blank[Integer]]]], pair[x, y]]]"
    );
}

#[test]
fn tagged_subvalues_attach_to_the_head_symbol_not_to_upvalues() {
    let mut session = Session::new();

    session.eval_text("f /: f[x_][y_] := pair[x, y]").unwrap();

    assert_eq!(
        session.eval_text("f[1][2]").unwrap().value.to_string(),
        "pair[1, 2]"
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("SubValues[f]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]][Pattern[y, Blank[]]]], pair[x, y]]]"
    );
}

#[test]
fn tagset_preserves_tag_ownership_for_upvalues() {
    let mut session = Session::new();

    session.eval_text("g /: f[g[x_]] := x").unwrap();

    assert_eq!(session.eval_text("f[g[7]]").unwrap().value.to_string(), "7");
    assert_eq!(
        session.eval_text("UpValues[g]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[g[Pattern[x, Blank[]]]]], x]]"
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[]"
    );
}

#[test]
fn unset_can_remove_symbol_downvalue_and_upvalue_definitions() {
    let mut session = Session::new();

    session.eval_text("x = 1").unwrap();
    session.eval_text("x =.").unwrap();
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "x");

    session.eval_text("f[x_] := x").unwrap();
    session.eval_text("f[y_] =.").unwrap();
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    session.eval_text("a /: h[a] := 1").unwrap();
    session.eval_text("a /: h[a] =.").unwrap();
    assert_eq!(
        session.eval_text("UpValues[a]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(session.eval_text("h[a]").unwrap().value.to_string(), "h[a]");
}

#[test]
fn explicit_tagunset_removes_tagged_subvalue_definitions() {
    let mut session = Session::new();

    session.eval_text("f /: f[x_][y_] := pair[x, y]").unwrap();
    session.eval_text("TagUnset[f, f[x_][y_]]").unwrap();

    assert_eq!(
        session.eval_text("SubValues[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("f[1][2]").unwrap().value.to_string(),
        "f[1][2]"
    );
}

#[test]
fn tag_definition_messages_distinguish_missing_tags_from_invalid_positions() {
    let mut session = Session::new();

    let missing = session.eval_text("f /: g[x_] := 1").unwrap();
    assert!(
        missing
            .messages
            .iter()
            .any(|message| message.symbol == "TagSetDelayed" && message.tag == "tagnf")
    );

    let invalid_position = session.eval_text("f /: g[h[f[x_]]] := 1").unwrap();
    assert!(
        invalid_position
            .messages
            .iter()
            .any(|message| message.symbol == "TagSetDelayed" && message.tag == "tagpos")
    );

    let invalid_unset = session.eval_text("TagUnset[f, g[h[f[x_]]]]").unwrap();
    assert!(
        invalid_unset
            .messages
            .iter()
            .any(|message| message.symbol == "TagUnset" && message.tag == "tagpos")
    );
}

#[test]
fn string_targets_flow_through_clear_tag_and_unset_normalization() {
    let mut session = Session::new();

    session.eval_text("TagSet[\"f\", f[x_], x]").unwrap();
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "1");

    session.eval_text("Unset[TagSet[\"f\", f[x_]]]").unwrap();
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    session
        .eval_text("MessageName[\"f\", \"usage\"] = \"desc\"")
        .unwrap();
    session
        .eval_text("Unset[MessageName[\"f\", \"usage\"]]")
        .unwrap();
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[]"
    );

    session.eval_text("f[x_] := x").unwrap();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session.eval_text("f::msg = \"boom\"").unwrap();
    session.eval_text("ClearAll[\"f\"]").unwrap();

    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[]"
    );
}

#[test]
fn clear_name_patterns_expand_before_mutation_and_preserve_exact_targets() {
    let mut session = Session::new();

    session.eval_text("cl213Exact = 7").unwrap();
    let exact = session.eval_text("Clear[\"cl213Exact\"]").unwrap();
    assert!(exact.messages.is_empty());
    assert_eq!(
        session.eval_text("cl213Exact").unwrap().value.to_string(),
        "cl213Exact"
    );

    session.eval_text("cl213Symbol = 8").unwrap();
    session.eval_text("Clear[cl213Symbol]").unwrap();
    assert_eq!(
        session.eval_text("cl213Symbol").unwrap().value.to_string(),
        "cl213Symbol"
    );

    session.eval_text("cl213Protected = 5").unwrap();
    session.eval_text("cl213Open = 6").unwrap();
    session.eval_text("Protect[cl213Protected]").unwrap();
    let cleared = session.eval_text("Clear[\"Global`cl213*\"]").unwrap();
    assert!(
        cleared
            .messages
            .iter()
            .any(|message| message.symbol == "Clear" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("cl213Protected")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session.eval_text("cl213Open").unwrap().value.to_string(),
        "cl213Open"
    );

    let unmatched = session
        .eval_text("Clear[\"Global`cl213NoMatch*\"]")
        .unwrap();
    assert!(unmatched.messages.is_empty());
    assert_eq!(
        session
            .eval_text("NameQ[\"Global`cl213NoMatch*\"]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn clearall_and_remove_expand_name_patterns() {
    let mut session = Session::new();

    session.eval_text("cl213All = 1").unwrap();
    session.eval_text("cl213All::tag = \"tag\"").unwrap();
    session
        .eval_text("SetAttributes[cl213All, HoldFirst]")
        .unwrap();
    session.eval_text("ClearAll[\"Global`cl213All*\"]").unwrap();
    assert_eq!(
        session.eval_text("cl213All").unwrap().value.to_string(),
        "cl213All"
    );
    assert_eq!(
        session
            .eval_text("Messages[cl213All]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[cl213All]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    session.eval_text("cl213RemoveKeep = 1").unwrap();
    session.eval_text("cl213RemoveDrop = 2").unwrap();
    session.eval_text("Protect[cl213RemoveKeep]").unwrap();
    let removed = session
        .eval_text("Remove[\"Global`cl213Remove*\"]")
        .unwrap();
    assert!(
        removed
            .messages
            .iter()
            .any(|message| message.symbol == "Remove" && message.tag == "rmptc")
    );
    assert_eq!(
        session
            .eval_text("cl213RemoveKeep")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("cl213RemoveDrop")
            .unwrap()
            .value
            .to_string(),
        "cl213RemoveDrop"
    );
}

#[test]
fn upset_stores_immediate_upvalues_on_inferred_argument_heads() {
    let mut session = Session::new();

    session.eval_text("UpSet[f[g[x_]], 1]").unwrap();

    assert_eq!(session.eval_text("f[g[a]]").unwrap().value.to_string(), "1");
    assert_eq!(
        session.eval_text("UpValues[g]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[g[Pattern[x, Blank[]]]]], 1]]"
    );
}

#[test]
fn upsetdelayed_can_store_on_multiple_inferred_tag_symbols() {
    let mut session = Session::new();

    session
        .eval_text("UpSetDelayed[f[g[x_], h[y_]], pair[x, y]]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("f[g[a], h[b]]")
            .unwrap()
            .value
            .to_string(),
        "pair[a, b]"
    );
    assert_eq!(
        session.eval_text("UpValues[g]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[g[Pattern[x, Blank[]]], h[Pattern[y, Blank[]]]]], pair[x, y]]]"
    );
    assert_eq!(
        session.eval_text("UpValues[h]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[f[g[Pattern[x, Blank[]]], h[Pattern[y, Blank[]]]]], pair[x, y]]]"
    );
}

#[test]
fn upset_emits_nosym_when_no_valid_argument_tag_symbol_exists() {
    let mut session = Session::new();

    let outcome = session.eval_text("UpSet[f[x_], 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "1");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "UpSet" && message.tag == "nosym")
    );
}
