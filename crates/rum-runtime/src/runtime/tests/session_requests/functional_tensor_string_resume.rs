use super::*;

#[test]
fn resume_eval_request_with_input_can_finish_array_callback_helpers() {
    let mut session = Session::new();

    let first = session
        .eval_request("ArrayReduce[Function[List[InputString[\"reduce: \"], Slot[1]]], {a, b}, {}]")
        .expect_err("ArrayReduce elementwise callback should suspend on the first item");
    assert_input_prompt(&first.error, "reduce: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("ArrayReduce should continue to the second item after resuming");
    assert_input_prompt(&second.error, "reduce: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("ArrayReduce should finish after resumed callbacks");
    assert_eq!(
        response.value.to_string(),
        "List[List[\"one\", a], List[\"two\", b]]"
    );

    let first = session
        .eval_request("ArrayFilter[Function[List[InputString[\"filter: \"], Slot[1]]], {a, b}, 0]")
        .expect_err("ArrayFilter callback should suspend on the first window");
    assert_input_prompt(&first.error, "filter: ");

    let second = session
        .resume_eval_request_with_input("left")
        .expect_err("ArrayFilter should continue to the second window after resuming");
    assert_input_prompt(&second.error, "filter: ");

    let response = session
        .resume_eval_request_with_input("right")
        .expect("ArrayFilter should finish after resumed callbacks");
    assert_eq!(
        response.value.to_string(),
        "List[List[\"left\", List[a]], List[\"right\", List[b]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_inner_term_and_reducer_callbacks() {
    let mut session = Session::new();

    let term = session
        .eval_request(
            "Inner[Function[List[InputString[\"term: \"], Slot[1], Slot[2]]], {a}, {b}, Function[List[InputString[\"inner: \"], Slot[1]]]]",
        )
        .expect_err("Inner term callback should suspend before reduction");
    assert_input_prompt(&term.error, "term: ");

    let reducer = session
        .resume_eval_request_with_input("mul")
        .expect_err("Inner reducer callback should run after the term resumes");
    assert_input_prompt(&reducer.error, "inner: ");

    let response = session
        .resume_eval_request_with_input("sum")
        .expect("Inner should finish after the reducer resumes");
    assert_eq!(
        response.value.to_string(),
        "List[\"sum\", List[\"mul\", a, b]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_string_apply_list_callbacks() {
    let mut session = Session::new();

    let first = session
        .eval_request("StringApply[Function[List[InputString[\"apply: \"]]], {\"ab\", \"cd\"}]")
        .expect_err("StringApply callback should suspend on the first string");
    assert_input_prompt(&first.error, "apply: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("StringApply should continue to the second string after resuming");
    assert_input_prompt(&second.error, "apply: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("StringApply should finish after resumed callbacks");
    assert_eq!(response.value.to_string(), "List[\"one\", \"two\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_delayed_string_rules() {
    let mut session = Session::new();

    let first = session
        .eval_request("StringReplace[{\"a\", \"a\"}, \"a\" :> InputString[\"replace: \"]]")
        .expect_err("StringReplace delayed rule should suspend on the first string");
    assert_input_prompt(&first.error, "replace: ");

    let second = session
        .resume_eval_request_with_input("x")
        .expect_err("StringReplace should continue to the second string after resuming");
    assert_input_prompt(&second.error, "replace: ");

    let response = session
        .resume_eval_request_with_input("y")
        .expect("StringReplace should finish after resumed delayed rules");
    assert_eq!(response.value.to_string(), "List[\"x\", \"y\"]");

    let first = session
        .eval_request("StringCases[{\"a\", \"b\"}, x_ :> InputString[\"case: \"]]")
        .expect_err("StringCases delayed rule should suspend on the first string");
    assert_input_prompt(&first.error, "case: ");

    let second = session
        .resume_eval_request_with_input("a")
        .expect_err("StringCases should continue to the second string after resuming");
    assert_input_prompt(&second.error, "case: ");

    let response = session
        .resume_eval_request_with_input("b")
        .expect("StringCases should finish after resumed delayed rules");
    assert_eq!(response.value.to_string(), "List[List[\"a\"], List[\"b\"]]");
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
