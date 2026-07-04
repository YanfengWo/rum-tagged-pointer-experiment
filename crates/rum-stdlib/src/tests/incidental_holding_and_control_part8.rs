use crate::install_core;
use rum_runtime::{Session, SessionRequestError, SessionRequestFailure};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn expect_input_prompt(error: SessionRequestFailure, prompt: &str) {
    match &error.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, prompt);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}

#[test]
fn generated_confirmation_failures_expose_current_oracle_metadata_slice() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FailureQ[$Canceled]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f][$Canceled]")
            .unwrap()
            .value
            .to_string(),
        "$Canceled"
    );
    assert_eq!(
        session
            .eval_text("Failsafe[f][Missing[]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[Missing[]], \"ConfirmationType\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Confirm\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[Failure[\"tag\", Association[\"Code\" -> 1]]], \"ConfirmationType\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Confirm\""
    );
    assert_eq!(
        session
            .eval_text(
                "Enclose[Confirm[Failure[\"tag\", Association[\"Code\" -> 1]]], \"Message\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "\"A failure of type \\\"tag\\\" occurred.\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Canceled], \"ConfirmationType\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Confirm\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Canceled], \"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"$Canceled encountered.\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[x, IntegerQ], \"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"IntegerQ[x] did not return True.\""
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[False], \"HeldTest\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[False]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmQuiet[First[f[]]], \"HeldMessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[First::nofirst]"
    );
}

#[test]
fn confirm_family_info_can_suspend_and_resume_inside_enclose() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "Enclose[ConfirmBy[x, IntegerQ, InputString[\"by info: \"]], \"Information\"]",
        )
        .expect_err("ConfirmBy information expression should suspend when it requests host input");
    expect_input_prompt(failure, "by info: ");
    let response = session
        .resume_eval_request_with_input("by-meta")
        .expect("resuming ConfirmBy information should finish property projection");
    assert_eq!(response.value.to_string(), "\"by-meta\"");

    let failure = session
        .eval_request(
            "Enclose[ConfirmMatch[x, _Integer, InputString[\"match info: \"]], \"Information\"]",
        )
        .expect_err(
            "ConfirmMatch information expression should suspend when it requests host input",
        );
    expect_input_prompt(failure, "match info: ");
    let response = session
        .resume_eval_request_with_input("match-meta")
        .expect("resuming ConfirmMatch information should finish property projection");
    assert_eq!(response.value.to_string(), "\"match-meta\"");

    let failure = session
        .eval_request(
            "Enclose[ConfirmAssert[False, InputString[\"assert info: \"]], \"Information\"]",
        )
        .expect_err(
            "ConfirmAssert information expression should suspend when it requests host input",
        );
    expect_input_prompt(failure, "assert info: ");
    let response = session
        .resume_eval_request_with_input("assert-meta")
        .expect("resuming ConfirmAssert information should finish property projection");
    assert_eq!(response.value.to_string(), "\"assert-meta\"");

    let failure = session
        .eval_request(
            "Enclose[ConfirmQuiet[InputString[\"body: \"]; First[f[]], All, InputString[\"quiet info: \"]], \"Information\"]",
        )
        .expect_err("ConfirmQuiet body should suspend before the message-producing expression");
    expect_input_prompt(failure, "body: ");
    let failure = session
        .resume_eval_request_with_input("continue")
        .expect_err("resuming ConfirmQuiet body should suspend in the information expression");
    expect_input_prompt(failure, "quiet info: ");
    let response = session
        .resume_eval_request_with_input("quiet-meta")
        .expect("resuming ConfirmQuiet information should finish property projection");
    assert_eq!(response.value.to_string(), "\"quiet-meta\"");
    assert!(response.messages.is_empty());
}
