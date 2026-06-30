use super::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const LONG_RUNNING_EVAL: &str = "Do[x = 1, 50000000]";
const TWO_TO_200: &str = "1606938044258990275541962092341162602522202993782792835301376";

fn host_session_with_large_iteration_budget() -> HostSession {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = 50_000_000;
    HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build()
}

#[test]
fn host_eval_reports_value_messages_prints_and_structured_outputs() {
    let mut session = HostSession::new();

    let outcome = session.eval("Print[1]; Message[General::stop]; 2");

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "2");
            assert_eq!(success.prints, vec!["1"]);
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "stop")
            );
            assert!(
                success
                    .outputs
                    .iter()
                    .any(|output| output.kind == HostStructuredOutputKind::Result)
            );
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn host_eval_exposes_typed_value_variants() {
    let mut session = HostSession::new();
    let cases = vec![
        ("2 + 3", "5", HostTyped::Integer(5)),
        (
            "2^200",
            TWO_TO_200,
            HostTyped::BigInteger(TWO_TO_200.to_string()),
        ),
        (
            "1/3",
            "1/3",
            HostTyped::Rational {
                numerator: "1".to_string(),
                denominator: "3".to_string(),
            },
        ),
        ("True", "True", HostTyped::Symbol("True".to_string())),
        ("\"hi\"", "\"hi\"", HostTyped::Str("hi".to_string())),
        (
            "{1, 2, 3}",
            "List[1, 2, 3]",
            HostTyped::List(vec![
                HostTyped::Integer(1),
                HostTyped::Integer(2),
                HostTyped::Integer(3),
            ]),
        ),
        (
            "f[1]",
            "f[1]",
            HostTyped::Other {
                head: "f".to_string(),
                text: "f[1]".to_string(),
            },
        ),
    ];

    for (input, expected_text, expected_typed) in cases {
        match session.eval(input) {
            HostEvalOutcome::Success(success) => {
                assert_eq!(success.value.text, expected_text, "input {input}");
                assert_eq!(success.value.typed, expected_typed, "input {input}");
                eprintln!(
                    "typed_value input={input:?} text={:?} typed={:?}",
                    success.value.text, success.value.typed
                );
            }
            other => panic!("expected success for {input}, got {other:?}"),
        }
    }
}

#[test]
fn host_suspend_resume_flow_wraps_input_requests() {
    let mut session = HostSession::new();

    let suspended = session.eval("StringJoin[InputString[\"name: \"], \"!\"]");
    match suspended {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, "name: ");
            assert!(!suspension.request.password);
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    let resumed = session.resume_with_input("rum");
    match resumed {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"rum!\""),
        other => panic!("expected resumed success, got {other:?}"),
    }
}

#[test]
fn host_parse_inspect_complete_and_is_complete_cover_editor_flows() {
    let mut session = HostSession::new();

    let parsed = session.parse("1 + 2").expect("parse should succeed");
    assert_eq!(parsed.syntax, "1 + 2");

    assert!(matches!(
        session.eval("localName = 1"),
        HostEvalOutcome::Success(_)
    ));

    let inspect = session.inspect(HostInspectRequest::new("localName", 3));
    assert!(inspect.found);
    assert_eq!(
        inspect
            .info
            .as_ref()
            .map(|info| info.qualified_name.as_str()),
        Some("Global`localName")
    );

    let completion = session.complete(HostCompletionRequest::new("localName", 3));
    assert_eq!(completion.replacement_start, 0);
    assert_eq!(completion.replacement_end, "localName".len());
    assert_eq!(completion.prefix, "loc");
    assert_eq!(completion.matches, vec!["localName"]);

    let incomplete = session.is_complete("f[\n  1");
    assert_eq!(incomplete.status, HostCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));
}

#[test]
fn host_failures_use_stable_failure_codes() {
    let mut session = HostSession::new();

    // `1 2` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let outcome = session.eval("f[1,]");

    match outcome {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Syntax);
            assert!(!failure.message.is_empty());
        }
        other => panic!("expected syntax failure, got {other:?}"),
    }
}

