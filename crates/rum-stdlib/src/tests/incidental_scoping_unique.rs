use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn unique_supports_list_and_attribute_forms() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("Unique[{}]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Unique[{x, y}]")
            .unwrap()
            .value
            .to_string(),
        "List[x$1, y$1]"
    );
    assert_eq!(
        session
            .eval_text("Unique[{\"a\", b}]")
            .unwrap()
            .value
            .to_string(),
        "List[a2, b$3]"
    );
    assert_eq!(
        session
            .eval_text("Map[Attributes, Unique[{p, q}, {Orderless, Protected, Temporary}]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Orderless, Protected, Temporary], List[Orderless, Protected, Temporary]]"
    );
}
