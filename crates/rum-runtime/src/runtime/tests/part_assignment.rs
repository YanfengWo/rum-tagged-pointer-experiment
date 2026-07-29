use super::*;
use crate::runtime::test_builtins::register_test_arithmetic_builtins;

fn assert_no_false_part_protection(messages: &[RuntimeMessage]) {
    assert!(
        messages
            .iter()
            .all(|message| !message.text.contains("Part is Protected")),
        "unexpected protected-Part diagnostic: {messages:?}"
    );
}

fn test_key_builtin(_session: &mut Session, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
    Ok(Expr::call(head.clone(), args.to_vec()))
}

#[test]
fn set_positional_part_supports_flat_nested_and_value_semantics() {
    let mut session = session_with_test_control_builtins();

    let flat = session
        .eval_text("ee = {1, 2, 3}; {ee[[2]] = 99, ee}")
        .unwrap();
    assert_eq!(flat.value.to_string(), "List[99, List[1, 99, 3]]");
    assert!(flat.messages.is_empty());

    let nested = session
        .eval_text("ff = {{1, 2}, {3, 4}}; {ff[[1, 2]] = 9, ff}")
        .unwrap();
    assert_eq!(
        nested.value.to_string(),
        "List[9, List[List[1, 9], List[3, 4]]]"
    );
    assert!(nested.messages.is_empty());

    let aliased = session
        .eval_text("x = {1, 2, 3}; y = x; y[[1]] = 9; {x, y}")
        .unwrap();
    assert_eq!(
        aliased.value.to_string(),
        "List[List[1, 2, 3], List[9, 2, 3]]"
    );
    assert!(aliased.messages.is_empty());
}

#[test]
fn set_positional_part_rederives_after_rhs_and_orders_selectors_left_to_right() {
    let mut session = session_with_test_control_builtins();

    let reassigned = session
        .eval_text("g = {1, 2, 3}; {g[[2]] = (g = {7, 8}; 5), g}")
        .unwrap();
    assert_eq!(reassigned.value.to_string(), "List[5, List[7, 5]]");
    assert!(reassigned.messages.is_empty());

    let ordered = session
        .eval_text(
            "m = {{1, 2}, {3, 4}}; order = {}; \
             {m[[(AppendTo[order, first]; 1), (AppendTo[order, second]; 2)]] = \
                (AppendTo[order, rhs]; 9), order, m}",
        )
        .unwrap();
    assert_eq!(
        ordered.value.to_string(),
        "List[9, List[rhs, first, second], List[List[1, 9], List[3, 4]]]"
    );
    assert!(ordered.messages.is_empty());

    let selector_reassigned = session
        .eval_text("s = {1, 2, 3}; {s[[(s = {7, 8}; 2)]] = 5, s}")
        .unwrap();
    assert_eq!(selector_reassigned.value.to_string(), "List[5, List[7, 5]]");
    assert!(selector_reassigned.messages.is_empty());

    let invalidated_path = session
        .eval_text("short = {1, 2, 3}; {short[[(short = {7}; 2)]] = 5, short}")
        .unwrap();
    assert_eq!(invalidated_path.value.to_string(), "List[5, List[7]]");
    assert!(invalidated_path.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "partw"
            && message.text == "Part 2 of List[7] does not exist."
    }));
    assert_no_false_part_protection(&invalidated_path.messages);
}

#[test]
fn set_positional_part_refuses_a_target_cleared_by_a_selector() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("c = {1, 2, 3}; {c[[(Clear[c]; 2)]] = 5, c}")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[5, c]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "argform"
            && message.text.contains("current List value")
    }));
    assert_no_false_part_protection(&outcome.messages);
}

#[test]
fn set_positional_part_rechecks_protection_after_rhs_and_selector_callbacks() {
    let mut session = session_with_test_control_builtins();

    let rhs_protected = session
        .eval_text("r = {1, 2, 3}; {r[[2]] = (Protect[r]; 5), r}")
        .unwrap();
    assert_eq!(rhs_protected.value.to_string(), "List[5, List[1, 2, 3]]");
    assert!(rhs_protected.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "wrsym"
            && message.text == "Symbol r is Protected."
    }));
    assert_no_false_part_protection(&rhs_protected.messages);
    session.eval_text("Unprotect[r]").unwrap();

    let selector_protected = session
        .eval_text("p = {1, 2, 3}; {p[[(Protect[p]; 2)]] = 5, p}")
        .unwrap();
    assert_eq!(
        selector_protected.value.to_string(),
        "List[5, List[1, 2, 3]]"
    );
    assert!(selector_protected.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "wrsym"
            && message.text == "Symbol p is Protected."
    }));
    assert_no_false_part_protection(&selector_protected.messages);
}

