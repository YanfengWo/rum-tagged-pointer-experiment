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
