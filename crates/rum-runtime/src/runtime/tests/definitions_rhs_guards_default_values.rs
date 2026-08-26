use super::*;

fn default_guard_session() -> Session {
    session_with_test_control_builtins()
}

#[test]
fn defaultvalue_rhs_guards_accept_decline_and_coexist_in_order() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "Default[guardedDefault] := \"one\" /; MatchQ[defaultSelector, 1]; \
             Default[guardedDefault] := \"two\" /; MatchQ[defaultSelector, 2]; \
             Default[guardedDefault] := \"other\"; \
             guardedDefault[x_.] := x",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "defaultSelector = 1; first = guardedDefault[]; \
                 defaultSelector = 2; second = guardedDefault[]; \
                 defaultSelector = 3; third = guardedDefault[]; \
                 {first, second, third}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"one\", \"two\", \"other\"]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[guardedDefault]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[guardedDefault]], Condition[\"one\", MatchQ[defaultSelector, 1]]], RuleDelayed[HoldPattern[Default[guardedDefault]], Condition[\"two\", MatchQ[defaultSelector, 2]]], RuleDelayed[HoldPattern[Default[guardedDefault]], \"other\"]]"
    );
}

#[test]
fn same_defaultvalue_lhs_and_guards_redefine_in_place() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "Default[redefinedDefault] := \"old one\" /; MatchQ[defaultSelector, 1]; \
             Default[redefinedDefault] := \"two\" /; MatchQ[defaultSelector, 2]; \
             Default[redefinedDefault] := \"new one\" /; MatchQ[defaultSelector, 1]; \
             redefinedDefault[x_.] := x",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "defaultSelector = 1; first = redefinedDefault[]; \
                 defaultSelector = 2; second = redefinedDefault[]; \
                 {first, second}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"new one\", \"two\"]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[redefinedDefault]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[redefinedDefault]], Condition[\"new one\", MatchQ[defaultSelector, 1]]], RuleDelayed[HoldPattern[Default[redefinedDefault]], Condition[\"two\", MatchQ[defaultSelector, 2]]]]"
    );
}

#[test]
fn nested_defaultvalue_guards_run_left_to_right_and_second_decline_falls_back() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "defaultGuardOrder = 0; \
             Default[orderedDefault] := Condition[7, Condition[(defaultGuardOrder = defaultGuardOrder * 10 + 1; True), (defaultGuardOrder = defaultGuardOrder * 10 + 2; False)]]; \
             Default[orderedDefault] := 9; \
             orderedDefault[x_.] := x",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{orderedDefault[], defaultGuardOrder}")
            .unwrap()
            .value
            .to_string(),
        "List[9, 12]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[orderedDefault]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[orderedDefault]], Condition[7, Condition[CompoundExpression[Set[defaultGuardOrder, Plus[Times[defaultGuardOrder, 10], 1]], True], CompoundExpression[Set[defaultGuardOrder, Plus[Times[defaultGuardOrder, 10], 2]], False]]]], RuleDelayed[HoldPattern[Default[orderedDefault]], 9]]"
    );
}

#[test]
fn every_non_true_defaultvalue_guard_result_declines() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "Default[nonBooleanDefault] := symbolResult /; defaultUndecided; \
             Default[nonBooleanDefault] := integerResult /; 7; \
             Default[nonBooleanDefault] := callResult /; defaultUnknown[]; \
             Default[nonBooleanDefault] := fallback; \
             nonBooleanDefault[x_.] := x",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("nonBooleanDefault[]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[nonBooleanDefault]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[nonBooleanDefault]], Condition[symbolResult, defaultUndecided]], RuleDelayed[HoldPattern[Default[nonBooleanDefault]], Condition[integerResult, 7]], RuleDelayed[HoldPattern[Default[nonBooleanDefault]], Condition[callResult, defaultUnknown[]]], RuleDelayed[HoldPattern[Default[nonBooleanDefault]], fallback]]"
    );
}

#[test]
fn immediate_and_delayed_defaultvalue_heads_split_guards_at_the_right_time() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "defaultInstallCount = 0; \
             Set[Default[defaultImmediate], Condition[(defaultInstallCount = defaultInstallCount + 1; 7), (defaultInstallCount = defaultInstallCount + 1; True)]]; \
             SetDelayed[Default[defaultDelayed], Condition[8, True]]; \
             defaultImmediate[x_.] := x; \
             defaultDelayed[x_.] := x",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "{defaultInstallCount, defaultImmediate[], defaultDelayed[], defaultInstallCount}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[2, 7, 8, 2]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[defaultImmediate]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[defaultImmediate]], Condition[7, True]]]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[defaultDelayed]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[defaultDelayed]], Condition[8, True]]]"
    );
}

#[test]
fn unset_clears_every_defaultvalue_guard_variant_and_table_roundtrip_restores_them() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "Default[defaultClear] := one /; MatchQ[defaultSelector, 1]; \
             Default[defaultClear] := two /; MatchQ[defaultSelector, 2]; \
             defaultClear[x_.] := x; \
             savedDefaultGuards = DownValues[Default]",
        )
        .unwrap();
    let before = session
        .eval_text("DefaultValues[defaultClear]")
        .unwrap()
        .value
        .to_string();

    session.eval_text("Unset[Default[defaultClear]]").unwrap();
    assert_eq!(
        session
            .eval_text("DefaultValues[defaultClear]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("defaultClear[]")
            .unwrap()
            .value
            .to_string(),
        "defaultClear[]"
    );

    session
        .eval_text("DownValues[Default] = savedDefaultGuards")
        .unwrap();
    assert_eq!(
        session
            .eval_text(
                "defaultSelector = 1; first = defaultClear[]; \
                 defaultSelector = 2; second = defaultClear[]; \
                 {first, second}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[one, two]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[defaultClear]")
            .unwrap()
            .value
            .to_string(),
        before
    );
}

#[test]
fn declining_default_guard_leaves_optional_matching_without_a_default() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "Default[withDecliningDefault] := 7 /; False; \
             withDecliningDefault[x_.] := x; \
             withoutDefault[x_.] := x",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{withDecliningDefault[], withoutDefault[]}")
            .unwrap()
            .value
            .to_string(),
        "List[withDecliningDefault[], withoutDefault[]]"
    );
    assert_eq!(
        session
            .eval_text("DefaultValues[withDecliningDefault]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[Default[withDecliningDefault]], Condition[7, False]]]"
    );
}

#[test]
fn definition_inspection_rebuilds_all_remaining_definition_guard_forms() {
    let mut session = default_guard_session();

    session
        .eval_text(
            "inspectUp /: inspectHead[inspectUp, x_] := upBody /; MatchQ[x, 1]; \
             inspectSub[1][x_] := subBody /; MatchQ[x, 1]; \
             Default[inspectDefault] := 7 /; True",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("Definition[inspectUp]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[SetDelayed[inspectHead[inspectUp, Pattern[x, Blank[]]], Condition[upBody, MatchQ[x, 1]]]]"
    );
    assert_eq!(
        session
            .eval_text("Definition[inspectSub]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[SetDelayed[inspectSub[1][Pattern[x, Blank[]]], Condition[subBody, MatchQ[x, 1]]]]"
    );
    assert_eq!(
        session
            .eval_text("Definition[inspectDefault]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[SetDelayed[Default[inspectDefault], Condition[7, True]]]"
    );
}
