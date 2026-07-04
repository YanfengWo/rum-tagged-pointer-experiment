use super::*;

fn assert_has_message(outcome: &EvalOutcome, symbol: &str, tag: &str) {
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag),
        "expected {symbol}::{tag}, got {:?}",
        outcome.messages
    );
}

#[test]
fn level_reports_wolfram_style_invalid_levelspec_messages() {
    let mut session = session_with_test_rewrite_builtins();

    let three_item_levelspec = session.eval_text("Level[f[a], {1, 2, 3}]").unwrap();
    assert_eq!(
        three_item_levelspec.value.to_string(),
        "Level[f[a], List[1, 2, 3]]"
    );
    assert_has_message(&three_item_levelspec, "Level", "level");

    let symbolic_levelspec = session.eval_text("Level[f[a], {1, x}]").unwrap();
    assert_eq!(
        symbolic_levelspec.value.to_string(),
        "Level[f[a], List[1, x]]"
    );
    assert_has_message(&symbolic_levelspec, "Level", "level");

    let with_apply_head = session.eval_text("Level[f[a], {1, 2, 3}, h]").unwrap();
    assert_eq!(
        with_apply_head.value.to_string(),
        "Level[f[a], List[1, 2, 3], h]"
    );
    assert_has_message(&with_apply_head, "Level", "level");
}

#[test]
fn traversal_family_reports_wolfram_style_invalid_levelspec_messages() {
    let mut session = session_with_test_rewrite_builtins();

    let cases = session.eval_text("Cases[f[a], _, {1, 2, 3}]").unwrap();
    assert_eq!(
        cases.value.to_string(),
        "Cases[f[a], Blank[], List[1, 2, 3]]"
    );
    assert_has_message(&cases, "Cases", "level");

    let delete_cases = session.eval_text("DeleteCases[f[a], _, x]").unwrap();
    assert_eq!(
        delete_cases.value.to_string(),
        "DeleteCases[f[a], Blank[], x]"
    );
    assert_has_message(&delete_cases, "DeleteCases", "level");

    let count = session.eval_text("Count[f[a], _, {1, x}]").unwrap();
    assert_eq!(count.value.to_string(), "Count[f[a], Blank[], List[1, x]]");
    assert_has_message(&count, "Count", "level");

    let member_q = session.eval_text("MemberQ[f[a], _, {1, 2, 3}]").unwrap();
    assert_eq!(
        member_q.value.to_string(),
        "MemberQ[f[a], Blank[], List[1, 2, 3]]"
    );
    assert_has_message(&member_q, "MemberQ", "level");

    let free_q = session.eval_text("FreeQ[f[a], _, x]").unwrap();
    assert_eq!(free_q.value.to_string(), "FreeQ[f[a], Blank[], x]");
    assert_has_message(&free_q, "FreeQ", "level");

    let position = session.eval_text("Position[f[a], _, {1, 2, 3}]").unwrap();
    assert_eq!(
        position.value.to_string(),
        "Position[f[a], Blank[], List[1, 2, 3]]"
    );
    assert_has_message(&position, "Position", "level");
}

#[test]
fn cases_and_position_report_wolfram_style_invalid_match_limit_messages() {
    let mut session = session_with_test_rewrite_builtins();

    let cases_limit = session.eval_text("Cases[f[a], _, {1, 2}, x]").unwrap();
    assert_eq!(
        cases_limit.value.to_string(),
        "Cases[f[a], Blank[], List[1, 2], x]"
    );
    assert_has_message(&cases_limit, "Cases", "innf");
    assert!(cases_limit.messages.iter().any(|message| {
        message.symbol == "Cases" && message.tag == "innf" && message.text.contains("position 4")
    }));

    let position_limit = session.eval_text("Position[f[a], _, {1, 2}, x]").unwrap();
    assert_eq!(
        position_limit.value.to_string(),
        "Position[f[a], Blank[], List[1, 2], x]"
    );
    assert_has_message(&position_limit, "Position", "innf");
    assert!(position_limit.messages.iter().any(|message| {
        message.symbol == "Position" && message.tag == "innf" && message.text.contains("position 4")
    }));
}
