use super::*;

#[test]
fn and_or_short_circuit_side_effects_and_compare_exact_values() {
    let mut session = core_session();

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

    session.eval_text("nandSide = seed").unwrap();
    assert_eq!(
        session
            .eval_text("Nand[False, nandSide = changed]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("nandSide").unwrap().value.to_string(),
        "seed"
    );

    session.eval_text("norSide = seed").unwrap();
    assert_eq!(
        session
            .eval_text("Nor[True, norSide = changed]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("norSide").unwrap().value.to_string(),
        "seed"
    );

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
            .eval_text("Equal[1, 2, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Equal[u, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "Equal[u, 1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Equal[1, u, 2]")
            .unwrap()
            .value
            .to_string(),
        "Equal[1, u, 2]"
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
        "Less[\"a\", \"b\"]"
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
            .eval_text("Less[\"b\", u, \"a\"]")
            .unwrap()
            .value
            .to_string(),
        "Less[\"b\", u, \"a\"]"
    );
    assert_eq!(
        session
            .eval_text("Greater[\"a\", u, \"b\"]")
            .unwrap()
            .value
            .to_string(),
        "Greater[\"a\", u, \"b\"]"
    );
    assert_eq!(
        session
            .eval_text("Less[\"b\", \"a\", u]")
            .unwrap()
            .value
            .to_string(),
        "Less[\"b\", \"a\", u]"
    );
    assert_eq!(
        session
            .eval_text("Less[cmpSym, cmpSym]")
            .unwrap()
            .value
            .to_string(),
        "Less[cmpSym, cmpSym]"
    );
    assert_eq!(
        session
            .eval_text("LessEqual[cmpSym, cmpSym]")
            .unwrap()
            .value
            .to_string(),
        "LessEqual[cmpSym, cmpSym]"
    );
    assert_eq!(
        session
            .eval_text("Less[1, u, 0]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("LessEqual[1, u, 0]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("LessEqual[1, u, 1]")
            .unwrap()
            .value
            .to_string(),
        "LessEqual[1, u, 1]"
    );
    assert_eq!(
        session
            .eval_text("Greater[0, u, 1]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("GreaterEqual[0, u, 0]")
            .unwrap()
            .value
            .to_string(),
        "GreaterEqual[0, u, 0]"
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
fn equal_and_unequal_leave_mixed_symbolic_chains_intact() {
    let mut session = core_session();

    for (input, expected) in [
        ("Equal[a, a, b]", "Equal[a, a, b]"),
        ("Equal[1, u, u, 1]", "Equal[1, u, u, 1]"),
        ("Equal[1, 1, u, 2]", "Equal[1, 1, u, 2]"),
        ("Unequal[a, b, a]", "Unequal[a, b, a]"),
        ("Unequal[1, u, 2, u]", "Unequal[1, u, 2, u]"),
        ("Unequal[1, 2, u, 3, 1]", "Unequal[1, 2, u, 3, 1]"),
    ] {
        assert_eq!(
            session.eval_text(input).unwrap().value.to_string(),
            expected
        );
    }
}

#[test]
fn unequal_wide_value_index_detects_late_duplicates() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Apply[Unequal, Range[20]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Apply[Unequal, Append[Range[20], 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn wide_numeric_order_chains_reduce_in_one_pass() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Apply[Less, Range[400]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Apply[Less, Append[Range[20], u]]")
            .unwrap()
            .value
            .to_string(),
        "Less[20, u]"
    );
}

#[test]
fn comparison_operator_forms_apply_single_arguments_on_the_left() {
    let mut session = core_session();

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
