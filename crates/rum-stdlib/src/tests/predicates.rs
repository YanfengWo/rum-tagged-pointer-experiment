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
    assert_eq!(
        session
            .eval_text("{VectorQ[{1, 2}, IntegerQ], VectorQ[{1, a}, IntegerQ], MatrixQ[{{1, 2}, {3, 4}}, IntegerQ], MatrixQ[{{1, 2}, {3, a}}, IntegerQ], ArrayQ[{{1, 2}, {3, 4}}, _], ArrayQ[{{1, 2}, {3, 4}}, 1 | 2], ArrayQ[{{1, 2}, {3, 4}}, 3], ArrayQ[{{1, 2}, {3, 4}}, 2, IntegerQ], ArrayQ[{{1, 2}, {3, a}}, 2, IntegerQ]}")
            .unwrap()
            .value
            .to_string(),
        "List[True, False, True, False, True, True, False, True, False]"
    );
    assert_eq!(
        session
            .eval_text("VectorQ[{1}, Function[1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("VectorQ[{{1}}, Function[InputString[\"should not ask\"]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(
                "calls = 0; {VectorQ[Range[25], Function[x, calls = calls + 1; IntegerQ[x]]], calls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[True, 25]"
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
fn collection_same_test_pairs_run_once_each_without_native_stack_recursion() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "duplicateCalls = 0; relationCalls = 0; {DuplicateFreeQ[Range[100], SameTest -> Function[(duplicateCalls = duplicateCalls + 1; False)]], duplicateCalls, ContainsNone[Range[100], Range[100], SameTest -> Function[(relationCalls = relationCalls + 1; False)]], relationCalls}",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[True, 4950, True, 10000]");
}

#[test]
fn predicate_operator_forms_are_inert_and_apply_to_data() {
    let mut session = core_session();

    let inert = session
        .eval_text(
            "{ContainsAll[{a}], ContainsAny[{z}], ContainsExactly[{a, b}], \
             ContainsNone[{z}], ContainsOnly[{a, b}], Between[{1, 3}], \
             AllTrue[EvenQ], AnyTrue[OddQ], NoneTrue[OddQ], MatchQ[_Integer]}",
        )
        .unwrap();
    assert!(inert.messages.is_empty());
    assert_eq!(
        inert.value.to_string(),
        "List[ContainsAll[List[a]], ContainsAny[List[z]], ContainsExactly[List[a, b]], ContainsNone[List[z]], ContainsOnly[List[a, b]], Between[List[1, 3]], AllTrue[EvenQ], AnyTrue[OddQ], NoneTrue[OddQ], MatchQ[Blank[Integer]]]"
    );

    let applied = session
        .eval_text(
            "{ContainsAll[{a}][{a, b}], ContainsAny[{z}][{a, b}], \
             ContainsExactly[{a, b}][{b, a}], ContainsNone[{z}][{a, b}], \
             ContainsOnly[{a, b}][{a}], Between[{1, 3}][2], \
             AllTrue[EvenQ][{2, 4}], AnyTrue[OddQ][{2, 3}], \
             NoneTrue[OddQ][{2, 4}], MatchQ[_Integer][3]}",
        )
        .unwrap();
    assert!(applied.messages.is_empty());
    assert_eq!(
        applied.value.to_string(),
        "List[True, False, True, True, True, True, True, True, True, True]"
    );

    let excluded_data_forms = session
        .eval_text("{OrderedQ[{1, 2, 3}], Lookup[<|\"a\" -> 1|>, \"a\"]}")
        .unwrap();
    assert!(excluded_data_forms.messages.is_empty());
    assert_eq!(excluded_data_forms.value.to_string(), "List[True, 1]");
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
    assert_eq!(
        session
            .eval_text(
                "calls = 0; {OrderedQ[Range[25], Function[{a, b}, calls = calls + 1; Less[a, b]]], calls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[True, 24]"
    );
    assert_eq!(
        session
            .eval_text(
                "calls = 0; {LexicographicOrder[Range[25], Range[25], Function[{a, b}, calls = calls + 1; Less[a, b]]], calls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[0, 50]"
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
            .eval_text(
                "calls = 0; {AllTrue[Range[25], Function[x, calls = calls + 1; Positive[x]]], calls}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[True, 25]"
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
