use super::*;

#[test]
fn abort_and_checkabort_cover_current_first_pass_subset() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Abort[]").unwrap().value.to_string(),
        "$Aborted"
    );
    assert_eq!(
        session
            .eval_text("CheckAbort[Abort[], rescued]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("CheckAbort[Abort[], x = 1, PropagateAborts -> True]")
            .unwrap()
            .value
            .to_string(),
        "$Aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    assert_eq!(
        session
            .eval_text("CheckAbort[Abort[], rescued, {PropagateAborts -> False}]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(
        session
            .eval_text("CheckAbort[Abort[], rescued, \"PropagateAborts\" -> False]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(
        session
            .eval_text("CheckAbort[CheckAbort[Abort[], rescued, {PropagateAborts -> True}], outer]")
            .unwrap()
            .value
            .to_string(),
        "outer"
    );
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[CheckAbort[Abort[], rescued, {PropagateAborts -> True, PropagateAborts -> False}], outer]"
            )
            .unwrap()
            .value
            .to_string(),
        "outer"
    );
    let invalid_value = session
        .eval_text("CheckAbort[Abort[], rescued, PropagateAborts -> bad]")
        .unwrap();
    assert_eq!(invalid_value.value.to_string(), "rescued");
    assert!(
        invalid_value
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "opttfa")
    );
}

#[test]
fn withcleanup_runs_cleanup_on_values_throws_and_aborts() {
    let mut session = Session::new();

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[x = 1, x = 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Catch[WithCleanup[Throw[1, tag], x = 3], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "3");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[Abort[], x = 4]")
            .unwrap()
            .value
            .to_string(),
        "$Aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "4");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[Abort[], x = 1, x = 5]")
            .unwrap()
            .value
            .to_string(),
        "$Aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "5");
}

#[test]
fn failsafe_wrapper_preserves_failure_like_inputs_and_supports_test_forms() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Failsafe[Plus][1, $Failed, 3]")
            .unwrap()
            .value
            .to_string(),
        "$Failed"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[Function[Plus[Slot[1], Slot[1]]], Positive][4]")
            .unwrap()
            .value
            .to_string(),
        "8"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failsafe[Function[Plus[Slot[1], Slot[1]]], Positive][-1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failsafe[f, Function[p]][1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failsafe[f, Function[1]][1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[Divide, Function[UnsameQ[Slot[2], 0]], Function[Missing[\"ZeroDiv\"]]][1, 0]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"ZeroDiv\"]"
    );
}
