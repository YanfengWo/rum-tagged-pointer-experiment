use super::*;

#[test]
fn unique_generates_fresh_symbols_for_current_prefix_subset() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Unique[]").unwrap().value.to_string(),
        "$1"
    );
    assert_eq!(
        session
            .eval_text("Unique[\"thm$\"]")
            .unwrap()
            .value
            .to_string(),
        "thm$2"
    );
    assert_eq!(
        session.eval_text("Unique[x]").unwrap().value.to_string(),
        "x$3"
    );
    assert_eq!(
        session
            .eval_text("Unique[\"Private`tag\"]")
            .unwrap()
            .value
            .to_string(),
        "Private`tag4"
    );
}

#[test]
fn unique_skips_existing_generated_names() {
    let mut session = Session::new();

    session.eval_text("x1 = occupied").unwrap();
    session.eval_text("x$2 = occupied").unwrap();

    assert_eq!(
        session
            .eval_text("Unique[\"x\"]")
            .unwrap()
            .value
            .to_string(),
        "x2"
    );
    assert_eq!(
        session.eval_text("Unique[x]").unwrap().value.to_string(),
        "x$3"
    );
}

#[test]
fn unique_registers_generated_symbols_as_known_names() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Unique[\"Private`tag\"]")
            .unwrap()
            .value
            .to_string(),
        "Private`tag1"
    );
    assert_eq!(
        session
            .eval_text("Names[\"Private`tag*\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Private`tag1\"]"
    );
}

#[test]
fn unique_reports_current_invalid_boundaries() {
    let mut session = Session::new();

    let invalid_value = session.eval_text("Unique[1]").unwrap();
    assert_eq!(invalid_value.value.to_string(), "Unique[1]");
    assert_eq!(invalid_value.messages[0].symbol, "Unique");
    assert_eq!(invalid_value.messages[0].tag, "usym");
    assert_eq!(
        invalid_value.messages[0].text,
        "1 is not a symbol or a valid symbol name."
    );

    let invalid_name = session.eval_text("Unique[\"1x\"]").unwrap();
    assert_eq!(invalid_name.value.to_string(), "Unique[\"1x\"]");
    assert_eq!(invalid_name.messages[0].symbol, "Unique");
    assert_eq!(invalid_name.messages[0].tag, "usym");
    assert_eq!(
        invalid_name.messages[0].text,
        "\"1x\" is not a symbol or a valid symbol name."
    );

    let too_many = session.eval_text("Unique[\"x\", y]").unwrap();
    assert_eq!(too_many.value.to_string(), "Unique[\"x\", y]");
    assert_eq!(too_many.messages[0].symbol, "Unique");
    assert_eq!(too_many.messages[0].tag, "argtu");
}
