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

#[test]
fn input_print_and_pause_cover_current_first_pass_io_subset() {
    let mut session = Session::new();

    let printed = session.eval_text("Print[\"x=\", 2]").unwrap();
    assert_eq!(printed.value.to_string(), "Null");
    assert_eq!(printed.prints, vec!["x=2"]);

    let paused = session.eval_text("Pause[0]").unwrap();
    assert_eq!(paused.value.to_string(), "Null");
    assert!(paused.prints.is_empty());

    let invalid_pause = session.eval_text("Pause[-1]").unwrap();
    assert_eq!(invalid_pause.value.to_string(), "Pause[-1]");
    assert!(
        invalid_pause
            .messages
            .iter()
            .any(|message| message.symbol == "Pause" && message.tag == "numnm")
    );

    let failure = session
        .eval_request("InputString[1, FieldMasked -> True]")
        .expect_err("InputString should accept non-string prompts and surface masked input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "1");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("secret")
        .expect("resuming masked InputString should return the supplied text");
    assert_eq!(response.value.to_string(), "\"secret\"");

    let failure = session
        .eval_request("Input[\"expr: \"]")
        .expect_err("Input should suspend until the host provides an expression");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("1 + 4")
        .expect("resuming Input should parse and evaluate the supplied expression");
    assert_eq!(response.value.to_string(), "5");
    assert!(response.prints.is_empty());

    let failure = session
        .eval_request("Input[\"expr: \", FieldMasked -> True]")
        .expect_err("Input should pass FieldMasked through to the host request");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("2 + 3")
        .expect("resuming masked Input should parse and evaluate the supplied expression");
    assert_eq!(response.value.to_string(), "5");

    session
        .eval_text("SetOptions[InputString, FieldMasked -> True]")
        .unwrap();
    let failure = session
        .eval_request("InputString[\"default masked: \"]")
        .expect_err("InputString should honor FieldMasked from SetOptions");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "default masked: ");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("secret")
        .expect("resuming default-masked InputString should return the supplied text");
    assert_eq!(response.value.to_string(), "\"secret\"");
    session
        .eval_text("SetOptions[InputString, FieldMasked -> False]")
        .unwrap();

    let unknown = session.eval_text("InputString[BadOption -> 1]").unwrap();
    assert_eq!(unknown.value.to_string(), "InputString[Rule[BadOption, 1]]");
    assert!(
        unknown
            .messages
            .iter()
            .any(|message| message.symbol == "InputString" && message.tag == "optx")
    );

    let non_option = session.eval_text("InputString[\"p\", 1]").unwrap();
    assert_eq!(non_option.value.to_string(), "InputString[\"p\", 1]");
    assert!(
        non_option
            .messages
            .iter()
            .any(|message| message.symbol == "InputString" && message.tag == "nonopt")
    );

    let invalid_option_name = session.eval_text("InputString[\"p\", 1 -> 2]").unwrap();
    assert_eq!(
        invalid_option_name.value.to_string(),
        "InputString[\"p\", Rule[1, 2]]"
    );
    assert!(
        invalid_option_name
            .messages
            .iter()
            .any(|message| message.symbol == "InputString" && message.tag == "optrs")
    );

    let input_unknown = session.eval_text("Input[BadOption -> 1]").unwrap();
    assert_eq!(input_unknown.value.to_string(), "Input[Rule[BadOption, 1]]");
    assert!(
        input_unknown
            .messages
            .iter()
            .any(|message| message.symbol == "Input" && message.tag == "optx")
    );

    let input_non_option = session.eval_text("Input[1, 2]").unwrap();
    assert_eq!(input_non_option.value.to_string(), "Input[1, 2]");
    assert!(
        input_non_option
            .messages
            .iter()
            .any(|message| message.symbol == "Input" && message.tag == "nonopt")
    );
}

