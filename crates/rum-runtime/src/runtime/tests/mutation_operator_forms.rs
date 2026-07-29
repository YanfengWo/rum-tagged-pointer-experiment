use super::*;

#[test]
fn kernel_mutation_operator_forms_apply_to_raw_held_targets() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(
            "x = 10; sx = 10; tx = 10; dx = 10; z = 7; ap = {1, 2}; pp = {1, 2}; \
             {{AddTo[5][x], x}, {SubtractFrom[3][sx], sx}, {TimesBy[4][tx], tx}, \
              {DivideBy[2][dx], dx}, {ApplyTo[Function[t, t + 1]][z], z}, \
              {AppendTo[3][ap], ap}, {PrependTo[0][pp], pp}}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[15, 15], List[7, 7], List[40, 40], List[5, 5], List[8, 8], \
         List[List[1, 2, 3], List[1, 2, 3]], List[List[0, 1, 2], List[0, 1, 2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_capture_defers_its_value_until_application() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("c = 0; w = AppendTo[c = c + 1]; l = {}; {c, w[l], c, l}")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[0, List[1], 1, List[1]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_and_add_to_direct_calls_match_their_operator_forms() {
    let mut operator_session = session_with_test_control_builtins();
    let operator = operator_session
        .eval_text("x = {1, 2}; y = 10; {{AppendTo[3][x], x}, {AddTo[5][y], y}}")
        .unwrap();

    let mut direct_session = session_with_test_control_builtins();
    let direct = direct_session
        .eval_text("x = {1, 2}; y = 10; {{AppendTo[x, 3], x}, {AddTo[y, 5], y}}")
        .unwrap();

    assert_eq!(operator.value, direct.value);
    assert_eq!(
        direct.value.to_string(),
        "List[List[List[1, 2, 3], List[1, 2, 3]], List[15, 15]]"
    );
    assert_eq!(operator.messages, direct.messages);
    assert!(direct.messages.is_empty());
}

#[test]
fn mutation_captures_are_quiet_but_invalid_applications_report_argx() {
    let mut session = session_with_test_control_builtins();

    let captures = session
        .eval_text(
            "{AddTo[1], SubtractFrom[1], TimesBy[1], DivideBy[1], ApplyTo[1], \
             AppendTo[1], PrependTo[1]}",
        )
        .unwrap();
    assert_eq!(
        captures.value.to_string(),
        "List[AddTo[1], SubtractFrom[1], TimesBy[1], DivideBy[1], ApplyTo[1], \
         AppendTo[1], PrependTo[1]]"
    );
    assert!(captures.messages.is_empty());

    for (source, expected, count) in [
        ("AppendTo[3][]", "AppendTo[3][]", 0),
        ("AppendTo[3][x, y]", "AppendTo[3][x, y]", 2),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == "AppendTo"
                && message.tag == "argx"
                && message.text
                    == format!("AppendTo[3] called with {count} arguments; 1 argument is expected.")
        }));
    }

    for (source, expected_count) in [("AddTo[]", 0), ("AddTo[a, b, c]", 3)] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), source);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == "AddTo"
                && message.tag == "argx"
                && message.text == format!("expected exactly 2 argument(s), got {expected_count}")
        }));
    }
}

#[test]
fn mutation_operator_form_preserves_direct_protection_failure() {
    let mut operator_session = session_with_test_control_builtins();
    let operator = operator_session
        .eval_text("Protect[p]; AppendTo[1][p]")
        .unwrap();

    let mut direct_session = session_with_test_control_builtins();
    let direct = direct_session
        .eval_text("Protect[p]; AppendTo[p, 1]")
        .unwrap();

    assert_eq!(operator.value, direct.value);
    assert_eq!(operator.value.to_string(), "AppendTo[p, 1]");
    for outcome in [&operator, &direct] {
        assert!(!outcome.messages.is_empty());
        assert!(outcome.messages.iter().all(|message| {
            message.symbol == "AppendTo"
                && message.tag == "wrsym"
                && message.text == "Symbol p is Protected."
        }));
    }
}

