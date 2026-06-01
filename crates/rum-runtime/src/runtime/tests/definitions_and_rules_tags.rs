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
