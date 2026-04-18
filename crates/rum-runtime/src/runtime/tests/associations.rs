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
        "Association[Rule[a, 1], Rule[b, 2]]"
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
        "2"
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
            .eval_text(
                "Lookup[{Association[a -> Association[b -> 2]], Association[a -> Association[b -> 3]]}, {Key[a], Key[b]}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[2, 3]"
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
            .eval_text("Values[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session.eval_text("Key[a]").unwrap().value.to_string(),
        "Key[a]"
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
            .eval_text("KeyTake[Association[a -> 1, b -> 2, c -> 3], {b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
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
            .eval_text("KeyMap[f, Association[a -> 1, b -> 2]]")
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
            .eval_text("KeyMemberQ[Association[a -> 1, b -> 2], a]")
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
            .eval_text("KeySort[Association[b -> 2, a -> 1, c -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
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
            .eval_text("AssociationMap[f, {a, b}]")
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
            .eval_text("KeyUnion[Association[a -> 1, b -> 2], Association[b -> 9, c -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text(
                "KeyIntersection[Association[a -> 1, b -> 2, c -> 3], Association[b -> 9, c -> 8]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("KeyComplement[Association[a -> 1, b -> 2, c -> 3], Association[b -> 9]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociateTo[Association[a -> 1, b -> 2], b -> 9]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 9]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDropFrom[Association[a -> 1, b -> 2, c -> 3], {a, c}]")
            .unwrap()
            .value
            .to_string(),
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
            .eval_text("Catenate[{Association[a -> 1], Association[b -> 2]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
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
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &]")
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
    assert_eq!(
        session
            .eval_text("Lookup[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "Lookup[List[Rule[a, 1]], a]"
    );
    assert_eq!(
        session
            .eval_text("Keys[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Keys[List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("Values[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Values[List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "KeyTake[List[Rule[a, 1]], a]"
    );
    assert_eq!(
        session
            .eval_text("KeyMap[f, List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "KeyMap[f, List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("KeySelect[List[a -> 1], TrueQ]")
            .unwrap()
            .value
            .to_string(),
        "KeySelect[List[Rule[a, 1]], TrueQ]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "KeySort[List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationThread[{a}, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "AssociationThread[List[a], List[1, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[f, a]")
            .unwrap()
            .value
            .to_string(),
        "AssociationMap[f, a]"
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
    assert_eq!(
        session
            .eval_text("AssociateTo[Association[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "AssociateTo[Association[Rule[a, 1]], a]"
    );
    assert_eq!(
        session
            .eval_text("Merge[Association[a -> 1], Total]")
            .unwrap()
            .value
            .to_string(),
        "Merge[Association[Rule[a, 1]], Total]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Catenate[Association[Rule[a, 1]]]"
    );
    assert_eq!(
        session.eval_text("Counts[a]").unwrap().value.to_string(),
        "Counts[a]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[a, f]")
            .unwrap()
            .value
            .to_string(),
        "GroupBy[a, f]"
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

    let outcome = session.eval_text("KeyMemberQ[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMemberQ[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMemberQ" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyFreeQ[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyFreeQ[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyFreeQ" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeySelect[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySelect[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySelect" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeySortBy[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySortBy[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySortBy" && message.tag == "argx")
    );

    let outcome = session.eval_text("AssociationThread[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationThread[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationThread" && message.tag == "argx")
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
            .any(|message| message.symbol == "KeyUnion" && message.tag == "argx")
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
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "argx")
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
            .any(|message| message.symbol == "KeyComplement" && message.tag == "argx")
    );

    let outcome = session.eval_text("Missing[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Missing[]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Missing" && message.tag == "argx")
    );
}
