use super::*;
use rum_runtime::SessionRequestError;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn sum_and_product_reuse_concrete_iterator_specs() {
    let mut session = core_session();

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
fn iterator_spec_matrix_covers_counts_lists_ranges_steps_and_rejections() {
    let cases = [
        ("x = 0; Do[x = x + 1, {3}]; x", "3"),
        ("Table[x, {3}]", "List[x, x, x]"),
        ("Sum[x, {3}]", "Times[3, x]"),
        ("Product[x, {3}]", "Power[x, 3]"),
        ("x = 0; Do[x = x + i, {i, {2, 4}}]; x", "6"),
        ("Table[i, {i, {2, 4}}]", "List[2, 4]"),
        ("Sum[i, {i, {2, 4}}]", "6"),
        ("Product[i, {i, {2, 4}}]", "8"),
        ("x = 0; Do[x = x + i, {i, 3}]; x", "6"),
        ("Table[i, {i, 3}]", "List[1, 2, 3]"),
        ("Sum[i, {i, 3}]", "6"),
        ("Product[i, {i, 3}]", "6"),
        ("x = 0; Do[x = x + i, {i, 2, 6, 2}]; x", "12"),
        ("Table[i, {i, 2, 6, 2}]", "List[2, 4, 6]"),
        ("Sum[i, {i, 2, 6, 2}]", "12"),
        ("Product[i, {i, 2, 6, 2}]", "48"),
        ("x = 0; Do[x = x + i, {i, Times[-1, 3], 1, 2}]; x", "-3"),
        ("Table[i, {i, Times[-1, 3], 1, 2}]", "List[-3, -1, 1]"),
        ("Sum[i, {i, Times[-1, 3], 1, 2}]", "-3"),
        ("Product[i, {i, Times[-1, 3], 1, 2}]", "3"),
    ];

    for (source, expected) in cases {
        let mut session = core_session();
        assert_eq!(
            session.eval_text(source).unwrap().value.to_string(),
            expected,
            "{source}"
        );
    }

    for (source, symbol, expected) in [
        ("Do[x, {i, 1, 3, 0}]", "Do", "Do[x, List[i, 1, 3, 0]]"),
        (
            "Table[i, {i, 1, 3, 0}]",
            "Table",
            "Table[i, List[i, 1, 3, 0]]",
        ),
        ("Do[x, {i, 1.5}]", "Do", "Do[x, List[i, 3/2]]"),
        ("Table[i, {i, 1.5}]", "Table", "Table[i, List[i, 3/2]]"),
    ] {
        let mut session = core_session();
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == "argform"),
            "{source} messages {:?}",
            outcome.messages
        );
    }

    for (source, symbol, expected) in [
        ("Sum[i, {i, 1, 3, 0}]", "Sum", "Sum[i, List[i, 1, 3, 0]]"),
        (
            "Product[i, {i, 1, 3, 0}]",
            "Product",
            "Product[i, List[i, 1, 3, 0]]",
        ),
        ("Sum[i, {i, 1.5}]", "Sum", "Sum[i, List[i, 3/2]]"),
        (
            "Product[i, {i, 1.5}]",
            "Product",
            "Product[i, List[i, 3/2]]",
        ),
    ] {
        let mut session = core_session();
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == "argform"),
            "{source} messages {:?}",
            outcome.messages
        );
    }
}

#[test]
fn sum_and_product_leave_unsupported_specs_unevaluated_with_argform() {
    let mut session = core_session();

    let cases = [
        ("Sum[i, {i, 1, n}]", "Sum", "Sum[i, List[i, 1, n]]"),
        (
            "Product[i, {i, 1, n}]",
            "Product",
            "Product[i, List[i, 1, n]]",
        ),
        ("Sum[x, 3]", "Sum", "Sum[x, 3]"),
        ("Product[x, 3]", "Product", "Product[x, 3]"),
    ];

    for (source, symbol, expected) in cases {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == "argform"),
            "{source} messages {:?}",
            outcome.messages
        );
    }
}

