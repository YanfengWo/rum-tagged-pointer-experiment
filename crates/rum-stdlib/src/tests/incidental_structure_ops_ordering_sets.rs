use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn sort_with_ordering_function_evaluates_comparator() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, Greater]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 2, 1]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, #1 > #2 &]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 2, 1]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, Order[#1, #2] === 1 &]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{c, a, b}, Order[#1, #2] === 1 &]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Sort[f[3, 1, 2], Less]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{1, 3, 2, 3, 1}, Less]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 1, 2, 3, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, #1 + #2 &]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 1, 2]"
    );
}
