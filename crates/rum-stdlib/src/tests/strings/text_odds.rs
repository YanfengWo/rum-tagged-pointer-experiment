use super::*;

#[test]
fn string_pad_right_mirrors_left_padding_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringPadRight[\"7\", 3]"),
        "\"7  \""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadRight[{\"7\", \"88\"}, 3, \"0\"]"),
        "List[\"700\", \"880\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadRight[\"abcd\", 2]"),
        "\"abcd\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadRight[\"a\", 3, \"\"]"),
        "StringPadRight[\"a\", 3, \"\"]"
    );
}

#[test]
fn string_lines_splits_lf_and_crlf_with_trailing_newline_rule() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringLines[\"a\\r\\nb\\n\\nc\"]"),
        "List[\"a\", \"b\", \"\", \"c\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLines[\"a\\n\"]"),
        "List[\"a\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLines[\"\"]"),
        "List[\"\"]"
    );
}

#[test]
fn decimal_string_formats_exact_numbers_with_fixed_fraction_digits() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "DecimalString[157/50, 2]"),
        "\"3.14\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[1/3, 4]"),
        "\"0.3333\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[2, 3]"),
        "\"2.000\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[-7/8, 3]"),
        "\"-0.875\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[999/1000, 2]"),
        "\"1.00\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[1/200, 2]"),
        "\"0.01\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[-1/200, 2]"),
        "\"-0.01\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[5/2, 0]"),
        "\"3\""
    );
    assert_eq!(
        eval_to_string(&mut session, "DecimalString[-1/1000, 2]"),
        "\"-0.00\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "DecimalString[123456789123456789123456789/100000000000000000000000000, 5]"
        ),
        "\"1.23457\""
    );
}

#[test]
fn decimal_string_invalid_forms_remain_inert() {
    let mut session = core_session();

    let non_number = session.eval_text("DecimalString[x, 2]").unwrap();
    assert_eq!(non_number.value.to_string(), "DecimalString[x, 2]");
    assert!(non_number.messages.is_empty());

    let invalid_digits = session.eval_text("DecimalString[1/2, -1]").unwrap();
    assert_eq!(invalid_digits.value.to_string(), "DecimalString[1/2, -1]");
    assert!(invalid_digits.messages.is_empty());

    let arity = session.eval_text("DecimalString[1]").unwrap();
    assert_eq!(arity.value.to_string(), "DecimalString[1]");
    assert!(
        arity
            .messages
            .iter()
            .any(|message| message.symbol == "DecimalString" && message.tag == "argx")
    );
}

#[test]
fn string_text_odds_report_invalid_arities() {
    let mut session = core_session();

    let pad = session.eval_text("StringPadRight[\"a\"]").unwrap();
    assert_eq!(pad.value.to_string(), "StringPadRight[\"a\"]");
    assert!(
        pad.messages
            .iter()
            .any(|message| message.symbol == "StringPadRight" && message.tag == "argtu")
    );

    let lines = session.eval_text("StringLines[]").unwrap();
    assert_eq!(lines.value.to_string(), "StringLines[]");
    assert!(
        lines
            .messages
            .iter()
            .any(|message| message.symbol == "StringLines" && message.tag == "argx")
    );
}
