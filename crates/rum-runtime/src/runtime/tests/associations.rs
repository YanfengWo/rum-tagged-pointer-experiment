use super::*;

#[test]
fn association_and_associationq_cover_first_pass_rule_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Association[a -> 1, b :> 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], RuleDelayed[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1, b -> 2, a -> 3]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Association[{a -> 1, b -> 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Association[{a -> 1, {b -> 2}, Association[c -> 3]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("<|a -> 1, b -> 2, a -> 3|>")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationQ[Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1, b -> 2, a -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 3], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationQ[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn lookup_keys_values_and_key_cover_first_pass_association_queries() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1, b -> 2], b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; assoc = Association[k :> (x = x + 1)]; {assoc[k], Lookup[assoc, k], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 2]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], b, 99]")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], b]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> Association[b -> 2]], {Key[a], Key[b]}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2]], Missing[\"KeyAbsent\", b]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], Key[b], 99]")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], Key[b]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1, b -> 2], {b, a, c}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, Missing[\"KeyAbsent\", c]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], {a, b}, 99]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 99]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], b, 99, Hold]")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], Association[a -> 2, b -> 3]}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], Association[b -> 2]}, a, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], Association[b -> 2]}, a, 0, Hold]")
            .unwrap()
            .value
            .to_string(),
        "List[Hold[1], 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, a]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Lookup[a -> 1, c]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", c]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, Key[b]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, Key[c]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", c]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, {b, a, c}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, Missing[\"KeyAbsent\", c]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{{a -> 1}, {b -> 2}}, a, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], {b -> 2}}, a, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[{Association[a -> Association[b -> 2]], Association[a -> Association[b -> 3]]}, {Key[a], Key[b]}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[Association[Rule[b, 2]], Missing[\"KeyAbsent\", b]], List[Association[Rule[b, 3]], Missing[\"KeyAbsent\", b]]]"
    );
    assert_eq!(
        session
            .eval_text("Keys[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Keys[List[a -> 1, b -> 2], Hold]")
            .unwrap()
            .value
            .to_string(),
        "List[Hold[a], Hold[b]]"
    );
    assert_eq!(
        session
            .eval_text("Keys[{Association[a -> 1], {b -> 2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b]]"
    );
    assert_eq!(
        session
            .eval_text("Values[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Values[{Association[a -> 1], {b -> 2}}, Hold]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Hold[1]], List[Hold[2]]]"
    );
    assert_eq!(
        session.eval_text("Key[a]").unwrap().value.to_string(),
        "Key[a]"
    );
    assert_eq!(
        session
            .eval_text("Key[a][Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Key[b][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Key[2][Association[2 -> \"two\", 1 -> \"one\"]]")
            .unwrap()
            .value
            .to_string(),
        "\"two\""
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1, b -> 2][a]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1][b]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1][]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
}

