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

#[test]
fn table_supports_count_iterators_and_nested_iterators() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("Table[x, 3]").unwrap().value.to_string(),
        "List[x, x, x]"
    );
    assert_eq!(
        session
            .eval_text("Table[Null, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[Null, Null, Null]"
    );
    assert_eq!(
        session
            .eval_text("Table[i^2, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 4, 9]"
    );
    assert_eq!(
        session
            .eval_text("Table[i, {i, {2, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[i + 1, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[1 + i, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("Table[{i, i, 0}, {i, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 1, 0], List[2, 2, 0], List[3, 3, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Table[i*j, {i, 2}, {j, i}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], List[2, 4]]"
    );
    assert_eq!(
        session
            .eval_text("Length[Table[i, {i, 1200}]]")
            .unwrap()
            .value
            .to_string(),
        "1200"
    );
    assert_eq!(
        session
            .eval_text("Flatten[Table[j, {i, 2}, {j, 2}], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 1, 2]"
    );
    assert_eq!(
        session
            .eval_text("argLevels = {0, 1}; fvVars = {a, b}; Table[argLevels[[i]] -> fvVars[[i]], {i, 1, Length[argLevels]}]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[0, a], Rule[1, b]]"
    );
}

#[test]
fn confirm_quiet_catches_and_quiets_matching_messages() {
    let mut session = core_session();

    let outcome = session
        .eval_text("Enclose[ConfirmQuiet[First[f[]]], \"HeldMessage\"]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "Hold[First::nofirst]");
    assert!(outcome.messages.is_empty());
}
