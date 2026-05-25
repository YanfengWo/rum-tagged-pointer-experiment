use super::*;

#[test]
fn order_places_negative_powers_before_their_bases() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Order[b^-1, b]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[b, b^-1]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Order[a*b, a/b]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{a/b, a*b}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("OrderedQ[{a*b, a/b}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn negative_power_order_compares_bases_before_exponents() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Order[b^-1, a^2]")
            .unwrap()
            .value
            .to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Order[a^2, b^-1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[b^-2, b^-1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[f[b^-1], f[b]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Order[\"x\"^-1, \"x\"]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}
