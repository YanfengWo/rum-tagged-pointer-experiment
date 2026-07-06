use super::*;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

fn file_request_error(source: &str, configure: impl FnOnce(&mut Session)) -> SessionRequestError {
    let dir = TempDirGuard::new("request-error");
    let file = dir.write("request-error.rum", source);
    let mut session = session_with_test_control_builtins();
    install_test_file_io(&mut session);
    configure(&mut session);

    session
        .eval_file_request(&file)
        .expect_err("file request should fail")
        .error
}

fn assert_file_request_error(
    source: &str,
    configure: impl FnOnce(&mut Session),
    matches_error: impl FnOnce(SessionRequestError),
) {
    matches_error(file_request_error(source, configure));
}

#[test]
fn parse_request_returns_structured_syntax_diagnostics() {
    let session = Session::new();
    let failure = session
        .parse_request("f[1 ! 2]")
        .expect_err("malformed input should fail with a syntax diagnostic");

    match failure.error {
        SessionRequestError::Syntax(diagnostic) => {
            assert!(diagnostic.message.starts_with("expected ,"));
            assert!(!diagnostic.is_incomplete);
            assert!(diagnostic.rendered.contains("--> 1:5"));
            assert_eq!(diagnostic.spans.len(), 1);
            assert_eq!(diagnostic.spans[0].start.line, 1);
            assert_eq!(diagnostic.spans[0].start.column, 5);
            assert_eq!(diagnostic.spans[0].text, "!");
        }
        other => panic!("expected syntax diagnostic, got {other:?}"),
    }

    assert!(failure.messages.is_empty());
    assert!(failure.trace.is_empty());
}

#[test]
fn complete_request_returns_transport_friendly_matches() {
    let mut session = Session::new();
    session.eval_text("localName = 1").unwrap();

    let response = session.complete_request("loc");

    assert_eq!(response.prefix, "loc");
    assert_eq!(response.matches, vec!["localName"]);
}

#[test]
fn is_complete_request_distinguishes_complete_incomplete_and_invalid_input() {
    let session = Session::new();

    let complete = session.is_complete_request("f[1]");
    assert_eq!(complete.status, SessionCompletenessStatus::Complete);
    assert!(complete.indent.is_none());
    assert!(complete.diagnostic.is_none());

    let incomplete = session.is_complete_request("f[\n  1");
    assert_eq!(incomplete.status, SessionCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));
    assert!(
        incomplete
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| diagnostic.is_incomplete)
    );

    let invalid = session.is_complete_request("f[1 ! 2]");
    assert_eq!(invalid.status, SessionCompletenessStatus::Invalid);
    assert!(invalid.indent.is_none());
    assert!(
        invalid
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| !diagnostic.is_incomplete)
    );
}

#[test]
fn eval_request_returns_transport_friendly_success_data() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := x").unwrap();

    let response = session.eval_request("f[1]").unwrap();

    assert_eq!(response.value.to_string(), "1");
    assert!(response.messages.is_empty());
    assert!(response.trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input == "f[1]"
    )));
}

#[test]
fn eval_request_returns_transport_friendly_cycle_termination_data() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := g[x]").unwrap();
    session.eval_text("g[x_Integer] := f[x]").unwrap();

    let response = session
        .eval_request("f[1]")
        .expect("recursive evaluation should surface a termination value");

    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(response.trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input == "f[1]" || input == "g[1]"
    )));
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );
}

#[test]
fn inspect_request_aggregates_symbol_definitions_and_messages() {
    let mut session = Session::new();
    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f::usage = \"desc\"").unwrap();

    let response = session
        .inspect_request("f")
        .expect("known symbol should produce an inspect response");

    assert_eq!(response.info.qualified_name, "Global`f");
    assert!(response.info.has_down_values);
    assert_eq!(response.definitions.len(), 1);
    assert_eq!(response.message_catalog.len(), 1);
    assert_eq!(response.message_catalog[0].message_name, "f::usage");
}

#[test]
fn eval_request_surfaces_structured_input_requests() {
    let mut session = Session::new();

    let failure = session
        .eval_request("InputString[\"name: \"]")
        .expect_err("missing host input should become a structured request failure");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "name: ");
            assert!(!request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    assert_eq!(
        session.pending_input_request(),
        Some(&RuntimeInputRequest::new("name: "))
    );
}

