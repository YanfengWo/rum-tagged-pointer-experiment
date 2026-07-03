use super::*;

#[test]
fn table_supports_count_iterators_and_nested_iterators() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Table[x, 3]").unwrap().value.to_string(),
        "List[x, x, x]"
    );
    assert_eq!(
        session
            .eval_text("Table[Null, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[Null, Null, Null]"
    );
    assert_eq!(
        session
            .eval_text("Table[i^2, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 4, 9]"
    );
    assert_eq!(
        session
            .eval_text("Table[i, {i, {2, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[i + 1, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[1 + i, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[{i, i, 0}, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 1, 0], List[2, 2, 0], List[3, 3, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Table[i*j, {i, 2}, {j, i}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], List[2, 4]]"
    );
    assert_eq!(
        session
            .eval_text("Length[Table[i, {i, 1200}]]")
            .unwrap()
            .value
            .to_string(),
        "1200"
    );
    assert_eq!(
        session
            .eval_text("Flatten[Table[j, {i, 2}, {j, 2}], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 1, 2]"
    );
    assert_eq!(
        session
            .eval_text("argLevels = {0, 1}; fvVars = {a, b}; Table[argLevels[[i]] -> fvVars[[i]], {i, 1, Length[argLevels]}]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[0, a], Rule[1, b]]"
    );
}

#[test]
fn sum_and_product_reuse_concrete_iterator_specs() {
    let mut session = Session::new();

    let cases = [
        ("Sum[i, {i, 1, 10}]", "55"),
        ("Product[i, {i, 1, 5}]", "120"),
        ("Sum[i, {i, 0, 10, 2}]", "30"),
        ("Product[k, {k, 2, 6, 2}]", "48"),
        ("Sum[i j, {i, 1, 2}, {j, 1, 3}]", "18"),
        ("Sum[i, {i, {2, 3, 5}}]", "10"),
        ("Sum[i, {i, 5, 1}]", "0"),
        ("Product[i, {i, 5, 1}]", "1"),
        ("Sum[x, {3}]", "Times[3, x]"),
        ("Product[x, {3}]", "Power[x, 3]"),
        ("i = 100; Sum[i, {i, 1, 3}]", "6"),
        ("i", "100"),
    ];

    for (source, expected) in cases {
        assert_eq!(
            session.eval_text(source).unwrap().value.to_string(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn sum_and_product_leave_symbolic_bounds_unevaluated() {
    let mut session = Session::new();

    let cases = [
        ("Sum[i, {i, 1, n}]", "Sum[i, List[i, 1, n]]"),
        ("Product[i, {i, 1, n}]", "Product[i, List[i, 1, n]]"),
        ("Sum[x, 3]", "Sum[x, 3]"),
        ("Product[x, 3]", "Product[x, 3]"),
    ];

    for (source, expected) in cases {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(outcome.messages.is_empty(), "{source}");
    }
}

#[test]
fn sum_and_product_report_wolfram_kernel_attributes() {
    let mut session = Session::new();

    for name in ["Sum", "Product"] {
        let source = format!("Attributes[{name}]");
        assert_eq!(
            session.eval_text(&source).unwrap().value.to_string(),
            "List[HoldAll, Protected, ReadProtected]",
            "{source}"
        );
    }
}

#[test]
fn iterator_variables_expose_dynamic_ownvalues_without_leaking_assignments() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Table[OwnValues[i], {i, 1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[RuleDelayed[HoldPattern[i], 1]]]"
    );
    assert_eq!(
        session
            .eval_text("Table[i = 10; i, {i, 1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[10]"
    );
    assert_eq!(session.eval_text("i").unwrap().value.to_string(), "i");
}

#[test]
fn iterator_dynamic_values_interact_with_nested_block_shadowing() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Table[Block[{i}, i], {i, 1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    assert_eq!(
        session
            .eval_text("Table[Block[{i = 7}, i], {i, 1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[7]"
    );
}

#[test]
fn do_and_table_respect_resource_limits() {
    let mut table_session = Session::new();
    table_session.set_generated_element_limit(3);
    let table = table_session.eval_text("Table[x, 5]").unwrap();
    assert_eq!(
        table.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(
        table
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut do_session = Session::new();
    do_session.set_iteration_work_limit(3);
    let do_result = do_session.eval_text("Do[Null, 5]").unwrap();
    assert_eq!(
        do_result.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(
        do_result
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut iterator_session = Session::new();
    iterator_session.set_generated_element_limit(3);
    assert_eq!(
        iterator_session
            .eval_text("Table[i, {i, 5}]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );

    let mut memory_session = Session::new();
    assert_eq!(
        memory_session
            .eval_text("MemoryConstrained[Table[x, 100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
}

#[test]
fn large_do_count_loop_does_not_recurse_through_the_rust_stack() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Do[Null, {10000}]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
}

#[test]
fn table_propagates_return_to_function_boundaries() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Function[Table[If[Equal[i, 2], Return[i], i], {i, 3}]][]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("side = 0; Enclose[Confirm[1, side = 1]]; side")
            .unwrap()
            .value
            .to_string(),
        "0"
    );

    session
        .eval_text("g[] := Confirm[$Failed, info, tag]")
        .expect("should define tagged confirmation helper");

    assert_eq!(
        session
            .eval_text("Enclose[g[], \"Information\", tag]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
}

#[test]
fn confirm_quiet_catches_and_quiets_matching_messages() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Enclose[ConfirmQuiet[First[f[]]], \"HeldMessage\"]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "Hold[First::nofirst]");
    assert!(outcome.messages.is_empty());
}
