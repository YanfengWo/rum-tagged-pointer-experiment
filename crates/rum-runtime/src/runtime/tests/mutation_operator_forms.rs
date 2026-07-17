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
