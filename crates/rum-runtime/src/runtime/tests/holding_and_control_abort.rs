use super::*;

#[test]
fn abortprotect_assert_and_blockrandom_cover_current_first_pass_subset() {
    let mut session = Session::new();

    session.eval_text("x = 0").unwrap();
    assert_eq!(
        session
            .eval_text("CheckAbort[AbortProtect[x = 1; Abort[]; x = 2; x], rescued]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");
    assert_eq!(
        session
            .eval_text("AbortProtect[CheckAbort[Abort[], rescued]]")
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[AbortProtect[CheckAbort[Abort[], rescued, PropagateAborts -> True]], outer]"
            )
            .unwrap()
            .value
            .to_string(),
        "outer"
    );
    session.eval_text("x = 0; y = unset").unwrap();
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[AbortProtect[y = CheckAbort[Abort[], inner, PropagateAborts -> True]; x = 1], rescued]"
            )
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(
        session.eval_text("{x, y}").unwrap().value.to_string(),
        "List[1, $Aborted]"
    );
    session.eval_text("x = 0; y = unset").unwrap();
    assert_eq!(
        session
            .eval_text(
                "CheckAbort[AbortProtect[y = AbortProtect[x = 1; Abort[]; x = 2]; x = 3], rescued]"
            )
            .unwrap()
            .value
            .to_string(),
        "rescued"
    );
    assert_eq!(
        session.eval_text("{x, y}").unwrap().value.to_string(),
        "List[3, 2]"
    );

    assert_eq!(
        session
            .eval_text("Assert[InputString[\"never: \"]]")
            .unwrap()
            .value
            .to_string(),
        "Assert[InputString[\"never: \"]]"
    );
    assert!(session.pending_input_request().is_none());
    let disabled_no_test = session.eval_text("Assert[]").unwrap();
    assert_eq!(disabled_no_test.value.to_string(), "Assert[]");
    assert!(disabled_no_test.messages.is_empty());
    let disabled_extra_args = session.eval_text("Assert[1, 2, 3]").unwrap();
    assert_eq!(disabled_extra_args.value.to_string(), "Assert[1, 2, 3]");
    assert!(disabled_extra_args.messages.is_empty());

    let mut callback_session = Session::new();
    assert_eq!(
        callback_session
            .eval_text("Context[$AssertFunction]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        callback_session
            .eval_text("HoldComplete[InputString[\"held: \"]]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[InputString[\"held: \"]]"
    );
    assert!(callback_session.pending_input_request().is_none());

    callback_session
        .eval_text("$AssertFunction = Print")
        .unwrap();
    let callback = callback_session
        .eval_text("Assert[Equal[1, 2], tag]")
        .unwrap();
    assert_eq!(callback.value.to_string(), "Null");
    assert!(callback.messages.is_empty());
    assert_eq!(
        callback.prints,
        vec!["HoldComplete[Assert[Equal[1, 2], tag]]"]
    );
    let callback_true = callback_session.eval_text("Assert[True]").unwrap();
    assert_eq!(callback_true.value.to_string(), "Null");
    assert!(callback_true.prints.is_empty());
    let callback_symbolic = callback_session.eval_text("Assert[p]").unwrap();
    assert_eq!(callback_symbolic.value.to_string(), "Null");
    assert_eq!(callback_symbolic.messages.len(), 1);
    assert_eq!(callback_symbolic.messages[0].symbol, "Assert");
    assert_eq!(callback_symbolic.messages[0].tag, "asrttf");
    assert_eq!(
        callback_symbolic.messages[0].text,
        "Assertion test p evaluated to p that is neither True nor False."
    );
    assert_eq!(callback_symbolic.prints, vec!["HoldComplete[Assert[p]]"]);

    callback_session
        .eval_text("$AssertFunction = Function[InputString[\"assert: \"]]")
        .unwrap();
    let failure = callback_session
        .eval_request("Assert[False]")
        .expect_err("$AssertFunction callback should be able to suspend");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "assert: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = callback_session
        .resume_eval_request_with_input("ignored")
        .expect("resuming $AssertFunction callback should finish Assert");
    assert_eq!(response.value.to_string(), "Null");

    assert_eq!(
        session.eval_text("On[Assert]").unwrap().value.to_string(),
        "Null"
    );
    assert!(session.asserts_enabled());
    let no_test = session.eval_text("Assert[]").unwrap();
    assert_eq!(no_test.value.to_string(), "Null");
    assert_eq!(no_test.messages.len(), 1);
    assert_eq!(no_test.messages[0].symbol, "Assert");
    assert_eq!(no_test.messages[0].tag, "asrte");
    assert_eq!(no_test.messages[0].text, "Assertion Assert[] had no test.");
    let extra_args = session.eval_text("Assert[1, 2, 3]").unwrap();
    assert_eq!(extra_args.value.to_string(), "Null");
    assert_eq!(extra_args.messages.len(), 2);
    assert_eq!(extra_args.messages[0].symbol, "Assert");
    assert_eq!(extra_args.messages[0].tag, "asrttf");
    assert_eq!(
        extra_args.messages[0].text,
        "Assertion test 1 evaluated to 1 that is neither True nor False."
    );
    assert_eq!(extra_args.messages[1].symbol, "Assert");
    assert_eq!(extra_args.messages[1].tag, "asrtfe");
    assert_eq!(
        extra_args.messages[1].text,
        "Assertion 1 in Assert[1, 2, 3] failed."
    );
    let true_extra_args = session.eval_text("Assert[True, tag, extra]").unwrap();
    assert_eq!(true_extra_args.value.to_string(), "Null");
    assert!(true_extra_args.messages.is_empty());
    let asserted = session.eval_text("Assert[False, tag]").unwrap();
    assert_eq!(asserted.value.to_string(), "Null");
    assert_eq!(asserted.messages.len(), 1);
    assert_eq!(asserted.messages[0].symbol, "Assert");
    assert_eq!(asserted.messages[0].tag, "asrtfe");
    assert_eq!(
        asserted.messages[0].text,
        "Assertion False in Assert[False, tag] failed."
    );
    let untagged = session.eval_text("Assert[False]").unwrap();
    assert_eq!(untagged.value.to_string(), "Null");
    assert_eq!(untagged.messages.len(), 1);
    assert_eq!(untagged.messages[0].symbol, "Assert");
    assert_eq!(untagged.messages[0].tag, "asrtf");
    assert_eq!(untagged.messages[0].text, "Assertion False failed.");
    let symbolic = session.eval_text("Assert[p]").unwrap();
    assert_eq!(symbolic.value.to_string(), "Null");
    assert_eq!(symbolic.messages.len(), 2);
    assert_eq!(symbolic.messages[0].symbol, "Assert");
    assert_eq!(symbolic.messages[0].tag, "asrttf");
    assert_eq!(
        symbolic.messages[0].text,
        "Assertion test p evaluated to p that is neither True nor False."
    );
    assert_eq!(symbolic.messages[1].symbol, "Assert");
    assert_eq!(symbolic.messages[1].tag, "asrtf");
    assert_eq!(symbolic.messages[1].text, "Assertion p failed.");
    match session
        .eval_text("Assert[InputString[\"now: \"]]")
        .expect_err("enabled Assert should evaluate its test and suspend when input is needed")
    {
        EvalError::InputRequired { request } => assert_eq!(request.prompt, "now: "),
        other => panic!("expected input-required error, got {other:?}"),
    }
    assert_eq!(
        session.eval_text("Off[Assert]").unwrap().value.to_string(),
        "Null"
    );
    assert!(!session.asserts_enabled());
    assert_eq!(
        session
            .eval_text("Assert[Equal[1, 2], tag]")
            .unwrap()
            .value
            .to_string(),
        "Assert[Equal[1, 2], tag]"
    );
    assert_eq!(
        session
            .eval_text("On[Assert, tag]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert!(session.asserts_enabled());
    let asserted_after_tagged_on = session.eval_text("Assert[False, other]").unwrap();
    assert_eq!(asserted_after_tagged_on.value.to_string(), "Null");
    assert_eq!(asserted_after_tagged_on.messages.len(), 1);
    assert_eq!(asserted_after_tagged_on.messages[0].tag, "asrtfe");
    assert_eq!(
        asserted_after_tagged_on.messages[0].text,
        "Assertion False in Assert[False, other] failed."
    );
    assert_eq!(
        session
            .eval_text("Off[Assert, tag]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert!(!session.asserts_enabled());
    assert_eq!(
        session
            .eval_text("Assert[False]")
            .unwrap()
            .value
            .to_string(),
        "Assert[False]"
    );

    assert_eq!(
        session
            .eval_text("BlockRandom[x = 7; x]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn abortprotect_and_failsafe_resume_through_host_input() {
    let mut session = Session::new();
    session.eval_text("x = 0").unwrap();

    let failure = session
        .eval_request(
            "CheckAbort[AbortProtect[x = 1; Abort[]; InputString[\"body: \"]; x = 2; x], rescued]",
        )
        .expect_err("AbortProtect body should suspend before delayed abort propagates");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming protected body should let CheckAbort catch the delayed abort");
    assert_eq!(response.value.to_string(), "rescued");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    let failure = session
        .eval_request("CheckAbort[Abort[], InputString[\"fallback: \"]]")
        .expect_err("CheckAbort fallback should suspend when host input is needed");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "fallback: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("rescued")
        .expect("resuming CheckAbort fallback should finish the caught abort");
    assert_eq!(response.value.to_string(), "\"rescued\"");

    let failure = session
        .eval_request("CheckAbort[InputString[\"body: \"]; Abort[], InputString[\"fallback: \"]]")
        .expect_err("CheckAbort body should suspend before aborting");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let failure = session
        .resume_eval_request_with_input("continue")
        .expect_err("resuming the body should reach the abort and then suspend in fallback");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "fallback: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("rescued")
        .expect("resuming nested CheckAbort fallback should finish the caught abort");
    assert_eq!(response.value.to_string(), "\"rescued\"");

    let failure = session
        .eval_request(
            "CheckAbort[CheckAbort[Abort[], rescued, PropagateAborts -> MatchQ[InputString[\"option: \"], \"yes\"]], outer]",
        )
        .expect_err("CheckAbort option RHS should suspend before the protected body runs");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "option: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("yes")
        .expect("truthy CheckAbort option RHS should propagate the abort");
    assert_eq!(response.value.to_string(), "outer");

    let failure = session
        .eval_request(
            "CheckAbort[CheckAbort[Abort[], rescued, PropagateAborts -> MatchQ[InputString[\"option: \"], \"yes\"]], outer]",
        )
        .expect_err("CheckAbort option RHS should suspend again on the next call");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "option: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("no")
        .expect("falsy CheckAbort option RHS should keep the caught fallback");
    assert_eq!(response.value.to_string(), "rescued");

    let failure = session
        .eval_request("Failsafe[f, Function[MatchQ[InputString[\"test: \"], \"yes\"]]][arg]")
        .expect_err("Failsafe test should suspend when host input is needed");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "test: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("yes")
        .expect("truthy resumed Failsafe test should call the wrapped function");
    assert_eq!(response.value.to_string(), "f[arg]");

    let failure = session
        .eval_request("Failsafe[f, Function[MatchQ[InputString[\"test: \"], \"yes\"]]][arg]")
        .expect_err("Failsafe test should suspend again for the next call");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "test: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("no")
        .expect("falsy resumed Failsafe test should produce a failure object");
    assert_eq!(
        response.value.to_string(),
        "Failure[\"FailsafeFailed\", Association[Rule[\"MessageTemplate\", \"The test failed on the arguments `1`.\"], RuleDelayed[\"MessageParameters\", List[HoldForm[arg]]], Rule[\"Arguments\", Hold[arg]]]]"
    );

    let failure = session
        .eval_request("Failsafe[Function[InputString[\"fn: \"]], Function[True]][arg]")
        .expect_err("truthy Failsafe function branch should suspend when host input is needed");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "fn: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("value")
        .expect("resuming truthy Failsafe function branch should return the supplied text");
    assert_eq!(response.value.to_string(), "\"value\"");

    let failure = session
        .eval_request("Failsafe[f, Function[False], Function[InputString[\"fail: \"]]][arg]")
        .expect_err("falsy Failsafe fail-function branch should suspend when host input is needed");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "fail: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("fallback")
        .expect("resuming Failsafe fail-function branch should return the supplied text");
    assert_eq!(response.value.to_string(), "\"fallback\"");
}
