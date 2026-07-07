use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn tally_and_riffle_cover_first_occurrence_and_boundaries() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Tally[{a, b, a, 1 + 1, 2}]"),
        "List[List[a, 2], List[b, 1], List[2, 2]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Tally[{1, -1, 2}, SameQ[Abs[#1], Abs[#2]] &]"),
        "List[List[1, 2], List[2, 1]]"
    );
    assert_eq!(eval_to_string(&mut session, "Tally[{}]"), "List[]");
    assert_eq!(
        eval_to_string(&mut session, "Riffle[{a, b, c}, x]"),
        "List[a, x, b, x, c]"
    );
    assert_eq!(eval_to_string(&mut session, "Riffle[{a}, x]"), "List[a]");
    assert_eq!(
        eval_to_string(&mut session, "Riffle[{a, b, c, d}, x, 3]"),
        "List[a, b, c, x, d]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Riffle[{a, b, c, d, e}, x, {2, 4, 2}]"),
        "List[a, b, x, c, d, x, e]"
    );
}

#[test]
fn by_extrema_take_and_reverse_sort_cover_ties_and_boundaries() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "MaxBy[{aa, b, cc}, StringLength[ToString[#]] &]"
        ),
        "aa"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "MinBy[{aa, b, c}, StringLength[ToString[#]] &]"
        ),
        "b"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TakeLargestBy[{aa, b, cc, ddd}, StringLength[ToString[#]] &, 3]"
        ),
        "List[ddd, aa, cc]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TakeSmallestBy[{aa, b, cc, ddd}, StringLength[ToString[#]] &, 3]"
        ),
        "List[b, aa, cc]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ReverseSort[{3, 1, 2}]"),
        "List[3, 2, 1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ReverseSort[{3, 1, 2}, Less]"),
        "List[3, 2, 1]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ReverseSortBy[{bb, a, ccc}, StringLength[ToString[#]] &]"
        ),
        "List[ccc, bb, a]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TakeLargestBy[{aa, b}, StringLength[ToString[#]] &, 5]"
        ),
        "List[aa, b]"
    );
    assert_eq!(eval_to_string(&mut session, "ReverseSort[{}]"), "List[]");
}

#[test]
fn gather_by_groups_in_first_key_order() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "GatherBy[{aa, b, cc, d}, StringLength[ToString[#]] &]"
        ),
        "List[List[aa, cc], List[b, d]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "GatherBy[{}, StringLength[ToString[#]] &]"),
        "List[]"
    );
    assert_eq!(
        eval_to_string(&mut session, "GatherBy[{1, 2, 3, 4}, {OddQ, Mod[#, 3] &}]"),
        "List[List[List[1], List[3]], List[List[2], List[4]]]"
    );
}

#[test]
fn max_by_and_min_by_accept_top_counts() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "MaxBy[{1, -3, 2}, Abs, 2]"),
        "List[-3, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "MinBy[{1, -3, 2}, Abs, 2]"),
        "List[1, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "MaxBy[{1, -1, 2}, Abs, 2]"),
        "List[2, 1]"
    );
}

#[test]
fn collection_operator_forms_rewrite_to_data_first_calls() {
    let mut session = core_session();

    let applied = session
        .eval_text(
            "{Riffle[x][{a, b}], GatherBy[OddQ][{1, 2, 3}], MaxBy[Abs][{1, -3, 2}], \
             MinBy[Abs][{1, -3, 2}], DeleteDuplicatesBy[Abs][{-1, 1, 2}], \
             ReverseSortBy[Identity][{3, 1, 2}], TakeLargestBy[Abs, 2][{1, -3, 2}], \
             TakeSmallestBy[Abs, 1][{1, -3, 2}]}",
        )
        .unwrap();
    assert_eq!(
        applied.value.to_string(),
        "List[List[a, x, b], List[List[1, 3], List[2]], -3, 1, List[-1, 2], List[3, 2, 1], List[-3, 2], List[1]]"
    );
    assert!(applied.messages.is_empty());

    let inert = session
        .eval_text(
            "{Riffle[x], GatherBy[OddQ], MaxBy[Abs], MinBy[Abs], DeleteDuplicatesBy[Abs], \
             ReverseSortBy[Identity], TakeLargestBy[Abs, 2], TakeSmallestBy[Abs, 1]}",
        )
        .unwrap();
    assert_eq!(
        inert.value.to_string(),
        "List[Riffle[x], GatherBy[OddQ], MaxBy[Abs], MinBy[Abs], DeleteDuplicatesBy[Abs], ReverseSortBy[Identity], TakeLargestBy[Abs, 2], TakeSmallestBy[Abs, 1]]"
    );
    assert!(inert.messages.is_empty());
}

#[test]
fn collection_stats_support_normal_expression_heads_where_structure_ops_do() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "ReverseSort[f[3, 1, 2]]"),
        "f[3, 2, 1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Riffle[f[a, b], x]"),
        "f[a, x, b]"
    );
}

#[test]
fn collection_stats_report_invalid_arity_and_empty_selection() {
    let mut session = core_session();

    for (symbol, source, tag) in [
        ("Tally", "Tally[]", "argx"),
        ("MaxBy", "MaxBy[]", "argx"),
        ("MinBy", "MinBy[]", "argx"),
        ("TakeLargestBy", "TakeLargestBy[{a}]", "argx"),
        ("TakeSmallestBy", "TakeSmallestBy[{a}]", "argx"),
        ("ReverseSort", "ReverseSort[]", "argx"),
        ("ReverseSortBy", "ReverseSortBy[]", "argx"),
        ("GatherBy", "GatherBy[]", "argx"),
        ("Riffle", "Riffle[]", "argtu"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(
            outcome.value.to_string(),
            source.replace('{', "List[").replace('}', "]")
        );
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == tag),
            "{symbol} should emit {tag}"
        );
    }

    let max_empty = session.eval_text("MaxBy[{}, Identity]").unwrap();
    assert_eq!(max_empty.value.to_string(), "MaxBy[List[], Identity]");
    assert!(
        max_empty
            .messages
            .iter()
            .any(|message| message.symbol == "MaxBy" && message.tag == "empty")
    );

    let min_empty = session.eval_text("MinBy[{}, Identity]").unwrap();
    assert_eq!(min_empty.value.to_string(), "MinBy[List[], Identity]");
    assert!(
        min_empty
            .messages
            .iter()
            .any(|message| message.symbol == "MinBy" && message.tag == "empty")
    );

    let riffle_bad_position = session.eval_text("Riffle[{a, b}, x, 0]").unwrap();
    assert_eq!(
        riffle_bad_position.value.to_string(),
        "Riffle[List[a, b], x, 0]"
    );
    assert!(
        riffle_bad_position
            .messages
            .iter()
            .any(|message| message.symbol == "Riffle" && message.tag == "seqs")
    );
}

#[test]
fn tally_and_take_largest_by_support_top_counts_idiom() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "TakeLargestBy[Tally[Characters[\"mississippi\"]], Last, 3]"
        ),
        "List[List[\"i\", 4], List[\"s\", 4], List[\"p\", 2]]"
    );
}
