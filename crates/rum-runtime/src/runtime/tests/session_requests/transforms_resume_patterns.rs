use super::*;

#[test]
fn resume_eval_request_with_input_can_finish_patterntest_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "MatchQ[1, PatternTest[x_Integer, Function[x, MatchQ[InputString[\"cond: \"], \"yes\"]]]]",
        )
        .expect_err("PatternTest should suspend when its test function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish PatternTest-based MatchQ");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_sequence_patterntest_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "MatchQ[f[1, 2], f[PatternTest[x__, Function[x, MatchQ[InputString[\"cond: \"], \"yes\"]]], y_]]",
        )
        .expect_err("sequence PatternTest should suspend when its test function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish sequence PatternTest matching");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_except_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("MatchQ[a, Except[x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]]]")
        .expect_err("Except should suspend when its excluded branch requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Except-based MatchQ");

    assert_eq!(response.value.to_string(), "False");
}

#[test]
fn resume_eval_request_with_input_can_finish_repeated_patterntest_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "MatchQ[f[1], f[Repeated[PatternTest[x_Integer, Function[x, MatchQ[InputString[\"cond: \"], \"yes\"]]]]]]",
        )
        .expect_err("Repeated should suspend when its element matcher requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Repeated-based MatchQ");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_thread_with_suspending_arg_spec() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Thread[f[{a, b}, {c, d}, {e, f}], List, If[MatchQ[InputString[\"spec: \"], \"yes\"], {2}, All]]",
        )
        .expect_err("Thread should suspend while its argument-spec expression requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "spec: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Thread after the argument spec resolves");

    assert_eq!(
        response.value.to_string(),
        "List[f[List[a, b], c, List[e, f]], f[List[a, b], d, List[e, f]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_setattributes_with_suspending_attribute_spec() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "SetAttributes[f, If[MatchQ[InputString[\"attr: \"], \"yes\"], HoldFirst, HoldRest]]",
        )
        .expect_err("SetAttributes should suspend while its attribute spec requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "attr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish SetAttributes after the attribute spec resolves");

    assert_eq!(response.value.to_string(), "Null");
    assert_eq!(
        session
            .eval_text("f[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "f[Identity[1], 2]"
    );
}

#[test]
fn eval_request_reports_prints_on_failure_and_across_input_resume() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Print[\"before\"]; Throw[1]")
        .expect_err("uncaught Throw should fail the request");
    assert_eq!(failure.prints, vec!["before"]);

    let suspended = session
        .eval_request("Print[\"pre\"]; Input[\"expr: \"]; Print[\"post\"]")
        .expect_err("Input should suspend the request");
    match &suspended.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(suspended.prints, vec!["pre"]);

    let resumed = session
        .resume_eval_request_with_input("2 + 3")
        .expect("resume should keep both pre- and post-input prints");
    assert_eq!(resumed.value.to_string(), "Null");
    assert_eq!(resumed.prints, vec!["pre", "post"]);
}
