use super::*;

#[test]
fn queued_input_responses_feed_inputstring_without_prompting_again() {
    let mut session = Session::new();
    session.queue_input_response("Alice");

    let response = session
        .eval_request("InputString[\"name: \"]")
        .expect("queued host input should satisfy InputString");

    assert_eq!(response.value.to_string(), "\"Alice\"");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments() {
    let mut session = Session::new();
    session.eval_text("x = seed").unwrap();

    let failure = session
        .eval_request("List[x = List[x], InputString[\"name: \"], x = List[x]]")
        .expect_err("InputString should suspend the request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue from the suspended argument list");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed], \"Alice\", List[List[seed]]]"
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[seed]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_can_suspend_multiple_times_without_restarting() {
    let mut session = Session::new();
    session.eval_text("x = seed").unwrap();

    let first = session
        .eval_request(
            "List[x = List[x], InputString[\"first: \"], x = List[x], InputString[\"second: \"], x = List[x]]",
        )
        .expect_err("first InputString should suspend the request");

    match &first.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "first: ");
        }
        other => panic!("expected first input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("Alice")
        .expect_err("second InputString should suspend the resumed request");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Bob")
        .expect("second resume should finish the original request");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed], \"Alice\", List[List[seed]], \"Bob\", List[List[List[seed]]]]"
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[List[seed]]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend() {
    let mut session = Session::new();
    session
        .eval_text("name := InputString[\"name: \"]")
        .unwrap();
    session.eval_text("g[\"Alice\"] := done").unwrap();

    let failure = session
        .eval_request("g[name]")
        .expect_err("symbol evaluation should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue the suspended fixed-point evaluation");

    assert_eq!(response.value.to_string(), "done");
}

#[test]
fn resume_eval_request_with_input_finishes_with_bindings_without_reprompting() {
    let mut session = Session::new();

    let failure = session
        .eval_request("With[{x = InputString[\"name: \"]}, List[x, x]]")
        .expect_err("With binding should suspend when its initializer requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should reuse the suspended With binding");

    assert_eq!(response.value.to_string(), "List[\"Alice\", \"Alice\"]");
}

#[test]
fn resume_eval_request_with_input_preserves_block_scope_until_completion() {
    let mut session = Session::new();
    session.eval_text("x = global").unwrap();

    let failure = session
        .eval_request("Block[{x = local}, List[x, InputString[\"name: \"], x = List[x], x]]")
        .expect_err("Block body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue inside the active Block scope");

    assert_eq!(
        response.value.to_string(),
        "List[local, \"Alice\", List[local], List[local]]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "global");
}

#[test]
fn resume_eval_request_with_input_can_finish_delayed_message_templates() {
    let mut session = Session::new();
    session
        .eval_text("f::argx := InputString[\"msg: \"]")
        .unwrap();

    let failure = session
        .eval_request("Message[f::argx]")
        .expect_err("delayed message template should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish delayed message rendering");

    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "Alice"
    }));
}

#[test]
fn resume_eval_request_with_input_formats_delayed_message_templates_after_resume() {
    let mut session = Session::new();
    session
        .eval_text(
            "f::argx := If[MatchQ[InputString[\"msg: \"], \"Alice\"], \"bad arg `1` from Alice\", \"bad arg `1` from stranger\"]",
        )
        .unwrap();

    let failure = session
        .eval_request("Message[f::argx, 42]")
        .expect_err("delayed formatted message template should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish delayed message formatting after the template resolves");

    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "bad arg 42 from Alice"
    }));
}

#[test]
fn resume_eval_request_with_input_continues_replace_all_without_restarting_target() {
    let mut session = Session::new();

    let failure = session
        .eval_request("ReplaceAll[InputString[\"name: \"], x_String :> List[x, x]]")
        .expect_err("ReplaceAll target should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue ReplaceAll from the suspended target");

    assert_eq!(response.value.to_string(), "List[\"Alice\", \"Alice\"]");
}
