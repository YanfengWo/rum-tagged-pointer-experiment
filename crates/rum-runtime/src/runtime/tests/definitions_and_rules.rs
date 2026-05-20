use super::*;

#[test]
fn set_assigns_simple_symbol_and_lookup_works() {
    let mut session = Session::new();

    let assigned = session.eval_text("x = 42").unwrap();
    assert_eq!(assigned.value.to_string(), "42");

    let looked_up = session.eval_text("x").unwrap();
    assert_eq!(looked_up.value.to_string(), "42");

    let info = session.inspect("x").unwrap();
    assert!(info.has_own_value);
    assert!(!info.has_delayed_value);
}

#[test]
fn setdelayed_stores_rhs_without_immediate_evaluation() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("y := x").unwrap().value.to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("x = 9").unwrap().value.to_string(), "9");
    assert_eq!(session.eval_text("y").unwrap().value.to_string(), "9");

    let info = session.inspect("y").unwrap();
    assert!(!info.has_own_value);
    assert!(info.has_delayed_value);
    assert!(!info.has_down_values);
}
#[test]
fn ownvalues_lists_immediate_and_delayed_symbol_definitions() {
    let mut session = Session::new();

    session.eval_text("x = 42").unwrap();
    session.eval_text("y := x").unwrap();

    assert_eq!(
        session.eval_text("OwnValues[x]").unwrap().value.to_string(),
        "List[Rule[x, 42]]"
    );
    assert_eq!(
        session.eval_text("OwnValues[y]").unwrap().value.to_string(),
        "List[RuleDelayed[y, x]]"
    );
    assert_eq!(
        session
            .eval_text("OwnValues[\"missing\"]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn defaultvalues_lists_optional_defaults_and_clearall_removes_them() {
    let mut session = Session::new();

    session.eval_text("Default[h] = 7").unwrap();
    session.eval_text("Default[h, 1] = 9").unwrap();
    session.eval_text("h[Optional[x_Integer]] := x").unwrap();

    assert_eq!(
        session
            .eval_text("DefaultValues[h]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[h]], 7], RuleDelayed[HoldPattern[Default[h, 1]], 9]]"
    );
    assert_eq!(session.eval_text("h[]").unwrap().value.to_string(), "9");

    session.eval_text("Unset[Default[h, 1]]").unwrap();
    assert_eq!(session.eval_text("h[]").unwrap().value.to_string(), "7");
    assert_eq!(
        session
            .eval_text("DefaultValues[\"h\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[h]], 7]]"
    );

    session.eval_text("ClearAll[h]").unwrap();
    assert_eq!(
        session
            .eval_text("DefaultValues[h]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(session.eval_text("h[]").unwrap().value.to_string(), "h[]");
}

#[test]
fn information_returns_first_pass_symbol_summary() {
    let mut session = Session::new();

    session.eval_text("f::usage = \"desc\"").unwrap();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session.eval_text("Options[f] = {a -> 0}").unwrap();
    session.eval_text("Default[f] = 7").unwrap();
    session.eval_text("f[x_Integer] := x").unwrap();

    let info = session.eval_text("Information[f]").unwrap();
    let rendered = info.value.to_string();
    assert!(rendered.starts_with("InformationData[Association["));
    assert!(rendered.contains("Rule[\"FullName\", \"Global`f\"]"));
    assert!(rendered.contains("Rule[\"Usage\", \"desc\"]"));
    assert!(rendered.contains("Rule[\"Attributes\", List[HoldFirst]]"));
    assert!(rendered.contains("Rule[\"Options\", List[Rule[a, 0]]]"));
    assert!(
        rendered.contains("Rule[\"DefaultValues\", Information`InformationValueForm[DefaultValues, f, List[RuleDelayed[HoldPattern[Default[f]], 7]]]]")
    );
    assert!(
        rendered.contains("Rule[\"DownValues\", Information`InformationValueForm[DownValues, f, List[RuleDelayed[f[Pattern[x, Blank[Integer]]], x]]]]")
    );
    assert!(!rendered.contains("Rule[\"Symbol\""));

    session.eval_text("x = f").unwrap();
    let resolved = session.eval_text("Information[x]").unwrap();
    assert!(
        resolved
            .value
            .to_string()
            .contains("Rule[\"FullName\", \"Global`f\"]")
    );

    session.eval_text("f = 1").unwrap();
    assert_eq!(
        session
            .eval_text("Information[f]")
            .unwrap()
            .value
            .to_string(),
        "Information[1]"
    );
    assert!(
        session
            .eval_text("Information[\"f\"]")
            .unwrap()
            .value
            .to_string()
            .contains(
                "Rule[\"OwnValues\", Information`InformationValueForm[OwnValues, f, List[Rule[f, 1]]]]"
            )
    );
}

#[test]
fn information_uses_symbol_name_usage_fallback_and_value_forms() {
    let mut session = Session::new();

    session.eval_text("Default[g] = 3").unwrap();

    let rendered = session
        .eval_text("Information[\"g\"]")
        .unwrap()
        .value
        .to_string();
    assert!(rendered.contains("Rule[\"Usage\", \"Global`g\"]"));
    assert!(rendered.contains("Rule[\"OwnValues\", None]"));
    assert!(
        rendered.contains("Rule[\"DefaultValues\", Information`InformationValueForm[DefaultValues, g, List[RuleDelayed[HoldPattern[Default[g]], 3]]]]")
    );
}

#[test]
fn valueq_uses_rollback_trial_evaluation_without_side_effects() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("ValueQ[x]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("ValueQ[1]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("ValueQ[1 + 1]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("ValueQ[{x}]").unwrap().value.to_string(),
        "False"
    );

    session.eval_text("x = 1").unwrap();
    session.eval_text("y := x + 1").unwrap();
    session.eval_text("f[1] = 3").unwrap();

    assert_eq!(
        session.eval_text("ValueQ[x]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("ValueQ[y]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("ValueQ[f[1]]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("ValueQ[f[2]]").unwrap().value.to_string(),
        "False"
    );

    assert_eq!(
        session
            .eval_text("ValueQ[z = 9]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(session.eval_text("z").unwrap().value.to_string(), "z");

    let printed = session.eval_text("ValueQ[Print[1]]").unwrap();
    assert_eq!(printed.value.to_string(), "True");
    assert!(printed.prints.is_empty());

    let input = session
        .eval_request("ValueQ[InputString[\"name: \"]]")
        .unwrap();
    assert_eq!(input.value.to_string(), "True");
    assert!(input.prints.is_empty());
    assert!(input.messages.is_empty());
    assert!(session.pending_input_request().is_none());
}

#[test]
fn definition_inspection_builtins_accept_symbol_name_strings() {
    let mut session = Session::new();

    session.eval_text("x = 42").unwrap();
    session.eval_text("f[y_Integer] := y").unwrap();
    session.eval_text("a /: h[a] := 1").unwrap();
    session.eval_text("g[z_][w_Integer] := pair[z, w]").unwrap();

    assert_eq!(
        session
            .eval_text("OwnValues[\"x\"]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[x, 42]]"
    );
    assert_eq!(
        session
            .eval_text("DownValues[\"f\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[f[Pattern[y, Blank[Integer]]], y]]"
    );
    assert_eq!(
        session
            .eval_text("UpValues[\"a\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[h[a], 1]]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[\"g\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[g[Pattern[z, Blank[]]][Pattern[w, Blank[Integer]]], pair[z, w]]]"
    );
}

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
        "List[RuleDelayed[g[f[Pattern[x, Blank[Integer]]]], h[x]]]"
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
        "List[Rule[g[f[Pattern[x, Blank[Integer]]]], 1]]"
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
        "List[RuleDelayed[f[Pattern[x, Blank[Integer]]], x], RuleDelayed[f[Pattern[y, Blank[String]]], y]]"
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
        "List[RuleDelayed[f[Pattern[y, Blank[]]], 2]]"
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
        "List[RuleDelayed[g[f[Pattern[y, Blank[]]]], 2]]"
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
        "List[RuleDelayed[f[Pattern[a, Blank[]]][Pattern[b, Blank[]]], List[a, b]]]"
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
        "List[RuleDelayed[f[Pattern[x, Blank[]]][Pattern[y, Blank[Integer]]], pair[x, y]]]"
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
        "List[RuleDelayed[f[Pattern[x, Blank[]]][Pattern[y, Blank[]]], pair[x, y]]]"
    );
}

#[test]
fn tagset_preserves_tag_ownership_for_upvalues() {
    let mut session = Session::new();

    session.eval_text("g /: f[g[x_]] := x").unwrap();

    assert_eq!(session.eval_text("f[g[7]]").unwrap().value.to_string(), "7");
    assert_eq!(
        session.eval_text("UpValues[g]").unwrap().value.to_string(),
        "List[RuleDelayed[f[g[Pattern[x, Blank[]]]], x]]"
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
fn clear_and_clearall_manage_symbol_lifecycle_state() {
    let mut session = Session::new();

    session.eval_text("f[x_] := x").unwrap();
    session.eval_text("f /: g[f[x_]] := x").unwrap();
    session.eval_text("f::msg = \"boom\"").unwrap();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();

    session.eval_text("Clear[f]").unwrap();
    assert_eq!(
        session.eval_text("OwnValues[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[f::msg]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );

    session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
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
}

#[test]
fn remove_clears_current_symbol_state_and_declared_stubs() {
    let mut session = Session::new();

    session.eval_text("f = 1").unwrap();
    session.eval_text("f[x_] := x").unwrap();
    session.eval_text("f /: g[f[x_]] := x").unwrap();
    session.eval_text("f::msg = \"boom\"").unwrap();
    session.eval_text("Options[f] = {a -> 0}").unwrap();
    session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}")
        .unwrap();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session
        .eval_text("DeclarePackage[\"Demo`\", {\"h\"}]")
        .unwrap();

    let removed = session.eval_text("Remove[f, \"Demo`h\"]").unwrap();
    assert_eq!(removed.value.to_string(), "Null");
    assert!(removed.messages.is_empty());

    assert_eq!(session.eval_text("f").unwrap().value.to_string(), "f");
    assert_eq!(
        session.eval_text("OwnValues[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("Options[f]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
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
        session
            .eval_text("Names[\"Demo`h\"]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn remove_marks_same_evaluation_symbols_stale_and_allows_later_reuse() {
    let mut session = Session::new();

    session.eval_text("cl02r = 1").unwrap();

    assert_eq!(
        session
            .eval_text("Remove[cl02r]; {cl02r, OwnValues[cl02r], Names[\"cl02r\"]}")
            .unwrap()
            .value
            .to_string(),
        "List[Removed[\"cl02r\"], List[], List[]]"
    );
    assert_eq!(
        session.eval_text("cl02r").unwrap().value.to_string(),
        "cl02r"
    );
    assert_eq!(
        session
            .eval_text("cl02r = 2; {cl02r, OwnValues[cl02r], Names[\"cl02r\"]}")
            .unwrap()
            .value
            .to_string(),
        "List[2, List[Rule[cl02r, 2]], List[\"cl02r\"]]"
    );
}

#[test]
fn remove_marks_missing_same_evaluation_symbols_stale() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Remove[cl02Missing]; cl02Missing")
            .unwrap()
            .value
            .to_string(),
        "Removed[\"cl02Missing\"]"
    );
    assert_eq!(
        session.eval_text("cl02Missing").unwrap().value.to_string(),
        "cl02Missing"
    );
}

#[test]
fn remove_reports_protected_symbols_without_clearing_state() {
    let mut session = Session::new();

    session.eval_text("f = 1").unwrap();
    session.eval_text("Protect[f]").unwrap();

    let removed = session.eval_text("Remove[f]").unwrap();
    assert_eq!(removed.value.to_string(), "Null");
    assert!(
        removed
            .messages
            .iter()
            .any(|message| { message.symbol == "Remove" && message.tag == "rmptc" })
    );
    assert_eq!(session.eval_text("f").unwrap().value.to_string(), "1");
}

#[test]
fn protected_symbols_block_definition_and_clear_writes() {
    let mut session = Session::new();

    session.eval_text("Protect[f]").unwrap();

    let set = session.eval_text("f = 1").unwrap();
    assert_eq!(set.value.to_string(), "1");
    assert!(set.messages.iter().any(|message| {
        message.symbol == "Set" && message.tag == "wrsym" && message.text.contains("Protected")
    }));
    assert_eq!(
        session.eval_text("OwnValues[f]").unwrap().value.to_string(),
        "List[]"
    );

    let delayed = session.eval_text("f[x_] := x").unwrap();
    assert_eq!(delayed.value.to_string(), "Null");
    assert!(
        delayed
            .messages
            .iter()
            .any(|message| { message.symbol == "SetDelayed" && message.tag == "wrsym" })
    );
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    let tagged = session.eval_text("f /: g[f[x_]] := x").unwrap();
    assert_eq!(tagged.value.to_string(), "Null");
    assert!(
        tagged
            .messages
            .iter()
            .any(|message| { message.symbol == "TagSetDelayed" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("UpValues[f]").unwrap().value.to_string(),
        "List[]"
    );

    let usage = session.eval_text("f::usage = \"desc\"").unwrap();
    assert_eq!(usage.value.to_string(), "\"desc\"");
    assert!(
        usage
            .messages
            .iter()
            .any(|message| { message.symbol == "Set" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[]"
    );

    let cleared = session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(cleared.value.to_string(), "Null");
    assert!(
        cleared
            .messages
            .iter()
            .any(|message| { message.symbol == "ClearAll" && message.tag == "wrsym" })
    );

    let unset = session.eval_text("Unset[f]").unwrap();
    assert_eq!(unset.value.to_string(), "Null");
    assert!(
        unset
            .messages
            .iter()
            .any(|message| { message.symbol == "Unset" && message.tag == "wrsym" })
    );

    let tag_unset = session.eval_text("TagUnset[f, g[f[x_]]]").unwrap();
    assert_eq!(tag_unset.value.to_string(), "Null");
    assert!(
        tag_unset
            .messages
            .iter()
            .any(|message| { message.symbol == "TagUnset" && message.tag == "wrsym" })
    );

    session.eval_text("Unprotect[f]").unwrap();
    session.eval_text("f = 1").unwrap();
    assert_eq!(session.eval_text("f").unwrap().value.to_string(), "1");
}

#[test]
fn normalize_rules_lists_single_rules_rule_lists_and_preserves_delayedness() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("NormalizeRules[x_Integer -> 1]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Pattern[x, Blank[Integer]], 1]]"
    );

    session
        .eval_text("rules = {x_Integer -> 1, y_String :> y}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("NormalizeRules[rules]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Pattern[x, Blank[Integer]], 1], RuleDelayed[Pattern[y, Blank[String]], y]]"
    );
}

#[test]
fn inspect_rewrite_rules_reports_normalized_rules_and_delayed_flags() {
    let mut session = Session::new();
    let syntax = parse_expr("{x_Integer -> 1, y_String :> y}").unwrap();
    let expr = lower_syntax_expr(&syntax).unwrap();

    let rules = session.inspect_rewrite_rules(expr).unwrap().unwrap();

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].lhs.to_string(), "Pattern[x, Blank[Integer]]");
    assert_eq!(rules[0].rhs.to_string(), "1");
    assert!(!rules[0].delayed);
    assert_eq!(rules[1].lhs.to_string(), "Pattern[y, Blank[String]]");
    assert_eq!(rules[1].rhs.to_string(), "y");
    assert!(rules[1].delayed);
}

#[test]
fn inspect_rewrite_rules_can_suspend_and_resume_with_host_input() {
    let mut session = Session::new();
    session
        .eval_text("ruleSource[\"Alice\"] := {x_Integer -> 1, y_String :> y}")
        .unwrap();
    session
        .eval_text("rules := ruleSource[InputString[\"name: \"]]")
        .unwrap();
    let syntax = parse_expr("rules").unwrap();
    let expr = lower_syntax_expr(&syntax).unwrap();

    let failure = session
        .inspect_rewrite_rules(expr)
        .expect_err("rewrite-rule inspection should suspend on InputString");

    match failure {
        EvalError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required inspection failure, got {other:?}"),
    }

    let rules = session
        .resume_inspect_rewrite_rules_with_input("Alice")
        .expect("resume should continue rewrite-rule inspection")
        .expect("rule source should produce normalized rules");

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].lhs.to_string(), "Pattern[x, Blank[Integer]]");
    assert_eq!(rules[0].rhs.to_string(), "1");
    assert!(!rules[0].delayed);
    assert_eq!(rules[1].lhs.to_string(), "Pattern[y, Blank[String]]");
    assert_eq!(rules[1].rhs.to_string(), "y");
    assert!(rules[1].delayed);
}

#[test]
fn upset_stores_immediate_upvalues_on_inferred_argument_heads() {
    let mut session = Session::new();

    session.eval_text("UpSet[f[g[x_]], 1]").unwrap();

    assert_eq!(session.eval_text("f[g[a]]").unwrap().value.to_string(), "1");
    assert_eq!(
        session.eval_text("UpValues[g]").unwrap().value.to_string(),
        "List[Rule[f[g[Pattern[x, Blank[]]]], 1]]"
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
        "List[RuleDelayed[f[g[Pattern[x, Blank[]]], h[Pattern[y, Blank[]]]], pair[x, y]]]"
    );
    assert_eq!(
        session.eval_text("UpValues[h]").unwrap().value.to_string(),
        "List[RuleDelayed[f[g[Pattern[x, Blank[]]], h[Pattern[y, Blank[]]]], pair[x, y]]]"
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
