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
fn resume_eval_request_with_input_can_finish_while_sequence_helpers() {
    let mut session = core_session();

    let first = session
        .eval_request("TakeWhile[{a, b, c}, Function[SameQ[InputString[\"take: \"], \"yes\"]]]")
        .expect_err("TakeWhile criterion should suspend on the first item");
    assert_input_prompt(&first.error, "take: ");

    let second = session
        .resume_eval_request_with_input("yes")
        .expect_err("TakeWhile should continue to the second item after resuming");
    assert_input_prompt(&second.error, "take: ");

    let response = session
        .resume_eval_request_with_input("no")
        .expect("TakeWhile should finish at the first falsy resumed criterion");
    assert_eq!(response.value.to_string(), "List[a]");

    let first = session
        .eval_request(
            "LengthWhile[<|a -> 1, b -> 2|>, Function[SameQ[InputString[\"len: \"], \"yes\"]]]",
        )
        .expect_err("LengthWhile criterion should suspend on the first item");
    assert_input_prompt(&first.error, "len: ");

    let second = session
        .resume_eval_request_with_input("yes")
        .expect_err("LengthWhile should continue to the second item after resuming");
    assert_input_prompt(&second.error, "len: ");

    let response = session
        .resume_eval_request_with_input("no")
        .expect("LengthWhile should return the accepted prefix length");
    assert_eq!(response.value.to_string(), "1");
}
