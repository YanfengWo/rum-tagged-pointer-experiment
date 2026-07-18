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
