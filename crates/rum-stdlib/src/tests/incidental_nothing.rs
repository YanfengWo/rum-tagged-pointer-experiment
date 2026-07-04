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

#[test]
fn nothing_is_removed_from_direct_list_elements_only() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("{1, Nothing, 2}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("{Nothing, a, Nothing, b}")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("{Nothing, Nothing}")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("{a, {b, Nothing}}")
            .unwrap()
            .value
            .to_string(),
        "List[a, List[b]]"
    );
    assert_eq!(
        session
            .eval_text("FullForm[f[a, Nothing, b]]")
            .unwrap()
            .value
            .to_string(),
        "\"f[a, Nothing, b]\""
    );
}
