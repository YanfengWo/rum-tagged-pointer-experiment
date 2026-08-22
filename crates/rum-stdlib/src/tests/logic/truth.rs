use super::*;

#[test]
fn and_or_not_respect_current_short_circuit_and_identity_semantics() {
    let mut session = core_session();

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
        session.eval_text("Nand[]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Nand[False, x]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Nand[True, x]")
            .unwrap()
            .value
            .to_string(),
        "Not[x]"
    );
    assert_eq!(
        session
            .eval_text("Nand[True, Not[x]]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Nand[True, Less[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Nand[True, x, y]")
            .unwrap()
            .value
            .to_string(),
        "Nand[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Nand[x, And[y, z]]")
            .unwrap()
            .value
            .to_string(),
        "Nand[x, And[y, z]]"
    );
    assert_eq!(
        session
            .eval_text("Nand[True, And[y, z]]")
            .unwrap()
            .value
            .to_string(),
        "Not[And[y, z]]"
    );
    assert_eq!(
        session.eval_text("Nor[]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Nor[True, x]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Nor[False, x]")
            .unwrap()
            .value
            .to_string(),
        "Not[x]"
    );
    assert_eq!(
        session
            .eval_text("Nor[False, Not[x]]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Nor[False, Less[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Nor[False, x, y]")
            .unwrap()
            .value
            .to_string(),
        "Nor[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Nor[x, Or[y, z]]")
            .unwrap()
            .value
            .to_string(),
        "Nor[x, Or[y, z]]"
    );
    assert_eq!(
        session
            .eval_text("Nor[False, Or[y, z]]")
            .unwrap()
            .value
            .to_string(),
        "Not[Or[y, z]]"
    );
    assert_eq!(
        session.eval_text("Xor[]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Xor[True]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Xor[True, False]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Xor[True, True]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Xor[x, False]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session.eval_text("Xor[x, True]").unwrap().value.to_string(),
        "Not[x]"
    );
    assert_eq!(
        session
            .eval_text("Xor[Not[x], True]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Xor[Less[a, b], True]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[a, b]"
    );
    assert_eq!(
        session.eval_text("Xor[x, x]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Xor[x, y, x]").unwrap().value.to_string(),
        "y"
    );
    assert_eq!(
        session
            .eval_text("Xor[x, y, True]")
            .unwrap()
            .value
            .to_string(),
        "Not[Xor[x, y]]"
    );
    assert_eq!(
        session.eval_text("Equivalent[]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[x]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[True, x]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[False, x]")
            .unwrap()
            .value
            .to_string(),
        "Not[x]"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[False, Not[x]]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[False, Less[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[True, False, x]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[x, x]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[x, x, y]")
            .unwrap()
            .value
            .to_string(),
        "Equivalent[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[x, Equivalent[y, z]]")
            .unwrap()
            .value
            .to_string(),
        "Equivalent[x, Equivalent[y, z]]"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[x, Equivalent[x, y]]")
            .unwrap()
            .value
            .to_string(),
        "Equivalent[x, Equivalent[x, y]]"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[True, x, y]")
            .unwrap()
            .value
            .to_string(),
        "And[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Equivalent[False, x, y]")
            .unwrap()
            .value
            .to_string(),
        "And[Not[x], Not[y]]"
    );
    assert_eq!(
        session
            .eval_text("Implies[True, x]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Implies[False, x]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Implies[x, True]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Implies[x, False]")
            .unwrap()
            .value
            .to_string(),
        "Not[x]"
    );
    assert_eq!(
        session
            .eval_text("Implies[Not[x], False]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Implies[Less[a, b], False]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Implies[x, x]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Implies[x, y]")
            .unwrap()
            .value
            .to_string(),
        "Implies[x, y]"
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
    assert_eq!(
        session
            .eval_text("Not[LessEqual[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "Greater[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Not[GreaterEqual[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "Less[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Not[Less[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "Not[Less[a, b, c]]"
    );
    assert_eq!(
        session
            .eval_text("Not[Equal[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "Not[Equal[a, b, c]]"
    );
}

#[test]
fn wide_xor_and_equivalent_indexes_preserve_parity_and_uniqueness() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "SameQ[Apply[Xor, Join[Range[20], {1, 1}]], Apply[Xor, Join[Range[2, 20], {1}]]]"
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Length[Apply[Equivalent, Join[Range[20], Range[20]]]]")
            .unwrap()
            .value
            .to_string(),
        "20"
    );
}
