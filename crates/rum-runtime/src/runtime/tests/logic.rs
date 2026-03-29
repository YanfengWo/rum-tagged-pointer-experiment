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
}
