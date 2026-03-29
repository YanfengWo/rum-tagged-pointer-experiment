use super::*;

#[test]
fn inner_arrayfilter_and_arrayreduce_cover_current_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Inner[Times, {1, 2}, {3, 4}]")
            .unwrap()
            .value
            .to_string(),
        "11"
    );
    assert_eq!(
        session
            .eval_text("Inner[Times, {{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}, Plus]")
            .unwrap()
            .value
            .to_string(),
        "List[List[19, 22], List[43, 50]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, a, b]], f[List[a, b, c]], f[List[b, c, c]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {{a, b, c}, {d, e, f}, {g, h, i}}, {1, 1}, Padding -> None]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[List[List[a, b, c], List[d, e, f], List[g, h, i]]]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, {1, 0, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, Missing[], b]], f[List[a, Missing[], c]], f[List[b, Missing[], c]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 1, Padding -> \"Periodic\"]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[c, a, b]], f[List[a, b, c]], f[List[b, c, a]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 1, Padding -> {{left, right}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[left, a, b]], f[List[a, b, c]], f[List[b, c, right]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 2, Padding -> {left, right}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[right, left, a, b, c]], f[List[left, a, b, c, left]], f[List[a, b, c, left, right]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c, d}, {1, 0, 1}, Padding -> None]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, Missing[], c]], f[List[b, Missing[], d]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "ArrayFilter[f, {{a, b, c}, {d, e, f}, {g, h, i}}, {1, 1}, Padding -> {{None, None}, {left, right}}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[f[List[List[left, a, b], List[left, d, e], List[left, g, h]]], f[List[List[a, b, c], List[d, e, f], List[g, h, i]]], f[List[List[b, c, right], List[e, f, right], List[h, i, right]]]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[Identity, {a, b, c, d}, {1, 1, 1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, a, a, b], List[a, a, b, c], List[a, b, c, d], List[b, c, d, d]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, c]], f[List[b, d]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{{a, b}, {c, d}}, {{e, f}, {g, h}}}, 1 ;; 2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, c, e, g]], f[List[b, d, f, h]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {{1, 2}, {3}}]")
            .unwrap()
            .value
            .to_string(),
        "f[List[List[a, b], List[c, d], List[e, f], List[g, h]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a], f[b]], List[f[c], f[d]]]"
    );
}

#[test]
fn array_functional_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let inner = session.eval_text("Inner[f, a]").unwrap();
    assert_eq!(inner.value.to_string(), "Inner[f, a]");
    assert!(inner
        .messages
        .iter()
        .any(|message| message.symbol == "Inner" && message.tag == "argtu"));

    let array_filter = session.eval_text("ArrayFilter[f, {a, b}, bad]").unwrap();
    assert_eq!(
        array_filter.value.to_string(),
        "ArrayFilter[f, List[a, b], bad]"
    );
    assert!(array_filter
        .messages
        .iter()
        .any(|message| message.symbol == "ArrayFilter" && message.tag == "argform"));

    let array_reduce = session.eval_text("ArrayReduce[f, {a, b}, bad]").unwrap();
    assert_eq!(
        array_reduce.value.to_string(),
        "ArrayReduce[f, List[a, b], bad]"
    );
    assert!(array_reduce
        .messages
        .iter()
        .any(|message| message.symbol == "ArrayReduce" && message.tag == "argform"));

    let array_reduce_grouped = session
        .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, {{1}, {1}}]")
        .unwrap();
    assert_eq!(
        array_reduce_grouped.value.to_string(),
        "ArrayReduce[f, List[List[a, b], List[c, d]], List[List[1], List[1]]]"
    );
    assert!(array_reduce_grouped
        .messages
        .iter()
        .any(|message| message.symbol == "ArrayReduce" && message.tag == "argform"));
}