#[test]
fn sum_and_product_report_declared_attributes() {
    let mut session = core_session();

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
    let mut session = core_session();

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
    let mut session = core_session();

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
    let mut table_session = core_session();
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

    let mut do_session = core_session();
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

    let mut iterator_session = core_session();
    iterator_session.set_generated_element_limit(3);
    assert_eq!(
        iterator_session
            .eval_text("Table[i, {i, 5}]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );

    let mut memory_session = core_session();
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
fn multi_iterator_table_charges_the_generated_product() {
    let mut session = core_session();
    session.set_generated_element_limit(5);

    assert_eq!(
        session
            .eval_text("Table[i + j, {i, 2}, {j, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[2, 3], List[3, 4]]"
    );

    let refused = session.eval_text("Table[0, {i, 2}, {j, 3}]").unwrap();
    assert_eq!(
        refused.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(refused.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "reslim"
            && message.text == "evaluation exceeded generated element limit (5)"
    }));
}

#[test]
fn multi_iterator_table_precharges_exact_arithmetic_counts() {
    let mut session = core_session();
    session.set_generated_element_limit(9_999);
    session.eval_text("calls = 0").unwrap();

    let refused = session
        .eval_text("Table[calls = calls + 1, {i, 1, 10^2}, {j, 1, 10^2}]")
        .unwrap();
    assert_eq!(
        refused.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert_eq!(session.eval_text("calls").unwrap().value.to_string(), "0");

    let mut overflow_session = core_session();
    overflow_session.set_generated_element_limit(usize::MAX);
    let factor = 1_usize << (usize::BITS / 2);
    let source = format!("Table[0, {{i, {factor}}}, {{j, {factor}}}]");
    let refused_overflow = overflow_session.eval_text(&source).unwrap();
    assert_eq!(
        refused_overflow.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
}

#[test]
fn multi_iterator_control_siblings_charge_iteration_product() {
    for (accepted_source, accepted_value, refused_source) in [
        (
            "calls = 0; Do[calls = calls + 1, {i, 2}, {j, 2}]; calls",
            "4",
            "Do[Null, {i, 2}, {j, 3}]",
        ),
        ("Sum[1, {i, 2}, {j, 2}]", "4", "Sum[1, {i, 2}, {j, 3}]"),
        (
            "Product[2, {i, 2}, {j, 2}]",
            "16",
            "Product[2, {i, 2}, {j, 3}]",
        ),
    ] {
        let mut session = core_session();
        session.set_iteration_work_limit(5);
        assert_eq!(
            session
                .eval_text(accepted_source)
                .unwrap()
                .value
                .to_string(),
            accepted_value,
            "{accepted_source}"
        );

        let refused = session.eval_text(refused_source).unwrap();
        assert_eq!(
            refused.value.to_string(),
            "TerminatedEvaluation[\"IterationWorkLimit\"]",
            "{refused_source}"
        );
        assert!(refused.messages.iter().any(|message| {
            message.symbol == "General"
                && message.tag == "reslim"
                && message.text == "evaluation exceeded iteration work limit (5)"
        }));
    }
}

#[test]
fn large_iteration_products_still_refuse() {
    for (source, termination, message) in [
        (
            "Table[0, {i, 1, 10^4}, {j, 1, 10^4}]",
            "TerminatedEvaluation[\"GeneratedElementLimit\"]",
            "evaluation exceeded generated element limit (100000)",
        ),
        (
            "n = 10^4; Table[0, {i, 1, n}, {j, 1, n}]",
            "TerminatedEvaluation[\"GeneratedElementLimit\"]",
            "evaluation exceeded generated element limit (100000)",
        ),
        (
            "n = 10^4; Table[Table[0, {j, 1, n}], {i, 1, n}]",
            "TerminatedEvaluation[\"GeneratedElementLimit\"]",
            "evaluation exceeded generated element limit (100000)",
        ),
        (
            "Table[0, {i, 1, 10^4}, {j, 1, i}]",
            "TerminatedEvaluation[\"GeneratedElementLimit\"]",
            "evaluation exceeded generated element limit (100000)",
        ),
        (
            "n = 10^4; Do[0, {i, 1, n}, {j, 1, n}]",
            "TerminatedEvaluation[\"IterationWorkLimit\"]",
            "evaluation exceeded iteration work limit (100000)",
        ),
        (
            "n = 10^4; Sum[1, {i, 1, n}, {j, 1, n}]",
            "TerminatedEvaluation[\"IterationWorkLimit\"]",
            "evaluation exceeded iteration work limit (100000)",
        ),
    ] {
        let mut session = core_session();
        let refused = session.eval_text(source).unwrap();
        assert_eq!(refused.value.to_string(), termination, "{source}");
        assert!(
            refused.messages.iter().any(|candidate| {
                candidate.symbol == "General"
                    && candidate.tag == "reslim"
                    && candidate.text == message
            }),
            "{source} messages {:?}",
            refused.messages
        );
    }
}

#[test]
fn non_retaining_consumers_do_not_propagate_their_product_into_table() {
    for (source, expected) in [
        ("Do[Table[i, {i, 1, 1000}], {200}]", "Null"),
        ("Do[Length[Table[i, {i, 1, 500}]], {500}]", "Null"),
    ] {
        let mut session = core_session();
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(outcome.messages.is_empty(), "{source}");
    }

    for (source, expected) in [
        ("Sum[Length[Table[i, {i, 1, 3}]], {2}]", "6"),
        ("Product[Length[Table[i, {i, 1, 3}]], {2}]", "9"),
    ] {
        let mut session = core_session();
        session.set_generated_element_limit(5);
        session.set_iteration_work_limit(5);
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(outcome.messages.is_empty(), "{source}");
    }
}

#[test]
fn nested_iteration_constructs_share_the_running_product() {
    let mut session = core_session();
    session.set_generated_element_limit(5);

    let refused = session
        .eval_text("n = 3; Table[Table[0, {j, 1, n}], {i, 1, n}]")
        .unwrap();
    assert_eq!(
        refused.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(refused.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "reslim"
            && message.text == "evaluation exceeded generated element limit (5)"
    }));
}

#[test]
fn iteration_product_scope_resets_between_statements_and_after_errors() {
    let mut session = core_session();
    session.set_generated_element_limit(5);

    let separate = session
        .eval_text("a = Table[0, {i, 3}]; b = Table[1, {j, 3}]; {a, b}")
        .unwrap();
    assert_eq!(
        separate.value.to_string(),
        "List[List[0, 0, 0], List[1, 1, 1]]"
    );
    assert!(separate.messages.is_empty());

    assert_eq!(
        session
            .eval_text("Table[0, {i, 3}, {j, 2}]")
            .unwrap()
            .value
            .to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert_eq!(
        session
            .eval_text("Table[2, {k, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 2, 2]"
    );
}

#[test]
fn suspended_nested_loop_retains_and_then_resets_the_running_product() {
    let mut session = core_session();
    session.set_iteration_work_limit(5);

    let failure = session
        .eval_request("Do[If[SameQ[i, 1], InputString[\"budget: \"], Do[Null, {j, 3}]], {i, 2}]")
        .expect_err("the first outer iteration should request input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "budget: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let resumed = session
        .resume_eval_request_with_input("ready")
        .expect("the resumed loop should terminate through the retained budget");
    assert_eq!(
        resumed.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(resumed.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "reslim"
            && message.text == "evaluation exceeded iteration work limit (5)"
    }));
    assert_eq!(
        session
            .eval_text("Do[Null, {k, 3}]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
}

#[test]
fn ordinary_nested_iteration_results_are_unchanged() {
    for (source, expected) in [
        (
            "n = 3; Table[i + j, {i, 1, n}, {j, 1, n}]",
            "List[List[2, 3, 4], List[3, 4, 5], List[4, 5, 6]]",
        ),
        (
            "Table[i + j, {i, 1, 3}, {j, 1, i}]",
            "List[List[2], List[3, 4], List[4, 5, 6]]",
        ),
        (
            "Table[Table[i + j, {j, 1, 2}], {i, 1, 2}]",
            "List[List[2, 3], List[3, 4]]",
        ),
        (
            "n = 3; calls = 0; Do[calls = calls + 1, {i, 1, n}, {j, 1, n}]; calls",
            "9",
        ),
        ("Sum[i + j, {i, 1, 3}, {j, 1, 3}]", "36"),
        ("Product[i + j, {i, 1, 2}, {j, 1, 2}]", "72"),
    ] {
        let mut session = core_session();
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(outcome.messages.is_empty(), "{source}");
    }
}

#[test]
fn table_memory_constraint_uses_projected_list_size_before_materialization() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("MemoryConstrained[Table[x, {1000000}], 64, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
}

#[test]
fn large_do_count_loop_does_not_recurse_through_the_rust_stack() {
    let mut session = core_session();

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
fn confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught() {
    let mut session = core_session();

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
fn iterator_lhs_normalization_preserves_lazy_and_container_lvalue_controls() {
    for (source, expected) in [
        (
            "ClearAll[i, f]; Normal[LazyTable[f[i] = i, {i, 1, 2}]]; DownValues[f]",
            "List[RuleDelayed[HoldPattern[f[1]], 1], RuleDelayed[HoldPattern[f[2]], 2]]",
        ),
        (
            "ClearAll[i, list]; list = {0, 0}; Do[list[[i]] = i, {i, 1, 2}]; list",
            "List[1, 2]",
        ),
        (
            "ClearAll[i, assoc]; assoc = <||>; Do[assoc[i] = i, {i, 1, 2}]; assoc",
            "Association[Rule[1, 1], Rule[2, 2]]",
        ),
    ] {
        let mut session = core_session();
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(
            outcome.messages.is_empty(),
            "{source} messages {:?}",
            outcome.messages
        );
    }
}
