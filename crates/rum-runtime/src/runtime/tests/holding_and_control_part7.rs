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
fn input_print_and_pause_cover_current_first_pass_io_subset() {
    let mut session = Session::new();

    let printed = session.eval_text("Print[\"x=\", 2]").unwrap();
    assert_eq!(printed.value.to_string(), "Null");
    assert_eq!(printed.prints, vec!["x=2"]);

    let paused = session.eval_text("Pause[0]").unwrap();
    assert_eq!(paused.value.to_string(), "Null");
    assert!(paused.prints.is_empty());

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
            .eval_text("Success[\"ok\", {\"Code\" -> 200, \"Body\" -> 7}][{\"Body\", \"Missing\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, Missing[\"KeyAbsent\", \"Missing\"]]"
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