#[test]
fn withcleanup_defers_cleanup_until_suspended_body_finishes() {
    let mut session = Session::new();
    session.eval_text("x = 0").unwrap();

    let failure = session
        .eval_request("WithCleanup[x = 1; InputString[\"body: \"]; x = 2; done, x = 3]")
        .expect_err("WithCleanup body should suspend before cleanup runs");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming WithCleanup body should finish and then run cleanup");
    assert_eq!(response.value.to_string(), "done");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "3");

    session.eval_text("x = 0").unwrap();
    let failure = session
        .eval_request("WithCleanup[x = 10, x = 1; InputString[\"body: \"]; x = 2; done, x = 3]")
        .expect_err("WithCleanup main body should suspend after init");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "body: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming three-argument WithCleanup body should run cleanup last");
    assert_eq!(response.value.to_string(), "done");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "3");

    session.eval_text("x = 0").unwrap();
    let failure = session
        .eval_request("WithCleanup[x = 1; done, InputString[\"cleanup: \"]; x = 2]")
        .expect_err("WithCleanup cleanup should suspend after a successful body");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cleanup: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming suspended cleanup should return the primary body value");
    assert_eq!(response.value.to_string(), "done");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "2");

    session.eval_text("x = 0").unwrap();
    let failure = session
        .eval_request(
            "CheckAbort[WithCleanup[Abort[], InputString[\"cleanup: \"]; x = 4], rescued]",
        )
        .expect_err("WithCleanup cleanup should suspend before propagating an abort");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cleanup: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "0");
    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming cleanup should let CheckAbort catch the original abort");
    assert_eq!(response.value.to_string(), "rescued");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "4");
}

