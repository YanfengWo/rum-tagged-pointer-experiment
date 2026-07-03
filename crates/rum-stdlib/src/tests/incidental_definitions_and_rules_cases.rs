use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn cases_and_deletecases_use_value_only_association_traversal() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Cases[Association[a -> 1, b -> f[2]], _Symbol, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Cases[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Association, f]"
    );
    assert_eq!(
        session
            .eval_text(
                "DeleteCases[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("DeleteCases[Association[a -> 1, b -> f[2]], f, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text(
                "DeleteCases[Association[a -> 1, b -> f[2]], Association, Infinity, Heads -> True]",
            )
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
}
