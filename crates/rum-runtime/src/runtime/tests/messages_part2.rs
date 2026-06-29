use super::*;

#[test]
fn invalid_builtin_argument_counts_emit_runtime_messages() {
    let mut session = Session::new();

    let if_outcome = session.eval_text("If[True]").unwrap();
    assert_eq!(if_outcome.value.to_string(), "If[True]");
    assert!(
        if_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "If" && message.tag == "argtu" })
    );

    let set_outcome = session.eval_text("Set[x]").unwrap();
    assert_eq!(set_outcome.value.to_string(), "Set[x]");
    assert!(
        set_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Set" && message.tag == "argx" })
    );
}

#[test]
fn unbalanced_context_frames_emit_runtime_messages() {
    let mut session = Session::new();

    let end_outcome = session.eval_text("End[]").unwrap();
    assert_eq!(end_outcome.value.to_string(), "End[]");
    assert!(
        end_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "End" && message.tag == "context" })
    );

    let end_package_outcome = session.eval_text("EndPackage[]").unwrap();
    assert_eq!(end_package_outcome.value.to_string(), "EndPackage[]");
    assert!(
        end_package_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "EndPackage" && message.tag == "context" })
    );
}

#[test]
fn explicit_parameter_function_arity_mismatch_emits_runtime_message() {
    let mut session = Session::new();

    let outcome = session.eval_text("Function[{x, y}, x][1]").unwrap();

    assert_eq!(outcome.value.to_string(), "Function[List[x, y], x][1]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Function" && message.tag == "argx" })
    );
}

#[test]
fn check_returns_fallback_when_messages_are_emitted() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Check[FindFile[\"Missing`\"], 99]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "99");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );
}

#[test]
fn check_leaves_value_unchanged_when_no_messages_are_emitted() {
    let mut session = Session::new();

    let outcome = session.eval_text("Check[Identity[7], 99]").unwrap();

    assert_eq!(outcome.value.to_string(), "7");
    assert!(outcome.messages.is_empty());
}

#[test]
fn quiet_suppresses_runtime_messages_from_nested_evaluation() {
    let mut session = Session::new();

    let outcome = session.eval_text("Quiet[FindFile[\"Missing`\"]]").unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(outcome.messages.is_empty());
}

#[test]
fn quiet_prevents_check_from_observing_suppressed_messages() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Check[Quiet[FindFile[\"Missing`\"]], 99]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(outcome.messages.is_empty());
}

#[test]
fn check_can_target_specific_message_selectors() {
    let mut session = Session::new();

    let non_matching = session
        .eval_text("Check[FindFile[\"Missing`\"], 99, \"Get::notfound\"]")
        .unwrap();
    assert_eq!(non_matching.value.to_string(), "$Failed");
    assert!(
        non_matching
            .messages
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );

    let matching = session
        .eval_text("Check[FindFile[\"Missing`\"], 99, \"FindFile::notfound\"]")
        .unwrap();
    assert_eq!(matching.value.to_string(), "99");
    assert!(
        matching
            .messages
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );
}

#[test]
fn check_can_match_by_message_symbol_only() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Check[Needs[\"Missing`\"], 99, Needs]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "99");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Get" && message.tag == "notfound" })
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Needs" && message.tag == "load" })
    );
}

#[test]
fn quiet_can_selectively_suppress_message_streams() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Quiet[Needs[\"Missing`\"], Get]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Get" && message.tag == "notfound" })
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Needs" && message.tag == "load" })
    );
}

#[test]
fn quiet_accepts_selector_lists() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Quiet[Needs[\"Missing`\"], {Get, \"Needs::load\"}]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(outcome.messages.is_empty());
}

#[test]
fn check_and_quiet_accept_message_name_selector_forms() {
    let mut session = Session::new();

    let check_outcome = session
        .eval_text("Check[FindFile[\"Missing`\"], 99, FindFile::notfound]")
        .unwrap();
    assert_eq!(check_outcome.value.to_string(), "99");
    assert!(
        check_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );

    let quiet_outcome = session
        .eval_text("Quiet[Needs[\"Missing`\"], {Get::notfound, Needs::load}]")
        .unwrap();
    assert_eq!(quiet_outcome.value.to_string(), "$Failed");
    assert!(quiet_outcome.messages.is_empty());
}

#[test]
fn invalid_message_selector_specs_emit_runtime_messages() {
    let mut session = Session::new();

    let check_outcome = session.eval_text("Check[Identity[1], 0, 1]").unwrap();
    assert_eq!(check_outcome.value.to_string(), "Check[Identity[1], 0, 1]");
    assert!(
        check_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Check" && message.tag == "argform" })
    );

    let quiet_outcome = session.eval_text("Quiet[Identity[1], 1]").unwrap();
    assert_eq!(quiet_outcome.value.to_string(), "Quiet[Identity[1], 1]");
    assert!(
        quiet_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Quiet" && message.tag == "argform" })
    );
}

#[test]
fn non_loader_evaluation_errors_emit_general_messages_at_top_level() {
    let mut session = Session::new();
    session.eval_text("loopA := loopB").unwrap();
    session.eval_text("loopB := loopA").unwrap();

    let error = session
        .eval_text("loopA")
        .expect_err("expected a cycle error from mutual delayed references");

    assert!(matches!(
        error,
        EvalError::EvaluationCycleDetected { ref expr } if expr == "loopA"
    ));

    assert_eq!(
        session.eval_text("$MessageList").unwrap().value.to_string(),
        "List[HoldForm[General::cycle]]"
    );
    assert!(
        session
            .take_messages()
            .iter()
            .any(|message| { message.symbol == "General" && message.tag == "cycle" })
    );
}

#[test]
fn check_can_catch_non_loader_evaluation_errors_via_general_messages() {
    let mut session = Session::new();
    session.eval_text("loopA := loopB").unwrap();
    session.eval_text("loopB := loopA").unwrap();

    let outcome = session.eval_text("Check[loopA, 99, General]").unwrap();

    assert_eq!(outcome.value.to_string(), "99");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "General" && message.tag == "cycle" })
    );
}

#[test]
fn check_can_match_specific_general_error_selectors() {
    let mut session = Session::new();
    session.eval_text("loopA := loopB").unwrap();
    session.eval_text("loopB := loopA").unwrap();

    let non_matching = session
        .eval_text("Check[loopA, 99, \"General::reclim\"]")
        .expect_err("expected a cycle error when selector does not match");
    assert!(matches!(
        non_matching,
        EvalError::EvaluationCycleDetected { ref expr } if expr == "loopA"
    ));
    assert!(
        session
            .take_messages()
            .iter()
            .any(|message| { message.symbol == "General" && message.tag == "cycle" })
    );

    let matching = session
        .eval_text("Check[loopA, 99, \"General::cycle\"]")
        .unwrap();
    assert_eq!(matching.value.to_string(), "99");
    assert!(
        matching
            .messages
            .iter()
            .any(|message| { message.symbol == "General" && message.tag == "cycle" })
    );
}

#[test]
fn quiet_suppresses_general_error_messages_without_masking_errors() {
    let mut session = Session::new();
    session.eval_text("loopA := loopB").unwrap();
    session.eval_text("loopB := loopA").unwrap();

    let error = session
        .eval_text("Quiet[loopA]")
        .expect_err("expected Quiet to preserve the underlying evaluation error");

    assert!(matches!(
        error,
        EvalError::EvaluationCycleDetected { ref expr } if expr == "loopA"
    ));
    assert!(session.take_messages().is_empty());
}
