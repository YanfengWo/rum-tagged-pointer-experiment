use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_select_predicate() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "Select[{1, 2}, Function[x, If[SameQ[x, 2], MatchQ[InputString[\"keep: \"], \"yes\"], False]]]",
        )
        .expect_err("Select predicate should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "keep: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Select after the predicate resolves");

    assert_eq!(response.value.to_string(), "List[2]");
}

#[test]
fn resume_eval_request_with_input_preserves_do_iterator_scope_until_completion() {
    let mut session = core_session();
    session.eval_text("i = global").unwrap();
    session.eval_text("seen = none").unwrap();

    let failure = session
        .eval_request(
            "Do[If[SameQ[i, 2], seen = List[i, InputString[\"iter: \"], i], Null], {i, 3}]; List[seen, i]",
        )
        .expect_err("Do iterator body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "iter: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resume should continue inside the active Do iterator scope");

    assert_eq!(
        response.value.to_string(),
        "List[List[2, \"ok\", 2], global]"
    );
    assert_eq!(session.eval_text("i").unwrap().value.to_string(), "global");
}
