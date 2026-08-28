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
fn held_updates_evaluate_rebound_part_selectors_for_all_eleven_heads() {
    for (name, setup, call, expected) in [
        (
            "AddTo",
            "x = {10, 20}",
            "AddTo[x[[$selector]], 5]",
            "List[15, List[15, 20]]",
        ),
        (
            "SubtractFrom",
            "x = {10, 20}",
            "SubtractFrom[x[[$selector]], 5]",
            "List[5, List[5, 20]]",
        ),
        (
            "TimesBy",
            "x = {10, 20}",
            "TimesBy[x[[$selector]], 2]",
            "List[20, List[20, 20]]",
        ),
        (
            "DivideBy",
            "x = {10, 20}",
            "DivideBy[x[[$selector]], 2]",
            "List[5, List[5, 20]]",
        ),
        (
            "ApplyTo",
            "x = {10, 20}",
            "ApplyTo[x[[$selector]], f]",
            "List[f[10], List[f[10], 20]]",
        ),
        (
            "AppendTo",
            "x = {{a}, {b}}",
            "AppendTo[x[[$selector]], z]",
            "List[List[a, z], List[List[a, z], List[b]]]",
        ),
        (
            "PrependTo",
            "x = {{a}, {b}}",
            "PrependTo[x[[$selector]], z]",
            "List[List[z, a], List[List[z, a], List[b]]]",
        ),
        (
            "Increment",
            "x = {10, 20}",
            "Increment[x[[$selector]]]",
            "List[10, List[11, 20]]",
        ),
        (
            "Decrement",
            "x = {10, 20}",
            "Decrement[x[[$selector]]]",
            "List[10, List[9, 20]]",
        ),
        (
            "PreIncrement",
            "x = {10, 20}",
            "PreIncrement[x[[$selector]]]",
            "List[11, List[11, 20]]",
        ),
        (
            "PreDecrement",
            "x = {10, 20}",
            "PreDecrement[x[[$selector]]]",
            "List[9, List[9, 20]]",
        ),
    ] {
        for selector in ["kk", "1"] {
            let call = call.replace("$selector", selector);
            let source = format!("kk = 2; {setup}; kk = 1; {{{call}, x}}");
            let mut session = core_session();
            let outcome = session.eval_text(&source).unwrap();

            assert_eq!(outcome.value.to_string(), expected, "{name}: {source}");
            assert!(
                outcome.messages.is_empty(),
                "{name}: {:?}",
                outcome.messages
            );
        }
    }
}

