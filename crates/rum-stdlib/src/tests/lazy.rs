use crate::install_core;
use rum_runtime::{Capability, EvalError, Session};
use std::sync::atomic::Ordering;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_text(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn lazy_range_builds_canonical_infinite_descriptors() {
    let mut session = core_session();

    for (source, expected) in [
        ("LazyRange[]", "LazyList[LazyRange[1, 1]]"),
        ("LazyRange[4]", "LazyList[LazyRange[4, 1]]"),
        ("LazyRange[4, 3/2]", "LazyList[LazyRange[4, 3/2]]"),
        ("x = 2; LazyRange[x + 1, x]", "LazyList[LazyRange[3, 2]]"),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }

    assert_eq!(
        eval_text(&mut session, "Attributes[LazyList]"),
        "List[HoldAllComplete, Protected]"
    );
    assert_eq!(
        eval_text(&mut session, "Attributes[LazyRange]"),
        "List[Protected]"
    );
    assert!(
        crate::catalog::LAZY_BUILTINS
            .iter()
            .all(|spec| spec.capability == Capability::Pure)
    );
}

#[test]
fn lazy_range_rejects_invalid_arguments_without_partial_values() {
    let mut session = core_session();

    let invalid_value = session.eval_text("LazyRange[x]").unwrap();
    assert_eq!(invalid_value.value.to_string(), "LazyRange[x]");
    assert!(
        invalid_value
            .messages
            .iter()
            .any(|message| message.symbol == "LazyRange" && message.tag == "argform")
    );

    let invalid_arity = session.eval_text("LazyRange[1, 2, 3]").unwrap();
    assert_eq!(invalid_arity.value.to_string(), "LazyRange[1, 2, 3]");
    assert!(
        invalid_arity
            .messages
            .iter()
            .any(|message| message.symbol == "LazyRange" && message.tag == "argtu")
    );
}

#[test]
fn lazy_list_is_opaque_structural_and_inert() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyRange[1, 1], _List]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyRange[1, 1], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "Total[LazyRange[1, 1]]"),
        "Total[LazyList[LazyRange[1, 1]]]"
    );
    assert_eq!(
        eval_text(&mut session, "LazyRange[1, 1]"),
        "LazyList[LazyRange[1, 1]]"
    );
    assert_eq!(
        eval_text(&mut session, "SameQ[LazyRange[1, 1], LazyRange[1, 1]]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "SameQ[LazyRange[1, 1], LazyRange[2, 1]]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "held = 0; LazyList[held = 1]; held"),
        "0"
    );
}

#[test]
fn lazy_realization_observes_the_session_cancel_flag() {
    let mut session = core_session();
    let lazy_list = session.eval_text("LazyRange[1, 1]").unwrap().value;
    let cancel = session.interrupt_handle();
    cancel.store(true, Ordering::Relaxed);

    assert!(matches!(
        crate::lazy::realize_all(&mut session, &lazy_list),
        Err(EvalError::Cancelled)
    ));
}

#[test]
fn bounded_realizers_pull_lazy_ranges() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[10, 5], 3]"),
        "List[10, 15, 20]"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[10, 5], UpTo[3]]"),
        "List[10, 15, 20]"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[1/2, 1/3], 3]"),
        "List[1/2, 5/6, 7/6]"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[10, 5], 0]"),
        "List[]"
    );
    assert_eq!(eval_text(&mut session, "First[LazyRange[7, 2]]"), "7");
}

#[test]
fn normal_on_infinite_lazy_range_fails_closed_at_iteration_ceiling() {
    let mut session = core_session();
    session.set_iteration_work_limit(3);

    let outcome = session.eval_text("Normal[LazyRange[1, 1]]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
    assert_eq!(eval_text(&mut session, "1 + 1"), "2");
}

#[test]
fn lazy_transformers_build_canonical_opaque_descriptors() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "LazyMap[f, LazyRange[1, 1]]",
            "LazyList[LazyMap[f, LazyRange[1, 1]]]",
        ),
        (
            "LazySelect[p, LazyRange[2, 3]]",
            "LazyList[LazySelect[p, LazyRange[2, 3]]]",
        ),
        (
            "LazyTake[LazyRange[1, 1], 4]",
            "LazyList[LazyTake[LazyRange[1, 1], 4]]",
        ),
        (
            "LazyTake[LazyRange[1, 1], UpTo[4]]",
            "LazyList[LazyTake[LazyRange[1, 1], 4]]",
        ),
        (
            "LazyDrop[LazyRange[1, 1], 4]",
            "LazyList[LazyDrop[LazyRange[1, 1], 4]]",
        ),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }

    for symbol in ["LazyDrop", "LazyMap", "LazySelect", "LazyTake"] {
        assert_eq!(
            eval_text(&mut session, &format!("Attributes[{symbol}]")),
            "List[Protected]"
        );
    }
    assert_eq!(
        eval_text(
            &mut session,
            "MatchQ[LazyTake[LazyRange[1, 1], 3], _LazyList]"
        ),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTake[LazyRange[1, 1], 3], _List]"),
        "False"
    );
}

