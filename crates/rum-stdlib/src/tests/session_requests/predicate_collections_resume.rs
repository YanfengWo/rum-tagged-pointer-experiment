use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_preserves_duplicate_free_pair_progress() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "DuplicateFreeQ[{a, b, c}, SameTest -> Function[SameQ[InputString[\"same: \"], \"yes\"]]]",
        )
        .expect_err("DuplicateFreeQ SameTest should suspend on the first pair");
    assert_input_prompt(&first.error, "same: ");

    let second = session
        .resume_eval_request_with_input("no")
        .expect_err("DuplicateFreeQ should continue to the second pair");
    assert_input_prompt(&second.error, "same: ");

    let third = session
        .resume_eval_request_with_input("no")
        .expect_err("DuplicateFreeQ should continue to the third pair");
    assert_input_prompt(&third.error, "same: ");

    let response = session
        .resume_eval_request_with_input("no")
        .expect("DuplicateFreeQ should finish after all pairs resume");
    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_preserves_same_test_matrix_progress() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "ContainsNone[{a, b}, {x, y}, SameTest -> Function[SameQ[InputString[\"matrix: \"], \"yes\"]]]",
        )
        .expect_err("ContainsNone SameTest should suspend on the first pair");
    assert_input_prompt(&first.error, "matrix: ");

    for completed_pair in 1..=4 {
        let result = session.resume_eval_request_with_input("no");
        if completed_pair < 4 {
            let failure = result.expect_err("ContainsNone should continue to the next pair");
            assert_input_prompt(&failure.error, "matrix: ");
        } else {
            let response = result.expect("ContainsNone should finish after all pairs resume");
            assert_eq!(response.value.to_string(), "True");
        }
    }
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
