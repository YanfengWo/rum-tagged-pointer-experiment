use super::*;

#[test]
fn function_accepts_explicit_attributes_for_application() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Function[x, Hold[x], HoldAll][Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1]]"
    );
    assert_eq!(
        session
            .eval_text("Function[Hold[Slot[1]], HoldAll][Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1]]"
    );
    assert_eq!(
        session
            .eval_text("Function[{x, y}, Hold[x, y], HoldFirst][Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1], 2]"
    );
}
