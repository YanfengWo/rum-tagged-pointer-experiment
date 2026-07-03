use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_cases_over_association_values_only() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "Cases[Association[a -> 1, b -> f[2]], x_Symbol /; MatchQ[InputString[\"cond: \"], \"yes\"], Infinity, Heads -> True]",
        )
        .expect_err("association-aware Cases traversal should suspend when a matched head condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("yes")
        .expect_err("association-aware Cases should continue into the next matching value head");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish association-aware Cases after the condition resolves");

    assert_eq!(response.value.to_string(), "List[Association, f]");
}
