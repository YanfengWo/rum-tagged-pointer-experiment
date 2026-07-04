use super::*;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn held_update_builtins_mutate_immediate_symbol_values() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("l = {}; {AppendTo[l, a], l}")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[a]]"
    );
    assert_eq!(
        session
            .eval_text("l = {b}; {PrependTo[l, a], l}")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("x = 1; {AddTo[x, 2], x}")
            .unwrap()
            .value
            .to_string(),
        "List[3, 3]"
    );
    assert_eq!(
        session
            .eval_text("x = 10; {SubtractFrom[x, 2], TimesBy[x, 3], DivideBy[x, 4], x}")
            .unwrap()
            .value
            .to_string(),
        "List[8, 24, 6, 6]"
    );
    assert_eq!(
        session
            .eval_text("x = 2; {ApplyTo[x, f], x}")
            .unwrap()
            .value
            .to_string(),
        "List[f[2], f[2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 2; {x //= Function[Plus[Slot[1], 3]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[5, 5]"
    );
    assert_eq!(
        session
            .eval_text("x = 1; {Increment[x], x}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("x = 1; {PreIncrement[x], x, Decrement[x], x, PreDecrement[x], x}")
            .unwrap()
            .value
            .to_string(),
        "List[2, 2, 2, 1, 0, 0]"
    );
}

#[test]
fn held_update_syntax_mutates_module_locals() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "Module[{l = {}, i = 0}, AppendTo[l, a]; ++i; i++; i += 2; i -= 1; i *= 3; i /= 3; i--; --i; {l, i}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[a], 1]"
    );
}

#[test]
fn held_updates_mutate_single_exact_part_targets() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("x = {1, 2}; {AddTo[x[[1]], 3], x}")
            .unwrap()
            .value
            .to_string(),
        "List[4, List[4, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = {10, 4}; {SubtractFrom[x[[1]], 3], TimesBy[x[[2]], 5], DivideBy[x[[1]], 7], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[7, 20, 1, List[1, 20]]"
    );
    assert_eq!(
        session
            .eval_text("x = {10, 20}; {x[[2]]++, x, ++x[[1]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[20, List[10, 21], 11, List[11, 21]]"
    );
    assert_eq!(
        session
            .eval_text("x = {2, 3}; {x[[1]]--, --x[[2]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[2, 2, List[1, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = {{a}, {b}}; {AppendTo[x[[1]], z], PrependTo[x[[2]], q], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, z], List[q, b], List[List[a, z], List[q, b]]]"
    );
    assert_eq!(
        session
            .eval_text("x = {2}; {ApplyTo[x[[1]], f], x}")
            .unwrap()
            .value
            .to_string(),
        "List[f[2], List[f[2]]]"
    );
    assert_eq!(
        session
            .eval_text("x = {{a}}; {ApplyTo[x[[1]], List], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[a]], List[List[List[a]]]]"
    );
    assert_eq!(
        session
            .eval_text("Module[{x = {1, 2}}, x[[1]] += 3; x]")
            .unwrap()
            .value
            .to_string(),
        "List[4, 2]"
    );
    assert_eq!(
        session
            .eval_text("x = {1, 2}; {AddTo[x[[1]], (x = {9, 9}; 3)], x}")
            .unwrap()
            .value
            .to_string(),
        "List[12, List[12, 9]]"
    );
}

#[test]
fn held_updates_mutate_bulk_exact_part_targets() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("x = {1, 2}; {AddTo[x[[All]], 1], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[2, 3], List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("x = {1, 2}; {x[[All]]++, x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("x = {{1, 2}, {3, 4}}; {AddTo[x[[All, 1]], 10], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[11, 13], List[List[11, 2], List[13, 4]]]"
    );
    assert_eq!(
        session
            .eval_text("x = {a, b, c}; {AppendTo[x[[{1, 3}]], z], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, c, z], List[List[a, c, z], b, List[a, c, z]]]"
    );
    assert_eq!(
        session
            .eval_text("x = {1, 2}; {ApplyTo[x[[All]], f], x}")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[1, 2]], List[f[List[1, 2]], f[List[1, 2]]]]"
    );
}

#[test]
fn held_updates_allow_empty_bulk_part_targets_without_mutation() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("x = {1, 2}; {AddTo[x[[{}]], 1], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[], List[1, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = {1, 2}; {Increment[x[[{}]]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[], List[1, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = {1, 2}; {AppendTo[x[[{}]], a], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[1, 2]]"
    );
}

#[test]
fn held_updates_cover_duplicate_bulk_part_targets() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("x = {1, 2}; {AddTo[x[[{1, 1}]], {10, 20}], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[11, 21], List[21, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = {{1, 2}, {3, 4}}; {AddTo[x[[All, {1, 1}]], 10], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[11, 11], List[13, 13]], List[List[11, 2], List[13, 4]]]"
    );
    assert_eq!(
        session
            .eval_text("x = {{1, 2}, {3, 4}}; {AddTo[x[[{1, 1}, All]], 10], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[11, 12], List[11, 12]], List[List[11, 12], List[3, 4]]]"
    );
}

#[test]
fn held_updates_reject_nested_part_selector_lists_without_mutation() {
    let mut session = core_session();

    let outcome = session
        .eval_text("x = {{1, 2}, {3, 4}}; AddTo[x[[{{1}, {1, 2}}]], 10]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AddTo[Part[x, List[List[1], List[1, 2]]], 10]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "pkspec1")
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "pkspec1")
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[1, 2], List[3, 4]]"
    );
}

