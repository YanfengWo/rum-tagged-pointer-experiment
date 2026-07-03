use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn association_values_keep_nothing_but_list_inputs_drop_it() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Association[a -> Nothing]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Nothing]]"
    );
    assert_eq!(
        session
            .eval_text("Association[{a -> 1, Nothing, b -> 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
}
