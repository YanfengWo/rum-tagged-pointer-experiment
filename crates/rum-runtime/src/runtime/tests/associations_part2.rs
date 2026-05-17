use super::*;

#[test]
fn counts_explicit_key_specs_cover_nested_duplicate_and_empty_edges() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Counts[{}, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 0], Rule[b, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{}, {}]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}, {a, a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], Rule[b, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{{a, b}, a, {a, b}}, {{a, b}, a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[List[a, b], 2], Rule[a, 1], Rule[b, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}, {a -> aa, b -> bb}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[Rule[a, aa], 0], Rule[Rule[b, bb], 0]]"
    );

    let invalid = session
        .eval_text("Counts[{a, b, a}, Association[a -> 1]]")
        .unwrap();
    assert_eq!(
        invalid.value.to_string(),
        "Counts[List[a, b, a], Association[Rule[a, 1]]]"
    );
    assert!(invalid.messages.iter().any(|message| {
        message.symbol == "Counts"
            && message.tag == "invl"
            && message.text == "The argument Association[Rule[a, 1]] is not a list."
    }));
}

#[test]
fn group_by_rule_specs_transform_group_values() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2, 3, 4}, OddQ -> (# + 10 &)]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, List[11, 13]], Rule[False, List[12, 14]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[OddQ -> (# + 10 &)][{1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, List[11]], Rule[False, List[12]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2}, OddQ :> (# + 10 &)]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, List[11]], Rule[False, List[12]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2, 3, 4}, OddQ -> (# + 10 &), Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, 13], Rule[False, 14]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[Association[a -> 1, b -> 2, c -> 3], OddQ -> (# + 10 &)]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[a, 11], Rule[c, 13]]], Rule[False, Association[Rule[b, 12]]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[Association[a -> 1, b -> 2, c -> 3], OddQ -> (# + 10 &), Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, 13], Rule[False, 12]]"
    );
}

#[test]
fn group_by_rule_specs_consume_delayed_association_values_for_key_and_value() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(
                "x = 0; {GroupBy[Association[a :> (x = x + 1), b :> (x = x + 1)], Identity -> (# + 10 &)], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, Association[RuleDelayed[a, 12]]], Rule[3, Association[RuleDelayed[b, 14]]]], 4]"
    );
}

#[test]
fn group_by_nested_specs_recurse_through_lists_and_associations() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2}, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2}, {}, Last]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{}, {}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[Association[], {}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2, 3}, {OddQ}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, List[1, 3]], Rule[False, List[2]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{OddQ, Identity}][{1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[1, List[1]]]], Rule[False, Association[Rule[2, List[2]]]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{}][{1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2, 3, 4}, {OddQ, Mod[#, 3] &}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[1, List[1]], Rule[0, List[3]]]], Rule[False, Association[Rule[2, List[2]], Rule[1, List[4]]]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2, 3, 4}, {OddQ, Mod[#, 3] &}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[1, 1], Rule[0, 3]]], Rule[False, Association[Rule[2, 2], Rule[1, 4]]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{1, 2, 3, 4}, {OddQ -> (# + 10 &), Mod[#, 3] &}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[2, List[11]], Rule[1, List[13]]]], Rule[False, Association[Rule[0, List[12]], Rule[2, List[14]]]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[Association[a -> 1, b -> 2, c -> 3, d -> 4], {OddQ, Mod[#, 3] &}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[1, Association[Rule[a, 1]]], Rule[0, Association[Rule[c, 3]]]]], Rule[False, Association[Rule[2, Association[Rule[b, 2]]], Rule[1, Association[Rule[d, 4]]]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[a -> 1, b -> 2, c -> 3, d -> 4], {OddQ, Mod[#, 3] &}, Last]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[True, Association[Rule[1, 1], Rule[0, 3]]], Rule[False, Association[Rule[2, 2], Rule[1, 4]]]]"
    );
}

#[test]
fn group_by_nested_specs_rekey_delayed_association_values_per_level() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(
                "x = 0; {GroupBy[Association[a :> (x = x + 1), b :> (x = x + 1)], {Identity, Identity}], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, Association[Rule[3, Association[RuleDelayed[a, Set[x, Plus[x, 1]]]]]]], Rule[2, Association[Rule[4, Association[RuleDelayed[b, Set[x, Plus[x, 1]]]]]]]], 4]"
    );
}
