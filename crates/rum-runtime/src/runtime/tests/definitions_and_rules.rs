use super::*;

#[test]
fn set_assigns_simple_symbol_and_lookup_works() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn self_referential_ownvalue_returns_symbol_without_message() {
    let mut session = session_with_test_control_builtins();

    let outcome = session.eval_text("a = a; a").unwrap();

    assert_eq!(outcome.value.to_string(), "a");
    assert!(outcome.messages.is_empty());
}

#[test]
fn self_referential_delayed_ownvalue_returns_symbol_without_message() {
    let mut session = session_with_test_control_builtins();

    let outcome = session.eval_text("b := b; b").unwrap();

    assert_eq!(outcome.value.to_string(), "b");
    assert!(outcome.messages.is_empty());
}

#[test]
fn immediate_self_alias_preserves_downvalues() {
    let mut session = session_with_test_control_builtins();

    let alias = session.eval_text("s[x_] := x + 1; s = s; s").unwrap();
    let applied = session.eval_text("s[5]").unwrap();

    assert_eq!(alias.value.to_string(), "s");
    assert!(alias.messages.is_empty());
    assert_eq!(applied.value.to_string(), "6");
    assert!(applied.messages.is_empty());
}

#[test]
fn delayed_self_alias_preserves_downvalues() {
    let mut session = session_with_test_control_builtins();

    let alias = session.eval_text("c[x_] := x + 1; c := c; c").unwrap();
    let applied = session.eval_text("c[5]").unwrap();

    assert_eq!(alias.value.to_string(), "c");
    assert!(alias.messages.is_empty());
    assert_eq!(applied.value.to_string(), "6");
    assert!(applied.messages.is_empty());
}

#[test]
fn ownvalues_lists_immediate_and_delayed_symbol_definitions() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("x = 42").unwrap();
    session.eval_text("y := x").unwrap();

    assert_eq!(
        session.eval_text("OwnValues[x]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[x], 42]]"
    );
    assert_eq!(
        session.eval_text("OwnValues[y]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[y], x]]"
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
    let mut session = session_with_test_control_builtins();

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
    assert_eq!(
        session
            .eval_text("Default[h, 1]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(session.eval_text("h[]").unwrap().value.to_string(), "9");

    session.eval_text("Default[g, 1] = 11").unwrap();
    assert_eq!(
        session.eval_text("Default[g]").unwrap().value.to_string(),
        "11"
    );
    assert_eq!(
        session
            .eval_text("Default[g, 1]")
            .unwrap()
            .value
            .to_string(),
        "11"
    );
    assert_eq!(
        session
            .eval_text("Default[g, 2]")
            .unwrap()
            .value
            .to_string(),
        "Default[g, 2]"
    );

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
fn defaultvalues_and_definition_include_explicit_syntax_information() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("DefaultValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[SyntaxInformation[f]], List[Rule[\"ArgumentsPattern\", List[Blank[]]]]]]"
    );
    assert_eq!(
        session
            .eval_text("Definition[f]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[Set[SyntaxInformation[f], List[Rule[\"ArgumentsPattern\", List[Blank[]]]]]]"
    );

    let rendered = session
        .eval_text("Information[f]")
        .unwrap()
        .value
        .to_string();
    assert!(
        rendered.contains("Rule[\"DefaultValues\", Information`InformationValueForm[DefaultValues, f, List[RuleDelayed[HoldPattern[SyntaxInformation[f]], List[Rule[\"ArgumentsPattern\", List[Blank[]]]]]]]]")
    );

    session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(
        session
            .eval_text("DefaultValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn information_returns_first_pass_symbol_summary() {
    let mut session = session_with_test_control_builtins();

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
        rendered.contains("Rule[\"DownValues\", Information`InformationValueForm[DownValues, f, List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[Integer]]]], x]]]]")
    );
    assert!(!rendered.contains("Rule[\"Symbol\""));

    session.eval_text("xq = 5").unwrap();
    let direct = session.eval_text("Information[xq]").unwrap();
    assert!(
        direct
            .value
            .to_string()
            .contains("Rule[\"FullName\", \"Global`xq\"]")
    );
    assert!(
        direct.value.to_string().contains(
            "Rule[\"OwnValues\", Information`InformationValueForm[OwnValues, xq, List[RuleDelayed[HoldPattern[xq], 5]]]]"
        )
    );
    let by_name = session.eval_text("Information[\"xq\"]").unwrap();
    assert_eq!(direct.value, by_name.value);
    assert_eq!(
        session
            .eval_text("Attributes[Information]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst, Protected]"
    );
}

#[test]
fn information_uses_catalog_and_symbol_name_usage_fallbacks_and_stored_message_tags() {
    let mut session = session_with_test_control_builtins();

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

    session.eval_text("m::tag = \"detail\"").unwrap();
    let message_info = session
        .eval_text("Information[m]")
        .unwrap()
        .value
        .to_string();
    assert!(message_info.contains("Rule[\"Messages\", List[\"tag\"]]"));

    let builtin = session
        .eval_text("Information[\"Set\"]")
        .unwrap()
        .value
        .to_string();
    assert!(
        builtin.contains("Rule[\"Usage\", \"Assigns an immediate value to a symbol or pattern.\"]")
    );
    assert!(builtin.contains(
        "Rule[\"Messages\", List[\"argform\", \"argx\", \"kval\", \"pkspec1\", \"shape\", \"write\", \"wrsym\"]]"
    ));
}

#[test]
fn definition_returns_held_first_pass_definition_statements() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f::usage = \"desc\"").unwrap();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session.eval_text("Options[f] = {a -> 0}").unwrap();
    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f = 1").unwrap();
    session.eval_text("Default[f] = 7").unwrap();

    assert_eq!(
        session
            .eval_text("Definition[f]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[Set[Attributes[f], List[HoldFirst]], Set[f::usage, \"desc\"], Set[Options[f], List[Rule[a, 0]]], Set[f, 1], SetDelayed[f[Pattern[x, Blank[Integer]]], x], Set[Default[f], 7]]"
    );
    assert_eq!(session.eval_text("f").unwrap().value.to_string(), "1");
    assert_eq!(
        session
            .eval_text("Definition[\"f\"]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[Set[Attributes[f], List[HoldFirst]], Set[f::usage, \"desc\"], Set[Options[f], List[Rule[a, 0]]], Set[f, 1], SetDelayed[f[Pattern[x, Blank[Integer]]], x], Set[Default[f], 7]]"
    );
}

#[test]
fn definition_returns_empty_hold_for_symbols_without_definitions() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Definition[cl02NoDefinition]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[]"
    );
}

#[test]
fn valueq_uses_rollback_trial_evaluation_without_side_effects() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
        "List[RuleDelayed[HoldPattern[x], 42]]"
    );
    assert_eq!(
        session
            .eval_text("DownValues[\"f\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[y, Blank[Integer]]]], y]]"
    );
    assert_eq!(
        session
            .eval_text("UpValues[\"a\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[h[a]], 1]]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[\"g\"]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[g[Pattern[z, Blank[]]][Pattern[w, Blank[Integer]]]], pair[z, w]]]"
    );
}
