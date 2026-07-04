use super::super::*;
use super::support::*;

// Moved from crates/rum-runtime/src/runtime/tests/holding_and_control_failure_messages.rs.

#[test]
fn failure_message_template_properties_cover_current_subset() {
    let mut session = core_session();

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
    session.eval_text("delayedParamsSide = 0").unwrap();
    assert_eq!(
        session
            .eval_text("Failure[\"tag\", Association[\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" :> (delayedParamsSide = 1; {a})]][\"Message\"]")
            .unwrap()
            .value
            .to_string(),
        "\"bad a\""
    );
    assert_eq!(
        session
            .eval_text("delayedParamsSide")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    let failure = session
        .eval_request("Failure[\"tag\", Association[\"MessageTemplate\" -> \"bad `1`\", \"MessageParameters\" :> {InputString[\"param: \"]}]][\"Message\"]")
        .expect_err("delayed Failure MessageParameters should suspend through host input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "param: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resuming Failure MessageParameters should finish message rendering");
    assert_eq!(response.value.to_string(), "\"bad ok\"");
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
}

// Moved from crates/rum-runtime/src/runtime/tests/holding_and_control_part5.rs.

#[test]
fn failure_and_enclose_confirm_family_cover_first_pass_error_flow() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("FailureQ[Failure[\"oops\", {\"Information\" -> 1}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Enclose[Confirm[$Failed]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Failed, info], \"Information\"]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[2, IntegerQ]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[x, IntegerQ, info], \"Function\"]")
            .unwrap()
            .value
            .to_string(),
        "IntegerQ"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmMatch[2, _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmMatch[x, _Integer, info], \"Pattern\"]")
            .unwrap()
            .value
            .to_string(),
        "Blank[Integer]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[True]]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[False, info], \"Information\"]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );

    session
        .eval_text("f::argx := StringJoin[InputString[\"msg: \"], \" `1`\"]")
        .unwrap();
    let failure = session
        .eval_request("Enclose[Confirm[Failure[\"tag\", {\"MessageTemplate\" -> MessageName[f, \"argx\"], \"MessageParameters\" -> {x}}]], \"Message\"]")
        .expect_err("Enclose string property handler should suspend while rendering Failure MessageName template");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("bad")
        .expect("resuming Enclose property handler should finish rendering Failure message");
    assert_eq!(response.value.to_string(), "\"bad x\"");
}

// Moved from crates/rum-runtime/src/runtime/tests/holding_and_control_part8.rs.

#[test]
fn confirm_failure_wrapping_resumes_delayed_source_failure_message_template() {
    let mut session = core_session();

    session
        .eval_text("g::argx := StringJoin[InputString[\"msg: \"], \" `1`\"]")
        .unwrap();
    let failure = session
        .eval_request("Enclose[Confirm[Failure[\"tag\", {\"MessageTemplate\" -> MessageName[g, \"argx\"], \"MessageParameters\" -> {x}}]], \"MessageTemplate\"]")
        .expect_err("Confirm should suspend while rendering the source Failure message");
    expect_input_prompt(failure, "msg: ");

    let response = session
        .resume_eval_request_with_input("bad")
        .expect("resuming source Failure message rendering should finish Confirm wrapping");
    assert_eq!(response.value.to_string(), "\"bad x\"");

    let failure = session
        .eval_request("Enclose[Confirm[Failure[\"tag\", {\"MessageTemplate\" -> MessageName[g, \"argx\"], \"MessageParameters\" -> {x}}]], \"MessageParameters\"]")
        .expect_err("Confirm should suspend before producing wrapped message parameters");
    expect_input_prompt(failure, "msg: ");

    let response = session
        .resume_eval_request_with_input("ignored")
        .expect("resuming source Failure message rendering should expose wrapped parameters");
    assert_eq!(response.value.to_string(), "List[]");
}

// Moved from crates/rum-runtime/src/runtime/tests/messages.rs.

#[test]
fn message_list_reports_last_top_level_messages() {
    let mut session = core_session();

    session.eval_text("Message[FindFile::notfound]").unwrap();

    assert_eq!(
        session.eval_text("$MessageList").unwrap().value.to_string(),
        "List[HoldForm[FindFile::notfound]]"
    );

    let mut string_session = core_session();
    string_session
        .eval_text("Message[FindFile::notfound]")
        .unwrap();
    assert_eq!(
        string_session
            .eval_text("ToString[$MessageList, InputForm]")
            .unwrap()
            .value
            .to_string(),
        "\"{HoldForm[FindFile::notfound]}\""
    );
}