#[test]
fn lazy_transformers_reject_invalid_inputs_without_partial_values() {
    let mut session = core_session();

    for (source, expected, symbol, tag) in [
        (
            "LazyMap[f, {1, 2}]",
            "LazyMap[f, List[1, 2]]",
            "LazyMap",
            "argform",
        ),
        (
            "LazySelect[p, {1, 2}]",
            "LazySelect[p, List[1, 2]]",
            "LazySelect",
            "argform",
        ),
        (
            "LazyTake[LazyRange[1, 1], -1]",
            "LazyTake[LazyList[LazyRange[1, 1]], -1]",
            "LazyTake",
            "argform",
        ),
        (
            "LazyTake[LazyRange[1, 1], UpTo[-1]]",
            "LazyTake[LazyList[LazyRange[1, 1]], UpTo[-1]]",
            "LazyTake",
            "argform",
        ),
        (
            "LazyDrop[LazyRange[1, 1], UpTo[2]]",
            "LazyDrop[LazyList[LazyRange[1, 1]], UpTo[2]]",
            "LazyDrop",
            "argform",
        ),
        ("LazyMap[f]", "LazyMap[f]", "LazyMap", "argx"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == tag),
            "source {source}"
        );
    }
}

#[test]
fn finite_lazy_transformer_compositions_realize_to_lists() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "Normal[LazyTake[LazyRange[1, 1], 5]]",
            "List[1, 2, 3, 4, 5]",
        ),
        (
            "Normal[LazyTake[LazyMap[Function[x, x^2], LazyRange[1, 1]], 4]]",
            "List[1, 4, 9, 16]",
        ),
        (
            "Normal[LazyTake[LazyDrop[LazyRange[1, 1], 3], 3]]",
            "List[4, 5, 6]",
        ),
        (
            "Normal[LazySelect[Function[x, 1], LazyTake[LazyRange[1, 1], 3]]]",
            "List[]",
        ),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }
}

#[test]
fn lazy_select_filters_under_bounded_realization() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Take[LazySelect[EvenQ, LazyRange[1, 1]], 3]"),
        "List[2, 4, 6]"
    );
}

#[test]
fn never_matching_lazy_select_fails_closed_at_source_pull_ceiling() {
    let mut session = core_session();
    session.set_iteration_work_limit(4);

    let outcome = session
        .eval_text("Normal[LazySelect[Function[x, False], LazyRange[1, 1]]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
}

#[test]
fn lazy_table_realizes_arithmetic_iterator_forms() {
    let mut session = core_session();

    for (source, expected) in [
        ("Normal[LazyTable[i^2, {i, 1, 5}]]", "List[1, 4, 9, 16, 25]"),
        (
            "Normal[LazyTable[i, {i, 2, 10, 2}]]",
            "List[2, 4, 6, 8, 10]",
        ),
        ("Normal[LazyTable[i, {i, 3}]]", "List[1, 2, 3]"),
        ("Normal[LazyTable[i, {i, 5, 1}]]", "List[]"),
        ("Normal[LazyTable[i, {i, 5, 1, -2}]]", "List[5, 3, 1]"),
        (
            "Normal[LazyTable[i, {i, 1/2, 2, 1/2}]]",
            "List[1/2, 1, 3/2, 2]",
        ),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }

    assert_eq!(
        eval_text(&mut session, "Attributes[LazyTable]"),
        "List[HoldAll, Protected]"
    );
}

#[test]
fn lazy_table_holds_its_body_and_binds_the_iterator_lexically() {
    let mut session = core_session();

    assert_eq!(
        eval_text(
            &mut session,
            "n = 0; i = 99; lz = LazyTable[(n = n + 1); i^2, {i, 1, 3}]; n"
        ),
        "0"
    );
    assert_eq!(eval_text(&mut session, "Normal[lz]"), "List[1, 4, 9]");
    assert_eq!(eval_text(&mut session, "n"), "3");
}

#[test]
fn lazy_table_rejects_invalid_specs_without_partial_values() {
    let mut session = core_session();

    for (source, expected, tag) in [
        ("LazyTable[i]", "LazyTable[i]", "argx"),
        ("LazyTable[i, bad]", "LazyTable[i, bad]", "argform"),
        (
            "LazyTable[i, {1, 1, 3}]",
            "LazyTable[i, List[1, 1, 3]]",
            "argform",
        ),
        (
            "LazyTable[i, {i, missing}]",
            "LazyTable[i, List[i, missing]]",
            "argform",
        ),
        (
            "LazyTable[i, {i, 1, 3, 0}]",
            "LazyTable[i, List[i, 1, 3, 0]]",
            "argform",
        ),
        (
            "LazyTable[i, {i, {1, 2}}]",
            "LazyTable[i, List[i, List[1, 2]]]",
            "argform",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "LazyTable" && message.tag == tag),
            "source {source}"
        );
    }
}

