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
            .eval_text("Dot[{1, 2, 3}, {4, 5, 6}]")
            .unwrap()
            .value
            .to_string(),
        "32"
    );
    assert_eq!(
        session
            .eval_text("{{1, 2}, {3, 4}} . {5, 6}")
            .unwrap()
            .value
            .to_string(),
        "List[17, 39]"
    );
    assert_eq!(
        session
            .eval_text("Dot[{1, 2}, {{1, 2}, {3, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[7, 10]"
    );
    assert_eq!(
        session
            .eval_text("Dot[{{1, 2}, {3, 4}}, {{1, 0}, {0, 1}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], List[3, 4]]"
    );
    assert_eq!(
        session
            .eval_text("Dot[{a, b}, {c, d}]")
            .unwrap()
            .value
            .to_string(),
        "Plus[Times[a, c], Times[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Dot[{1/2, 2}, {2, 1/2}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(session.eval_text("Dot[x]").unwrap().value.to_string(), "x");
    assert_eq!(
        session.eval_text("Dot[]").unwrap().value.to_string(),
        "Dot[]"
    );
    assert_eq!(
        session
            .eval_text("Dot[{{1, 2}, {3, 4}}, {{1, 0}, {0, 1}}, {5, 6}]")
            .unwrap()
            .value
            .to_string(),
        "List[17, 39]"
    );
    assert_eq!(
        session.eval_text("Dot[x, y]").unwrap().value.to_string(),
        "Dot[x, y]"
    );
    assert_eq!(
        session
            .eval_text("Dot[{1, 2}, {3, 4, 5}]")
            .unwrap()
            .value
            .to_string(),
        "Dot[List[1, 2], List[3, 4, 5]]"
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
fn array_and_outer_cover_current_dense_generation_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("List[Context[Array], Context[Outer]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"System`\", \"System`\"]"
    );
    assert_eq!(
        session.eval_text("Array[f, 3]").unwrap().value.to_string(),
        "List[f[1], f[2], f[3]]"
    );
    assert_eq!(
        session
            .eval_text("Array[f, {2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[1, 1], f[1, 2], f[1, 3]], List[f[2, 1], f[2, 2], f[2, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("Array[f, {2}, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[f[0], f[1]]"
    );
    assert_eq!(
        session
            .eval_text("Array[f, {2}, {10}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[10], f[11]]"
    );
    assert_eq!(
        session
            .eval_text("Array[f, {2}, 1, g]")
            .unwrap()
            .value
            .to_string(),
        "g[f[1], f[2]]"
    );
    assert_eq!(
        session
            .eval_text("Array[f, {2}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[Plus[1, a]]]"
    );
    assert_eq!(
        session.eval_text("Array[f, {}]").unwrap().value.to_string(),
        "f[]"
    );

    assert_eq!(
        session
            .eval_text("Outer[f, {a, b}, {x, y}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a, x], f[a, y]], List[f[b, x], f[b, y]]]"
    );
    assert_eq!(
        session
            .eval_text("Outer[f, {a, b}, {{x, y}, {u, v}}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a, List[x, y]], f[a, List[u, v]]], List[f[b, List[x, y]], f[b, List[u, v]]]]"
    );
    assert_eq!(
        session
            .eval_text("Outer[f, g[a, b], g[x, y]]")
            .unwrap()
            .value
            .to_string(),
        "g[g[f[a, x], f[a, y]], g[f[b, x], f[b, y]]]"
    );
}

#[test]
fn array_and_outer_respect_resource_limits() {
    let mut array_session = Session::new();
    array_session.set_generated_element_limit(3);
    let array = array_session.eval_text("Array[f, 5]").unwrap();
    assert_eq!(
        array.value.to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );
    assert!(
        array
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut outer_session = Session::new();
    outer_session.set_iteration_work_limit(3);
    let outer = outer_session.eval_text("Outer[f, {a, b}, {x, y}]").unwrap();
    assert_eq!(
        outer.value.to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );
    assert!(
        outer
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut memory_session = Session::new();
    assert_eq!(
        memory_session
            .eval_text("MemoryConstrained[Array[f, {100}], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
}

#[test]
fn array_functional_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let inner = session.eval_text("Inner[f, a]").unwrap();
    assert_eq!(inner.value.to_string(), "Inner[f, a]");
    assert!(
        inner
            .messages
            .iter()
            .any(|message| message.symbol == "Inner" && message.tag == "argtu")
    );

    let array_filter = session.eval_text("ArrayFilter[f, {a, b}, bad]").unwrap();
    assert_eq!(
        array_filter.value.to_string(),
        "ArrayFilter[f, List[a, b], bad]"
    );
    assert!(
        array_filter
            .messages
            .iter()
            .any(|message| message.symbol == "ArrayFilter" && message.tag == "argform")
    );

    let array_reduce = session.eval_text("ArrayReduce[f, {a, b}, bad]").unwrap();
    assert_eq!(
        array_reduce.value.to_string(),
        "ArrayReduce[f, List[a, b], bad]"
    );
    assert!(
        array_reduce
            .messages
            .iter()
            .any(|message| message.symbol == "ArrayReduce" && message.tag == "argform")
    );

    let array_reduce_grouped = session
        .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, {{1}, {1}}]")
        .unwrap();
    assert_eq!(
        array_reduce_grouped.value.to_string(),
        "ArrayReduce[f, List[List[a, b], List[c, d]], List[List[1], List[1]]]"
    );
    assert!(
        array_reduce_grouped
            .messages
            .iter()
            .any(|message| message.symbol == "ArrayReduce" && message.tag == "argform")
    );

    let array = session.eval_text("Array[f, bad]").unwrap();
    assert_eq!(array.value.to_string(), "Array[f, bad]");
    assert!(
        array
            .messages
            .iter()
            .any(|message| message.symbol == "Array" && message.tag == "argform")
    );

    let outer = session.eval_text("Outer[f, {a}, g[b]]").unwrap();
    assert_eq!(outer.value.to_string(), "Outer[f, List[a], g[b]]");
    assert!(
        outer
            .messages
            .iter()
            .any(|message| message.symbol == "Outer" && message.tag == "heads")
    );
}
