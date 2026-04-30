use super::*;

#[test]
fn and_or_not_respect_current_short_circuit_and_identity_semantics() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("And[]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Or[]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("And[True, True]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Or[False, False]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("And[x, True]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("And[x, y, True]")
            .unwrap()
            .value
            .to_string(),
        "And[x, y]"
    );
    assert_eq!(
        session
            .eval_text("And[a, And[b, c]]")
            .unwrap()
            .value
            .to_string(),
        "And[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("And[True, And[a, b], c]")
            .unwrap()
            .value
            .to_string(),
        "And[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("And[x, False, y]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Or[x, False]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Or[x, y, False]")
            .unwrap()
            .value
            .to_string(),
        "Or[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Or[a, Or[b, c]]")
            .unwrap()
            .value
            .to_string(),
        "Or[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Or[False, Or[a, b], c]")
            .unwrap()
            .value
            .to_string(),
        "Or[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Or[x, True, y]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Not[True]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Not[False]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Not[x]").unwrap().value.to_string(),
        "Not[x]"
    );
    assert_eq!(
        session.eval_text("Not[Not[x]]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Not[Equal[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Not[Unequal[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "Equal[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Not[Less[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Not[Greater[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "LessEqual[a, b]"
    );
}

#[test]
fn and_or_short_circuit_side_effects_and_compare_exact_values() {
    let mut session = Session::new();

    session.eval_text("x = seed").unwrap();
    assert_eq!(
        session
            .eval_text("And[False, x = changed]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "seed");

    session.eval_text("y = seed").unwrap();
    assert_eq!(
        session
            .eval_text("Or[True, y = changed]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(session.eval_text("y").unwrap().value.to_string(), "seed");

    assert_eq!(
        session
            .eval_text("Equal[2, 1 + 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Equal[x, x]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Equal[a, b]").unwrap().value.to_string(),
        "Equal[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, 2, 3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Unequal[a, b]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[a, a, b]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[a, b, b]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[a, b, b]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, u, 1]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[1, u, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, 2, 2, a]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Less[1, 2, 3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Less[1, 1]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Less[\"a\", \"b\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Less[\"a\", \"b\", u]")
            .unwrap()
            .value
            .to_string(),
        "Less[\"a\", \"b\", u]"
    );
    assert_eq!(
        session
            .eval_text("GreaterEqual[2, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("LessEqual[1, 3/2, 2]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Less[a, b]").unwrap().value.to_string(),
        "Less[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Less[1, 2, u]")
            .unwrap()
            .value
            .to_string(),
        "Less[2, u]"
    );
    assert_eq!(
        session
            .eval_text("Less[u, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "Less[u, 1]"
    );
    assert_eq!(
        session
            .eval_text("Less[1, 2, u, 3, 4]")
            .unwrap()
            .value
            .to_string(),
        "Less[2, u, 3]"
    );
    assert_eq!(
        session
            .eval_text("Less[1, 2, u, 3, 2]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Less[u, 2, 1, v]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("LessEqual[1, 1, u, 2, 2]")
            .unwrap()
            .value
            .to_string(),
        "LessEqual[1, u, 2]"
    );
    assert_eq!(
        session
            .eval_text("Greater[5, 4, u, 3, 2]")
            .unwrap()
            .value
            .to_string(),
        "Greater[4, u, 3]"
    );
    assert_eq!(
        session
            .eval_text("GreaterEqual[3, 3, u]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[3, u]"
    );
    assert_eq!(
        session
            .eval_text("Equal[1, 1, u]")
            .unwrap()
            .value
            .to_string(),
        "Equal[1, 1, u]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, 2, u]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[1, 2, u]"
    );
}

#[test]
fn comparison_operator_forms_apply_single_arguments_on_the_left() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("LessThan[3]").unwrap().value.to_string(),
        "LessThan[3]"
    );
    assert_eq!(
        session.eval_text("LessThan[]").unwrap().value.to_string(),
        "LessThan[]"
    );
    assert_eq!(
        session
            .eval_text("LessThan[3, 4]")
            .unwrap()
            .value
            .to_string(),
        "LessThan[3, 4]"
    );
    assert_eq!(
        session
            .eval_text("LessThan[3][2, 4]")
            .unwrap()
            .value
            .to_string(),
        "LessThan[3][2, 4]"
    );
    assert_eq!(
        session
            .eval_text("LessThan[3][2]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("LessThan[3][4]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("LessThan[x][1]")
            .unwrap()
            .value
            .to_string(),
        "Less[1, x]"
    );
    assert_eq!(
        session
            .eval_text("GreaterThan[3][4]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("LessEqualThan[3][3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("GreaterEqualThan[3][3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("EqualTo[3][3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("UnequalTo[3][4]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SameAs[f[a]][f[a]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SameAs[f[a]][f[b]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}