#[test]
fn held_update_part_selectors_run_left_to_right_after_binary_rhs_preparation() {
    let mut session = core_session();
    assert_eq!(
        session
            .eval_text(
                "order = {}; x = {{1, 2}, {3, 4}}; \
                 AddTo[x[[(AppendTo[order, first]; 1), (AppendTo[order, second]; 2)]], 10]; \
                 {order, x}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[first, second], List[List[1, 12], List[3, 4]]]"
    );

    for (name, setup, call, expected) in [
        (
            "AddTo",
            "x = {10}",
            "AddTo[x[[kk]], (AppendTo[order, rhs]; 5)]",
            "List[List[rhs, selector], 15, List[15]]",
        ),
        (
            "SubtractFrom",
            "x = {10}",
            "SubtractFrom[x[[kk]], (AppendTo[order, rhs]; 5)]",
            "List[List[rhs, selector], 5, List[5]]",
        ),
        (
            "TimesBy",
            "x = {10}",
            "TimesBy[x[[kk]], (AppendTo[order, rhs]; 5)]",
            "List[List[rhs, selector], 50, List[50]]",
        ),
        (
            "DivideBy",
            "x = {10}",
            "DivideBy[x[[kk]], (AppendTo[order, rhs]; 5)]",
            "List[List[rhs, selector], 2, List[2]]",
        ),
        (
            "ApplyTo",
            "x = {10}",
            "ApplyTo[x[[kk]], (AppendTo[order, rhs]; f)]",
            "List[List[rhs, selector], f[10], List[f[10]]]",
        ),
        (
            "AppendTo",
            "x = {{a}}",
            "AppendTo[x[[kk]], (AppendTo[order, rhs]; z)]",
            "List[List[rhs, selector], List[a, z], List[List[a, z]]]",
        ),
        (
            "PrependTo",
            "x = {{a}}",
            "PrependTo[x[[kk]], (AppendTo[order, rhs]; z)]",
            "List[List[rhs, selector], List[z, a], List[List[z, a]]]",
        ),
    ] {
        let source = format!(
            "order = {{}}; kk := (AppendTo[order, selector]; 1); {setup}; \
             result = {call}; {{order, result, x}}"
        );
        let mut session = core_session();
        let outcome = session.eval_text(&source).unwrap();

        assert_eq!(outcome.value.to_string(), expected, "{name}: {source}");
        assert!(
            outcome.messages.is_empty(),
            "{name}: {:?}",
            outcome.messages
        );
    }
}

#[test]
fn held_update_part_selectors_rederive_owner_state_after_callbacks() {
    let mut session = core_session();
    assert_eq!(
        session
            .eval_text(
                "x = {1, 2}; sel := (x = {10, 20}; 2); \
                 {AddTo[x[[sel]], 5], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[25, List[10, 25]]"
    );

    session
        .eval_text("x = {1, 2}; sel := (Clear[x]; 1)")
        .unwrap();
    let cleared = session.eval_text("AddTo[x[[sel]], 5]").unwrap();
    assert!(
        cleared
            .messages
            .iter()
            .any(|message| { message.symbol == "AddTo" && message.tag == "rvalue" })
    );
    assert_eq!(
        session.eval_text("ValueQ[x]").unwrap().value.to_string(),
        "False"
    );

    session
        .eval_text("x = {1, 2}; sel := (Remove[x]; 1)")
        .unwrap();
    let removed = session.eval_text("AddTo[x[[sel]], 5]").unwrap();
    assert!(
        removed
            .messages
            .iter()
            .any(|message| { message.symbol == "AddTo" && message.tag == "rvalue" })
    );
    assert_eq!(
        session.eval_text("ValueQ[x]").unwrap().value.to_string(),
        "False"
    );

    session
        .eval_text("x = {1, 2}; sel := (Protect[x]; 1)")
        .unwrap();
    let protected = session.eval_text("AddTo[x[[sel]], 5]").unwrap();
    assert!(
        protected
            .messages
            .iter()
            .any(|message| { message.symbol == "AddTo" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[x]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );

    session
        .eval_text("Unprotect[x]; x = {1, 2}; sel := (Unprotect[x]; 1); Protect[x]")
        .unwrap();
    let unprotected = session.eval_text("AddTo[x[[sel]], 5]").unwrap();
    assert_eq!(unprotected.value.to_string(), "6");
    assert!(unprotected.messages.is_empty());
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[6, 2]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[x]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn held_update_invalid_selector_diagnostics_use_the_evaluated_offender() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "calls = 0; sel := (calls = calls + 1; evaluatedBad); \
             x = {1}; AddTo[x[[sel]], 5]",
        )
        .unwrap();

    let selector_messages = outcome
        .messages
        .iter()
        .filter(|message| message.tag == "pkspec1")
        .collect::<Vec<_>>();
    assert!(!selector_messages.is_empty());
    assert!(
        selector_messages
            .iter()
            .all(|message| message.text.contains("evaluatedBad"))
    );
    assert_eq!(session.eval_text("calls").unwrap().value.to_string(), "1");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "List[1]");
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

#[test]
fn apply_to_rechecks_protection_at_store_time() {
    let mut session = core_session();

    session
        .eval_text("x = 1; f = Function[v, (Protect[x]; v + 100)]")
        .unwrap();
    let blocked = session.eval_text("ApplyTo[x, f]").unwrap();

    // message emitted by the store-time guard
    assert_eq!(blocked.messages[0].symbol, "ApplyTo");
    assert_eq!(blocked.messages[0].tag, "wrsym");
    // the write did NOT land: x is still its pre-update value
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    // and the symbol is (now) protected
    assert_eq!(
        session
            .eval_text("Attributes[x]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );
}
