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
fn assert_function_none_restores_normal_assert_control() {
    let mut session = Session::new();

    session.eval_text("$AssertFunction = None").unwrap();
    assert_eq!(
        session
            .eval_text("Assert[False]")
            .unwrap()
            .value
            .to_string(),
        "Assert[False]"
    );

    assert_eq!(
        session.eval_text("On[Assert]").unwrap().value.to_string(),
        "Null"
    );
    let asserted = session.eval_text("Assert[False]").unwrap();
    assert_eq!(asserted.value.to_string(), "Null");
    assert_eq!(asserted.messages.len(), 1);
    assert_eq!(asserted.messages[0].symbol, "Assert");
    assert_eq!(asserted.messages[0].tag, "asrtf");
    assert_eq!(asserted.messages[0].text, "Assertion False failed.");

    assert_eq!(
        session.eval_text("Off[Assert]").unwrap().value.to_string(),
        "Null"
    );
    session.eval_text("$AssertFunction = Identity").unwrap();
    let callback = session.eval_text("Assert[False]").unwrap();
    assert_eq!(callback.value.to_string(), "Null");
    assert!(callback.messages.is_empty());
}
