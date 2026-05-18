use super::*;

fn expect_input_prompt(error: SessionRequestFailure, prompt: &str) {
    match &error.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, prompt);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}

#[test]
fn enclose_handler_can_suspend_and_resume_through_host_input() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Enclose[Confirm[$Failed], Function[InputString[\"handler: \"]]]")
        .expect_err("Enclose handler should suspend when it requests host input");
    expect_input_prompt(failure, "handler: ");

    let response = session
        .resume_eval_request_with_input("handled")
        .expect("resuming Enclose handler should return the handler value");
    assert_eq!(response.value.to_string(), "\"handled\"");
}

#[test]
fn confirm_information_can_suspend_and_resume_inside_enclose() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Enclose[Confirm[$Failed, InputString[\"info: \"]], \"Information\"]")
        .expect_err("Confirm information expression should suspend when it requests host input");
    expect_input_prompt(failure, "info: ");

    let response = session
        .resume_eval_request_with_input("metadata")
        .expect("resuming Confirm information should finish Enclose property projection");
    assert_eq!(response.value.to_string(), "\"metadata\"");
}

#[test]
fn confirm_family_info_can_suspend_and_resume_inside_enclose() {
    let mut session = Session::new();

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

#[test]
fn confirm_predicate_and_test_can_suspend_and_resume_inside_enclose() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "Enclose[ConfirmBy[x, Function[MatchQ[InputString[\"by test: \"], \"yes\"]]]]",
        )
        .expect_err("ConfirmBy predicate should suspend when it requests host input");
    expect_input_prompt(failure, "by test: ");
    let response = session
        .resume_eval_request_with_input("yes")
        .expect("truthy resumed ConfirmBy predicate should return the original expression");
    assert_eq!(response.value.to_string(), "x");

    let failure = session
        .eval_request("Enclose[ConfirmAssert[MatchQ[InputString[\"assert test: \"], \"yes\"]]]")
        .expect_err("ConfirmAssert test should suspend when it requests host input");
    expect_input_prompt(failure, "assert test: ");
    let response = session
        .resume_eval_request_with_input("yes")
        .expect("truthy resumed ConfirmAssert test should return Null");
    assert_eq!(response.value.to_string(), "Null");
}

#[test]
fn generated_confirmation_failures_expose_current_oracle_metadata_slice() {
    let mut session = Session::new();

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
        "Hold[First::first]"
    );
}
