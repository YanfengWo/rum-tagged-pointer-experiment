use super::*;

#[test]
fn nothing_resolves_as_system_symbol() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Context[Nothing]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Names[\"System`Nothing\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"System`Nothing\"]"
    );
}

#[test]
fn nothing_removal_happens_after_sequence_splicing_for_lists_only() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("{a, Sequence[Nothing, b], c}")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("f[a, Sequence[Nothing, b], c]")
            .unwrap()
            .value
            .to_string(),
        "f[a, Nothing, b, c]"
    );
}