#[test]
fn held_updates_report_missing_or_invalid_targets_without_writing() {
    let mut session = core_session();

    let missing = session.eval_text("AddTo[x, 2]").unwrap();
    assert_eq!(missing.value.to_string(), "AddTo[x, 2]");
    assert_eq!(missing.messages[0].symbol, "AddTo");
    assert_eq!(missing.messages[0].tag, "rvalue");

    let missing_pre = session.eval_text("PreIncrement[y]").unwrap();
    assert_eq!(missing_pre.value.to_string(), "PreIncrement[y]");
    assert_eq!(missing_pre.messages[0].symbol, "PreIncrement");
    assert_eq!(missing_pre.messages[0].tag, "rvalue");

    let missing_apply = session.eval_text("ApplyTo[y, f]").unwrap();
    assert_eq!(missing_apply.value.to_string(), "ApplyTo[y, f]");
    assert_eq!(missing_apply.messages[0].symbol, "ApplyTo");
    assert_eq!(missing_apply.messages[0].tag, "rvalue");

    let invalid_append = session.eval_text("l = 1; AppendTo[l, a]").unwrap();
    assert_eq!(invalid_append.value.to_string(), "AppendTo[l, a]");
    assert_eq!(invalid_append.messages[0].symbol, "AppendTo");
    assert_eq!(invalid_append.messages[0].tag, "normal");
    assert_eq!(session.eval_text("l").unwrap().value.to_string(), "1");

    let invalid_prepend = session.eval_text("l = 1; PrependTo[l, a]").unwrap();
    assert_eq!(invalid_prepend.value.to_string(), "PrependTo[l, a]");
    assert_eq!(invalid_prepend.messages[0].symbol, "PrependTo");
    assert_eq!(invalid_prepend.messages[0].tag, "normal");
    assert_eq!(session.eval_text("l").unwrap().value.to_string(), "1");

    let invalid_target = session.eval_text("SubtractFrom[f[1], 2]").unwrap();
    assert_eq!(invalid_target.value.to_string(), "SubtractFrom[f[1], 2]");
    assert_eq!(invalid_target.messages[0].symbol, "SubtractFrom");
    assert_eq!(invalid_target.messages[0].tag, "rvalue");
    assert!(invalid_target.messages[0].text.contains("f[1]"));

    let invalid_literal_target = session.eval_text("AppendTo[1, a]").unwrap();
    assert_eq!(invalid_literal_target.value.to_string(), "AppendTo[1, a]");
    assert_eq!(invalid_literal_target.messages[0].symbol, "AppendTo");
    assert_eq!(invalid_literal_target.messages[0].tag, "rvalue");
    assert!(invalid_literal_target.messages[0].text.contains("1"));
}

#[test]
fn held_updates_respect_protected_symbols() {
    let mut session = core_session();

    session.eval_text("x = 1; Protect[x]").unwrap();
    let blocked = session.eval_text("AddTo[x, 2]").unwrap();

    assert_eq!(blocked.value.to_string(), "AddTo[x, 2]");
    assert_eq!(blocked.messages[0].symbol, "AddTo");
    assert_eq!(blocked.messages[0].tag, "wrsym");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}
