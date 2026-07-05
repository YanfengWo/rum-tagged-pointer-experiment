use super::*;
use rum_runtime::{EvalError, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[path = "holding_and_control_part6/cleanup_throw.rs"]
mod cleanup_throw;

#[test]
fn abort_and_checkabort_cover_current_first_pass_subset() {
    let mut session = core_session();

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
    let mut session = core_session();

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
    let mut session = core_session();

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
fn top_level_break_and_continue_report_nofwd_messages() {
    let mut session = core_session();

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
