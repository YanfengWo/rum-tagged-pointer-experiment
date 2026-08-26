use super::*;

fn up_guard_session() -> Session {
    session_with_test_control_builtins()
}

#[test]
fn upvalue_rhs_guards_accept_decline_and_coexist_in_order() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "u /: h[u, x_] := \"one\" /; MatchQ[x, 1]; \
             u /: h[u, x_] := \"two\" /; MatchQ[x, 2]; \
             u /: h[u, x_] := \"other\"",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{h[u, 1], h[u, 2], h[u, 3]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"one\", \"two\", \"other\"]"
    );
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[h[u, Pattern[x, Blank[]]]], Condition[\"one\", MatchQ[x, 1]]], RuleDelayed[HoldPattern[h[u, Pattern[x, Blank[]]]], Condition[\"two\", MatchQ[x, 2]]], RuleDelayed[HoldPattern[h[u, Pattern[x, Blank[]]]], \"other\"]]"
    );
}

#[test]
fn same_alpha_canonical_upvalue_lhs_and_guards_redefine_in_place() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "u /: h[u, x_] := \"old one\" /; MatchQ[x, 1]; \
             u /: h[u, z_] := \"two\" /; MatchQ[z, 2]; \
             u /: h[u, y_] := \"new one\" /; MatchQ[y, 1]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{h[u, 1], h[u, 2], h[u, 3]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"new one\", \"two\", h[u, 3]]"
    );
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[h[u, Pattern[y, Blank[]]]], Condition[\"new one\", MatchQ[y, 1]]], RuleDelayed[HoldPattern[h[u, Pattern[z, Blank[]]]], Condition[\"two\", MatchQ[z, 2]]]]"
    );
}

#[test]
fn nested_upvalue_guards_run_left_to_right_and_second_decline_falls_back() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "upGuardOrder = 0; \
             u /: ordered[u, x_] := Condition[\"guarded\", Condition[(upGuardOrder = upGuardOrder * 10 + 1; True), (upGuardOrder = upGuardOrder * 10 + 2; False)]]; \
             u /: ordered[u, x_] := \"fallback\"",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{ordered[u, 1], upGuardOrder}")
            .unwrap()
            .value
            .to_string(),
        "List[\"fallback\", 12]"
    );
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[ordered[u, Pattern[x, Blank[]]]], Condition[\"guarded\", Condition[CompoundExpression[Set[upGuardOrder, Plus[Times[upGuardOrder, 10], 1]], True], CompoundExpression[Set[upGuardOrder, Plus[Times[upGuardOrder, 10], 2]], False]]]], RuleDelayed[HoldPattern[ordered[u, Pattern[x, Blank[]]]], \"fallback\"]]"
    );
}

#[test]
fn every_non_true_upvalue_guard_result_declines() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "u /: nonBoolean[u, x_] := symbolResult /; upUndecided; \
             u /: nonBoolean[u, x_] := integerResult /; 7; \
             u /: nonBoolean[u, x_] := callResult /; upUnknown[x]; \
             u /: nonBoolean[u, x_] := fallback",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("nonBoolean[u, 1]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        "List[RuleDelayed[HoldPattern[nonBoolean[u, Pattern[x, Blank[]]]], Condition[symbolResult, upUndecided]], RuleDelayed[HoldPattern[nonBoolean[u, Pattern[x, Blank[]]]], Condition[integerResult, 7]], RuleDelayed[HoldPattern[nonBoolean[u, Pattern[x, Blank[]]]], Condition[callResult, upUnknown[x]]], RuleDelayed[HoldPattern[nonBoolean[u, Pattern[x, Blank[]]]], fallback]]"
    );
}

#[test]
fn upvalue_rhs_guards_share_the_guarded_body_option_scope() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "Options[optionHead] = {accept -> False}; \
             optionTag /: optionHead[optionTag, x_, OptionsPattern[]] := guarded[x, OptionValue[accept]] /; OptionValue[accept]; \
             optionTag /: optionHead[optionTag, x_, OptionsPattern[]] := fallback[x, OptionValue[accept]]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{optionHead[optionTag, 1], optionHead[optionTag, 1, accept -> True]}")
            .unwrap()
            .value
            .to_string(),
        "List[fallback[1, False], guarded[1, True]]"
    );
    assert_eq!(
        session
            .eval_text("UpValues[optionTag]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[optionHead[optionTag, Pattern[x, Blank[]], OptionsPattern[]]], Condition[guarded[x, OptionValue[accept]], OptionValue[accept]]], RuleDelayed[HoldPattern[optionHead[optionTag, Pattern[x, Blank[]], OptionsPattern[]]], fallback[x, OptionValue[accept]]]]"
    );
}

