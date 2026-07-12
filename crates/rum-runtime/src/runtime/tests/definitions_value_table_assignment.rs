use super::*;

#[test]
fn downvalues_identity_round_trip_restores_definition_behavior() {
    let mut session = session_with_test_control_builtins();

    let assigned = session
        .eval_text(
            "f[1] := 111; f[x_] := x + 1; saved = DownValues[f]; ClearAll[f]; DownValues[f] = saved",
        )
        .unwrap();

    assert_eq!(
        assigned.value.to_string(),
        "List[RuleDelayed[HoldPattern[f[1]], 111], RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], Plus[x, 1]]]"
    );
    assert!(assigned.messages.is_empty());
    assert_eq!(
        session.eval_text("{f[1], f[5]}").unwrap().value.to_string(),
        "List[111, 6]"
    );
}

#[test]
fn ownvalues_identity_round_trip_restores_the_owned_value() {
    let mut session = session_with_test_control_builtins();

    let assigned = session
        .eval_text("a = 10; saved = OwnValues[a]; ClearAll[a]; OwnValues[a] = saved")
        .unwrap();

    assert_eq!(
        assigned.value.to_string(),
        "List[RuleDelayed[HoldPattern[a], 10]]"
    );
    assert!(assigned.messages.is_empty());
    assert_eq!(session.eval_text("a").unwrap().value.to_string(), "10");
}

#[test]
fn upvalues_and_subvalues_identity_round_trips_restore_behavior() {
    let mut session = session_with_test_control_builtins();

    let up_assigned = session
        .eval_text(
            "u /: h[u[x_]] := x + 1; savedUp = UpValues[u]; ClearAll[u]; UpValues[u] = savedUp",
        )
        .unwrap();
    assert!(up_assigned.messages.is_empty());
    assert_eq!(session.eval_text("h[u[4]]").unwrap().value.to_string(), "5");

    let sub_assigned = session
        .eval_text(
            "s[x_][y_] := x + y; savedSub = SubValues[s]; ClearAll[s]; SubValues[s] = savedSub",
        )
        .unwrap();
    assert!(sub_assigned.messages.is_empty());
    assert_eq!(session.eval_text("s[2][3]").unwrap().value.to_string(), "5");
}

#[test]
fn cross_symbol_downvalues_copy_fails_closed_on_head_mismatch() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("f[x_] := x + 1; g[x_] := old[x]")
        .unwrap();
    let before = session.eval_text("DownValues[g]").unwrap().value;
    let assigned = session.eval_text("DownValues[g] = DownValues[f]").unwrap();

    assert_eq!(
        assigned.value.to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], Plus[x, 1]]]"
    );
    assert!(assigned.messages.iter().any(|message| {
        message.symbol == "DownValues"
            && message.tag == "setval"
            && message.text.contains("does not define the target")
    }));
    assert_eq!(
        session.eval_text("g[3]").unwrap().value.to_string(),
        "old[3]"
    );
    assert_eq!(session.eval_text("DownValues[g]").unwrap().value, before);
}

#[test]
fn cross_symbol_downvalues_copy_works_after_target_substitution() {
    let mut session = session_with_test_control_builtins();

    let assigned = session
        .eval_text("f[x_] := x + 1; DownValues[g] = (DownValues[f] /. f -> g)")
        .unwrap();

    assert!(assigned.messages.is_empty());
    assert_eq!(session.eval_text("g[3]").unwrap().value.to_string(), "4");
}

#[test]
fn malformed_value_table_rhs_leaves_the_existing_table_unchanged() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("g[x_] := old[x]").unwrap();
    let before = session.eval_text("DownValues[g]").unwrap().value;
    let assigned = session.eval_text("DownValues[g] = 42").unwrap();

    assert_eq!(assigned.value.to_string(), "42");
    assert!(assigned.messages.iter().any(|message| {
        message.symbol == "DownValues"
            && message.tag == "setval"
            && message.text.contains("must be assigned a List")
    }));
    assert_eq!(
        session.eval_text("g[3]").unwrap().value.to_string(),
        "old[3]"
    );
    assert_eq!(session.eval_text("DownValues[g]").unwrap().value, before);
}

#[test]
fn successful_value_table_assignment_replaces_instead_of_merging() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f[0] := old").unwrap();
    let assigned = session
        .eval_text("DownValues[f] = {RuleDelayed[HoldPattern[f[1]], new]}")
        .unwrap();

    assert!(assigned.messages.is_empty());
    assert_eq!(session.eval_text("f[0]").unwrap().value.to_string(), "f[0]");
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "new");
    assert_eq!(
        session.eval_text("DownValues[f]").unwrap().value,
        assigned.value
    );
}

#[test]
fn later_mismatched_rule_cannot_partially_replace_a_value_table() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("g[0] := 7").unwrap();
    let before = session.eval_text("DownValues[g]").unwrap().value;
    let assigned = session
        .eval_text(
            "DownValues[g] = {RuleDelayed[HoldPattern[g[1]], 11], RuleDelayed[HoldPattern[f[x_]], x]}",
        )
        .unwrap();

    assert!(assigned.messages.iter().any(|message| {
        message.symbol == "DownValues"
            && message.tag == "setval"
            && message.text.contains("RuleDelayed[HoldPattern[f[")
    }));
    assert_eq!(session.eval_text("g[0]").unwrap().value.to_string(), "7");
    assert_eq!(session.eval_text("g[1]").unwrap().value.to_string(), "g[1]");
    assert_eq!(session.eval_text("DownValues[g]").unwrap().value, before);
}

#[test]
fn protected_value_table_target_rejects_replacement() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f[x_] := old[x]; Protect[f]").unwrap();
    let before = session.eval_text("DownValues[f]").unwrap().value;
    let assigned = session
        .eval_text("DownValues[f] = {RuleDelayed[HoldPattern[f[x_]], new[x]]}")
        .unwrap();

    assert!(
        assigned
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "wrsym")
    );
    assert_eq!(
        session.eval_text("f[3]").unwrap().value.to_string(),
        "old[3]"
    );
    assert_eq!(session.eval_text("DownValues[f]").unwrap().value, before);
}

#[test]
fn value_table_assignment_accepts_immediate_delayed_and_unwrapped_rules() {
    let mut session = session_with_test_control_builtins();

    let assigned = session
        .eval_text(
            "n = 1; DownValues[f] = {Rule[HoldPattern[f[1]], n], RuleDelayed[f[x_], n]}; n = 2",
        )
        .unwrap();

    assert!(assigned.messages.is_empty());
    assert_eq!(
        session.eval_text("{f[1], f[3]}").unwrap().value.to_string(),
        "List[1, 2]"
    );
}
