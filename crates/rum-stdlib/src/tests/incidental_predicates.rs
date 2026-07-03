use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn predicate_q_family_covers_current_exact_and_structural_subsets() {
    let mut session = core_session();

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
