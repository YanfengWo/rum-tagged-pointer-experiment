use super::*;

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
