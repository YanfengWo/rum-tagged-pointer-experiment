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
