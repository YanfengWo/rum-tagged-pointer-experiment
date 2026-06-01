use super::*;

#[test]
fn failure_message_template_properties_cover_current_subset() {
    let mut session = Session::new();

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
