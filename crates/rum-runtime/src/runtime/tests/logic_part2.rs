use super::*;

#[test]
fn equal_reduces_adjacent_mixed_comparable_pairs_only() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Equal[1, \"1\", u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Equal[\"1\", 1, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Equal[1, u, \"1\"]")
            .unwrap()
            .value
            .to_string(),
        "Equal[1, u, \"1\"]"
    );
    assert_eq!(
        session
            .eval_text("Equal[\"a\", u, \"b\"]")
            .unwrap()
            .value
            .to_string(),
        "Equal[\"a\", u, \"b\"]"
    );
}

#[test]
fn equal_and_unequal_handle_boolean_pairs_without_overreaching() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Equal[True, False]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, False]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Equal[True, 1]")
            .unwrap()
            .value
            .to_string(),
        "Equal[True, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, 1]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[True, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, False, True]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, x, True]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[True, x, True]"
    );
}

#[test]
fn unequal_detects_repeated_known_prefix_values_before_unknown_tail() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Unequal[1, 2, 1, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[1, \"1\", 1, u]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Unequal[u, 1, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[u, 1, 2, 1]"
    );
    assert_eq!(
        session
            .eval_text("Unequal[True, False, 1, u]")
            .unwrap()
            .value
            .to_string(),
        "Unequal[True, False, 1, u]"
    );
}
