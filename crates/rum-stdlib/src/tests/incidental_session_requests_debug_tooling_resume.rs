use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}

#[test]
fn resume_eval_request_with_input_can_finish_constraint_limit_and_fallback() {
    let mut session = core_session();

    let limit = session
        .eval_request("TimeConstrained[1 + 1, If[SameQ[InputString[\"limit: \"], \"go\"], 1, 1]]")
        .expect_err("TimeConstrained limit evaluation should suspend");
    assert_input_prompt(&limit.error, "limit: ");

    let response = session
        .resume_eval_request_with_input("go")
        .expect("TimeConstrained should evaluate the body after the limit resumes");
    assert_eq!(response.value.to_string(), "2");

    let fallback = session
        .eval_request("MemoryConstrained[Range[100], 20, InputString[\"memfb: \"]]")
        .expect_err("MemoryConstrained fallback evaluation should suspend");
    assert_input_prompt(&fallback.error, "memfb: ");

    let response = session
        .resume_eval_request_with_input("fallback")
        .expect("MemoryConstrained should finish after the fallback resumes");
    assert_eq!(response.value.to_string(), "\"fallback\"");
}

#[test]
fn resume_eval_request_with_input_can_finish_test_report_input_and_sametest() {
    let mut session = core_session();

    let input = session
        .eval_request("Lookup[TestReport[{TestCreate[InputString[\"test: \"], \"ok\"]}], \"TestsSucceededCount\"]")
        .expect_err("TestReport test input capture should suspend");
    assert_input_prompt(&input.error, "test: ");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("TestReport should finish the test case after input resumes");
    assert_eq!(response.value.to_string(), "1");

    let same_test = session
        .eval_request(
            "Lookup[TestReport[{TestCreate[1, 2, {SameTest -> Function[SameQ[InputString[\"same: \"], \"yes\"]]}]}], \"TestsSucceededCount\"]",
        )
        .expect_err("TestReport SameTest evaluation should suspend");
    assert_input_prompt(&same_test.error, "same: ");

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("TestReport should finish after SameTest resumes");
    assert_eq!(response.value.to_string(), "1");
}
