use super::*;
use rum_runtime::SessionRequestError;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn exception_registry_round_trips_and_reregistration_fails_closed() {
    let mut session = core_session();

    let inert = session
        .eval_request("Exception[\"Unregistered\", <||>]")
        .expect("constructing an Exception object never raises");
    assert_eq!(
        inert.value.to_string(),
        "Exception[\"Unregistered\", Association[]]"
    );
    assert!(inert.messages.is_empty());

    let registered = session
        .eval_text(
            "RegisterExceptionType[\"NetError\"]; \
             RegisterExceptionType[\"Timeout\", \"NetError\"]; \
             ExceptionTypes[]",
        )
        .unwrap();
    assert_eq!(
        registered.value.to_string(),
        "List[Rule[\"NetError\", \"Exception\"], Rule[\"Timeout\", \"NetError\"]]"
    );
    assert!(registered.messages.is_empty());

    let reregistered = session
        .eval_text("RegisterExceptionType[\"Timeout\", \"Other\"]")
        .unwrap();
    assert_eq!(
        reregistered.value.to_string(),
        "Rule[\"Timeout\", \"NetError\"]"
    );
    assert_eq!(reregistered.messages.len(), 1);
    assert_eq!(reregistered.messages[0].symbol, "RegisterExceptionType");
    assert_eq!(reregistered.messages[0].tag, "reregister");
    assert_eq!(
        session
            .eval_text("ExceptionTypes[]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"NetError\", \"Exception\"], Rule[\"Timeout\", \"NetError\"]]"
    );
}

#[test]
fn exception_dispatch_honors_exact_ancestry_handler_order_and_outer_reraise() {
    let mut session = core_session();
    session
        .eval_text(
            "RegisterExceptionType[\"NetError\"]; \
             RegisterExceptionType[\"Timeout\", \"NetError\"]",
        )
        .unwrap();

    let exact = session
        .eval_text(
            "CatchExceptions[ThrowException[\"Timeout\", <|\"ms\" -> 30|>], \
             {Exception[\"Timeout\", p_] :> p}]",
        )
        .unwrap();
    assert_eq!(exact.value.to_string(), "Association[Rule[\"ms\", 30]]");

    let parent = session
        .eval_text(
            "CatchExceptions[ThrowException[\"Timeout\", <|\"ms\" -> 30|>], \
             {Exception[\"NetError\", p_] :> {\"caught-parent\", p}}]",
        )
        .unwrap();
    assert_eq!(
        parent.value.to_string(),
        "List[\"caught-parent\", Association[Rule[\"ms\", 30]]]"
    );

    let author_priority = session
        .eval_text(
            "CatchExceptions[ThrowException[\"Timeout\"], \
             {Exception[\"Exception\", _] :> \"root-first\", \
              Exception[\"Timeout\", _] :> \"exact-second\"}]",
        )
        .unwrap();
    assert_eq!(author_priority.value.to_string(), "\"root-first\"");

    let outer = session
        .eval_text(
            "CatchExceptions[\
               CatchExceptions[ThrowException[\"Timeout\"], \
                 {Exception[\"Other\", _] :> \"inner\"}], \
               {Exception[\"NetError\", _] :> \"outer\"}]",
        )
        .unwrap();
    assert_eq!(outer.value.to_string(), "\"outer\"");

    let catch_all = session
        .eval_text("CatchExceptions[ThrowException[\"NetError\"]]")
        .unwrap();
    assert_eq!(
        catch_all.value.to_string(),
        "Exception[\"NetError\", Association[]]"
    );
}

#[test]
fn unregistered_and_uncaught_exceptions_use_fail_closed_in_band_values() {
    let mut session = core_session();

    let unregistered = session
        .eval_request("ThrowException[\"Unregistered\"]")
        .expect("uncaught exceptions must remain in-band request values");
    assert_eq!(
        unregistered.value.to_string(),
        concat!(
            "Failure[\"UncaughtException\", Association[",
            "Rule[\"Type\", \"Exception\"], ",
            "Rule[\"Payload\", Association[",
            "Rule[\"OriginalType\", \"Unregistered\"], ",
            "Rule[\"OriginalPayload\", Association[]]]]]]"
        )
    );
    assert_eq!(unregistered.termination, None);
    assert_eq!(unregistered.messages.len(), 2);
    assert_eq!(unregistered.messages[0].symbol, "ThrowException");
    assert_eq!(unregistered.messages[0].tag, "unreg");
    assert_eq!(unregistered.messages[1].symbol, "ThrowException");
    assert_eq!(unregistered.messages[1].tag, "nocatch");

    session
        .eval_text("RegisterExceptionType[\"NetError\"]")
        .unwrap();
    let uncaught = session
        .eval_request("ThrowException[\"NetError\"]")
        .expect("uncaught registered exceptions must not become host failures");
    assert_eq!(
        uncaught.value.to_string(),
        "Failure[\"UncaughtException\", Association[Rule[\"Type\", \"NetError\"], Rule[\"Payload\", Association[]]]]"
    );
    assert_eq!(uncaught.termination, None);
    assert_eq!(uncaught.messages.len(), 1);
    assert_eq!(uncaught.messages[0].symbol, "ThrowException");
    assert_eq!(uncaught.messages[0].tag, "nocatch");
}

#[test]
fn classic_and_exception_channels_do_not_cross_catch() {
    let mut session = core_session();
    session
        .eval_text("RegisterExceptionType[\"NetError\"]")
        .unwrap();

    let through_catch = session
        .eval_request("Catch[ThrowException[\"NetError\"]]")
        .expect("classic Catch must let the in-band exception result reach the host");
    assert_eq!(
        through_catch.value.to_string(),
        "Failure[\"UncaughtException\", Association[Rule[\"Type\", \"NetError\"], Rule[\"Payload\", Association[]]]]"
    );
    assert_eq!(through_catch.messages.len(), 1);
    assert_eq!(through_catch.messages[0].tag, "nocatch");

    let through_exception_catch = session
        .eval_request("CatchExceptions[Throw[42], {Exception[_, _] :> \"x\"}]")
        .expect_err("classic Throw must propagate through CatchExceptions");
    assert_eq!(through_exception_catch.messages.len(), 1);
    assert_eq!(through_exception_catch.messages[0].symbol, "Throw");
    assert_eq!(through_exception_catch.messages[0].tag, "nocatch");
}

#[test]
fn cleanup_runs_and_enclose_does_not_intercept_exceptions() {
    let mut session = core_session();
    session
        .eval_text("RegisterExceptionType[\"NetError\"]")
        .unwrap();

    let cleaned = session
        .eval_text(
            "CatchExceptions[\
               WithCleanup[ThrowException[\"NetError\"], Print[\"cleanup\"]], \
               {Exception[_, _] :> \"caught\"}]",
        )
        .unwrap();
    assert_eq!(cleaned.value.to_string(), "\"caught\"");
    assert_eq!(cleaned.prints, vec!["cleanup"]);

    let through_enclose = session
        .eval_text("Enclose[ThrowException[\"NetError\"]]")
        .unwrap();
    assert_eq!(
        through_enclose.value.to_string(),
        "Failure[\"UncaughtException\", Association[Rule[\"Type\", \"NetError\"], Rule[\"Payload\", Association[]]]]"
    );
    assert_eq!(through_enclose.messages.len(), 1);
    assert_eq!(through_enclose.messages[0].tag, "nocatch");
}

#[test]
fn catch_exceptions_dispatch_survives_body_suspension() {
    let mut session = core_session();
    session
        .eval_text("RegisterExceptionType[\"NetError\"]")
        .unwrap();

    let failure = session
        .eval_request(
            "CatchExceptions[InputString[\"body: \"]; ThrowException[\"NetError\"], \
             {Exception[\"NetError\", _] :> \"caught\"}]",
        )
        .expect_err("the exception body should suspend for input");
    match failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let resumed = session
        .resume_eval_request_with_input("ok")
        .expect("resumed exception body should dispatch to its handler");
    assert_eq!(resumed.value.to_string(), "\"caught\"");
}
