use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_deletecases_after_deleting_an_association_head() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "DeleteCases[Association[a -> 1, b -> f[2]], x_Symbol /; MatchQ[InputString[\"cond: \"], \"yes\"], Infinity, Heads -> True]",
        )
        .expect_err("association-aware DeleteCases should suspend when a matched head condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish association-aware DeleteCases after the condition resolves");

    assert_eq!(response.value.to_string(), "Association[]");
}
