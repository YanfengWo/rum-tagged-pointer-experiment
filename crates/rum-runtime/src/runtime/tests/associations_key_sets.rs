use super::*;

#[test]
fn association_thread_and_key_set_ops_cover_first_pass_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("AssociationThread[{a, b}, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[{a, b}, u]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, u], Rule[b, u]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[u, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[u, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[{}, u]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[u, {}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[u, List]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[a -> 1]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {AssociationThread[a :> (x = x + 1)], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[{a, b} :> z]")
            .unwrap()
            .value
            .to_string(),
        "Association[RuleDelayed[a, z], RuleDelayed[b, z]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[z :> {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[RuleDelayed[z, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {AssociationThread[{a, b} :> {x = x + 1, x = x + 1}], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]], RuleDelayed[b, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[a, {1}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[{a}, b]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("KeyUnion[{Association[a -> 1, b -> 2], Association[b -> 9, c -> 3]}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2], Rule[c, Missing[\"KeyAbsent\", c]]], Association[Rule[a, Missing[\"KeyAbsent\", a]], Rule[b, 9], Rule[c, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyUnion[{Association[a -> 1], Association[a -> 2, b -> 3]}, f]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, f[b]]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeyUnion[{Association[a :> (x = x + 1)], Association[]}], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, Missing[\"KeyAbsent\", a]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeyUnion[{Association[a :> (x = x + 1)], Association[]}, f], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, f[a]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeyIntersection[{Association[a -> 1, b -> 2, c -> 3], Association[b -> 9, c -> 8]}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2], Rule[c, 3]], Association[Rule[b, 9], Rule[c, 8]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {KeyIntersection[{Association[a :> (x = x + 1), b -> 2], Association[a -> 9]}], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[Rule[a, 9]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeyComplement[{Association[a -> 1, b -> 2, c -> 3], Association[b -> 9]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {KeyComplement[{Association[a :> (x = x + 1), b -> 2], Association[b -> 9]}], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeyUnion[{{a -> 1, b -> 2}, {b -> 9, c -> 3}}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2], Rule[c, Missing[\"KeyAbsent\", c]]], Association[Rule[a, Missing[\"KeyAbsent\", a]], Rule[b, 9], Rule[c, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyIntersection[{{a -> 1, b -> 2}, {b -> 9, c -> 3}}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2]], Association[Rule[b, 9]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyComplement[{{a -> 1, b -> 2}, {b -> 9}}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyUnion[{a -> 1, {b -> 2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, Missing[\"KeyAbsent\", b]]], Association[Rule[a, Missing[\"KeyAbsent\", a]], Rule[b, 2]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyComplement[{a -> 1, {b -> 2}}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 1], Rule[b, 2]]"
    );
}
