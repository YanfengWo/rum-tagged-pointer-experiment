use super::*;

#[test]
fn predicate_q_family_covers_current_exact_and_structural_subsets() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("TrueQ[True]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("TrueQ[If[True, True, False]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("TrueQ[If[True, False, True]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("BooleanQ[False]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("BooleanQ[x]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Boole[True]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Boole[False]").unwrap().value.to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Boole[Less[1, 2]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Boole[x]").unwrap().value.to_string(),
        "Boole[x]"
    );
    assert_eq!(
        session
            .eval_text("Boole[{True, False, x, Less[1, 2]}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0, Boole[x], 1]"
    );
    assert_eq!(
        session
            .eval_text("Boole[{{True}, {False, x}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], List[0, Boole[x]]]"
    );
    assert_eq!(
        session.eval_text("AtomQ[5]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("AtomQ[f[]]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("ListQ[{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("ListQ[f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("StringQ[\"abc\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("IntegerQ[1/2]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("IntegerQ[2 + 3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("NumberQ[1/2]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("NumberQ[x]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("NumericQ[1/2]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("NumericQ[x]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("EvenQ[4]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("EvenQ[3]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("OddQ[3]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("OddQ[1/2]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("EvenQ[{1, 2, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[False, True, False]"
    );
    assert_eq!(
        session
            .eval_text("OddQ[{{1}, {2, x}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[True], List[False, False]]"
    );
    assert_eq!(
        session
            .eval_text("Positive[3/4]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("Positive[0]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Positive[x]").unwrap().value.to_string(),
        "Positive[x]"
    );
    assert_eq!(
        session
            .eval_text("Negative[-3/4]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("NonPositive[-3/4]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("NonNegative[3/4]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Positive[{-1, 0, 1, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[False, False, True, Positive[x]]"
    );
    assert_eq!(
        session
            .eval_text("Negative[{-1, 0, 1, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[True, False, False, Negative[x]]"
    );
    assert_eq!(
        session
            .eval_text("NonPositive[{-1, 0, 1, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[True, True, False, NonPositive[x]]"
    );
    assert_eq!(
        session
            .eval_text("NonNegative[{-1, 0, 1, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[False, True, True, NonNegative[x]]"
    );
    assert_eq!(
        session
            .eval_text("PossibleZeroQ[0]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("PossibleZeroQ[1/3]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("PossibleZeroQ[{0, 1/3, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[True, False, False]"
    );
    assert_eq!(
        session
            .eval_text("MachineNumberQ[1]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("RealValuedNumberQ[1/2]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("RealValuedNumericQ[x]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("PrimeQ[7]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session.eval_text("PrimeQ[9]").unwrap().value.to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("PrimeQ[{2, 9, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[True, False, False]"
    );
    assert_eq!(
        session
            .eval_text("AlgebraicIntegerQ[3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("AlgebraicIntegerQ[1/2]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("AlgebraicIntegerQ[{3, 1/2, x}]")
            .unwrap()
            .value
            .to_string(),
        "List[True, False, False]"
    );
    assert_eq!(
        session
            .eval_text("DateObjectQ[DateObject[{2020, 1, 1}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("DateObjectQ[TimeObject[{1, 2, 3}]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("TimeObjectQ[TimeObject[{1, 2, 3}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("TimeObjectQ[DateObject[{2020, 1, 1}]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("SameQ[]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SameQ[2, 1 + 1]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SameQ[f[a], f[a], f[a]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("UnsameQ[1, 2, 3]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("UnsameQ[1, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn list_relation_and_shape_predicates_cover_current_exact_subsets() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("DuplicateFreeQ[{a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("DuplicateFreeQ[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("Order[a, b]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Order[b, a]").unwrap().value.to_string(),
        "-1"
    );
    assert_eq!(
        session.eval_text("Order[2, 10]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("NumericalOrder[10, 2]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("LexicographicOrder[{1, 2}, {1, 3}]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("LexicographicOrder[\"2\", \"10\"]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("AlphabeticOrder[\"a\", \"b\"]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{a, b, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{b, a}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{2, 10}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{10, 2}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[f[b, a]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Order[f[2], f[10]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{f[2], f[10]}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{f[10], f[2]}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Order[f[1], f[1, 0]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[{2}, {10}]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[Association[a -> 2], Association[a -> 10]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{Association[a -> 2], Association[a -> 10]}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[Association[b -> 2, a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    let atom_ordered_q = session.eval_text("OrderedQ[1]").unwrap();
    assert_eq!(atom_ordered_q.value.to_string(), "OrderedQ[1]");
    assert!(
        atom_ordered_q
            .messages
            .iter()
            .any(|message| message.symbol == "OrderedQ" && message.tag == "normal")
    );
    assert_eq!(
        session
            .eval_text("ContainsAll[{a, b, c}, {b, a}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("ContainsAny[{a, b}, {x, b}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("ContainsNone[{a, b}, {x, y}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("ContainsOnly[{a, a, b}, {a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("ContainsExactly[{a, b, a}, {b, a, a}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("SubsetQ[{a, b}, {a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("IntersectingQ[{a, b}, {x, b}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("DisjointQ[{a, b}, {x, y}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Between[3/2, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("VectorQ[{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatrixQ[{{a, b}, {c, d}}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatrixQ[{{a}, {b, c}}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("ArrayQ[{{a, b}, {c, d}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("ArrayQ[{{a}, {b, c}}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn truth_aggregates_and_coprime_q_cover_current_subsets() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("AllTrue[{1, 2, 3}, Positive]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("AllTrue[f[g[1], h[2]], Positive, {2}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("AnyTrue[{1, -2, 3}, Negative]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("NoneTrue[{1, 2, 3}, Negative]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("AllTrue[{1}, f]")
            .unwrap()
            .value
            .to_string(),
        "AllTrue[List[1], f]"
    );
    assert_eq!(
        session
            .eval_text("CoprimeQ[8, 15]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("CoprimeQ[6, 10, 15]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}
