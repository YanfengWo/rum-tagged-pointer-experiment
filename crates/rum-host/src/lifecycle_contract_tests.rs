use super::*;

fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected successful evaluation, got {other:?}"),
    }
}

#[test]
fn panic_poison_blocks_public_entries_until_reset() {
    let mut session = HostSession::new();

    match session.eval_with_test_panic() {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Panic),
        other => panic!("expected panic failure, got {other:?}"),
    }
    assert!(session.poisoned_for_test());

    let parse = session
        .parse("1 + 1")
        .expect_err("poisoned parse should fail");
    assert_eq!(parse.code, HostFailureCode::Poisoned);

    match session.eval("1 + 1") {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Poisoned),
        other => panic!("expected poisoned eval failure, got {other:?}"),
    }

    assert!(!session.inspect_symbol("x").found);
    assert!(
        session
            .complete(HostCompletionRequest::new("x", 1))
            .matches
            .is_empty()
    );
    assert_eq!(
        session.is_complete("1 + 1").status,
        HostCompletenessStatus::Invalid
    );

    session.reset();
    assert!(!session.poisoned_for_test());
    assert_eq!(
        session
            .parse("1 + 1")
            .expect("reset parse should succeed")
            .syntax,
        "1 + 1"
    );
    match session.eval("1 + 1") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "2"),
        other => panic!("expected reset eval success, got {other:?}"),
    }
}

#[test]
fn reset_rebuilds_all_runtime_accumulation_dimensions() {
    let mut session = HostSession::new();

    assert_eq!(successful_eval(session.eval("x = 41")).value.text, "41");
    assert_eq!(
        successful_eval(session.eval("f[n_] := n^2; f[3]"))
            .value
            .text,
        "9"
    );
    assert_eq!(
        successful_eval(session.eval("SetAttributes[g, Orderless]; Attributes[g]"))
            .value
            .text,
        "List[Orderless]"
    );
    assert_eq!(
        successful_eval(
            session.eval("Options[h] = {Foo -> 0}; SetOptions[h, Foo -> 1]; Options[h]")
        )
        .value
        .text,
        "List[Rule[Foo, 1]]"
    );
    assert_eq!(
        successful_eval(session.eval("g::note = \"text\"; Messages[g]"))
            .value
            .text,
        "List[g::note]"
    );
    assert_eq!(
        successful_eval(session.eval("Begin[\"Custom`\"]; $Context"))
            .value
            .text,
        "\"Custom`\""
    );
    let module_before_reset = successful_eval(session.eval("Module[{t}, t]")).value.text;
    assert_eq!(module_before_reset, "Custom`t$1");
    let random_before_reset = successful_eval(session.eval("SeedRandom[7]; RandomInteger[100]"))
        .value
        .text;
    assert_eq!(random_before_reset, "23");

    session.reset();
    let mut fresh = HostSession::new();

    assert_eq!(session.definitions(), fresh.definitions());
    assert_eq!(session.contexts(), fresh.contexts());

    let reset_x = successful_eval(session.eval("x"));
    assert_eq!(reset_x.value.text, "x");
    assert_eq!(reset_x.value.typed, HostTyped::Symbol("x".to_string()));
    assert_eq!(successful_eval(session.eval("f[3]")).value.text, "f[3]");
    assert_eq!(
        successful_eval(session.eval("Attributes[g]")).value.text,
        "List[]"
    );
    assert_eq!(
        successful_eval(session.eval("Options[h]")).value.text,
        "List[]"
    );
    assert_eq!(
        successful_eval(session.eval("Messages[g]")).value.text,
        "List[]"
    );

    let reset_context = successful_eval(session.eval("$Context")).value.text;
    let fresh_context = successful_eval(fresh.eval("$Context")).value.text;
    assert_eq!(reset_context, fresh_context);
    assert_eq!(reset_context, "\"Global`\"");

    let module_after_reset = successful_eval(session.eval("Module[{t}, t]")).value.text;
    let fresh_module = successful_eval(fresh.eval("Module[{t}, t]")).value.text;
    assert_eq!(module_after_reset, fresh_module);

    let random_after_reset = successful_eval(session.eval("SeedRandom[7]; RandomInteger[100]"))
        .value
        .text;
    let fresh_random = successful_eval(fresh.eval("SeedRandom[7]; RandomInteger[100]"))
        .value
        .text;
    assert_eq!(random_after_reset, fresh_random);
    assert_eq!(random_after_reset, random_before_reset);
}

#[test]
fn reset_between_requests_isolates_implicit_session_state() {
    let mut session = HostSession::new();

    let request_1_counter = successful_eval(session.eval("counter = 1; counter"));
    assert_eq!(request_1_counter.value.text, "1");
    let request_1_random = successful_eval(session.eval("SeedRandom[1]; RandomInteger[10^6]"))
        .value
        .text;

    // Independent requests pass explicit state as request data; implicit session state does not
    // survive the reset between them.
    session.reset();

    let request_2_counter = successful_eval(session.eval("counter"));
    assert_eq!(request_2_counter.value.text, "counter");
    assert_eq!(
        request_2_counter.value.typed,
        HostTyped::Symbol("counter".to_string())
    );
    let request_2_random = successful_eval(session.eval("SeedRandom[1]; RandomInteger[10^6]"))
        .value
        .text;
    assert_eq!(request_2_random, request_1_random);
}

#[test]
fn abandoned_poisoned_session_does_not_contaminate_a_fresh_session() {
    let mut abandoned = HostSession::new();

    assert_eq!(
        successful_eval(
            abandoned.eval("x = 41; y = 42; f[n_] := n^2; g[n_] := n + 1; f[3] + g[3]")
        )
        .value
        .text,
        "13"
    );
    assert_eq!(
        successful_eval(abandoned.eval("Module[{t}, t]")).value.text,
        "t$1"
    );
    assert_eq!(
        successful_eval(abandoned.eval("SeedRandom[7]; RandomInteger[100]"))
            .value
            .text,
        "23"
    );
    match abandoned.eval_with_test_panic() {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Panic),
        other => panic!("expected panic failure, got {other:?}"),
    }
    assert!(abandoned.poisoned_for_test());
    drop(abandoned);

    let mut session = HostSession::new();
    let mut fresh = HostSession::new();

    assert_eq!(session.definitions(), fresh.definitions());
    assert_eq!(session.contexts(), fresh.contexts());
    assert_eq!(successful_eval(session.eval("2 + 2")).value.text, "4");

    let undefined = successful_eval(session.eval("x"));
    assert_eq!(undefined.value.text, "x");
    assert_eq!(undefined.value.typed, HostTyped::Symbol("x".to_string()));

    let session_module = successful_eval(session.eval("Module[{t}, t]")).value.text;
    let fresh_module = successful_eval(fresh.eval("Module[{t}, t]")).value.text;
    assert_eq!(session_module, fresh_module);

    let session_random = successful_eval(session.eval("SeedRandom[3]; RandomInteger[9]"))
        .value
        .text;
    let fresh_random = successful_eval(fresh.eval("SeedRandom[3]; RandomInteger[9]"))
        .value
        .text;
    assert_eq!(session_random, fresh_random);
}
