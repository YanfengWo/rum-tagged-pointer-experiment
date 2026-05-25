use super::*;

#[test]
fn level_reports_wolfram_style_invalid_levelspec_messages() {
    let mut session = Session::new();

    let three_item_levelspec = session.eval_text("Level[f[a], {1, 2, 3}]").unwrap();
    assert_eq!(
        three_item_levelspec.value.to_string(),
        "Level[f[a], List[1, 2, 3]]"
    );
    assert!(
        three_item_levelspec
            .messages
            .iter()
            .any(|message| message.symbol == "Level" && message.tag == "level")
    );

    let symbolic_levelspec = session.eval_text("Level[f[a], {1, x}]").unwrap();
    assert_eq!(
        symbolic_levelspec.value.to_string(),
        "Level[f[a], List[1, x]]"
    );
    assert!(
        symbolic_levelspec
            .messages
            .iter()
            .any(|message| message.symbol == "Level" && message.tag == "level")
    );

    let with_apply_head = session.eval_text("Level[f[a], {1, 2, 3}, h]").unwrap();
    assert_eq!(
        with_apply_head.value.to_string(),
        "Level[f[a], List[1, 2, 3], h]"
    );
    assert!(
        with_apply_head
            .messages
            .iter()
            .any(|message| message.symbol == "Level" && message.tag == "level")
    );
}
