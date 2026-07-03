use super::*;

#[test]
fn equal_reduces_adjacent_mixed_comparable_pairs_only() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Equal[1, \"1\", u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Equal[\"1\", 1, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Equal[1, u, \"1\"]")
            .unwrap()
            .value
            .to_string(),
        "Equal[1, u, \"1\"]"
    );
    assert_eq!(
        session
            .eval_text("Equal[\"a\", u, \"b\"]")
            .unwrap()
            .value
            .to_string(),
        "Equal[\"a\", u, \"b\"]"
    );
}

#[test]
fn equal_and_unequal_handle_boolean_pairs_without_overreaching() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Equal[True, False]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, False]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Equal[True, 1]")
            .unwrap()
            .value
            .to_string(),
        "Equal[True, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, 1]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[True, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, False, True]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, x, True]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[True, x, True]"
    );
}

#[test]
fn unequal_detects_repeated_known_prefix_values_before_unknown_tail() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Unequal[1, 2, 1, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, \"1\", 1, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[u, 1, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[u, 1, 2, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, False, 1, u]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[True, False, 1, u]"
    );
}

#[test]
fn equality_operator_forms_accept_multi_argument_applications() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("EqualTo[3][]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("UnequalTo[3][]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("SameAs[3][]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("EqualTo[3][3, 3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("EqualTo[3][3, 4]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("EqualTo[x][1, 1]")
            .unwrap()
            .value
            .to_string(),
        "Equal[1, 1, x]"
    );
    assert_eq!(
        session
            .eval_text("UnequalTo[3][4, 5]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("UnequalTo[3][4, 3]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("SameAs[f[a]][f[a], f[a]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SameAs[f[a]][f[a], f[b]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn order_comparison_operator_forms_report_application_arity() {
    let mut session = core_session();

    for (input, expected, count) in [
        ("LessThan[3][]", "LessThan[3][]", 0),
        ("LessThan[3][2, 4]", "LessThan[3][2, 4]", 2),
        ("GreaterThan[3][]", "GreaterThan[3][]", 0),
        ("LessEqualThan[3][2, 3]", "LessEqualThan[3][2, 3]", 2),
        ("GreaterEqualThan[3][2, 3]", "GreaterEqualThan[3][2, 3]", 2),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected);
        let symbol = expected.split('[').next().unwrap();
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "argx"
                && message.text
                    == format!("{symbol}[3] called with {count} arguments; 1 argument is expected.")
        }));
    }
}
