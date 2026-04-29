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
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[CheckAbort[Abort[], rescued, {}, PropagateAborts -> True], outer]"
            )
            .unwrap()
            .value
            .to_string(),
        "outer"
    );
    assert_eq!(
        session
            .eval_text("CheckAbort[Abort[], rescued, {}, {}]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[Abort[], rescued, PropagateAborts -> False, PropagateAborts -> True]"
            )
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    session
        .eval_text("SetOptions[CheckAbort, PropagateAborts -> True]")
        .unwrap();
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[CheckAbort[Abort[], rescued, {}], outer, PropagateAborts -> False]"
            )
            .unwrap()
            .value
            .to_string(),
        "outer"
    );
    assert_eq!(
        session
            .eval_text("CheckAbort[CheckAbort[Abort[], rescued, PropagateAborts -> False], outer, PropagateAborts -> False]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    session
        .eval_text("SetOptions[CheckAbort, PropagateAborts -> Automatic]")
        .unwrap();
    let invalid_extra_option = session.eval_text("CheckAbort[1, 2, 3, 4]").unwrap();
    assert_eq!(
        invalid_extra_option.value.to_string(),
        "CheckAbort[1, 2, 3, 4]"
    );
    assert!(
        invalid_extra_option
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "nonopt")
    );
    let missing_args = session.eval_text("CheckAbort[]").unwrap();
    assert_eq!(missing_args.value.to_string(), "CheckAbort[]");
    assert!(
        missing_args
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "argrx")
    );
    let missing_fallback = session.eval_text("CheckAbort[1]").unwrap();
    assert_eq!(missing_fallback.value.to_string(), "CheckAbort[1]");
    assert!(
        missing_fallback
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "argr")
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

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Catch[WithCleanup[Throw[1, tag], x = 9, x = 6], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "6");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("CheckAbort[WithCleanup[x = 1; done, (x = 7; Abort[])], aborted]")
            .unwrap()
            .value
            .to_string(),
        "aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "7");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[x = 1; done, x = 2; Return[20]]")
            .unwrap()
            .value
            .to_string(),
        "done"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Function[WithCleanup[x = 1; Return[10], x = 2; Return[20]]; 99][]")
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text(
                "Catch[Function[WithCleanup[x = 1; Throw[10, tag], x = 2; Return[20]]; 99][], tag]"
            )
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("CheckAbort[Function[WithCleanup[x = 1; Abort[], x = 2; Return[20]]; 99][], aborted]")
            .unwrap()
            .value
            .to_string(),
        "aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");
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
    assert_eq!(
        session.eval_text("Failsafe[]").unwrap().value.to_string(),
        "Failsafe[]"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, t, g, h][x]")
            .unwrap()
            .value
            .to_string(),
        "Failsafe[f, t, g, h][x]"
    );
}
