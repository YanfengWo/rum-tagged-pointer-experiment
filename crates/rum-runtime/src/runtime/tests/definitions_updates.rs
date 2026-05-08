use super::*;

#[test]
fn held_update_builtins_mutate_immediate_symbol_values() {
    let mut session = Session::new();

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
            .eval_text("x = 1; {AddTo[x, 2], x}")
            .unwrap()
            .value
            .to_string(),
        "List[3, 3]"
    );
    assert_eq!(
        session
            .eval_text("x = 1; {Increment[x], x}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
}

#[test]
fn held_update_syntax_mutates_module_locals() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Module[{l = {}, i = 0}, AppendTo[l, a]; i++; i += 2; {l, i}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], 3]"
    );
}

#[test]
fn held_updates_report_missing_or_invalid_targets_without_writing() {
    let mut session = Session::new();

    let missing = session.eval_text("AddTo[x, 2]").unwrap();
    assert_eq!(missing.value.to_string(), "AddTo[x, 2]");
    assert_eq!(missing.messages[0].symbol, "AddTo");
    assert_eq!(missing.messages[0].tag, "rvalue");

    let invalid_append = session.eval_text("l = 1; AppendTo[l, a]").unwrap();
    assert_eq!(invalid_append.value.to_string(), "AppendTo[l, a]");
    assert_eq!(invalid_append.messages[0].symbol, "AppendTo");
    assert_eq!(invalid_append.messages[0].tag, "normal");
    assert_eq!(session.eval_text("l").unwrap().value.to_string(), "1");

    let invalid_target = session.eval_text("AddTo[f[1], 2]").unwrap();
    assert_eq!(invalid_target.value.to_string(), "AddTo[f[1], 2]");
    assert_eq!(invalid_target.messages[0].symbol, "AddTo");
    assert_eq!(invalid_target.messages[0].tag, "argform");
}

#[test]
fn held_updates_respect_protected_symbols() {
    let mut session = Session::new();

    session.eval_text("x = 1; Protect[x]").unwrap();
    let blocked = session.eval_text("AddTo[x, 2]").unwrap();

    assert_eq!(blocked.value.to_string(), "AddTo[x, 2]");
    assert_eq!(blocked.messages[0].symbol, "AddTo");
    assert_eq!(blocked.messages[0].tag, "wrsym");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}
