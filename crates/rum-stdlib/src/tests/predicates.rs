use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn list_relation_and_shape_predicates_cover_current_exact_subsets() {
    let mut session = core_session();

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
        session
            .eval_text("Order[f, List]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Order[a, A]").unwrap().value.to_string(),
        "1"
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
            .eval_text("OrderedQ[{f, List}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{a, A}]")
            .unwrap()
            .value
            .to_string(),
        "True"
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
            .eval_text("OrderedQ[{Power[a, b], Times[a, b], Plus[a, b], f[a], List[a]}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{List[a], f[a]}]")
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
            .eval_text("Order[List[a], f[a]]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Order[Rule[a, 1], f[a]]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Order[Hold[a], h[a]]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Order[Plus[a, b], Times[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Order[Power[a, b], Plus[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[Plus[a, b], f[a]]")
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
            .eval_text("Order[1, \"x\"]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[\"x\", x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[x, f[y]]")
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
        "False"
    );
    assert_eq!(
        session
            .eval_text("SubsetQ[{a, b, c}, {b, a}]")
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
fn collection_predicates_accept_same_test_options() {
    let mut session = core_session();
    let same_abs = "SameTest -> (SameQ[Abs[#1], Abs[#2]] &)";

    assert_eq!(
        session
            .eval_text(&format!("DuplicateFreeQ[{{1, -1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(&format!("IntersectingQ[{{1}}, {{-1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(&format!("DisjointQ[{{1}}, {{-1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(&format!("SubsetQ[{{1, 2}}, {{-1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(&format!("ContainsAll[{{1, 2}}, {{-1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(&format!("ContainsAny[{{1, 2}}, {{-1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(&format!(
                "ContainsExactly[{{1, -1}}, {{-1, 1}}, {same_abs}]"
            ))
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(&format!("ContainsNone[{{1}}, {{-1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(&format!("ContainsOnly[{{1, -1}}, {{1}}, {same_abs}]"))
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Options[DuplicateFreeQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[SameTest, Automatic]]"
    );
}

#[test]
fn ordering_predicates_accept_custom_comparators() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("OrderedQ[{3, 2, 1}, Greater]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{1, 3, 2}, Greater]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("LexicographicOrder[{3}, {2}, Greater]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("LexicographicOrder[{2}, {3}, Greater]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
}

#[test]
fn truth_aggregates_and_coprime_q_cover_current_subsets() {
    let mut session = core_session();

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
