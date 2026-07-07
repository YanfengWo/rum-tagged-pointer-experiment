use super::super::*;

// Moved from crates/rum-runtime/src/runtime/tests/associations_key_ops.rs.

#[test]
fn key_predicates_take_drop_map_and_sort_cover_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyExistsQ[Association[a -> 1, b -> 2], b]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[Association[a -> 1], b]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[a -> 1, a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[Association[a -> 1, b -> 2, c -> 3], {b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2], Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[Association[a -> 1, b -> 2, c -> 3], b]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {KeyTake[Association[a :> (x = x + 1)], a], KeyDrop[Association[a :> (x = x + 1), b -> 2], b], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[Association[a -> 1, Key[a] -> 7], Key[a]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[Association[a -> 1, Key[a] -> 7, b -> 2], {Key[a], b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[Key[a], 7]]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[a -> 1, a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[a -> 1, a]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[{Association[a -> 1, b -> 2], {a -> 3, c -> 4}}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1]], Association[Rule[a, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[{Association[a -> 1, b -> 2], {a -> 3, c -> 4}}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2]], Association[Rule[c, 4]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[{a -> 1, b -> 2}, a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMap[f, Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[f[a], 1], Rule[f[b], 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyMap[f][Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[f[a], 1], Rule[f[b], 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeyMap[f, Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[f[a], Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeyValueMap[f, Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, 1], f[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyValueMap[f][Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, 1], f[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[Association[a -> 1, b -> 2], a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[{a -> 1, b -> 2}, a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[Association[a -> 1, b -> 2], z]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[{a -> 1, b -> 2}, z]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySelect[Association[a -> 1, b -> 2, c -> 3], MatchQ[#, Alternatives[a, c]] &]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySelect[MatchQ[#, Alternatives[a, c]] &][Association[a -> 1, b -> 2, c -> 3]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySelect[{a -> 1, a -> 2, b -> 3}, MatchQ[#, a] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeySelect[Association[a :> (x = x + 1)], True &], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[Association[b -> 2, a -> 1, c -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[List[b -> 2, a -> 1, c -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySort[Association[bb -> 2, a -> 1, ccc -> 3], Function[Less[StringLength[ToString[Slot[1]]], StringLength[ToString[Slot[2]]]]]]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySort[List[bb -> 2, a -> 1], Function[Less[StringLength[ToString[Slot[1]]], StringLength[ToString[Slot[2]]]]]]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {KeySort[Association[a :> (x = x + 1)]], KeySort[Association[a :> (x = x + 1)], OrderedQ[{#1, #2}] &], x}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySortBy[Association[a -> 1, bb -> 2, ccc -> 3], StringLength[ToString[#]] &]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeySortBy[Association[bb -> 2, a -> 1, ccc -> 3], StringLength[ToString[#]] &, Greater]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[ccc, 3], Rule[bb, 2], Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeySortBy[List[bb -> 2, a -> 1, ccc -> 3], StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeySortBy[StringLength[ToString[#]] &][Association[bb -> 2, a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {KeySortBy[Association[a :> (x = x + 1)], Identity], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[f, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[a]], Rule[b, f[b]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[f][{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[a]], Rule[b, f[b]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[Identity, Association[a -> 1, b :> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], RuleDelayed[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[(h[#] -> #) &, Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[h[Rule[a, 1]], Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {AssociationMap[Identity, Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    let invalid_association_map = session
        .eval_text("AssociationMap[f, Association[a -> 1, b -> 2]]")
        .unwrap();
    assert_eq!(
        invalid_association_map.value.to_string(),
        "Association[f[Rule[a, 1]], f[Rule[b, 2]]]"
    );
    assert_eq!(
        invalid_association_map
            .messages
            .iter()
            .filter(|message| message.symbol == "AssociationMap" && message.tag == "invrlf")
            .count(),
        2
    );
}
