use super::*;

#[test]
fn withcleanup_defers_cleanup_until_suspended_body_finishes() {
    let mut session = Session::new();
    session.eval_text("x = 0").unwrap();

    let failure = session
        .eval_request("WithCleanup[x = 1; InputString[\"body: \"]; x = 2; done, x = 3]")
        .expect_err("WithCleanup body should suspend before cleanup runs");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming WithCleanup body should finish and then run cleanup");
    assert_eq!(response.value.to_string(), "done");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "3");

    session.eval_text("x = 0").unwrap();
    let failure = session
        .eval_request("WithCleanup[x = 10, x = 1; InputString[\"body: \"]; x = 2; done, x = 3]")
        .expect_err("WithCleanup main body should suspend after init");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming three-argument WithCleanup body should run cleanup last");
    assert_eq!(response.value.to_string(), "done");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "3");

    session.eval_text("x = 0").unwrap();
    let failure = session
        .eval_request("WithCleanup[x = 1; done, InputString[\"cleanup: \"]; x = 2]")
        .expect_err("WithCleanup cleanup should suspend after a successful body");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cleanup: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming suspended cleanup should return the primary body value");
    assert_eq!(response.value.to_string(), "done");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    let failure = session
        .eval_request(
            "CheckAbort[WithCleanup[Abort[], InputString[\"cleanup: \"]; x = 4], rescued]",
        )
        .expect_err("WithCleanup cleanup should suspend before propagating an abort");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cleanup: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "0");
    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming cleanup should let CheckAbort catch the original abort");
    assert_eq!(response.value.to_string(), "rescued");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "4");
}
