use super::*;

#[test]
fn resume_eval_request_with_input_can_finish_nest_callback_loop() {
    let mut session = Session::new();

    let first = session
        .eval_request("Nest[Function[List[Slot[1], InputString[\"nest: \"]]], seed, 2]")
        .expect_err("Nest callback should suspend on the first iteration");
    assert_input_prompt(&first.error, "nest: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("Nest should continue to the second iteration after resuming the first");
    assert_input_prompt(&second.error, "nest: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("Nest should finish after the second resumed callback");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed, \"one\"], \"two\"]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_nestlist_callback_loop() {
    let mut session = Session::new();

    let failure = session
        .eval_request("NestList[Function[List[Slot[1], InputString[\"nestlist: \"]]], seed, 1]")
        .expect_err("NestList callback should suspend");
    assert_input_prompt(&failure.error, "nestlist: ");

    let response = session
        .resume_eval_request_with_input("one")
        .expect("NestList should preserve collected history after resuming");

    assert_eq!(
        response.value.to_string(),
        "List[seed, List[seed, \"one\"]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_fixedpoint_callback_loop() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "FixedPoint[Function[If[SameQ[Slot[1], seed], InputString[\"fixed: \"], Slot[1]]], seed]",
        )
        .expect_err("FixedPoint callback should suspend");
    assert_input_prompt(&failure.error, "fixed: ");

    let response = session
        .resume_eval_request_with_input("done")
        .expect("FixedPoint should continue until the resumed value is stable");

    assert_eq!(response.value.to_string(), "\"done\"");
}

#[test]
fn resume_eval_request_with_input_can_finish_fixedpointlist_callback_loop() {
    let mut session = Session::new();

    let failure = session
        .eval_request(
            "FixedPointList[Function[If[SameQ[Slot[1], seed], InputString[\"fixedlist: \"], Slot[1]]], seed]",
        )
        .expect_err("FixedPointList callback should suspend");
    assert_input_prompt(&failure.error, "fixedlist: ");

    let response = session
        .resume_eval_request_with_input("done")
        .expect("FixedPointList should preserve collected history after resuming");

    assert_eq!(response.value.to_string(), "List[seed, \"done\", \"done\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_nestwhile_test_and_step() {
    let mut session = Session::new();

    let test = session
        .eval_request(
            "NestWhile[Function[List[Slot[1], InputString[\"step: \"]]], seed, Function[If[SameQ[Slot[1], seed], SameQ[InputString[\"test: \"], \"yes\"], False]]]",
        )
        .expect_err("NestWhile test should suspend before the first step");
    assert_input_prompt(&test.error, "test: ");

    let step = session
        .resume_eval_request_with_input("yes")
        .expect_err("NestWhile step should suspend after the truthy test resumes");
    assert_input_prompt(&step.error, "step: ");

    let response = session
        .resume_eval_request_with_input("s")
        .expect("NestWhile should finish when the next test is false");

    assert_eq!(response.value.to_string(), "List[seed, \"s\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_nestwhilelist_test_and_step() {
    let mut session = Session::new();

    let test = session
        .eval_request(
            "NestWhileList[Function[List[Slot[1], InputString[\"steplist: \"]]], seed, Function[If[SameQ[Slot[1], seed], SameQ[InputString[\"testlist: \"], \"yes\"], False]]]",
        )
        .expect_err("NestWhileList test should suspend before the first step");
    assert_input_prompt(&test.error, "testlist: ");

    let step = session
        .resume_eval_request_with_input("yes")
        .expect_err("NestWhileList step should suspend after the truthy test resumes");
    assert_input_prompt(&step.error, "steplist: ");

    let response = session
        .resume_eval_request_with_input("s")
        .expect("NestWhileList should preserve collected history after resuming");

    assert_eq!(response.value.to_string(), "List[seed, List[seed, \"s\"]]");
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
