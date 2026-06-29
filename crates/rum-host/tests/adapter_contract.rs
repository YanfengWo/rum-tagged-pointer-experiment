use rum_host::{
    HostCompletenessStatus, HostCompletionRequest, HostConfig, HostEvalOutcome, HostFailureCode,
    HostInputKind, HostInspectRequest, HostIsCompleteRequest, HostSession,
    HostStructuredOutputKind,
};

#[test]
fn public_contract_covers_eval_outputs_without_runtime_types() {
    let mut session = HostSession::builder().with_tracing(true).build();

    let outcome = session.eval("f[x_Integer] := x; Print[\"seen\"]; Message[General::stop]; f[3]");

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "3");
            assert_eq!(success.prints, vec!["seen"]);
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "stop")
            );
            assert!(success.trace.iter().any(|event| {
                matches!(event, rum_host::HostTraceEvent::RuleApplied { source, .. } if source == "DownValue")
            }));
            assert_eq!(
                success.outputs.last().map(|output| output.kind),
                Some(HostStructuredOutputKind::Result)
            );
            assert!(
                success
                    .outputs
                    .iter()
                    .any(|output| output.kind == HostStructuredOutputKind::Trace)
            );
        }
        other => panic!("expected host success, got {other:?}"),
    }
}

#[test]
fn public_contract_preserves_input_suspension_and_resume_boundaries() {
    let mut session = HostSession::new();

    match session.eval("Input[\"expr: \"] + 2") {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::Input);
            assert_eq!(suspension.request.prompt, "expr: ");
            assert_eq!(
                session
                    .pending_input_request()
                    .map(|request| request.prompt),
                Some("expr: ".to_string())
            );
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    match session.resume_with_input("5") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "7"),
        other => panic!("expected resumed success, got {other:?}"),
    }

    match session.resume_with_input("unused") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert!(failure.message.contains("no suspended evaluation"));
        }
        other => panic!("expected no-pending-input failure, got {other:?}"),
    }
}

#[test]
fn public_contract_covers_editor_requests_and_failures() {
    let mut session = HostSession::new();

    assert!(matches!(
        session.eval("localName = 42"),
        HostEvalOutcome::Success(_)
    ));

    let inspect = session.inspect(HostInspectRequest::new("localName", 4));
    assert!(inspect.found);
    assert_eq!(
        inspect
            .info
            .as_ref()
            .map(|info| info.qualified_name.as_str()),
        Some("Global`localName")
    );
    assert!(
        inspect
            .definitions
            .iter()
            .any(|definition| definition.rhs == "42")
    );

    let completion = session.complete(HostCompletionRequest::new("localName", 4));
    assert_eq!(completion.prefix, "loca");
    assert_eq!(completion.replacement_start, 0);
    assert_eq!(completion.replacement_end, "localName".len());
    assert_eq!(completion.matches, vec!["localName"]);

    let incomplete = session.is_complete(HostIsCompleteRequest::new("f[\n  1"));
    assert_eq!(incomplete.status, HostCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));

    // `1 2` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let invalid = session.is_complete("f[1,]");
    assert_eq!(invalid.status, HostCompletenessStatus::Invalid);
    assert!(invalid.diagnostic.is_some());

    match session.eval("f[1,]") {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Syntax),
        other => panic!("expected syntax failure, got {other:?}"),
    }
}

#[test]
fn public_config_facade_controls_core_builtin_installation() {
    let disabled_config = HostConfig::default().with_core_builtins(false);
    let disabled = HostSession::builder().with_config(disabled_config).build();
    assert!(!disabled.configuration().install_core_builtins);

    assert!(
        disabled
            .complete(HostCompletionRequest::new("Ide", 3))
            .matches
            .is_empty()
    );
    let disabled_identity = disabled.inspect_symbol("Identity");
    assert!(disabled_identity.found);
    assert_eq!(
        disabled_identity
            .info
            .and_then(|info| info.summary)
            .as_deref(),
        None
    );

    let mut enabled = HostSession::new();
    assert_eq!(
        enabled
            .complete(HostCompletionRequest::new("Ide", 3))
            .matches,
        vec!["Identity"]
    );
    assert!(
        enabled
            .inspect_symbol("Identity")
            .info
            .and_then(|info| info.summary)
            .is_some()
    );
    match enabled.eval("1 + 2") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "3"),
        other => panic!("expected evaluated Plus with installed builtins, got {other:?}"),
    }
}
