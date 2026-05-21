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
fn checkabort_validates_entire_option_lists_before_evaluating_body() {
    let mut session = Session::new();

    session.eval_text("propagate = True").unwrap();
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[CheckAbort[Abort[], rescued, PropagateAborts -> propagate], outer]"
            )
            .unwrap()
            .value
            .to_string(),
        "outer"
    );
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[CheckAbort[Abort[], rescued, PropagateAborts :> propagate], outer]"
            )
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );

    let unknown_after_valid = session
        .eval_text("CheckAbort[Abort[], rescued, {PropagateAborts -> False, BadOpt -> 1}]")
        .unwrap();
    assert_eq!(
        unknown_after_valid.value.to_string(),
        "CheckAbort[Abort[], rescued, List[Rule[PropagateAborts, False], Rule[BadOpt, 1]]]"
    );
    assert!(
        unknown_after_valid
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "optx")
    );

    let non_option_after_valid = session
        .eval_text("CheckAbort[Abort[], rescued, {PropagateAborts -> False, 3}]")
        .unwrap();
    assert_eq!(
        non_option_after_valid.value.to_string(),
        "CheckAbort[Abort[], rescued, List[Rule[PropagateAborts, False], 3]]"
    );
    assert!(non_option_after_valid.messages.iter().any(|message| {
        message.symbol == "CheckAbort"
            && message.tag == "nonopt"
            && message
                .text
                .contains("List[Rule[PropagateAborts, False], 3]")
    }));

    let top_level_unknown_after_valid = session
        .eval_text("CheckAbort[Abort[], rescued, PropagateAborts -> False, BadOpt -> 1]")
        .unwrap();
    assert_eq!(
        top_level_unknown_after_valid.value.to_string(),
        "CheckAbort[Abort[], rescued, Rule[PropagateAborts, False], Rule[BadOpt, 1]]"
    );
    assert!(
        top_level_unknown_after_valid
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "optx")
    );

    let side_effect_blocked_by_invalid_list = session
        .eval_text("x = 0; CheckAbort[Abort[], rescued, {PropagateAborts -> (x = 1; False), 3}]; x")
        .unwrap();
    assert_eq!(side_effect_blocked_by_invalid_list.value.to_string(), "0");
    assert!(
        side_effect_blocked_by_invalid_list
            .messages
            .iter()
            .any(|message| message.symbol == "CheckAbort" && message.tag == "nonopt")
    );
}

