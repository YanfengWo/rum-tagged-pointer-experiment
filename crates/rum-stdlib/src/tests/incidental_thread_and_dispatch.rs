use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn dispatch_stage_guards_preserve_value_and_builtin_semantics() {
    let mut session = core_session();

    assert_eq!(session.eval_text("k[1]").unwrap().value.to_string(), "k[1]");
    session.eval_text("k[1] = 9").unwrap();
    assert_eq!(session.eval_text("k[1]").unwrap().value.to_string(), "9");

    session.eval_text("m /: f[m] := ok").unwrap();
    assert_eq!(session.eval_text("f[m]").unwrap().value.to_string(), "ok");

    session.eval_text("d[1][2] := 12").unwrap();
    assert_eq!(
        session.eval_text("d[1][2]").unwrap().value.to_string(),
        "12"
    );

    assert_eq!(
        session.eval_text("SameQ[1, 1]").unwrap().value.to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("If[True, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Length[{1, 2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}
