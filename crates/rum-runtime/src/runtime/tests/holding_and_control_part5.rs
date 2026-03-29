use super::*;

#[test]
fn table_supports_count_iterators_and_nested_iterators() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Table[x, 3]").unwrap().value.to_string(),
        "List[x, x, x]"
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
            .eval_text("Table[i*j, {i, 2}, {j, i}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], List[2, 4]]"
    );
}

#[test]
fn table_propagates_return_to_function_boundaries() {
    let mut session = Session::new();

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
fn failure_and_enclose_confirm_family_cover_first_pass_error_flow() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("FailureQ[Failure[\"oops\", {\"Information\" -> 1}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FailureQ[Enclose[Confirm[$Failed]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Enclose[Confirm[$Failed, info], \"Information\"]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[2, IntegerQ]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmBy[x, IntegerQ, info], \"Function\"]")
            .unwrap()
            .value
            .to_string(),
        "IntegerQ"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmMatch[2, _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmMatch[x, _Integer, info], \"Pattern\"]")
            .unwrap()
            .value
            .to_string(),
        "Blank[Integer]"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[True]]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("Enclose[ConfirmAssert[False, info], \"Information\"]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
}

#[test]
fn confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("side = 0; Enclose[Confirm[1, side = 1]]; side")
            .unwrap()
            .value
            .to_string(),
        "0"
    );

    session
        .eval_text("g[] := Confirm[$Failed, info, tag]")
        .expect("should define tagged confirmation helper");

    assert_eq!(
        session
            .eval_text("Enclose[g[], \"Information\", tag]")
            .unwrap()
            .value
            .to_string(),
        "info"
    );
}

#[test]
fn confirm_quiet_catches_and_quiets_matching_messages() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("Enclose[ConfirmQuiet[First[f[]]], \"HeldMessage\"]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "Hold[First::first]");
    assert!(outcome.messages.is_empty());
}
