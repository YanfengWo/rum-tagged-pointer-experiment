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
    assert!(missing
        .messages
        .iter()
        .any(|message| message.symbol == "TagSetDelayed" && message.tag == "tagnf"));

    let invalid_position = session.eval_text("f /: g[h[f[x_]]] := 1").unwrap();
    assert!(invalid_position
        .messages
        .iter()
        .any(|message| message.symbol == "TagSetDelayed" && message.tag == "tagpos"));

    let invalid_unset = session.eval_text("TagUnset[f, g[h[f[x_]]]]").unwrap();
    assert!(invalid_unset
        .messages
        .iter()
        .any(|message| message.symbol == "TagUnset" && message.tag == "tagpos"));
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
