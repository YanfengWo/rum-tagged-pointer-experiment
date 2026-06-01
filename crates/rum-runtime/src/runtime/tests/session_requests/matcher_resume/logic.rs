use super::*;

#[test]
fn logic_short_circuit_can_avoid_or_resume_input_requests() {
    let mut session = Session::new();

    let response = session
        .eval_request("And[False, InputString[\"rhs: \"]]")
        .expect("And should short-circuit before requesting input");
    assert_eq!(response.value.to_string(), "False");
    assert!(session.pending_input_request().is_none());

    let failure = session
        .eval_request("And[InputString[\"lhs: \"], False]")
        .expect_err("leading InputString should still suspend the request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("text")
        .expect("resume should continue And evaluation after the suspended input");

    assert_eq!(response.value.to_string(), "False");
    assert!(session.pending_input_request().is_none());

    let response = session
        .eval_request("Nand[False, InputString[\"rhs: \"]]")
        .expect("Nand should short-circuit before requesting input");
    assert_eq!(response.value.to_string(), "True");
    assert!(session.pending_input_request().is_none());

    let failure = session
        .eval_request("Nand[InputString[\"lhs: \"], False]")
        .expect_err("leading InputString should still suspend the Nand request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("text")
        .expect("resume should continue Nand evaluation after the suspended input");

    assert_eq!(response.value.to_string(), "True");
    assert!(session.pending_input_request().is_none());

    let response = session
        .eval_request("Nor[True, InputString[\"rhs: \"]]")
        .expect("Nor should short-circuit before requesting input");
    assert_eq!(response.value.to_string(), "False");
    assert!(session.pending_input_request().is_none());

    let failure = session
        .eval_request("Nor[InputString[\"lhs: \"], True]")
        .expect_err("leading InputString should still suspend the Nor request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("text")
        .expect("resume should continue Nor evaluation after the suspended input");

    assert_eq!(response.value.to_string(), "False");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn truth_aggregates_short_circuit_and_resume_input_requests() {
    let mut session = Session::new();

    let response = session
        .eval_request(
            "AnyTrue[{1, 2}, Function[x, If[SameQ[x, 1], True, MatchQ[InputString[\"rhs: \"], \"yes\"]]]]",
        )
        .expect("AnyTrue should short-circuit before requesting later inputs");
    assert_eq!(response.value.to_string(), "True");
    assert!(session.pending_input_request().is_none());

    let failure = session
        .eval_request(
            "AllTrue[{1, 2}, Function[x, If[SameQ[x, 1], True, MatchQ[InputString[\"rhs: \"], \"yes\"]]]]",
        )
        .expect_err("AllTrue should suspend when a later criterion application requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish AllTrue after the suspended criterion resolves");

    assert_eq!(response.value.to_string(), "True");
    assert!(session.pending_input_request().is_none());
}
