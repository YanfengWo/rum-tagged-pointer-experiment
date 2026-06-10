use super::*;

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
        "List[2, List[RuleDelayed[HoldPattern[cl02r], 2]], List[\"cl02r\"]]"
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
fn set_list_destructuring_assigns_leaf_targets() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ClearAll[a, b]; {a, b} = {1, 2}; {a, b, DownValues[List], OwnValues[a], OwnValues[b]}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, List[], List[RuleDelayed[HoldPattern[a], 1]], List[RuleDelayed[HoldPattern[b], 2]]]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[a, b, c]; {{a, b}, c} = {{1, 2}, 3}; {a, b, c}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Module[{dom, rng}, {dom, rng} = {x, y}; {dom, rng}]")
            .unwrap()
            .value
            .to_string(),
        "List[x, y]"
    );

    let mismatch = session
        .eval_text("ClearAll[a, b]; {a, b} = {1, 2, 3}")
        .unwrap();
    assert_eq!(mismatch.value.to_string(), "List[1, 2, 3]");
    assert!(
        mismatch
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "shape")
    );
    assert_eq!(
        session.eval_text("{a, b}").unwrap().value.to_string(),
        "List[a, b]"
    );
}

#[test]
fn definition_lhs_evaluates_call_heads_before_storing_rules() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(
                "ClearAll[g, h, f, q, x]; h = f; g[h[x_]] := 1; {g[f[a]], g[h[a]], DownValues[g]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[1, 1, List[RuleDelayed[HoldPattern[g[f[Pattern[x, Blank[]]]]], 1]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "ClearAll[g, h, f, q, x]; h = f; f = q; g[h[x_]] := 1; {g[q[a]], DownValues[g]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[1, List[RuleDelayed[HoldPattern[g[q[Pattern[x, Blank[]]]]], 1]]]"
    );
}

#[test]
fn definition_lhs_evaluates_non_pattern_constants_before_storing_rules() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ClearAll[f, p, a]; p = a; f[p] := hit; {f[a], f[p], DownValues[f]}")
            .unwrap()
            .value
            .to_string(),
        "List[hit, hit, List[RuleDelayed[HoldPattern[f[a]], hit]]]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[f, x]; x = 1; f[x_] := x; {f[a], DownValues[f]}")
            .unwrap()
            .value
            .to_string(),
        "List[a, List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], x]]]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[f, h, x]; h = Integer; f[x_h] := x; {f[3], DownValues[f]}")
            .unwrap()
            .value
            .to_string(),
        "List[3, List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[Integer]]]], x]]]"
    );
}

#[test]
fn definition_lhs_evaluates_outer_call_head_before_selecting_definition_owner() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(
                "ClearAll[h, f, x]; h = f; h[x_] := 2; {f[a], h[a], DownValues[f], DownValues[h]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[2, 2, List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], 2]], List[]]"
    );
    assert_eq!(
        session
            .eval_text(
                "ClearAll[h, f, x]; h := f; h[x_] := 2; {f[a], h[a], DownValues[f], DownValues[h]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[2, 2, List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], 2]], List[]]"
    );
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
