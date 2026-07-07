use super::*;

#[test]
fn select_positions_returns_position_paths_matching_position() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("SelectPositions[{1, 2, 3, 4}, EvenQ]")
            .unwrap()
            .value
            .to_string(),
        "List[List[2], List[4]]"
    );
    assert_eq!(
        session
            .eval_text("SelectPositions[{1, 2, 3, 4, 6}, EvenQ, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[2], List[4]]"
    );
    assert_eq!(
        session
            .eval_text("SelectPositions[{1, 2, 3, 4}, OddQ, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("SelectPositions[x, EvenQ]")
            .unwrap()
            .value
            .to_string(),
        "SelectPositions[x, EvenQ]"
    );
}

#[test]
fn duplicates_returns_repeated_values_once_in_first_appearance_order() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Duplicates[{1, 2, 1, 3, 2, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Duplicates[{1, 2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Duplicates[x]")
            .unwrap()
            .value
            .to_string(),
        "Duplicates[x]"
    );
}

#[test]
fn binary_search_uses_canonical_order_and_not_found_missing() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("BinarySearch[{1, 3, 5, 7, 9}, 7]")
            .unwrap()
            .value
            .to_string(),
        "4"
    );
    assert_eq!(
        session
            .eval_text("BinarySearch[{1, 3, 5, 7, 9}, 4]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotFound\"]"
    );
    assert_eq!(
        session
            .eval_text("BinarySearch[{a, b, c}, b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("BinarySearch[x, 3]")
            .unwrap()
            .value
            .to_string(),
        "BinarySearch[x, 3]"
    );
}
