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
