use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn group_by_nested_specs_recurse_through_lists_and_associations() {
    let mut session = core_session();

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
fn group_by_wide_list_nested_and_reducer_phases_preserve_order_and_call_counts() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                r#"n = 200;
                   keyCalls = 0;
                   reducerCalls = 0;
                   reduced = GroupBy[
                       Range[n],
                       Function[x, keyCalls = keyCalls + 1; x],
                       Function[{group}, reducerCalls = reducerCalls + 1; group]
                   ];
                   outerCalls = 0;
                   innerCalls = 0;
                   nested = GroupBy[
                       Range[n],
                       {Function[x, outerCalls = outerCalls + 1; x],
                        Function[x, innerCalls = innerCalls + 1; x]}
                   ];
                   SameQ[
                       {Keys[reduced], First[Lookup[reduced, 1]], First[Lookup[reduced, n]],
                        keyCalls, reducerCalls, Keys[nested],
                        First[Lookup[Lookup[nested, 1], 1]],
                        First[Lookup[Lookup[nested, n], n]], outerCalls, innerCalls},
                       {Range[n], 1, n, n, n, Range[n], 1, n, n, n}
                   ]"#,
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn group_by_rule_specs_transform_group_values() {
    let mut session = core_session();

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