#[test]
fn immediate_and_delayed_upvalue_heads_split_guards_at_the_right_time() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "upInstallCount = 0; \
             UpSet[upSetHead[upSetTag, x_], Condition[(upInstallCount = upInstallCount + 1; \"up-set\"), (upInstallCount = upInstallCount + 1; True)]]; \
             TagSet[tagSetTag, tagSetHead[tagSetTag, x_], Condition[(upInstallCount = upInstallCount + 1; \"tag-set\"), (upInstallCount = upInstallCount + 1; True)]]; \
             UpSetDelayed[upSetDelayedHead[upSetDelayedTag, x_], Condition[\"up-set-delayed\", MatchQ[x, 1]]]; \
             TagSetDelayed[tagSetDelayedTag, tagSetDelayedHead[tagSetDelayedTag, x_], Condition[\"tag-set-delayed\", MatchQ[x, 1]]]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "{upInstallCount, upSetHead[upSetTag, 0], tagSetHead[tagSetTag, 0], upSetDelayedHead[upSetDelayedTag, 1], tagSetDelayedHead[tagSetDelayedTag, 1], upInstallCount}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[4, \"up-set\", \"tag-set\", \"up-set-delayed\", \"tag-set-delayed\", 4]"
    );
    assert_eq!(
        session
            .eval_text("UpValues[upSetTag]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[upSetHead[upSetTag, Pattern[x, Blank[]]]], Condition[\"up-set\", True]]]"
    );
    assert_eq!(
        session
            .eval_text("UpValues[tagSetDelayedTag]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[tagSetDelayedHead[tagSetDelayedTag, Pattern[x, Blank[]]]], Condition[\"tag-set-delayed\", MatchQ[x, 1]]]]"
    );
}

#[test]
fn tagunset_clears_every_upvalue_guard_variant_and_table_roundtrip_restores_them() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "u /: clearHead[u, x_] := one /; MatchQ[x, 1]; \
             u /: clearHead[u, x_] := two /; MatchQ[x, 2]; \
             savedUpGuards = UpValues[u]",
        )
        .unwrap();
    let before = session.eval_text("UpValues[u]").unwrap().value.to_string();

    session.eval_text("TagUnset[u, clearHead[u, z_]]").unwrap();
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("clearHead[u, 1]")
            .unwrap()
            .value
            .to_string(),
        "clearHead[u, 1]"
    );

    session.eval_text("UpValues[u] = savedUpGuards").unwrap();
    assert_eq!(
        session
            .eval_text("{clearHead[u, 1], clearHead[u, 2], clearHead[u, 3]}")
            .unwrap()
            .value
            .to_string(),
        "List[one, two, clearHead[u, 3]]"
    );
    assert_eq!(
        session.eval_text("UpValues[u]").unwrap().value.to_string(),
        before
    );
}

#[test]
fn wide_upvalue_identity_index_keeps_guard_variants_distinct() {
    let mut session = up_guard_session();

    session
        .eval_text(
            "Do[With[{k = i}, wideUpTag /: wideUpHead[wideUpTag, k, x_] := k], {i, 1, 32}]; \
             wideUpTag /: wideUpHead[wideUpTag, 99, x_] := oldOne /; MatchQ[x, 1]; \
             wideUpTag /: wideUpHead[wideUpTag, 99, x_] := two /; MatchQ[x, 2]; \
             wideUpTag /: wideUpHead[wideUpTag, 99, y_] := newOne /; MatchQ[y, 1]; \
             wideUpTag /: wideUpHead[wideUpTag, 99, x_] := other",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{wideUpHead[wideUpTag, 99, 1], wideUpHead[wideUpTag, 99, 2], wideUpHead[wideUpTag, 99, 3]}")
            .unwrap()
            .value
            .to_string(),
        "List[newOne, two, other]"
    );
    let Expr::Call { args, .. } = session.eval_text("UpValues[wideUpTag]").unwrap().value else {
        panic!("UpValues[wideUpTag] should be a List");
    };
    assert_eq!(args.len(), 35);
}
