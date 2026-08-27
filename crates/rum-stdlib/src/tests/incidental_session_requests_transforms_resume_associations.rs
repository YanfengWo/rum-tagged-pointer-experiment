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
fn resume_eval_request_with_input_preserves_associationmap_list_progress() {
    let mut session = core_session();

    let first = session
        .eval_request("AssociationMap[Function[InputString[\"map: \"]], {a, b}]")
        .expect_err("AssociationMap should suspend on the first list key");
    match &first.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "map: "),
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("AssociationMap should continue to the second list key");
    match &second.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "map: "),
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("two")
        .expect("AssociationMap should retain the first mapped value");
    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, \"one\"], Rule[b, \"two\"]]"
    );
}

#[test]
fn resume_eval_request_with_input_preserves_keyselect_progress() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "KeySelect[<|a -> 1, b -> 2|>, Function[SameQ[InputString[\"key: \"], \"yes\"]]]",
        )
        .expect_err("KeySelect should suspend on the first key");
    match &first.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "key: "),
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("yes")
        .expect_err("KeySelect should continue to the second key");
    match &second.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "key: "),
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("KeySelect should retain the first selected entry");
    assert_eq!(response.value.to_string(), "Association[Rule[a, 1]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_keysort_after_suspending_comparer() {
    let source = "KeySort[{bb -> 2, a -> 1}, Function[If[SameQ[Slot[1], a], MatchQ[InputString[\"cmp: \"], \"yes\"], True]]]";
    let mut uninterrupted_session = core_session();
    uninterrupted_session.queue_input_response("yes");
    let uninterrupted = uninterrupted_session
        .eval_request(source)
        .expect("queued input should complete KeySort");
    let mut session = core_session();

    let failure = session
        .eval_request(source)
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
    assert_eq!(response.value, uninterrupted.value);
}
