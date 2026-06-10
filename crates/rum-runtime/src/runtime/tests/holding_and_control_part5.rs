use super::*;

#[test]
fn table_supports_count_iterators_and_nested_iterators() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Table[x, 3]").unwrap().value.to_string(),
        "List[x, x, x]"
    );
    assert_eq!(
        session
            .eval_text("Table[i^2, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 4, 9]"
    );
    assert_eq!(
        session
            .eval_text("Table[i, {i, {2, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[i*j, {i, 2}, {j, i}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], List[2, 4]]"
    );
    assert_eq!(
        session
            .eval_text("Length[Table[i, {i, 1200}]]")
            .unwrap()
            .value
            .to_string(),
        "1200"
    );
    assert_eq!(
        session
            .eval_text("Flatten[Table[j, {i, 2}, {j, 2}], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 1, 2]"
    );
}

#[test]
fn do_and_table_respect_resource_limits() {
    let mut table_session = Session::new();
    table_session.set_generated_element_limit(3);
    let table = table_session.eval_text("Table[x, 5]").unwrap();
    assert_eq!(
        table.value.to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );
    assert!(
        table
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut do_session = Session::new();
    do_session.set_iteration_work_limit(3);
    let do_result = do_session.eval_text("Do[Null, 5]").unwrap();
    assert_eq!(
        do_result.value.to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );
    assert!(
        do_result
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut iterator_session = Session::new();
    iterator_session.set_generated_element_limit(3);
    assert_eq!(
        iterator_session
            .eval_text("Table[i, {i, 5}]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );

    let mut memory_session = Session::new();
    assert_eq!(
        memory_session
            .eval_text("MemoryConstrained[Table[x, 100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
}

#[test]
fn table_propagates_return_to_function_boundaries() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Function[Table[If[Equal[i, 2], Return[i], i], {i, 3}]][]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn failure_and_enclose_confirm_family_cover_first_pass_error_flow() {
    let mut session = Session::new();

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

#[test]
fn confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("side = 0; Enclose[Confirm[1, side = 1]]; side")
            .unwrap()
            .value
            .to_string(),
        "0"
    );

    session
        .eval_text("g[] := Confirm[$Failed, info, tag]")
        .expect("should define tagged confirmation helper");

    assert_eq!(
        session
            .eval_text("Enclose[g[], \"Information\", tag]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
}

#[test]
fn confirm_quiet_catches_and_quiets_matching_messages() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Enclose[ConfirmQuiet[First[f[]]], \"HeldMessage\"]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "Hold[First::nofirst]");
    assert!(outcome.messages.is_empty());
}