#[test]
fn set_positional_part_selector_resume_is_single_pass_and_rederives_the_owner() {
    const SOURCE: &str = "g = {1, 2, 3}; seen = {}; \
        g[[(AppendTo[seen, before]; InputString[\"index: \"]; \
             AppendTo[seen, after]; g = {7, 8}; 2)]] = 9; \
        {g, seen}";

    let mut resumed_session = session_with_test_control_builtins();
    let failure = resumed_session
        .eval_request(SOURCE)
        .expect_err("selector InputString should suspend the request");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "index: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let resumed = resumed_session
        .resume_eval_request_with_input("ready")
        .expect("selector resume should finish the original Part assignment");
    assert_eq!(
        resumed.value.to_string(),
        "List[List[7, 9], List[before, after]]"
    );
    assert!(resumed.messages.is_empty());

    let mut queued_session = session_with_test_control_builtins();
    queued_session.queue_input_response("ready");
    let queued = queued_session
        .eval_request(SOURCE)
        .expect("queued selector input should finish without suspension");

    assert_eq!(queued.value, resumed.value);
    assert_eq!(queued.messages, resumed.messages);
}

#[test]
fn set_positional_part_invalid_targets_report_without_protected_part_fallback() {
    for (source, expected_symbol, expected_tag, expected_value) in [
        (
            "a = {1, 2}; {a[[0]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {1, 2}; {a[[-1]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {1, 2}; {a[[3]] = 9, a}",
            "Part",
            "partw",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {1, 2}; {a[[bad]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {f[1]}; {a[[1, 1]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[f[1]]]",
        ),
        (
            "{({1, 2})[[1]] = 9, {1, 2}}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
    ] {
        let mut session = session_with_test_control_builtins();
        let outcome = session.eval_text(source).unwrap();

        assert_eq!(outcome.value.to_string(), expected_value, "{source}");
        assert!(
            outcome.messages.iter().any(|message| {
                message.symbol == expected_symbol && message.tag == expected_tag
            }),
            "missing {expected_symbol}::{expected_tag} for {source}: {:?}",
            outcome.messages
        );
        assert_no_false_part_protection(&outcome.messages);
    }
}

#[test]
fn set_positional_part_route_preserves_other_lvalues_and_definition_lhs_forms() {
    let mut session = session_with_test_control_builtins();
    register_test_arithmetic_builtins(&mut session);
    session.register_builtin(BuiltinSpec::ordinary(
        "Key",
        "Test-local association key wrapper.",
        test_key_builtin,
    ));

    let associations = session
        .eval_text(
            "aa = <|1 -> 2|>; bb = <|1 -> 2|>; \
             aa[[Key[1]]] = 99; bb[1] = 99; {aa, bb}",
        )
        .unwrap();
    assert_eq!(
        associations.value.to_string(),
        "List[Association[Rule[1, 99]], Association[Rule[1, 99]]]"
    );
    assert!(associations.messages.is_empty());

    let compound = session
        .eval_text(
            "dd = {1, 2, 3}; cc = {{1}, {2}}; \
             dd[[2]] += 10; AppendTo[cc[[1]], 9]; {dd, cc}",
        )
        .unwrap();
    assert_eq!(
        compound.value.to_string(),
        "List[List[1, 12, 3], List[List[1, 9], List[2]]]"
    );
    assert!(compound.messages.is_empty());

    let ordinary = session
        .eval_text(
            "{left, right} = {1, 2}; \
             f[x_] := x; f[1] = 2; {{left, right}, f[1], f[3]}",
        )
        .unwrap();
    assert_eq!(ordinary.value.to_string(), "List[List[1, 2], 2, 3]");
    assert!(ordinary.messages.is_empty());
}
