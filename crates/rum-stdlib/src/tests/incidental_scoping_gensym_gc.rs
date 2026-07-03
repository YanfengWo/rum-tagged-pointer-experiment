use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn recursive_module_sweep_reclaims_dead_retained_gensyms() {
    let mut session = core_session();

    session
        .eval_text(
            "ClearAll[gcRec050]; gcRec050[n_] := Module[{r}, If[n <= 0, 0, r = gcRec050[n - 1] + 1; r]]",
        )
        .unwrap();

    for _ in 0..4 {
        assert_eq!(eval_string(&mut session, "Do[gcRec050[12], {15}]"), "Null");
        assert!(
            session.retained_candidate_definition_count() > 0,
            "recursive Module workload should retain escaped gensyms before sweep"
        );

        session.collect_and_sweep_dead_gensyms(&[]);

        assert_eq!(session.retained_candidate_definition_count(), 0);
        assert!(session.gc_candidate_gensym_count() == 0);
    }

    assert_eq!(eval_string(&mut session, "gcRec050[5]"), "5");
}