#[test]
fn missingq_success_and_terminatedevaluation_cover_current_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MissingQ[Missing[]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MissingQ[Missing[\"KeyAbsent\"]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("MissingQ[1]").unwrap().value.to_string(),
        "False"
    );
    let malformed_failure = session.eval_text("Failure[\"oops\"]").unwrap();
    assert_eq!(malformed_failure.value.to_string(), "Failure[\"oops\"]");
    assert!(malformed_failure.messages.is_empty());
    let malformed_success = session.eval_text("Success[\"ok\"]").unwrap();
    assert_eq!(malformed_success.value.to_string(), "Success[\"ok\"]");
    assert!(malformed_success.messages.is_empty());
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200]]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", Association[Rule[\"Code\", 200]]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "200"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[Code -> 200, Body -> 7]][Code]")
            .unwrap()
            .value
            .to_string(),
        "200"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][Code]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Code]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][{\"Body\", \"Missing\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", \"Missing\"]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[Code -> 200, Body -> 7]][{Body, Missing}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", Missing]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][\"Properties\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"Code\", \"Body\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200]][All]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", All]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", Association[\"Code\" -> 200, \"Body\" -> 7]][Properties]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Properties]"
    );
    assert_eq!(
        session
            .eval_text("Success[tag, Association[Code -> 200]][Code]")
            .unwrap()
            .value
            .to_string(),
        "Success[tag, Association[Rule[Code, 200]]][Code]"
    );
    assert_eq!(
        session
            .eval_text("Success[1, Association[Code -> 200]][Code]")
            .unwrap()
            .value
            .to_string(),
        "Success[1, Association[Rule[Code, 200]]][Code]"
    );
    assert_eq!(
        session
            .eval_text(
                "Success[\"ok\", Association[Properties -> 1, \"Properties\" -> 2]][\"Properties\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Properties, \"Properties\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200}][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", List[Rule[\"Code\", 200]]][\"Code\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", \"Code\" -> 200][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", Rule[\"Code\", 200]][\"Code\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", 1][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", 1][\"Properties\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", 1][All]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", 1][All]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500, \"Body\" -> 7}][\"Missing\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"Missing\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", \"Code\" -> 500][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "500"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", \"Code\" :> x][All]")
            .unwrap()
            .value
            .to_string(),
        "Association[RuleDelayed[\"Code\", x]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", \"Code\" :> x][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" :> x}][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", List[RuleDelayed[\"Code\", x]]][\"Code\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][All]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"Code\", 500]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][\"All\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"All\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500, \"Body\" -> 7}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Body\", \"Code\", \"Message\", \"StyledMessage\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Code -> 500, Body -> 7}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Message\", \"StyledMessage\", Body, Code]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"A failure of type \\\"oops\\\" occurred.\""
    );
    assert_eq!(
        session
            .eval_text("Failure[tag, {\"Code\" -> 500}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"An unknown failure occurred.\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"A failure of type \\\"oops\\\" occurred.\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[tag, {\"Code\" -> 500}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"An unknown failure occurred.\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "\"oops\""
    );
    assert_eq!(
        session
            .eval_text("Failure[tag, {}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "tag"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Tag\" -> override}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "override"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Tag -> override}][Tag]")
            .unwrap()
            .value
            .to_string(),
        "override"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Tag -> override}][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "\"oops\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", 1][\"Tag\"]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", 1][\"Tag\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", 1][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", 1][\"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", 1][All]")
            .unwrap()
            .value
            .to_string(),
        "Failure[\"oops\", 1][All]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"HeldMessageTemplate\", \"Message\", \"MessageName\", \"MessageParameters\", \"MessageTemplate\", \"StyledMessage\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad ``\", \"MessageParameters\" -> {x}}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"bad x\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[\"bad `1`\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" -> {x}}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotApplicable\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `1`\"}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad `1`\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> 1}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Message\", \"MessageTemplate\", \"StyledMessage\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> 1}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"HeldMessageTemplate\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> 1}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", \"MessageName\"]"
    );
    assert_eq!(
        session
            .eval_text(
                "Failure[\"tag\", {\"MessageTemplate\" -> 1, \"HeldMessageTemplate\" -> 2}][\"HeldMessageTemplate\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Failure[\"tag\", {\"MessageTemplate\" -> 1, \"MessageName\" -> 3}][\"MessageName\"]"
            )
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> \"bad `Name`\", \"MessageParameters\" -> <|\"Name\" -> \"x\"|>}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    session.eval_text("f::named = \"bad `Name`\"").unwrap();
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad arg x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"named\"], \"MessageParameters\" -> <|\"Name\" -> \"x\"|>}][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad x\""
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"StyledMessage\"]")
            .unwrap()
            .value
            .to_string(),
        "Style[\"bad arg x\", \"Message\"]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "Hold[f::argx]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "\"argx\""
    );
    session
        .eval_text("g::argx := StringJoin[InputString[\"msg: \"], \" `1`\"]")
        .unwrap();
    let failure = session
        .eval_request("Failure[\"tag\", {\"MessageTemplate\" -> MessageName[g, \"argx\"], \"MessageParameters\" -> {x}}][\"Message\"]")
        .expect_err("delayed Failure MessageName template should suspend through host input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("bad")
        .expect("resuming Failure MessageName template should finish message rendering");
    assert_eq!(response.value.to_string(), "\"bad x\"");
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Code -> 500, Body -> 7}][Code]")
            .unwrap()
            .value
            .to_string(),
        "500"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Message -> 1, StyledMessage -> 2}][Message]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"MessageName\" -> 1}][\"MessageName\"]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"HeldMessageTemplate\" -> 2}][\"HeldMessageTemplate\"]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Failure[\"oops\", {\"Code\" -> 500, \"Body\" -> 7}][{\"Body\", \"Missing\"}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"NotAvailable\", List[\"Body\", \"Missing\"]]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {Code -> 500, Body -> 7}][{Body, Missing}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"NotAvailable\", List[Body, Missing]]]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Message\" -> 1, \"StyledMessage\" -> 2}][{\"Message\", \"StyledMessage\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Failure[\"oops\", {\"Code\" -> 500}][Properties]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotAvailable\", Properties]"
    );
    assert_eq!(
        session
            .eval_text("Failure[1, {\"Code\" -> 500}][All]")
            .unwrap()
            .value
            .to_string(),
        "Failure[1, Association[Rule[\"Code\", 500]]][All]"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[$Aborted]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[$Failed]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    let malformed_failure_q = session.eval_text("FailureQ[Failure[\"oops\"]]").unwrap();
    assert_eq!(malformed_failure_q.value.to_string(), "True");
    assert!(malformed_failure_q.messages.is_empty());
    assert_eq!(
        session
            .eval_text("FailureQ[Failure[\"oops\", bad]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Failure[1, {\"Code\" -> 500}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[]"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\", \"extra\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\", \"extra\"]"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[TerminatedEvaluation[\"RecursionLimit\"]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\"][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"][\"Properties\"]"
    );

    let mut recursion_session = Session::new();
    recursion_session.set_recursion_limit(32);
    recursion_session
        .eval_text("grow[x_] := grow[f[x]]")
        .unwrap();
    let recursion = recursion_session.eval_text("grow[1]").unwrap();
    assert_eq!(
        recursion.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert!(
        recursion
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );

    let mut iteration_session = Session::new();
    iteration_session.set_evaluation_pass_limit(1);
    let iteration = iteration_session
        .eval_text("f[f = Hold, Identity[1]]")
        .unwrap();
    assert_eq!(
        iteration.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(
        iteration
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );

    let mut request_session = Session::new();
    request_session.set_recursion_limit(32);
    request_session
        .eval_request("grow[x_] := grow[f[x]]")
        .unwrap();
    let request = request_session.eval_request("grow[1]").unwrap();
    assert_eq!(
        request.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert!(
        request
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );

    let mut data_session = Session::new();
    data_session.set_recursion_limit(32);
    data_session.eval_text("grow[x_] := grow[f[x]]").unwrap();
    let data = data_session.eval_text("EvaluationData[grow[1]]").unwrap();
    assert_eq!(
        data_session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![data.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        data_session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![data.value, Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"RecursionLimit\""
    );
}
