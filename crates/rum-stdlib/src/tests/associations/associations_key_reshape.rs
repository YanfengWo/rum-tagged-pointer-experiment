use super::*;

#[test]
fn key_group_by_groups_association_entries_by_transformed_keys() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyGroupBy[<|\"aa\" -> 1, \"b\" -> 2|>, StringLength]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[2, Association[Rule[\"aa\", 1]]], Rule[1, Association[Rule[\"b\", 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyGroupBy[<|\"aa\" -> 1, \"bb\" -> 2|>, StringLength, HoldComplete]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[2, HoldComplete[Association[Rule[\"aa\", 1], Rule[\"bb\", 2]]]]]"
    );
}

#[test]
fn association_key_flatten_and_deflatten_transform_path_keys() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("AssociationKeyDeflatten[<|{\"a\", \"b\"} -> 1, {\"a\", \"c\"} -> 2|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"a\", Association[Rule[\"b\", 1], Rule[\"c\", 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationKeyFlatten[<|\"a\" -> <|\"b\" -> 1|>|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[List[\"a\", \"b\"], 1]]"
    );
}

#[test]
fn association_key_flatten_and_deflatten_round_trip_well_formed_inputs() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("AssociationKeyDeflatten[AssociationKeyFlatten[<|\"a\" -> <|\"b\" -> 1, \"c\" -> 2|>|>]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"a\", Association[Rule[\"b\", 1], Rule[\"c\", 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationKeyFlatten[AssociationKeyDeflatten[<|{\"a\", \"b\"} -> 1, {\"a\", \"c\"} -> 2|>]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[List[\"a\", \"b\"], 1], Rule[List[\"a\", \"c\"], 2]]"
    );
}

#[test]
fn merge_nested_recursively_merges_associations_and_combines_leaf_values() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("MergeNested[{<|\"db\" -> <|\"host\" -> \"a\"|>|>, <|\"db\" -> <|\"port\" -> 5432|>|>}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"db\", Association[Rule[\"host\", \"a\"], Rule[\"port\", 5432]]]]"
    );
    assert_eq!(
        session
            .eval_text("MergeNested[{<|\"a\" -> 1|>, <|\"a\" -> 2|>}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"a\", 2]]"
    );
}

#[test]
fn nested_association_additions_support_operator_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyGroupBy[f]")
            .unwrap()
            .value
            .to_string(),
        "Function[rum$keyedArg, KeyGroupBy[rum$keyedArg, f]]"
    );
    assert_eq!(
        session
            .eval_text("MergeNested[Last]")
            .unwrap()
            .value
            .to_string(),
        "Function[rum$keyedArg, MergeNested[rum$keyedArg, Last]]"
    );

    let outcome = session
        .eval_text(
            "{KeyGroupBy[StringLength][<|\"aa\" -> 1|>], MergeNested[Last][{<|\"a\" -> 1|>, <|\"a\" -> 2|>}]}",
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[2, Association[Rule[\"aa\", 1]]]], Association[Rule[\"a\", 2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn nested_association_additions_leave_malformed_inputs_inert() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "{KeyGroupBy[x, f], AssociationKeyDeflatten[x], AssociationKeyFlatten[x], MergeNested[y, Last]}",
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[KeyGroupBy[x, f], AssociationKeyDeflatten[x], AssociationKeyFlatten[x], MergeNested[y, Last]]"
    );
    assert!(outcome.messages.is_empty());
}
