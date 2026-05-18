use super::*;

#[test]
fn associateto_mutates_deep_key_path_targets() {
    let mut session = Session::new();
    session
        .eval_text("nested = Association[a -> Association[b -> Association[c -> 1, d -> 2]]]")
        .unwrap();

    let expected = "Association[Rule[a, Association[Rule[b, Association[Rule[c, 1], Rule[d, 2], Rule[e, 5]]]]]]";
    assert_eq!(
        session
            .eval_text("AssociateTo[nested[[Key[a], Key[b]]], e -> 5]")
            .unwrap()
            .value
            .to_string(),
        expected
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        expected
    );
}

#[test]
fn associateto_mutates_mixed_integer_and_key_part_targets() {
    let mut session = Session::new();
    session
        .eval_text("parts = {Association[a -> Association[b -> 1]]}")
        .unwrap();

    let expected = "List[Association[Rule[a, Association[Rule[b, 1], Rule[c, 3]]]]]";
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[1, Key[a]]], c -> 3]")
            .unwrap()
            .value
            .to_string(),
        expected
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        expected
    );
}

#[test]
fn associateto_mutates_bulk_nested_part_targets() {
    let mut session = Session::new();
    session
        .eval_text("parts = {{Association[a -> 1]}, {Association[a -> 2]}}")
        .unwrap();

    let expected = "List[List[Association[Rule[a, 1], Rule[b, 3]]], List[Association[Rule[a, 2], Rule[b, 3]]]]";
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[All, 1]], b -> 3]")
            .unwrap()
            .value
            .to_string(),
        expected
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        expected
    );
}

#[test]
fn keydropfrom_mutates_deep_key_path_targets() {
    let mut session = Session::new();
    session
        .eval_text("nested = Association[a -> Association[b -> Association[c -> 1, d -> 2]]]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("KeyDropFrom[nested[[Key[a], Key[b]]], {c, e}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[d, 2]]"
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        "Association[Rule[a, Association[Rule[b, Association[Rule[d, 2]]]]]]"
    );
}

#[test]
fn keydropfrom_mutates_mixed_integer_and_key_leaf_targets() {
    let mut session = Session::new();
    session
        .eval_text("parts = {Association[a -> Association[b -> 1, c -> 2]]}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("KeyDropFrom[parts[[1, Key[a]]], c]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 1]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, Association[Rule[b, 1]]]]]"
    );
}

#[test]
fn keydropfrom_rejects_bulk_nested_part_targets_without_mutation() {
    let mut session = Session::new();
    session
        .eval_text("parts = {{Association[a -> 1, b -> 2]}, {Association[a -> 3, b -> 4]}}")
        .unwrap();

    let outcome = session
        .eval_text("KeyDropFrom[parts[[All, 1]], a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyDropFrom[Part[parts, All, 1], a]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[List[Association[Rule[a, 1], Rule[b, 2]]], List[Association[Rule[a, 3], Rule[b, 4]]]]"
    );
}
