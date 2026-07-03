use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn table_propagates_return_to_function_boundaries() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Function[Table[If[Equal[i, 2], Return[i], i], {i, 3}]][]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}
