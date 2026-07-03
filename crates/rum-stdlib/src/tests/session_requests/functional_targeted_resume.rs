use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_mapat_callback_loop() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "MapAt[Function[List[InputString[\"mapat: \"], Slot[1]]], {a, b}, {{1}, {2}}]",
        )
        .expect_err("MapAt callback should suspend on the first target");
    assert_input_prompt(&first.error, "mapat: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("MapAt should continue to the second target after resuming the first");
    assert_input_prompt(&second.error, "mapat: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("MapAt should finish after the second resumed callback");

    assert_eq!(
        response.value.to_string(),
        "List[List[\"one\", a], List[\"two\", b]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_can_finish_subsetmap_callback_and_final_eval() {
    let mut session = core_session();
    session
        .eval_text("f[x, b] := InputString[\"final: \"]")
        .unwrap();

    let operator = session
        .eval_request(
            "SubsetMap[Function[List[If[SameQ[InputString[\"subset: \"], \"go\"], x, y]]], f[a, b], {1}]",
        )
        .expect_err("SubsetMap operator should suspend before replacements are applied");
    assert_input_prompt(&operator.error, "subset: ");

    let final_eval = session
        .resume_eval_request_with_input("go")
        .expect_err("SubsetMap should evaluate the replaced expression after the operator resumes");
    assert_input_prompt(&final_eval.error, "final: ");

    let response = session
        .resume_eval_request_with_input("done")
        .expect("SubsetMap should finish the resumed final evaluation");

    assert_eq!(response.value.to_string(), "\"done\"");
    assert!(session.pending_input_request().is_none());
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
