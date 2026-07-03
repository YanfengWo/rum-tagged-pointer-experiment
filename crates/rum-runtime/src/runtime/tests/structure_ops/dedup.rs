use super::*;

#[test]
fn delete_duplicates_keeps_first_structural_occurrence() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("DeleteDuplicates[{a, b, a, c, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("DeleteDuplicates[f[a, b, a, c, b]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("DeleteDuplicates[{1 + 1, 2, f[a], f[a]}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, f[a]]"
    );
}

#[test]
fn delete_duplicates_reports_current_invalid_boundaries() {
    let mut session = Session::new();

    let atomic = session.eval_text("DeleteDuplicates[a]").unwrap();
    assert_eq!(atomic.value.to_string(), "DeleteDuplicates[a]");
    assert!(
        atomic
            .messages
            .iter()
            .any(|message| message.symbol == "DeleteDuplicates" && message.tag == "normal")
    );

    let by_atomic = session.eval_text("DeleteDuplicatesBy[a, f]").unwrap();
    assert_eq!(by_atomic.value.to_string(), "DeleteDuplicatesBy[a, f]");
    assert!(
        by_atomic
            .messages
            .iter()
            .any(|message| message.symbol == "DeleteDuplicatesBy" && message.tag == "normal")
    );

    let by_one_arg = session.eval_text("DeleteDuplicatesBy[f]").unwrap();
    assert_eq!(by_one_arg.value.to_string(), "DeleteDuplicatesBy[f]");
    assert!(by_one_arg.messages.is_empty());
}
