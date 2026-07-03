use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_associationmap_over_association_entries() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "AssociationMap[If[SameQ[InputString[\"go: \"], \"yes\"], #, #] &, Association[a -> 1]]",
        )
        .expect_err("AssociationMap should suspend when the mapped entry function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "go: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("k")
        .expect("resume should finish AssociationMap after the mapped entry function resolves");

    assert_eq!(response.value.to_string(), "Association[Rule[a, 1]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_keysort_after_suspending_comparer() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "KeySort[{bb -> 2, a -> 1}, Function[If[SameQ[Slot[1], a], MatchQ[InputString[\"cmp: \"], \"yes\"], True]]]",
        )
        .expect_err("KeySort should suspend when its custom comparer requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cmp: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue KeySort after the custom comparer resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
}
