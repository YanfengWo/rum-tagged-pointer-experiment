use super::*;

#[test]
fn withcleanup_runs_cleanup_on_values_throws_and_aborts() {
    let mut session = Session::new();

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[x = 1, x = 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Catch[WithCleanup[Throw[1, tag], x = 3], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "3");

    let cleanup_throw = session
        .eval_text("Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], tag]")
        .expect_err("cleanup-time Throw with the active tag should bypass the current Catch");
    match cleanup_throw {
        EvalError::Thrown { value, tag, .. } => {
            assert_eq!(value.to_string(), "1");
            assert_eq!(tag.to_string(), "tag");
        }
        other => panic!("expected primary throw to bypass current Catch, got {other:?}"),
    }
    assert_eq!(
        session
            .eval_text("Catch[Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], tag], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    let dual_uncaught = session
        .eval_request("WithCleanup[Throw[1, tag], Throw[2, tag]]")
        .expect_err("top-level active cleanup throw should report cleanup and primary throws");
    match dual_uncaught.error {
        SessionRequestError::Other { message } => assert_eq!(
            message,
            "uncaught throw: 2 with tag tag; uncaught throw: 1 with tag tag"
        ),
        other => panic!("expected dual uncaught throw request error, got {other:?}"),
    }
    assert_eq!(dual_uncaught.messages.len(), 2);
    assert_eq!(dual_uncaught.messages[0].symbol, "Throw");
    assert_eq!(dual_uncaught.messages[0].tag, "nocatch");
    assert_eq!(
        dual_uncaught.messages[0].text,
        "Uncaught Throw[2, tag] returned to top level."
    );
    assert_eq!(dual_uncaught.messages[1].symbol, "Throw");
    assert_eq!(dual_uncaught.messages[1].tag, "nocatch");
    assert_eq!(
        dual_uncaught.messages[1].text,
        "Uncaught Throw[1, tag] returned to top level."
    );
    match session
        .eval_text("Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], other]")
        .expect_err("nonmatching Catch should keep both cleanup and primary throw information")
    {
        EvalError::ThrownBypassingCatch {
            value,
            tag,
            cleanup_value,
            cleanup_tag,
            ..
        } => {
            assert_eq!(value.to_string(), "1");
            assert_eq!(tag.to_string(), "tag");
            assert_eq!(cleanup_value.to_string(), "2");
            assert_eq!(cleanup_tag.to_string(), "tag");
        }
        other => panic!("expected bypassing throw with cleanup context, got {other:?}"),
    }

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[Abort[], x = 4]")
            .unwrap()
            .value
            .to_string(),
        "$Aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "4");
    let abort_cleanup_throw = session
        .eval_text("WithCleanup[Abort[], Throw[2, tag]]")
        .unwrap();
    assert_eq!(abort_cleanup_throw.value.to_string(), "$Aborted");
    assert!(abort_cleanup_throw.messages.is_empty());
    assert_eq!(
        session
            .eval_text("CheckAbort[WithCleanup[Abort[], Throw[2, tag]], caught]")
            .unwrap()
            .value
            .to_string(),
        "caught"
    );
    assert_eq!(
        session
            .eval_text("Catch[CheckAbort[WithCleanup[Abort[], Throw[2, tag]], caught], tag]")
            .unwrap()
            .value
            .to_string(),
        "caught"
    );

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[Abort[], x = 1, x = 5]")
            .unwrap()
            .value
            .to_string(),
        "$Aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "5");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Catch[WithCleanup[Throw[1, tag], x = 9, x = 6], tag]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "6");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("CheckAbort[WithCleanup[x = 1; done, (x = 7; Abort[])], aborted]")
            .unwrap()
            .value
            .to_string(),
        "aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "7");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("WithCleanup[x = 1; done, x = 2; Return[20]]")
            .unwrap()
            .value
            .to_string(),
        "done"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Function[WithCleanup[x = 1; Return[10], x = 2; Return[20]]; 99][]")
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text(
                "Catch[Function[WithCleanup[x = 1; Throw[10, tag], x = 2; Return[20]]; 99][], tag]"
            )
            .unwrap()
            .value
            .to_string(),
        "10"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("CheckAbort[Function[WithCleanup[x = 1; Abort[], x = 2; Return[20]]; 99][], aborted]")
            .unwrap()
            .value
            .to_string(),
        "aborted"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");
}

#[test]
fn uncaught_throw_request_messages_follow_wolfram_nocatch_shape() {
    let mut session = Session::new();

    let plain = session
        .eval_request("Throw[1]")
        .expect_err("top-level Throw should fail the request");
    assert_eq!(plain.messages.len(), 1);
    assert_eq!(plain.messages[0].symbol, "Throw");
    assert_eq!(plain.messages[0].tag, "nocatch");
    assert_eq!(
        plain.messages[0].text,
        "Uncaught Throw[1] returned to top level."
    );

    let through_active_catch = session
        .eval_request("Catch[WithCleanup[Throw[1, tag], Throw[2, tag]], tag]")
        .expect_err("cleanup-time Throw should force the primary throw past active Catch");
    assert_eq!(through_active_catch.messages.len(), 1);
    assert_eq!(through_active_catch.messages[0].symbol, "Throw");
    assert_eq!(through_active_catch.messages[0].tag, "nocatch");
    assert_eq!(
        through_active_catch.messages[0].text,
        "Uncaught Throw[1, tag] returned to top level."
    );

    let cleanup_only = session
        .eval_request("Catch[WithCleanup[Throw[1, tag], Throw[2, other]], tag]")
        .expect_err("nonmatching cleanup Throw should escape before the primary is caught");
    assert_eq!(cleanup_only.messages.len(), 1);
    assert_eq!(cleanup_only.messages[0].symbol, "Throw");
    assert_eq!(cleanup_only.messages[0].tag, "nocatch");
    assert_eq!(
        cleanup_only.messages[0].text,
        "Uncaught Throw[2, other] returned to top level."
    );

    let explicit_none = session
        .eval_request("Throw[1, None]")
        .expect_err("explicit None tag should still be visible in the diagnostic");
    assert_eq!(explicit_none.messages.len(), 1);
    assert_eq!(explicit_none.messages[0].symbol, "Throw");
    assert_eq!(explicit_none.messages[0].tag, "nocatch");
    assert_eq!(
        explicit_none.messages[0].text,
        "Uncaught Throw[1, None] returned to top level."
    );

    let explicit_none_cleanup = session
        .eval_request("WithCleanup[Throw[1, None], Throw[2, None]]")
        .expect_err("explicit None tags should be preserved through cleanup throw diagnostics");
    assert_eq!(explicit_none_cleanup.messages.len(), 2);
    assert_eq!(
        explicit_none_cleanup.messages[0].text,
        "Uncaught Throw[2, None] returned to top level."
    );
    assert_eq!(
        explicit_none_cleanup.messages[1].text,
        "Uncaught Throw[1, None] returned to top level."
    );

    session.eval_text("Off[Throw::nocatch]").unwrap();
    let suppressed = session
        .eval_request("Throw[1, tag]")
        .expect_err("Off[Throw::nocatch] suppresses only the diagnostic message");
    assert!(suppressed.messages.is_empty());
}

#[test]
fn catch_evaluates_symbol_valued_tag_form_before_matching() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("tag = \"$t$\"; Catch[Throw[42, tag], tag]")
            .unwrap()
            .value
            .to_string(),
        "42"
    );

    match session
        .eval_text("tag = \"$t$\"; Catch[Throw[42, \"$u$\"], tag]")
        .expect_err("symbol-valued catch tag should not match a different thrown string")
    {
        EvalError::Thrown { value, tag, .. } => {
            assert_eq!(value.to_string(), "42");
            assert_eq!(tag.to_string(), "\"$u$\"");
        }
        other => panic!("expected nonmatching throw to escape, got {other:?}"),
    }
}

#[test]
fn catch_tag_form_keeps_visible_side_effects() {
    let mut session = Session::new();

    let caught = session
        .eval_text("x = 0; Catch[Throw[1, \"$t$\"], (Print[\"tag\"]; x = 9; \"$t$\")]; x")
        .unwrap();
    assert_eq!(caught.value.to_string(), "9");
    assert_eq!(caught.prints, vec!["tag"]);

    let bypassed = session
        .eval_text("x = 0; Catch[Catch[Throw[1, \"$u$\"], (Print[\"tag\"]; x = 9; \"$t$\")], _]; x")
        .unwrap();
    assert_eq!(bypassed.value.to_string(), "9");
    assert_eq!(bypassed.prints, vec!["tag"]);
}
