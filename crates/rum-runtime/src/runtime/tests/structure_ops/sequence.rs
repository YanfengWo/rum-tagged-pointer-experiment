use super::*;

#[test]
fn take_drop_insert_delete_and_take_list_cover_current_structural_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], -2]")
            .unwrap()
            .value
            .to_string(),
        "f[c, d]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], {2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], {2, -1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[g[a, b], h[c, d]], All, {2}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b], None]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );

    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "f[c, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], -2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], {2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[g[a, b], h[c, d]], None, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("TakeDrop[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], f[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("Insert[f[a, b, c], x, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, x, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Insert[f[a, b, c], x, -1]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c, x]"
    );
    assert_eq!(
        session
            .eval_text("Delete[f[a, b, c, d], {2, -1}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("TakeList[f[a, b, c, d], {1, 2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[b, c], f[d]]"
    );
}

#[test]
fn take_drop_append_and_prepend_report_current_invalid_boundaries() {
    let mut session = Session::new();

    let outcome = session.eval_text("Take[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[f[a], 2]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Take" && message.tag == "take")
    );

    let outcome = session.eval_text("Take[5, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[5, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Take"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through 1 in 5."
    }));

    let outcome = session.eval_text("Take[5, All]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[5, All]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Take"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through -1 in 5."
    }));

    let outcome = session.eval_text("Drop[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Drop[f[a], 2]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Drop" && message.tag == "drop")
    );

    let outcome = session.eval_text("Drop[5, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "Drop[5, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Drop"
            && message.tag == "drop"
            && message.text == "Cannot drop positions 1 through 1 in 5."
    }));

    let outcome = session.eval_text("TakeDrop[5, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "TakeDrop[5, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "TakeDrop"
            && message.tag == "iseq"
            && message.text == "Invalid sequence specification 1 for an expression of length 0."
    }));

    let outcome = session.eval_text("Append[5, x]").unwrap();
    assert_eq!(outcome.value.to_string(), "Append[5, x]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Append" && message.tag == "normal")
    );

    let outcome = session.eval_text("Prepend[5, x]").unwrap();
    assert_eq!(outcome.value.to_string(), "Prepend[5, x]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Prepend" && message.tag == "normal")
    );

    let outcome = session.eval_text("Insert[f[a], x, 0]").unwrap();
    assert_eq!(outcome.value.to_string(), "Insert[f[a], x, 0]");

    let outcome = session.eval_text("Delete[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Delete[f[a], 2]");

    let outcome = session.eval_text("TakeList[f[a], {2}]").unwrap();
    assert_eq!(outcome.value.to_string(), "TakeList[f[a], List[2]]");

    let outcome = session.eval_text("TakeList[5, {1}]").unwrap();
    assert_eq!(outcome.value.to_string(), "TakeList[5, List[1]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "TakeList"
            && message.tag == "normal"
            && message.text
                == "Nonatomic expression expected at position 1 in TakeList[5, List[1]]."
    }));
}
