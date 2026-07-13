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

fn assert_has_message_text(outcome: &EvalOutcome, symbol: &str, tag: &str, text: &str) {
    assert!(
        outcome.messages.iter().any(|message| {
            message.symbol == symbol && message.tag == tag && message.text == text
        }),
        "expected {symbol}::{tag}: {text}, got {:?}",
        outcome.messages
    );
}

#[test]
fn position_operator_form_preserves_data_and_error_forms() {
    let mut session = session_with_test_rewrite_builtins();

    let wrapper = session.eval_text("Position[_Integer]").unwrap();
    assert_eq!(wrapper.value.to_string(), "Position[Blank[Integer]]");
    assert!(wrapper.messages.is_empty());

    let applied = session.eval_text("Position[_Integer][{1, a, 2}]").unwrap();
    let ordinary = session.eval_text("Position[{1, a, 2}, _Integer]").unwrap();
    assert_eq!(applied.value, ordinary.value);
    assert!(applied.messages.is_empty());
    assert!(ordinary.messages.is_empty());

    let data = session
        .eval_text("Position[f[a, f[a]], a, Infinity, 1, Heads -> False]")
        .unwrap();
    assert_eq!(data.value.to_string(), "List[List[1]]");
    assert!(data.messages.is_empty());

    let zero = session.eval_text("Position[]").unwrap();
    assert_eq!(zero.value.to_string(), "Position[]");
    assert_has_message_text(
        &zero,
        "Position",
        "argtu",
        "expected between 2 and 5 argument(s), got 0",
    );

    let invalid_application = session.eval_text("Position[_Integer][a, b]").unwrap();
    assert_eq!(
        invalid_application.value.to_string(),
        "Position[Blank[Integer]][a, b]"
    );
    assert_has_message_text(
        &invalid_application,
        "Position",
        "argx",
        "Position[Blank[Integer]] called with 2 arguments; 1 argument is expected.",
    );
}

#[test]
fn count_operator_form_preserves_data_and_error_forms() {
    let mut session = session_with_test_rewrite_builtins();

    let wrapper = session.eval_text("Count[_Integer]").unwrap();
    assert_eq!(wrapper.value.to_string(), "Count[Blank[Integer]]");
    assert!(wrapper.messages.is_empty());

    let applied = session.eval_text("Count[_Integer][{1, a, 2}]").unwrap();
    let ordinary = session.eval_text("Count[{1, a, 2}, _Integer]").unwrap();
    assert_eq!(applied.value, ordinary.value);
    assert!(applied.messages.is_empty());
    assert!(ordinary.messages.is_empty());

    let data = session
        .eval_text("Count[f[a, f[a]], a, Infinity, Heads -> False]")
        .unwrap();
    assert_eq!(data.value.to_string(), "2");
    assert!(data.messages.is_empty());

    let zero = session.eval_text("Count[]").unwrap();
    assert_eq!(zero.value.to_string(), "Count[]");
    assert_has_message_text(
        &zero,
        "Count",
        "argtu",
        "expected between 2 and 4 argument(s), got 0",
    );

    let invalid_application = session.eval_text("Count[_Integer][a, b]").unwrap();
    assert_eq!(
        invalid_application.value.to_string(),
        "Count[Blank[Integer]][a, b]"
    );
    assert_has_message_text(
        &invalid_application,
        "Count",
        "argx",
        "Count[Blank[Integer]] called with 2 arguments; 1 argument is expected.",
    );
}

#[test]
fn memberq_operator_form_preserves_data_and_error_forms() {
    let mut session = session_with_test_rewrite_builtins();

    let wrapper = session.eval_text("MemberQ[_Integer]").unwrap();
    assert_eq!(wrapper.value.to_string(), "MemberQ[Blank[Integer]]");
    assert!(wrapper.messages.is_empty());

    let applied = session.eval_text("MemberQ[_Integer][{a, 1}]").unwrap();
    let ordinary = session.eval_text("MemberQ[{a, 1}, _Integer]").unwrap();
    assert_eq!(applied.value, ordinary.value);
    assert!(applied.messages.is_empty());
    assert!(ordinary.messages.is_empty());

    let data = session
        .eval_text("MemberQ[f[g[a]], a, Infinity, Heads -> False]")
        .unwrap();
    assert_eq!(data.value.to_string(), "True");
    assert!(data.messages.is_empty());

    let zero = session.eval_text("MemberQ[]").unwrap();
    assert_eq!(zero.value.to_string(), "MemberQ[]");
    assert_has_message_text(
        &zero,
        "MemberQ",
        "argtu",
        "expected between 2 and 4 argument(s), got 0",
    );

    let invalid_application = session.eval_text("MemberQ[_Integer][a, b]").unwrap();
    assert_eq!(
        invalid_application.value.to_string(),
        "MemberQ[Blank[Integer]][a, b]"
    );
    assert_has_message_text(
        &invalid_application,
        "MemberQ",
        "argx",
        "MemberQ[Blank[Integer]] called with 2 arguments; 1 argument is expected.",
    );
}

#[test]
fn freeq_operator_form_preserves_data_and_error_forms() {
    let mut session = session_with_test_rewrite_builtins();

    let wrapper = session.eval_text("FreeQ[_Integer]").unwrap();
    assert_eq!(wrapper.value.to_string(), "FreeQ[Blank[Integer]]");
    assert!(wrapper.messages.is_empty());

    let applied = session.eval_text("FreeQ[_Integer][{a, b}]").unwrap();
    let ordinary = session.eval_text("FreeQ[{a, b}, _Integer]").unwrap();
    assert_eq!(applied.value, ordinary.value);
    assert!(applied.messages.is_empty());
    assert!(ordinary.messages.is_empty());

    let data = session
        .eval_text("FreeQ[f[g[a]], a, {1}, Heads -> False]")
        .unwrap();
    assert_eq!(data.value.to_string(), "True");
    assert!(data.messages.is_empty());

    let zero = session.eval_text("FreeQ[]").unwrap();
    assert_eq!(zero.value.to_string(), "FreeQ[]");
    assert_has_message_text(
        &zero,
        "FreeQ",
        "argtu",
        "expected between 2 and 4 argument(s), got 0",
    );

    let invalid_application = session.eval_text("FreeQ[_Integer][a, b]").unwrap();
    assert_eq!(
        invalid_application.value.to_string(),
        "FreeQ[Blank[Integer]][a, b]"
    );
    assert_has_message_text(
        &invalid_application,
        "FreeQ",
        "argx",
        "FreeQ[Blank[Integer]] called with 2 arguments; 1 argument is expected.",
    );
}

#[test]
fn level_reports_invalid_levelspec_messages() {
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
fn traversal_family_reports_invalid_levelspec_messages() {
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
fn cases_and_position_report_invalid_match_limit_messages() {
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