#[test]
fn key_exists_take_drop_map_and_keyvaluemap_cover_first_pass_subset() {
    let mut session = Session::new();

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
                "KeySortBy[Association[a -> 1, bb -> 2, ccc -> 3], StringLength[ToString[#]] &]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[bb, 2], Rule[ccc, 3]]"
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
            .eval_text("AssociationThread[{a, b}, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
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
            .eval_text("KeyComplement[{Association[a -> 1, b -> 2, c -> 3], Association[b -> 9]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
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
    let literal_associate = session
        .eval_text("AssociateTo[Association[a -> 1, b -> 2], b -> 9]")
        .unwrap();
    assert_eq!(
        literal_associate.value.to_string(),
        "AssociateTo[Association[Rule[a, 1], Rule[b, 2]], Rule[b, 9]]"
    );
    assert!(
        literal_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let literal_drop = session
        .eval_text("KeyDropFrom[Association[a -> 1, b -> 2, c -> 3], {a, c}]")
        .unwrap();
    assert_eq!(literal_drop.value.to_string(), "Association[Rule[b, 2]]");
    assert!(
        literal_drop
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "write")
    );
    session.eval_text("assoc = Association[a -> 1]").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[assoc, b -> 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    session.eval_text("assoc = Association[a -> 1]").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[assoc, {Association[b -> 2], c -> 3}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    session
        .eval_text("assoc = Association[a -> 1, b -> 2, c -> 3]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[assoc, {a, c}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[b, 2]]"
    );
    session
        .eval_text("assoc = Association[a -> 1, Key[a] -> 7, b -> 2]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[assoc, {Key[a], b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[Key[a], 7]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[Key[a], 7]]"
    );
    session.eval_text("parts = {Association[a -> 1]}").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[1]], b -> 2]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2]]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1], Association[a -> 2]}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[{1, 2}]], b -> 3]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 3]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 3]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1], 1}")
        .unwrap();
    let invalid_part_associate = session
        .eval_text("AssociateTo[parts[[{1, 2}]], b -> 3]")
        .unwrap();
    assert!(
        invalid_part_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invak")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]], 1]"
    );
    session
        .eval_text("nested = Association[a -> Association[b -> 1]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[nested[[Key[a]]], c -> 3]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Association[Rule[b, 1], Rule[c, 3]]]]"
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        "Association[Rule[a, Association[Rule[b, 1], Rule[c, 3]]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1, b -> 2]}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[parts[[1]], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[b, 2]]]"
    );
    session.eval_text("parts = {Association[a -> 1]}").unwrap();
    session.eval_text("Protect[parts]").unwrap();
    let blocked_part_associate = session
        .eval_text("AssociateTo[parts[[1]], b -> 2]")
        .unwrap();
    assert!(
        blocked_part_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]]]"
    );
    session.eval_text("Unprotect[parts]").unwrap();
    session
        .eval_text("nested = Association[a -> Association[b -> 1, c -> 2]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[nested[[Key[a]]], b]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[c, 2]]"
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        "Association[Rule[a, Association[Rule[c, 2]]]]"
    );
    session.eval_text("ClearAll[assoc]").unwrap();
    let missing_value = session.eval_text("AssociateTo[assoc, b -> 2]").unwrap();
    assert_eq!(
        missing_value.value.to_string(),
        "AssociateTo[assoc, Rule[b, 2]]"
    );
    assert!(
        missing_value
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "blnoval")
    );
    session.eval_text("assoc := Association[a -> 1]").unwrap();
    let delayed_value = session.eval_text("AssociateTo[assoc, b -> 2]").unwrap();
    assert_eq!(
        delayed_value.value.to_string(),
        "AssociateTo[assoc, Rule[b, 2]]"
    );
    assert!(
        delayed_value
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "blnoval")
    );
    session.eval_text("assoc = 1").unwrap();
    let invalid_value = session.eval_text("AssociateTo[assoc, b -> 2]").unwrap();
    assert_eq!(
        invalid_value.value.to_string(),
        "AssociateTo[assoc, Rule[b, 2]]"
    );
    assert!(
        invalid_value
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invak")
    );
    let invalid_spec = session.eval_text("AssociateTo[assoc, a]").unwrap();
    assert_eq!(invalid_spec.value.to_string(), "AssociateTo[assoc, a]");
    assert!(
        invalid_spec
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invlb")
    );
    session.eval_text("ClearAll[assoc]").unwrap();
    let missing_drop_target = session.eval_text("KeyDropFrom[assoc, a]").unwrap();
    assert_eq!(
        missing_drop_target.value.to_string(),
        "KeyDropFrom[assoc, a]"
    );
    assert!(
        missing_drop_target
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "blnoval")
    );
    session.eval_text("assoc = {a -> 1, b -> 2}").unwrap();
    let invalid_drop_target = session.eval_text("KeyDropFrom[assoc, a]").unwrap();
    assert_eq!(
        invalid_drop_target.value.to_string(),
        "KeyDropFrom[assoc, a]"
    );
    assert!(
        invalid_drop_target
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    session.eval_text("assoc = Association[b -> 2]").unwrap();
    session.eval_text("Protect[assoc]").unwrap();
    let blocked = session.eval_text("AssociateTo[assoc, c -> 9]").unwrap();
    assert_eq!(blocked.value.to_string(), "AssociateTo[assoc, Rule[c, 9]]");
    assert!(
        blocked
            .messages
            .iter()
            .any(|message| { message.symbol == "AssociateTo" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyValuePattern[a -> _]")
            .unwrap()
            .value
            .to_string(),
        "KeyValuePattern[List[Rule[a, Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1, b -> 2], KeyValuePattern[a -> _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1, b -> 2], KeyValuePattern[{b -> 2, a -> _}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1], KeyValuePattern[b -> _]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Merge[{Association[a -> 1, b -> 2], Association[a -> 3]}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Total[List[1, 3]]], Rule[b, Total[List[2]]]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{Association[a -> 1], {a -> 2, b -> 3}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Total[List[1, 2]]], Rule[b, Total[List[3]]]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{a -> 1, {a -> 2, b -> 3}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Total[List[1, 2]]], Rule[b, Total[List[3]]]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[Total][{Association[a -> 1, b -> 2], Association[a -> 3]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Total[List[1, 3]]], Rule[b, Total[List[2]]]]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[{Association[a -> 1], Association[b -> 2]}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> {1, 2}, b -> {3}]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[{Association[a -> {1, 2}], {3, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], Rule[b, 1]]"
    );
    assert_eq!(
        session
            .eval_text("CountsBy[{a, bb, ccc}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, 1], Rule[2, 1], Rule[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("CountsBy[StringLength[ToString[#]] &][{a, bb, ccc}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, 1], Rule[2, 1], Rule[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, List[a]], Rule[2, List[bb, dd]], Rule[3, List[ccc]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, a], Rule[2, dd], Rule[3, ccc]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[StringLength[ToString[#]] &][{a, bb, ccc, dd}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, List[a]], Rule[2, List[bb, dd]], Rule[3, List[ccc]]]"
    );
    assert_eq!(
        session
            .eval_text("PositionIndex[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, List[1, 3]], Rule[b, List[2]]]"
    );
    assert_eq!(
        session
            .eval_text("Missing[\"KeyAbsent\", a]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", a]"
    );
}

#[test]
fn association_family_keeps_invalid_forms_unevaluated_or_reports_arity() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Association[a]")
            .unwrap()
            .value
            .to_string(),
        "Association[a]"
    );
    let outcome = session.eval_text("Lookup[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "Lookup[a, b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Lookup" && message.tag == "invrl")
    );
    let outcome = session.eval_text("Lookup[{a -> 1, {b -> 2}}, a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Lookup[List[Rule[a, 1], List[Rule[b, 2]]], a]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Lookup" && message.tag == "invrl")
    );
    assert_eq!(
        session
            .eval_text("Keys[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
    assert_eq!(
        session
            .eval_text("Values[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[List[a -> 1, b -> 2], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    let outcome = session.eval_text("KeyMap[f, List[a -> 1]]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMap[f, List[Rule[a, 1]]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMap" && message.tag == "invak")
    );
    assert_eq!(
        session
            .eval_text("KeySelect[List[a -> 1], TrueQ]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    let outcome = session.eval_text("AssociationThread[{a}, {1, 2}]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociationThread[List[a], List[1, 2]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationThread" && message.tag == "idim")
    );
    let outcome = session.eval_text("AssociationMap[f, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationMap[f, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationMap" && message.tag == "invrp")
    );
    assert_eq!(
        session
            .eval_text("KeyUnion[Association[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "KeyUnion[Association[Rule[a, 1]], a]"
    );
    assert_eq!(
        session
            .eval_text("Normal[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Normal[List[Rule[a, 1]]]"
    );
    let outcome = session
        .eval_text("AssociateTo[Association[a -> 1], a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociateTo[Association[Rule[a, 1]], a]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invlb")
    );
    let outcome = session
        .eval_text("Merge[Association[a -> 1], Total]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Merge[Association[Rule[a, 1]], Total]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Merge" && message.tag == "list1")
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[List[a -> 1], b]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    let outcome = session
        .eval_text("Catenate[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Catenate[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Catenate" && message.tag == "invrp")
    );
    assert_eq!(
        session.eval_text("Counts[a]").unwrap().value.to_string(),
        "Counts[a]"
    );
    let outcome = session.eval_text("GroupBy[a, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "GroupBy[a, f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "GroupBy" && message.tag == "list1")
    );
    assert_eq!(
        session
            .eval_text("PositionIndex[a]")
            .unwrap()
            .value
            .to_string(),
        "PositionIndex[a]"
    );

    let outcome = session.eval_text("Key[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Key[]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Key" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyExistsQ[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyExistsQ[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyExistsQ" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyExistsQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "False");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyExistsQ" && message.tag == "invrl")
    );

    let outcome = session.eval_text("KeyMemberQ[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMemberQ[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMemberQ" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyMemberQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMemberQ[a, b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMemberQ" && message.tag == "invas")
    );

    let outcome = session.eval_text("KeyFreeQ[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyFreeQ[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyFreeQ" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyFreeQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyFreeQ[a, b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyFreeQ" && message.tag == "invas")
    );

    let outcome = session.eval_text("KeyMap[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyMap[f, rum$keyedArg]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyMap[f, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMap[f, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMap" && message.tag == "invak")
    );

    let outcome = session.eval_text("KeyValueMap[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyValueMap[f, rum$keyedArg]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyValueMap[f, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyValueMap[f, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyValueMap" && message.tag == "invak")
    );

    let outcome = session.eval_text("KeySelect[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeySelect[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeySelect[a, TrueQ]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySelect[a, TrueQ]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySelect" && message.tag == "invru")
    );

    let outcome = session.eval_text("AssociationMap[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, AssociationMap[f, rum$keyedArg]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("Merge[Total]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, Merge[rum$keyedArg, Total]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("CountsBy[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, CountsBy[rum$keyedArg, f]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("GroupBy[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, GroupBy[rum$keyedArg, f]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("AssociationMap[]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationMap[]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationMap" && message.tag == "argt")
    );

    let outcome = session.eval_text("AssociationMap[f, {a}, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationMap[f, List[a], b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationMap" && message.tag == "argt")
    );

    let outcome = session.eval_text("KeySortBy[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeySortBy[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeySort[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySort[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySort" && message.tag == "invrl")
    );

    let outcome = session.eval_text("KeySortBy[a, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySortBy[a, f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySortBy" && message.tag == "invrl")
    );

    let outcome = session.eval_text("AssociationThread[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationThread[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationThread" && message.tag == "invrk")
    );

    let outcome = session.eval_text("KeyUnion[Association[a -> 1]]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyUnion[Association[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyUnion" && message.tag == "invar")
    );

    let outcome = session
        .eval_text("KeyIntersection[Association[a -> 1]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyIntersection[Association[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "invar")
    );

    let outcome = session
        .eval_text("KeyComplement[Association[a -> 1]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[Association[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "invar")
    );

    let outcome = session.eval_text("KeyUnion[{}]").unwrap();
    assert_eq!(outcome.value.to_string(), "List[]");
    assert!(outcome.messages.is_empty());

    let outcome = session
        .eval_text("KeyUnion[{Association[a -> 1]}, f, g]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyUnion[List[Association[Rule[a, 1]]], f, g]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyUnion" && message.tag == "argt")
    );

    let outcome = session.eval_text("KeyIntersection[{}]").unwrap();
    assert_eq!(outcome.value.to_string(), "List[]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyIntersection[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyIntersection[]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyComplement[{}]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyComplement[List[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "empt")
    );

    let outcome = session
        .eval_text("KeyComplement[{Association[a -> 1]}, b]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[List[Association[Rule[a, 1]]], b]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "nonopt")
    );

    let outcome = session
        .eval_text("KeyComplement[{Association[a -> 1]}, x -> y]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[List[Association[Rule[a, 1]]], Rule[x, y]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "optx")
    );

    let outcome = session
        .eval_text("KeyUnion[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyUnion[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyUnion" && message.tag == "invas")
    );

    let outcome = session
        .eval_text("KeyIntersection[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyIntersection[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "invas")
    );

    let outcome = session
        .eval_text("KeyComplement[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "invrl")
    );

    let outcome = session.eval_text("Missing[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Missing[]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("Missing[a, b, c]").unwrap();
    assert_eq!(outcome.value.to_string(), "Missing[a, b, c]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("CountsBy[a, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "CountsBy[a, f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "CountsBy" && message.tag == "list")
    );

    let outcome = session.eval_text("CountsBy[{a}, f, g]").unwrap();
    assert_eq!(outcome.value.to_string(), "CountsBy[List[a], f, g]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "CountsBy" && message.tag == "argt")
    );

    let outcome = session.eval_text("Merge[{a}, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "Merge[List[a], f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Merge" && message.tag == "list1")
    );

    let outcome = session
        .eval_text("Merge[{Association[a -> 1]}, f, g]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Merge[List[Association[Rule[a, 1]]], f, g]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Merge" && message.tag == "argt")
    );

    let outcome = session.eval_text("GroupBy[{a}, f, g, h]").unwrap();
    assert_eq!(outcome.value.to_string(), "GroupBy[List[a], f, g, h]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "GroupBy" && message.tag == "argb")
    );
}