#[test]
fn kernel_mutation_heads_advertise_subvalues_hold_all() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(
            "{Attributes[AddTo], Attributes[SubtractFrom], Attributes[TimesBy], \
             Attributes[DivideBy], Attributes[ApplyTo], Attributes[AppendTo], \
             Attributes[PrependTo]}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_plain_list_preserves_alias_and_returned_value() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("x = {1, 2}; y = x; {AppendTo[y, 3], x, y}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[1, 2, 3], List[1, 2], List[1, 2, 3]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_non_list_head_still_runs_matching_user_code() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(
            "lst2 = f[]; f[z___] := (seen2 = \"RAN\"; g[z]); \
             {AppendTo[lst2, 1], seen2, lst2}",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[g[1], \"RAN\", g[1]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_nothing_keeps_list_normalization_semantics() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("x = {1, 2}; {AppendTo[x, Nothing], x}")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[List[1, 2], List[1, 2]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_list_upvalue_still_observes_the_old_value_and_fires() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(
            "x = {u}; u /: List[u, 1] := (seen = x; transformed); \
             {AppendTo[x, 1], seen, x}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[transformed, List[u], transformed]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_list_downvalue_still_observes_the_old_value_and_fires() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("x = {u}").unwrap();
    session
        .eval_text("List[u, 1] := (seen = x; transformed)")
        .unwrap();

    let appended = session.eval_text("AppendTo[x, 1]").unwrap();
    let observed = session.eval_text("seen").unwrap();
    let stored = session.eval_text("x").unwrap();

    assert_eq!(appended.value.to_string(), "transformed");
    assert_eq!(observed.value.to_string(), "List[u]");
    assert_eq!(stored.value.to_string(), "transformed");
    assert!(appended.messages.is_empty());
}

#[test]
fn append_to_with_tracing_preserves_the_existing_trace() {
    let mut session = Session::new();
    session.eval_text("x = {}").unwrap();

    let traced = session.eval_text("Trace[AppendTo[x, 1]]").unwrap();

    assert_eq!(
        traced.value.to_string(),
        "List[Hold[AppendTo[x, 1]], Hold[List[1]]]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "List[1]");
}

#[test]
fn append_to_with_monitor_preserves_old_and_new_display_observations() {
    let mut session = Session::new();
    session.eval_text("x = {}").unwrap();

    let monitored = session.eval_text("Monitor[AppendTo[x, 1], x]").unwrap();

    assert_eq!(monitored.value.to_string(), "List[1]");
    assert_eq!(monitored.prints, vec!["List[]", "List[1]"]);
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "List[1]");
}

#[test]
fn append_to_protected_list_target_preserves_value_and_message() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("protectedList = {1, 2}").unwrap();
    session.eval_text("Protect[protectedList]").unwrap();

    let outcome = session.eval_text("AppendTo[protectedList, 3]").unwrap();

    assert_eq!(outcome.value.to_string(), "AppendTo[protectedList, 3]");
    assert_eq!(outcome.messages.len(), 1);
    assert_eq!(outcome.messages[0].symbol, "AppendTo");
    assert_eq!(outcome.messages[0].tag, "wrsym");
    assert_eq!(
        outcome.messages[0].text,
        "Symbol protectedList is Protected."
    );
    assert_eq!(
        session
            .eval_text("protectedList")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
}

#[test]
fn append_to_rolls_back_if_fast_update_fails_after_definition_take() {
    let mut session = Session::new();
    let target = Symbol::new(GLOBAL_CONTEXT, "rollbackAppendTarget");
    session.eval_text("rollbackAppendTarget = {1, 2}").unwrap();
    drop(session.eval_text("rollbackAppendTarget").unwrap());
    session.fail_next_incremental_append_after_value_take(target);

    let outcome = session
        .eval_text("AppendTo[rollbackAppendTarget, 3]")
        .unwrap();

    assert_eq!(outcome.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"Cancelled\"]"
    );
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref())
            .expect("rollback must restore the definition value")
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("rollbackAppendTarget")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
}

#[test]
fn append_to_item_that_requires_target_evaluation_stays_on_evaluated_path() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("x = {1}; {AppendTo[x, Unevaluated[x]], x}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[1, List[1]], List[1, List[1]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_modified_list_attributes_stays_on_evaluated_path() {
    let mut session = Session::new();
    session.eval_text("x = {2}").unwrap();
    session.eval_text("SetAttributes[List, Orderless]").unwrap();

    let outcome = session.eval_text("AppendTo[x, 1]").unwrap();

    assert_eq!(outcome.value.to_string(), "List[1, 2]");
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[1, 2]"
    );
}

