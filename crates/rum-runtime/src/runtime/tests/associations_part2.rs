use super::*;

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