#[test]
fn host_boundary_catches_panic_and_poisons_session() {
    let mut session = HostSession::new();

    let caught = session.eval_with_test_panic();

    let (caught_code, caught_message) = match caught {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Panic);
            assert!(failure.message.contains("host panic boundary test"));
            (failure.code, failure.message)
        }
        other => panic!("expected panic failure, got {other:?}"),
    };
    assert!(session.poisoned_for_test());

    let followup = session.eval("1 + 1");
    match followup {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Poisoned);
            assert!(failure.message.contains("poisoned"));
            eprintln!(
                "host_boundary_catches_panic_and_poisons_session caught_code={caught_code:?} caught_message={caught_message:?} followup_code={:?} followup_message={:?}",
                failure.code, failure.message
            );
        }
        other => panic!("expected poisoned failure, got {other:?}"),
    }
}

#[test]
fn host_boundary_normal_eval_does_not_poison_session() {
    let mut session = HostSession::new();

    match session.eval("1 + 2") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "3"),
        other => panic!("expected normal success, got {other:?}"),
    }
    assert!(!session.poisoned_for_test());

    match session.eval("3 + 4") {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "7");
            eprintln!(
                "host_boundary_normal_eval_does_not_poison_session first=3 second=7 poisoned={}",
                session.poisoned_for_test()
            );
        }
        other => panic!("expected follow-up success, got {other:?}"),
    }
}

#[test]
fn host_cancel_handle_interrupts_running_eval_from_another_thread() {
    let mut session = host_session_with_large_iteration_budget();
    let cancel = session.cancel_handle();
    let (cancelled_tx, cancelled_rx) = mpsc::channel();

    let canceller = {
        let cancel = cancel.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            let cancelled_at = Instant::now();
            cancel.cancel();
            cancelled_tx
                .send(cancelled_at)
                .expect("test should receive cancel timestamp");
        })
    };

    let started = Instant::now();
    let outcome = session.eval(LONG_RUNNING_EVAL);
    let completed_at = Instant::now();
    let elapsed = completed_at.duration_since(started);
    let cancelled_at = cancelled_rx
        .recv()
        .expect("canceller should report cancel timestamp");
    canceller.join().expect("canceller should not panic");
    let cancel_latency = completed_at.duration_since(cancelled_at);

    eprintln!(
        "host_cancel_handle_interrupts_running_eval_from_another_thread elapsed_ms={} cancel_latency_ms={}",
        elapsed.as_millis(),
        cancel_latency.as_millis()
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "cancelled eval took {elapsed:?}"
    );
    assert!(
        cancel_latency < Duration::from_secs(1),
        "cancel observation took {cancel_latency:?}"
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "TerminatedEvaluation[\"Cancelled\"]");
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "cancel")
            );
        }
        other => panic!("expected graceful cancelled value, got {other:?}"),
    }
}

#[test]
fn host_eval_with_timeout_interrupts_running_eval() {
    let mut session = host_session_with_large_iteration_budget();

    let started = Instant::now();
    let outcome = session.eval_with_timeout(LONG_RUNNING_EVAL, Duration::from_millis(200));
    let elapsed = started.elapsed();

    eprintln!(
        "host_eval_with_timeout_interrupts_running_eval elapsed_ms={}",
        elapsed.as_millis()
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "timed eval took {elapsed:?}"
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(
                success.value.text,
                "TerminatedEvaluation[\"TimeConstraint\"]"
            );
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "timelim")
            );
        }
        other => panic!("expected graceful timeout value, got {other:?}"),
    }
}

#[test]
fn host_eval_with_unused_handle_and_generous_timeout_still_succeeds() {
    let mut session = HostSession::new();
    let _cancel = session.cancel_handle();

    let started = Instant::now();
    let outcome = session.eval_with_timeout("1 + 2", Duration::from_secs(5));
    let elapsed = started.elapsed();

    eprintln!(
        "host_eval_with_unused_handle_and_generous_timeout_still_succeeds elapsed_ms={}",
        elapsed.as_millis()
    );

    match outcome {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "3"),
        other => panic!("expected normal success, got {other:?}"),
    }
}
