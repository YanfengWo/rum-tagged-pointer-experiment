use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_array_callback_helpers() {
    let mut session = core_session();

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
    let mut session = core_session();

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

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
