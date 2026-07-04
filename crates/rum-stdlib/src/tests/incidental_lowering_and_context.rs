use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn applies_at_sugar_through_lowering_and_evaluation() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Length @ f[a, b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Length @ Identity @ f[a, b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn functional_operator_sugars_flow_through_lowering_and_first_pass_runtime() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("f @@ {a, b}").unwrap().value.to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("f @@@ {{a, b}, {c, d}}")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], f[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("Length @* Identity @ f[a, b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Length /* Identity @ f[a, b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Composition[]")
            .unwrap()
            .value
            .to_string(),
        "Identity"
    );
    assert_eq!(
        session
            .eval_text("Composition[Identity, f]")
            .unwrap()
            .value
            .to_string(),
        "f"
    );
    assert_eq!(
        session
            .eval_text("RightComposition[f]")
            .unwrap()
            .value
            .to_string(),
        "f"
    );
}