#[test]
fn exit_reports_a_request_level_code_and_stops_evaluation() {
    let mut session = Session::new();

    match session
        .eval_text("Exit[7]")
        .expect_err("Exit should terminate the current evaluation")
    {
        EvalError::Exit { code } => assert_eq!(code, 7),
        other => panic!("expected Exit control flow, got {other:?}"),
    }

    let invalid_code = session.eval_text("Exit[foo]").unwrap();
    assert_eq!(invalid_code.value.to_string(), "Exit[foo]");
    assert!(
        invalid_code
            .messages
            .iter()
            .any(|message| message.symbol == "Exit" && message.tag == "intm")
    );

    let invalid_arity = session.eval_text("Exit[1, 2]").unwrap();
    assert!(
        invalid_arity
            .messages
            .iter()
            .any(|message| message.symbol == "Exit" && message.tag == "argt")
    );

    let failure = session
        .eval_script_request("x = 1\nExit[5]\nx = 2")
        .expect_err("script Exit should surface at the request boundary");
    assert!(matches!(
        failure.error,
        SessionRequestError::Exit { code: 5 }
    ));
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
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

    let cleanup_throw = session
        .eval_text("Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], tag]")
        .expect_err("cleanup-time Throw with the active tag should bypass the current Catch");
    match cleanup_throw {
        EvalError::Thrown { value, tag, .. } => {
            assert_eq!(value.to_string(), "1");
            assert_eq!(tag.to_string(), "tag");
        }
        other => panic!("expected primary throw to bypass current Catch, got {other:?}"),
    }
    assert_eq!(
        session
            .eval_text("Catch[Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], tag], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    let dual_uncaught = session
        .eval_request("WithCleanup[Throw[1, tag], Throw[2, tag]]")
        .expect_err("top-level active cleanup throw should report cleanup and primary throws");
    match dual_uncaught.error {
        SessionRequestError::Other { message } => assert_eq!(
            message,
            "uncaught throw: 2 with tag tag; uncaught throw: 1 with tag tag"
        ),
        other => panic!("expected dual uncaught throw request error, got {other:?}"),
    }
    assert_eq!(dual_uncaught.messages.len(), 2);
    assert_eq!(dual_uncaught.messages[0].symbol, "Throw");
    assert_eq!(dual_uncaught.messages[0].tag, "nocatch");
    assert_eq!(
        dual_uncaught.messages[0].text,
        "Uncaught Throw[2, tag] returned to top level."
    );
    assert_eq!(dual_uncaught.messages[1].symbol, "Throw");
    assert_eq!(dual_uncaught.messages[1].tag, "nocatch");
    assert_eq!(
        dual_uncaught.messages[1].text,
        "Uncaught Throw[1, tag] returned to top level."
    );
    match session
        .eval_text("Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], other]")
        .expect_err("nonmatching Catch should keep both cleanup and primary throw information")
    {
        EvalError::ThrownBypassingCatch {
            value,
            tag,
            cleanup_value,
            cleanup_tag,
            ..
        } => {
            assert_eq!(value.to_string(), "1");
            assert_eq!(tag.to_string(), "tag");
            assert_eq!(cleanup_value.to_string(), "2");
            assert_eq!(cleanup_tag.to_string(), "tag");
        }
        other => panic!("expected bypassing throw with cleanup context, got {other:?}"),
    }

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
fn uncaught_throw_request_messages_follow_wolfram_nocatch_shape() {
    let mut session = Session::new();

    let plain = session
        .eval_request("Throw[1]")
        .expect_err("top-level Throw should fail the request");
    assert_eq!(plain.messages.len(), 1);
    assert_eq!(plain.messages[0].symbol, "Throw");
    assert_eq!(plain.messages[0].tag, "nocatch");
    assert_eq!(
        plain.messages[0].text,
        "Uncaught Throw[1] returned to top level."
    );

    let through_active_catch = session
        .eval_request("Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], tag]")
        .expect_err("cleanup-time Throw should force the primary throw past active Catch");
    assert_eq!(through_active_catch.messages.len(), 1);
    assert_eq!(through_active_catch.messages[0].symbol, "Throw");
    assert_eq!(through_active_catch.messages[0].tag, "nocatch");
    assert_eq!(
        through_active_catch.messages[0].text,
        "Uncaught Throw[1, tag] returned to top level."
    );

    let cleanup_only = session
        .eval_request("Catch[WithCleanup[Throw[1, tag], Throw[2, other]], tag]")
        .expect_err("nonmatching cleanup Throw should escape before the primary is caught");
    assert_eq!(cleanup_only.messages.len(), 1);
    assert_eq!(cleanup_only.messages[0].symbol, "Throw");
    assert_eq!(cleanup_only.messages[0].tag, "nocatch");
    assert_eq!(
        cleanup_only.messages[0].text,
        "Uncaught Throw[2, other] returned to top level."
    );

    let explicit_none = session
        .eval_request("Throw[1, None]")
        .expect_err("explicit None tag should still be visible in the diagnostic");
    assert_eq!(explicit_none.messages.len(), 1);
    assert_eq!(explicit_none.messages[0].symbol, "Throw");
    assert_eq!(explicit_none.messages[0].tag, "nocatch");
    assert_eq!(
        explicit_none.messages[0].text,
        "Uncaught Throw[1, None] returned to top level."
    );

    let explicit_none_cleanup = session
        .eval_request("WithCleanup[Throw[1, None], Throw[2, None]]")
        .expect_err("explicit None tags should be preserved through cleanup throw diagnostics");
    assert_eq!(explicit_none_cleanup.messages.len(), 2);
    assert_eq!(
        explicit_none_cleanup.messages[0].text,
        "Uncaught Throw[2, None] returned to top level."
    );
    assert_eq!(
        explicit_none_cleanup.messages[1].text,
        "Uncaught Throw[1, None] returned to top level."
    );

    session.eval_text("Off[Throw::nocatch]").unwrap();
    let suppressed = session
        .eval_request("Throw[1, tag]")
        .expect_err("Off[Throw::nocatch] suppresses only the diagnostic message");
    assert!(suppressed.messages.is_empty());
}

#[test]
fn top_level_break_and_continue_report_wolfram_nofwd_messages() {
    let mut session = Session::new();

    let break_failure = session
        .eval_request("Break[]")
        .expect_err("top-level Break should fail the request");
    assert_eq!(break_failure.messages.len(), 1);
    assert_eq!(break_failure.messages[0].symbol, "Break");
    assert_eq!(break_failure.messages[0].tag, "nofwd");
    assert_eq!(
        break_failure.messages[0].text,
        "No enclosing For, While, Until or Do found for Break[]."
    );

    let continue_failure = session
        .eval_request("Continue[]")
        .expect_err("top-level Continue should fail the request");
    assert_eq!(continue_failure.messages.len(), 1);
    assert_eq!(continue_failure.messages[0].symbol, "Continue");
    assert_eq!(continue_failure.messages[0].tag, "nofwd");
    assert_eq!(
        continue_failure.messages[0].text,
        "No enclosing For, While, Until or Do found for Continue[]."
    );

    session.eval_text("Off[Break::nofwd]").unwrap();
    let suppressed_break = session
        .eval_request("Break[]")
        .expect_err("Off[Break::nofwd] suppresses only the diagnostic message");
    assert!(suppressed_break.messages.is_empty());
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
            .eval_text("Failsafe[f, Positive][-1][All]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"MessageTemplate\", \"The test failed on the arguments `1`.\"], RuleDelayed[\"MessageParameters\", List[HoldForm[-1]]], Rule[\"Arguments\", Hold[-1]]]"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Positive][-1][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"The test failed on the arguments -1.\""
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f, Positive][-1][\"Function\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"Function\"]"
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
