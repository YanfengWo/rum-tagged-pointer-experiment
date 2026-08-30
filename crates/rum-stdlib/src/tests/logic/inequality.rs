use super::*;

fn assert_held_full_form(session: &mut Session, source: &str, expected: &str) {
    let outcome = session
        .eval_text(&format!("FullForm[HoldComplete[{source}]]"))
        .expect("held comparison FullForm should evaluate");
    assert_eq!(
        outcome.value.to_string(),
        format!("\"HoldComplete[{expected}]\""),
        "{source}"
    );
}

#[test]
fn comparison_parentheses_bound_chain_full_forms() {
    let mut session = core_session();

    for (source, expected) in [
        ("(a == b) == c", "Equal[Equal[a, b], c]"),
        ("a == (b == c)", "Equal[a, Equal[b, c]]"),
        ("(a < b) < c", "Less[Less[a, b], c]"),
        ("a < (b < c)", "Less[a, Less[b, c]]"),
        ("(a != b) != c", "Unequal[Unequal[a, b], c]"),
        ("a === (b === c)", "SameQ[a, SameQ[b, c]]"),
        ("a == b == c", "Equal[a, b, c]"),
    ] {
        assert_held_full_form(&mut session, source, expected);
    }
}

#[test]
fn mixed_comparison_chains_have_alternating_inequality_full_forms() {
    let mut session = core_session();

    for (source, expected) in [
        ("a < b == c", "Inequality[a, Less, b, Equal, c]"),
        (
            "a < b <= c == d",
            "Inequality[a, Less, b, LessEqual, c, Equal, d]",
        ),
    ] {
        assert_held_full_form(&mut session, source, expected);
    }
}

#[test]
fn parenthesized_and_unparenthesized_equality_results_remain_distinct() {
    let mut session = core_session();

    for (source, expected) in [
        ("(1 == 1) == 1", "Equal[True, 1]"),
        ("1 == (1 == 1)", "Equal[1, True]"),
        ("1 == 1 == 1", "True"),
    ] {
        assert_eq!(
            session.eval_text(source).unwrap().value.to_string(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn mixed_comparison_chain_evaluates_middle_operand_once() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "calls = 0; g[] := (calls = calls + 1; 2); result = 1 < g[] == 2; {result, calls}",
        )
        .expect("mixed comparison count probe should evaluate");

    assert_eq!(outcome.value.to_string(), "List[True, 1]");
}

#[test]
fn mixed_comparison_chain_does_not_short_circuit_operand_evaluation() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "calls = 0; k[] := (calls = calls + 1; 0); result = 1 < 0 == k[]; {result, calls}",
        )
        .expect("mixed comparison liveness probe should evaluate");

    assert_eq!(outcome.value.to_string(), "List[False, 1]");
}

#[test]
fn inequality_binary_symbolic_and_attribute_contracts_are_explicit() {
    let mut session = core_session();

    for (source, expected) in [
        ("Inequality[a, Less, b]", "Less[a, b]"),
        (
            "Inequality[1, Less, x, Less, 2]",
            "Inequality[1, Less, x, Less, 2]",
        ),
        ("Attributes[Inequality]", "List[Protected]"),
    ] {
        assert_eq!(
            session.eval_text(source).unwrap().value.to_string(),
            expected,
            "{source}"
        );
    }
}