#[test]
fn eval_request_take_pending_input_request_moves_and_clears_the_request() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Input[\"expr: \"]")
        .expect_err("Input should suspend the request");
    assert!(matches!(
        failure.error,
        SessionRequestError::InputRequired { .. }
    ));

    let request = session
        .take_pending_input_request()
        .expect("pending input request should be movable");
    assert_eq!(request.kind, RuntimeInputKind::Input);
    assert_eq!(request.prompt, "expr: ");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn file_request_maps_completed_syntax_io_and_missing_provider_errors() {
    assert_file_request_error(
        "f[1,]",
        |_| {},
        |error| match error {
            SessionRequestError::Syntax(diagnostic) => {
                assert!(!diagnostic.message.is_empty());
                assert!(!diagnostic.is_incomplete);
                assert!(diagnostic.rendered.contains("f[1,]"));
            }
            other => panic!("expected syntax request error, got {other:?}"),
        },
    );

    let missing_error = {
        let mut session = Session::new();
        install_test_file_io(&mut session);
        session
            .eval_file_request("missing-file.rum")
            .expect_err("missing file should fail")
            .error
    };
    match missing_error {
        SessionRequestError::Io { path, message } => {
            assert!(path.ends_with("missing-file.rum"));
            assert!(!message.is_empty());
        }
        other => panic!("expected IO request error, got {other:?}"),
    }

    let mut no_provider_session = Session::new();
    no_provider_session.set_file_io(None);
    let no_provider = no_provider_session
        .eval_file_request("missing-file.rum")
        .expect_err("file request without provider should fail")
        .error;
    match no_provider {
        SessionRequestError::Other { message } => {
            assert_eq!(message, "no file provider installed");
        }
        other => panic!("expected no-provider request error, got {other:?}"),
    }
}

#[test]
fn file_request_maps_resource_limit_exit_and_cancel_errors() {
    assert_file_request_error(
        "grow[x_] := grow[f[x]]\ngrow[1]",
        |session| session.set_recursion_limit(32),
        |error| match error {
            SessionRequestError::RecursionLimitExceeded { limit } => assert_eq!(limit, 32),
            other => panic!("expected recursion-limit request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "f[n_] := f[n + 1]\nf[0]",
        |session| session.set_native_stack_limit(512 * 1024),
        |error| match error {
            SessionRequestError::StackLimitExceeded { limit } => assert_eq!(limit, 512 * 1024),
            other => panic!("expected stack-limit request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Table[x, 5]",
        |session| session.set_generated_element_limit(3),
        |error| match error {
            SessionRequestError::GeneratedElementLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected generated-element request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Factorial[5]",
        |session| session.set_integer_work_limit(3),
        |error| match error {
            SessionRequestError::IntegerWorkLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected integer-work request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Do[Null, 5]",
        |session| session.set_iteration_work_limit(3),
        |error| match error {
            SessionRequestError::IterationWorkLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected iteration-work request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "1 //. {1 -> 2, 2 -> 1}",
        |session| session.set_replace_repeated_limit(3),
        |error| match error {
            SessionRequestError::IterationLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected replace-repeated request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Exit[5]",
        |_| {},
        |error| match error {
            SessionRequestError::Exit { code } => assert_eq!(code, 5),
            other => panic!("expected exit request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Do[x = 1, 50000000]",
        |session| {
            session.set_iteration_work_limit(50_000_000);
            session.set_eval_timeout(Some(Duration::from_millis(0)));
        },
        |error| match error {
            SessionRequestError::TimeConstraintExceeded => {}
            other => panic!("expected timeout request error, got {other:?}"),
        },
    );

    let dir = TempDirGuard::new("request-cancel");
    let file = dir.write("cancel.rum", "Do[x = 1, 50000000]");
    let mut session = session_with_test_control_builtins();
    install_test_file_io(&mut session);
    session.set_iteration_work_limit(50_000_000);
    let cancel = session.interrupt_handle();
    let canceller = {
        let cancel = cancel.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(25));
            cancel.store(true, Ordering::Relaxed);
        })
    };
    let cancelled = session
        .eval_file_request(&file)
        .expect_err("cancelled file request should fail")
        .error;
    canceller.join().expect("canceller should not panic");
    assert!(matches!(cancelled, SessionRequestError::Cancelled));
}
