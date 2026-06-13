use super::*;

#[test]
fn resume_eval_request_with_input_can_finish_repeated_timing_samples() {
    let mut session = Session::new();

    let first = session
        .eval_request("RepeatedTiming[InputString[\"rt: \"]]")
        .expect_err("RepeatedTiming should suspend on the first sample");
    assert_input_prompt(&first.error, "rt: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("RepeatedTiming should continue to the second sample after resuming");
    assert_input_prompt(&second.error, "rt: ");

    let third = session
        .resume_eval_request_with_input("two")
        .expect_err("RepeatedTiming should continue to the third sample after resuming");
    assert_input_prompt(&third.error, "rt: ");

    let response = session
        .resume_eval_request_with_input("three")
        .expect("RepeatedTiming should finish after all resumed samples");
    let rendered = response.value.to_string();
    assert!(rendered.starts_with("List["));
    assert!(rendered.ends_with(", \"three\"]"));
}

#[test]
fn resume_eval_request_with_input_can_finish_constraint_limit_and_fallback() {
    let mut session = Session::new();

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
fn resume_eval_request_with_input_can_finish_reap_handler_rendering() {
    let mut session = Session::new();

    let first = session
        .eval_request(
            "Reap[Sow[1, a]; Sow[2, b]; done, _, Function[List[InputString[\"reap: \"], Slot[1], Slot[2]]]]",
        )
        .expect_err("Reap handler should suspend on the first bucket");
    assert_input_prompt(&first.error, "reap: ");

    let second = session
        .resume_eval_request_with_input("x")
        .expect_err("Reap handler should continue to the second bucket after resuming");
    assert_input_prompt(&second.error, "reap: ");

    let response = session
        .resume_eval_request_with_input("y")
        .expect("Reap should finish after resumed handler buckets");
    assert_eq!(
        response.value.to_string(),
        "List[done, List[List[\"x\", a, List[1]], List[\"y\", b, List[2]]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_trace_capture() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Trace[InputString[\"trace: \"]]")
        .expect_err("Trace capture should suspend through the traced expression");
    assert_input_prompt(&failure.error, "trace: ");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("Trace should return the captured trace after input resumes");
    assert!(response.value.to_string().starts_with("List["));
}

#[test]
fn resume_eval_request_with_input_can_finish_test_report_input_and_sametest() {
    let mut session = Session::new();

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

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
