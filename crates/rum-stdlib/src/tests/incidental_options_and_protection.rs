use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn setoptions_updates_system_heads_defaults_for_cases_position_and_depth() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Cases[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    session
        .eval_text("SetOptions[Cases, Heads -> True]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Cases[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[f]"
    );

    assert_eq!(
        session
            .eval_text("Position[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0]]"
    );
    session
        .eval_text("SetOptions[Position, Heads -> False]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Position[f[x], f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    session
        .eval_text("SetOptions[Depth, Heads -> True]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}
