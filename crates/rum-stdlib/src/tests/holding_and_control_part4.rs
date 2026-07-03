use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn do_supports_count_iterators_and_nested_iterators() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("sum = 0; Do[sum = sum + 1, 3]; sum")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("sum = 0; Do[sum = sum + i, {i, 3}]; sum")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
    assert_eq!(
        session
            .eval_text("sum = 0; Do[sum = sum + i*j, {i, 2}, {j, i}]; sum")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("sum = 0; Do[sum = sum + i, {i, {2, 4}}]; sum")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
}

#[test]
fn while_until_and_for_support_first_pass_loop_control() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("x = 0; While[Less[x, 3], x = x + 1]; x")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("x = 0; Until[Greater[x, 2], x = x + 1]; x")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("sum = 0; For[i = 0, Less[i, 4], i = i + 1, sum = sum + i]; sum")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
}

#[test]
fn break_and_continue_are_handled_by_current_loop_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("sum = 0; Do[If[Equal[i, 2], Continue[], sum = sum + i], {i, 3}]; sum")
            .unwrap()
            .value
            .to_string(),
        "4"
    );
    assert_eq!(
        session
            .eval_text("x = 0; While[True, If[Equal[x, 2], Break[], x = x + 1]]; x")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "sum = 0; For[i = 0, Less[i, 4], i = i + 1, If[Equal[i, 2], Continue[], sum = sum + i]]; sum"
            )
            .unwrap()
            .value
            .to_string(),
        "4"
    );
}

#[test]
fn return_propagates_through_loop_bodies_to_function_boundaries() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Function[While[True, Return[7]]][]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Function[Do[Return[5], {3}]][]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}