#[test]
fn lazy_table_result_is_opaque_and_bounded_take_is_lazy() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTable[i, {i, 1, 3}], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTable[i, {i, 1, 3}], _List]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyTable[i^2, {i, 1, 1000000}], 3]"),
        "List[1, 4, 9]"
    );
}

#[test]
fn lazy_drop_and_rest_keep_the_tail_lazy() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Rest[LazyRange[10, 5]]"),
        "LazyList[LazyDrop[LazyRange[10, 5], 1]]"
    );
    assert_eq!(
        eval_text(&mut session, "First[Rest[LazyRange[10, 5]]]"),
        "15"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazyTake[Rest[LazyRange[10, 5]], 3]]"),
        "List[15, 20, 25]"
    );
}

#[test]
fn nested_lazy_map_and_select_eval_errors_propagate() {
    let mut session = core_session();

    for (source, expected_value, expected_tag) in [
        (
            "Normal[LazyTake[LazyMap[Function[x, Throw[x, mapTag]], LazyRange[1, 1]], 1]]",
            "1",
            "mapTag",
        ),
        (
            "Normal[LazySelect[Function[x, Throw[x, selectTag]], LazyTake[LazyRange[1, 1], 1]]]",
            "1",
            "selectTag",
        ),
    ] {
        match session
            .eval_text(source)
            .expect_err("nested Throw should propagate out of the lazy realizer")
        {
            EvalError::Thrown { value, tag, .. } => {
                assert_eq!(value.to_string(), expected_value, "source {source}");
                assert_eq!(tag.to_string(), expected_tag, "source {source}");
            }
            other => panic!("expected nested Throw from {source}, got {other:?}"),
        }
    }
}

#[test]
fn snapshot_roundtrip_rederives_composed_lazy_descriptor() {
    let mut source = core_session();
    assert_eq!(
        eval_text(
            &mut source,
            "f[x_] := x + 1; lz = LazyTake[LazyMap[f, LazyRange[3, 2]], 4]"
        ),
        "LazyList[LazyTake[LazyMap[f, LazyRange[3, 2]], 4]]"
    );
    let snapshot = source.snapshot_definitions_source();

    let mut restored = core_session();
    restored
        .eval_script(&snapshot)
        .expect("composed lazy-list snapshot should restore");

    assert_eq!(
        eval_text(&mut restored, "lz"),
        "LazyList[LazyTake[LazyMap[f, LazyRange[3, 2]], 4]]"
    );
    assert_eq!(eval_text(&mut restored, "Normal[lz]"), "List[4, 6, 8, 10]");
}

#[test]
fn snapshot_roundtrip_rederives_lazy_table_descriptor() {
    let mut source = core_session();
    assert_eq!(
        eval_text(&mut source, "lz = LazyTable[i^2, {i, 2, 6, 2}]"),
        "LazyList[LazyTake[LazyMap[Function[i, Power[i, 2]], LazyRange[2, 2]], 3]]"
    );
    let snapshot = source.snapshot_definitions_source();

    let mut restored = core_session();
    restored
        .eval_script(&snapshot)
        .expect("lazy-table snapshot should restore");

    assert_eq!(
        eval_text(&mut restored, "lz"),
        "LazyList[LazyTake[LazyMap[Function[i, Power[i, 2]], LazyRange[2, 2]], 3]]"
    );
    assert_eq!(eval_text(&mut restored, "Normal[lz]"), "List[4, 16, 36]");
}
