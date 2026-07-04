use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn extract_preserves_nested_key_and_empty_selector_shapes() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Extract[Association[a -> 1], {{{Key[a]}}}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1]]]"
    );

    let missing_nested_key = session
        .eval_text("Extract[Association[a -> 1], {{{Key[b]}}}]")
        .unwrap();
    assert_eq!(
        missing_nested_key.value.to_string(),
        "List[Association[Rule[b, Missing[\"KeyAbsent\", Key[b]]]]]"
    );
    assert!(
        missing_nested_key
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "keyw")
    );

    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{{}, {1}}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[Association[a -> 1], {{}}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1]]]"
    );
}

#[test]
fn part_and_extract_head_access_preserve_symbol_diagnostics() {
    let mut session = core_session();

    let part_head_all = session.eval_text("Part[f[a, b], 0, All]").unwrap();
    assert_eq!(part_head_all.value.to_string(), "Symbol[]");
    assert!(
        part_head_all
            .messages
            .iter()
            .any(|message| message.symbol == "Symbol" && message.tag == "argx")
    );

    let part_head_zero_list = session.eval_text("Part[f[a, b], 0, {0}]").unwrap();
    assert_eq!(part_head_zero_list.value.to_string(), "Symbol[Symbol]");
    assert!(
        part_head_zero_list
            .messages
            .iter()
            .any(|message| message.symbol == "Symbol" && message.tag == "string")
    );

    let extract_head_zero_list = session.eval_text("Extract[f[a, b], {0, {0}}]").unwrap();
    assert_eq!(extract_head_zero_list.value.to_string(), "Symbol[Symbol]");
    assert!(
        extract_head_zero_list
            .messages
            .iter()
            .any(|message| message.symbol == "Symbol" && message.tag == "string")
    );
}
