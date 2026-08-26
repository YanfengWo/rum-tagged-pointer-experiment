use super::*;

fn sub_guard_session() -> Session {
    session_with_test_control_builtins()
}

#[test]
fn subvalue_rhs_guards_accept_decline_and_coexist_in_order() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "s[1][x_] := \"one\" /; MatchQ[x, 1]; \
             s[1][x_] := \"two\" /; MatchQ[x, 2]; \
             s[1][x_] := \"other\"",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{s[1][1], s[1][2], s[1][3]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"one\", \"two\", \"other\"]"
    );
    assert_eq!(
        session.eval_text("SubValues[s]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[s[1][Pattern[x, Blank[]]]], Condition[\"one\", MatchQ[x, 1]]], RuleDelayed[HoldPattern[s[1][Pattern[x, Blank[]]]], Condition[\"two\", MatchQ[x, 2]]], RuleDelayed[HoldPattern[s[1][Pattern[x, Blank[]]]], \"other\"]]"
    );
}

#[test]
fn same_alpha_canonical_subvalue_lhs_and_guards_redefine_in_place() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "s[1][x_] := \"old one\" /; MatchQ[x, 1]; \
             s[1][z_] := \"two\" /; MatchQ[z, 2]; \
             s[1][y_] := \"new one\" /; MatchQ[y, 1]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{s[1][1], s[1][2], s[1][3]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"new one\", \"two\", s[1][3]]"
    );
    assert_eq!(
        session.eval_text("SubValues[s]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[s[1][Pattern[y, Blank[]]]], Condition[\"new one\", MatchQ[y, 1]]], RuleDelayed[HoldPattern[s[1][Pattern[z, Blank[]]]], Condition[\"two\", MatchQ[z, 2]]]]"
    );
}

#[test]
fn nested_subvalue_guards_run_left_to_right_and_second_decline_falls_back() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "subGuardOrder = 0; \
             orderedSub[1][x_] := Condition[\"guarded\", Condition[(subGuardOrder = subGuardOrder * 10 + 1; True), (subGuardOrder = subGuardOrder * 10 + 2; False)]]; \
             orderedSub[1][x_] := \"fallback\"",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{orderedSub[1][1], subGuardOrder}")
            .unwrap()
            .value
            .to_string(),
        "List[\"fallback\", 12]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[orderedSub]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[orderedSub[1][Pattern[x, Blank[]]]], Condition[\"guarded\", Condition[CompoundExpression[Set[subGuardOrder, Plus[Times[subGuardOrder, 10], 1]], True], CompoundExpression[Set[subGuardOrder, Plus[Times[subGuardOrder, 10], 2]], False]]]], RuleDelayed[HoldPattern[orderedSub[1][Pattern[x, Blank[]]]], \"fallback\"]]"
    );
}

#[test]
fn every_non_true_subvalue_guard_result_declines() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "nonBooleanSub[1][x_] := symbolResult /; subUndecided; \
             nonBooleanSub[1][x_] := integerResult /; 7; \
             nonBooleanSub[1][x_] := callResult /; subUnknown[x]; \
             nonBooleanSub[1][x_] := fallback",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("nonBooleanSub[1][1]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
    assert_eq!(
        session
            .eval_text("SubValues[nonBooleanSub]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[nonBooleanSub[1][Pattern[x, Blank[]]]], Condition[symbolResult, subUndecided]], RuleDelayed[HoldPattern[nonBooleanSub[1][Pattern[x, Blank[]]]], Condition[integerResult, 7]], RuleDelayed[HoldPattern[nonBooleanSub[1][Pattern[x, Blank[]]]], Condition[callResult, subUnknown[x]]], RuleDelayed[HoldPattern[nonBooleanSub[1][Pattern[x, Blank[]]]], fallback]]"
    );
}

#[test]
fn subvalue_rhs_guards_share_the_guarded_body_option_scope() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "Options[optionSub] = {accept -> False}; \
             optionSub[1][x_, OptionsPattern[]] := guarded[x, OptionValue[accept]] /; OptionValue[accept]; \
             optionSub[1][x_, OptionsPattern[]] := fallback[x, OptionValue[accept]]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{optionSub[1][1], optionSub[1][1, accept -> True]}")
            .unwrap()
            .value
            .to_string(),
        "List[fallback[1, False], guarded[1, True]]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[optionSub]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[optionSub[1][Pattern[x, Blank[]], OptionsPattern[]]], Condition[guarded[x, OptionValue[accept]], OptionValue[accept]]], RuleDelayed[HoldPattern[optionSub[1][Pattern[x, Blank[]], OptionsPattern[]]], fallback[x, OptionValue[accept]]]]"
    );
}

#[test]
fn immediate_and_delayed_subvalue_heads_split_guards_at_the_right_time() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "subInstallCount = 0; \
             Set[subImmediate[1][x_], Condition[(subInstallCount = subInstallCount + 1; \"immediate\"), (subInstallCount = subInstallCount + 1; True)]]; \
             SetDelayed[subDelayed[1][x_], Condition[\"delayed\", MatchQ[x, 1]]]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{subInstallCount, subImmediate[1][0], subDelayed[1][1], subInstallCount}",)
            .unwrap()
            .value
            .to_string(),
        "List[2, \"immediate\", \"delayed\", 2]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[subImmediate]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[subImmediate[1][Pattern[x, Blank[]]]], Condition[\"immediate\", True]]]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[subDelayed]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[subDelayed[1][Pattern[x, Blank[]]]], Condition[\"delayed\", MatchQ[x, 1]]]]"
    );
}

#[test]
fn unset_clears_every_subvalue_guard_variant_and_table_roundtrip_restores_them() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "subClear[1][x_] := one /; MatchQ[x, 1]; \
             subClear[1][x_] := two /; MatchQ[x, 2]; \
             savedSubGuards = SubValues[subClear]",
        )
        .unwrap();
    let before = session
        .eval_text("SubValues[subClear]")
        .unwrap()
        .value
        .to_string();

    session.eval_text("Unset[subClear[1][z_]]").unwrap();
    assert_eq!(
        session
            .eval_text("SubValues[subClear]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("subClear[1][1]")
            .unwrap()
            .value
            .to_string(),
        "subClear[1][1]"
    );

    session
        .eval_text("SubValues[subClear] = savedSubGuards")
        .unwrap();
    assert_eq!(
        session
            .eval_text("{subClear[1][1], subClear[1][2], subClear[1][3]}")
            .unwrap()
            .value
            .to_string(),
        "List[one, two, subClear[1][3]]"
    );
    assert_eq!(
        session
            .eval_text("SubValues[subClear]")
            .unwrap()
            .value
            .to_string(),
        before
    );
}

#[test]
fn wide_subvalue_identity_index_keeps_guard_variants_distinct() {
    let mut session = sub_guard_session();

    session
        .eval_text(
            "Do[With[{k = i}, wideSub[k][x_] := k], {i, 1, 32}]; \
             wideSub[99][x_] := oldOne /; MatchQ[x, 1]; \
             wideSub[99][x_] := two /; MatchQ[x, 2]; \
             wideSub[99][y_] := newOne /; MatchQ[y, 1]; \
             wideSub[99][x_] := other",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{wideSub[99][1], wideSub[99][2], wideSub[99][3]}")
            .unwrap()
            .value
            .to_string(),
        "List[newOne, two, other]"
    );
    let Expr::Call { args, .. } = session.eval_text("SubValues[wideSub]").unwrap().value else {
        panic!("SubValues[wideSub] should be a List");
    };
    assert_eq!(args.len(), 35);
}