#[test]
fn append_to_resumes_item_input_without_losing_or_replaying_the_update() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("inputList = {}").unwrap();

    let failure = session
        .eval_request("AppendTo[inputList, InputString[\"item: \"]]")
        .expect_err("the item should suspend before AppendTo runs");
    match failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "item: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(
        session.eval_text("inputList").unwrap().value.to_string(),
        "List[]"
    );

    let resumed = session
        .resume_eval_request_with_input("value")
        .expect("the suspended AppendTo should resume");

    assert_eq!(resumed.value.to_string(), "List[\"value\"]");
    assert_eq!(
        session.eval_text("inputList").unwrap().value.to_string(),
        "List[\"value\"]"
    );
}

#[test]
fn append_to_exact_part_preserves_outer_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text("o = {{1}, {2}}; a = o; {AppendTo[o[[1]], 3], a, o}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[1, 3], List[List[1], List[2]], List[List[1, 3], List[2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_exact_part_preserves_inner_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text("o = {{1}, {2}}; b = o[[1]]; {AppendTo[o[[1]], 3], b, o}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[1, 3], List[1], List[List[1, 3], List[2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_deep_exact_part_preserves_middle_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "o = {{{{1}}}}; middle = o[[1, 1]]; \
             {AppendTo[o[[1, 1, 1]], 2], middle, o}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[1, 2], List[List[1]], List[List[List[List[1, 2]]]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_exact_part_copies_list_shared_at_two_positions() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text("i = {1}; o = {i, i}; {AppendTo[o[[1]], 2], o}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[1, 2], List[List[1, 2], List[1]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_complex_part_selectors_keep_generic_behavior() {
    for (source, expected) in [
        (
            "o = {{1}, {2}}; {AppendTo[o[[All]], 3], o}",
            "List[List[List[1], List[2], 3], List[List[List[1], List[2], 3], \
             List[List[1], List[2], 3]]]",
        ),
        (
            "o = {{1}, {2}}; {AppendTo[o[[{1, 1}]], 3], o}",
            "List[List[List[1], List[1], 3], List[List[List[1], List[1], 3], List[2]]]",
        ),
        (
            "o = {{1}, {2}}; {AppendTo[o[[{1, 2}]], 3], o}",
            "List[List[List[1], List[2], 3], List[List[List[1], List[2], 3], \
             List[List[1], List[2], 3]]]",
        ),
        (
            "o = {{1}, {2}}; {AppendTo[o[[-1]], 3], o}",
            "List[List[2, 3], List[List[1], List[2, 3]]]",
        ),
        (
            "o = {{1}, {2}}; {AppendTo[o[[3]], 3], o}",
            "List[AppendTo[Part[o, 3], 3], List[List[1], List[2]]]",
        ),
    ] {
        let mut session = session_with_test_control_builtins();
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source: {source}");
        assert!(outcome.messages.is_empty(), "source: {source}");
    }
}

#[test]
fn append_to_association_key_part_stays_on_generic_path() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text("a = <|x -> {1}|>; {AppendTo[a[[Key[x]]], 2], a}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[AppendTo[Part[a, Key[x]], 2], Association[Rule[x, List[1]]]]"
    );
    assert_eq!(outcome.messages.len(), 6);
    assert!(outcome.messages.iter().all(|message| {
        matches!(message.symbol.as_str(), "Part" | "Set")
            && message.tag == "pkspec1"
            && message.text == "The expression Key[x] cannot be used as a part specification."
    }));
}

#[test]
fn append_to_exact_part_non_list_leaf_still_runs_definitions() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "o = {f[]}; f[z___] := (seen = \"RAN\"; g[z]); \
             {AppendTo[o[[1]], 1], seen, o}",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[g[1], \"RAN\", List[g[1]]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn append_to_exact_part_keeps_nothing_and_upvalue_behavior() {
    let mut nothing_session = session_with_test_control_builtins();
    let nothing = nothing_session
        .eval_text("o = {{1, 2}}; {AppendTo[o[[1]], Nothing], o}")
        .unwrap();
    assert_eq!(
        nothing.value.to_string(),
        "List[List[1, 2], List[List[1, 2]]]"
    );
    assert!(nothing.messages.is_empty());

    let mut upvalue_session = session_with_test_control_builtins();
    let upvalue = upvalue_session
        .eval_text(
            "o = {{u}, {2}}; u /: List[u, 3] := (seen = o; transformed); \
             {AppendTo[o[[1]], 3], seen, o}",
        )
        .unwrap();
    assert_eq!(
        upvalue.value.to_string(),
        "List[transformed, List[List[u], List[2]], List[transformed, List[2]]]"
    );
    assert!(upvalue.messages.is_empty());
}

#[test]
fn append_to_exact_part_with_tracing_preserves_existing_trace() {
    let mut session = Session::new();
    session.eval_text("o = {{1}, {2}}").unwrap();
    let outcome = session.eval_text("Trace[AppendTo[o[[1]], 3]]").unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[Hold[AppendTo[Part[o, 1], 3]], Hold[List[1, 3]]]"
    );
    assert_eq!(
        session.eval_text("o").unwrap().value.to_string(),
        "List[List[1, 3], List[2]]"
    );
}

#[test]
fn append_to_exact_part_with_monitor_preserves_spine_observations() {
    let mut session = Session::new();
    session.eval_text("o = {{1}, {2}}").unwrap();
    let outcome = session
        .eval_text("Monitor[AppendTo[o[[1]], 3], o]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[1, 3]");
    assert_eq!(
        outcome.prints,
        vec!["List[List[1], List[2]]", "List[List[1, 3], List[2]]"]
    );
}

#[test]
fn append_to_exact_part_protected_owner_still_refuses() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("o = {{1}, {2}}; Protect[o]").unwrap();
    let outcome = session.eval_text("AppendTo[o[[1]], 3]").unwrap();

    assert_eq!(outcome.value.to_string(), "AppendTo[Part[o, 1], 3]");
    assert!(!outcome.messages.is_empty());
    assert!(
        outcome
            .messages
            .iter()
            .all(|message| message.symbol == "AppendTo" && message.tag == "wrsym")
    );
    assert_eq!(
        session.eval_text("o").unwrap().value.to_string(),
        "List[List[1], List[2]]"
    );
}

#[test]
fn append_to_exact_part_rolls_back_multilevel_spine() {
    let mut session = Session::new();
    let target = Symbol::new(GLOBAL_CONTEXT, "rollbackPartTarget");
    session.eval_text("rollbackPartTarget = {{{{1}}}}").unwrap();
    session
        .eval_text("outerAlias = rollbackPartTarget")
        .unwrap();
    session
        .eval_text("middleAlias = rollbackPartTarget[[1, 1]]")
        .unwrap();
    drop(session.eval_text("rollbackPartTarget[[1, 1, 1]]").unwrap());
    session.fail_next_incremental_append_after_value_take(target);

    let outcome = session
        .eval_text("AppendTo[rollbackPartTarget[[1, 1, 1]], 2]")
        .unwrap();

    assert_eq!(outcome.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref())
            .expect("rollback must restore the whole definition spine")
            .to_string(),
        "List[List[List[List[1]]]]"
    );
    assert_eq!(
        session
            .eval_text("{rollbackPartTarget, outerAlias, middleAlias}")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[List[List[1]]]], List[List[List[List[1]]]], List[List[1]]]"
    );
}
