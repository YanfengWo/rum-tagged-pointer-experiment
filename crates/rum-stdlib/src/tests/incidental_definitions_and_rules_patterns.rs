use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn compiled_downvalue_matcher_falls_back_for_condition_patterns() {
    let mut session = core_session();

    session.eval_text("p[x_] /; x > 0 := x").unwrap();

    assert_eq!(session.eval_text("p[3]").unwrap().value.to_string(), "3");
    assert_eq!(
        session.eval_text("p[-1]").unwrap().value.to_string(),
        "p[-1]"
    );
}
