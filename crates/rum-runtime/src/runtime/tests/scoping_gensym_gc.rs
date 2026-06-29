use super::*;

#[test]
fn recursive_module_sweep_reclaims_dead_retained_gensyms() {
    let mut session = Session::new();

    session
        .eval_text(
            "ClearAll[gcRec050]; gcRec050[n_] := Module[{r}, If[n <= 0, 0, r = gcRec050[n - 1] + 1; r]]",
        )
        .unwrap();

    for _ in 0..4 {
        assert_eq!(eval_string(&mut session, "Do[gcRec050[12], {15}]"), "Null");
        assert!(
            retained_candidate_definition_count(&session) > 0,
            "recursive Module workload should retain escaped gensyms before sweep"
        );

        session.collect_and_sweep_dead_gensyms(&[]);

        assert_eq!(retained_candidate_definition_count(&session), 0);
        assert!(session.gc_candidate_gensyms.is_empty());
    }

    assert_eq!(eval_string(&mut session, "gcRec050[5]"), "5");
}

#[test]
fn returned_expr_extra_root_keeps_candidate_definition_live() {
    let mut session = Session::new();

    let outcome = session.eval_text("Module[{x}, x = 7; Hold[x]]").unwrap();
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);

    session.collect_and_sweep_dead_gensyms(std::slice::from_ref(&outcome.value));

    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate)),
        "returned Hold[x$] must keep x$ live while it is an extra root"
    );

    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(
        candidates
            .iter()
            .all(|candidate| !session.definitions.contains_key(candidate)),
        "candidate should sweep once the returned value is no longer a root"
    );
}

#[test]
fn global_root_keeps_escaped_candidate_until_root_is_cleared() {
    let mut session = Session::new();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcRoot050]; Module[{x}, x = 7; gcRoot050 = Hold[x]; 0]"
        ),
        "0"
    );
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, "First[gcRoot050]"), "7");
    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate))
    );

    session.eval_text("Clear[gcRoot050]").unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(
        candidates
            .iter()
            .all(|candidate| !session.definitions.contains_key(candidate))
    );
}

#[test]
fn live_candidate_definition_marks_transitive_candidate_dependencies() {
    let mut session = Session::new();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcTrans050]; Module[{a, b}, b = 11; a := b; gcTrans050 = Hold[a]; 0]"
        ),
        "0"
    );
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 2);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, "First[gcTrans050]"), "11");
    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate)),
        "rooted candidate a$ should keep candidate b$ live through a$'s definition"
    );
}

#[test]
fn unique_generated_symbols_are_not_module_gc_candidates() {
    let mut session = Session::new();

    let generated = eval_string(&mut session, "Unique[gcUnique050]");
    assert_eq!(session.gc_candidate_gensyms.len(), 0);

    assert_eq!(
        eval_string(&mut session, &format!("{generated} = 17")),
        "17"
    );
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, &generated), "17");
    assert_eq!(session.gc_candidate_gensyms.len(), 0);
}

fn eval_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn retained_candidate_definition_count(session: &Session) -> usize {
    session
        .gc_candidate_gensyms
        .iter()
        .filter(|candidate| session.definitions.contains_key(candidate))
        .count()
}
