use super::*;

#[test]
fn immediate_and_delayed_ownvalue_rhs_guards_accept_or_decline() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("immediateAccept = 6 /; True").unwrap();
    session.eval_text("immediateDecline = 7 /; False").unwrap();
    session.eval_text("delayedAccept := 8 /; True").unwrap();
    session.eval_text("delayedDecline := 9 /; False").unwrap();

    let outcome = session
        .eval_text("{immediateAccept, immediateDecline, delayedAccept, delayedDecline}")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[6, immediateDecline, 8, delayedDecline]"
    );
    assert!(outcome.messages.is_empty());

    assert_eq!(
        session
            .eval_text("OwnValues[immediateAccept]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[immediateAccept], Condition[6, True]]]"
    );
    assert_eq!(
        session
            .eval_text("Definition[delayedDecline]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[SetDelayed[delayedDecline, Condition[9, False]]]"
    );
}

#[test]
fn non_true_ownvalue_guards_decline_and_valueq_reports_false() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("falseGuard := 7 /; False").unwrap();
    session.eval_text("symbolGuard := 8 /; undecided").unwrap();
    session.eval_text("integerGuard := 9 /; 1").unwrap();
    session
        .eval_text("callGuard := 10 /; unknownGuard[]")
        .unwrap();

    let outcome = session
        .eval_text(
            "{falseGuard, symbolGuard, integerGuard, callGuard, ValueQ[falseGuard], ValueQ[symbolGuard], ValueQ[integerGuard], ValueQ[callGuard]}",
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[falseGuard, symbolGuard, integerGuard, callGuard, False, False, False, False]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn ownvalue_guards_run_left_to_right_and_decline_before_the_body() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "ownGuardOrder = 0; ownBodyRuns = 0; orderedOwn := Condition[(ownBodyRuns = ownBodyRuns + 1; 8), Condition[(ownGuardOrder = ownGuardOrder * 10 + 1; True), (ownGuardOrder = ownGuardOrder * 10 + 2; False)]]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{orderedOwn, ownGuardOrder, ownBodyRuns}")
            .unwrap()
            .value
            .to_string(),
        "List[orderedOwn, 12, 0]"
    );
}

#[test]
fn declining_ownvalue_guards_fall_through_inside_block_and_module() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "scopedOwn = 100; {Block[{scopedOwn = 9}, scopedOwn := 7 /; False; ValueQ[scopedOwn]], scopedOwn, Module[{localOwn = 9}, localOwn := 8 /; False; ValueQ[localOwn]]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[False, 100, False]"
    );

    session
        .eval_text("dynamicGuardedOwn := 7 /; False")
        .unwrap();
    let symbol = Symbol::new("Global`", "dynamicGuardedOwn");
    let depth = session.push_dynamic_own_value_binding(symbol, Some(Expr::small_integer(11)));
    assert_eq!(
        session
            .eval_text("dynamicGuardedOwn")
            .unwrap()
            .value
            .to_string(),
        "11"
    );
    session.truncate_dynamic_own_value_bindings(depth);
}

#[test]
fn a_later_guarded_ownvalue_replaces_the_single_existing_slot() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "ownReplacementCount = 0; replacedOwn := 1 /; (ownReplacementCount = ownReplacementCount + 1; True); replacedOwn := 2 /; (ownReplacementCount = ownReplacementCount + 10; True)",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{replacedOwn, ownReplacementCount}")
            .unwrap()
            .value
            .to_string(),
        "List[2, 10]"
    );
    assert_eq!(
        session
            .eval_text("OwnValues[replacedOwn]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[replacedOwn], Condition[2, CompoundExpression[Set[ownReplacementCount, Plus[ownReplacementCount, 10]], True]]]]"
    );
}

#[test]
fn ownvalue_rhs_guard_resumes_without_replaying_the_guard() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "ownGuardResumeCount = 0; suspendingOwn := 7 /; (ownGuardResumeCount = ownGuardResumeCount + 1; MatchQ[InputString[\"own guard: \"], \"yes\"])",
        )
        .unwrap();

    let failure = session
        .eval_request("{suspendingOwn, ownGuardResumeCount}")
        .expect_err("ownvalue rhs guard should suspend when it requests input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "own guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("accepting resumed ownvalue guard should evaluate its stored body");
    assert_eq!(response.value.to_string(), "List[7, 1]");
}
