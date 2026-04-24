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

    assert_eq!(
        session
            .eval_text("Assert[InputString[\"never: \"]]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert!(session.pending_input_request().is_none());

    session.enable_asserts(true);
    let asserted = session.eval_text("Assert[Equal[1, 2], tag]").unwrap();
    assert_eq!(asserted.value.to_string(), "Null");
    assert_eq!(asserted.messages.len(), 1);
    assert_eq!(asserted.messages[0].symbol, "Assert");
    assert_eq!(asserted.messages[0].tag, "fail");
    assert!(asserted.messages[0].text.contains("tag"));
    match session
        .eval_text("Assert[InputString[\"now: \"]]")
        .expect_err("enabled Assert should evaluate its test and suspend when input is needed")
    {
        EvalError::InputRequired { request } => assert_eq!(request.prompt, "now: "),
        other => panic!("expected input-required error, got {other:?}"),
    }

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
        "Failure[\"FailsafeFailure\", Association[Rule[\"Function\", f], Rule[\"Test\", Function[MatchQ[InputString[\"test: \"], \"yes\"]]], Rule[\"Arguments\", List[arg]]]]"
    );
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

    let failure = session
        .eval_request("InputString[1, FieldMasked -> True]")
        .expect_err("InputString should accept non-string prompts and surface masked input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
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
            assert_eq!(request.prompt, "expr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("1 + 4")
        .expect("resuming Input should parse and evaluate the supplied expression");
    assert_eq!(response.value.to_string(), "5");
    assert!(response.prints.is_empty());
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
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200}]")
            .unwrap()
            .value
            .to_string(),
        "Success[\"ok\", Association[Rule[\"Code\", 200]]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200, \"Body\" -> 7}][\"Code\"]")
            .unwrap()
            .value
            .to_string(),
        "200"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {Code -> 200, Body -> 7}][Code]")
            .unwrap()
            .value
            .to_string(),
        "200"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200, \"Body\" -> 7}][Code]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Code]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200, \"Body\" -> 7}][{\"Body\", \"Missing\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", \"Missing\"]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {Code -> 200, Body -> 7}][{Body, Missing}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", Missing]]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200, \"Body\" -> 7}][\"Properties\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Code\", \"Body\"]"
    );
    assert_eq!(
        session
            .eval_text("Success[\"ok\", {\"Code\" -> 200, \"Body\" -> 7}][Properties]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Properties]"
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
    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
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
    assert_eq!(
        session
            .eval_text("TerminatedEvaluation[\"RecursionLimit\"]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
}
