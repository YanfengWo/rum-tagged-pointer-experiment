use super::*;

#[test]
fn names_supports_current_first_pass_string_patterns() {
    let mut session = Session::new();

    session.eval_text("pkg`alpha = 1").unwrap();
    session.eval_text("pkg`beta = 2").unwrap();
    session.eval_text("pkg`local = 4").unwrap();
    session.eval_text("pkg`llocal = 5").unwrap();
    session.eval_text("pkg`lllocal = 6").unwrap();
    session.eval_text("local = 3").unwrap();

    assert_eq!(
        session
            .eval_text("Names[\"local\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"local\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[\"pkg`a*\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[StringExpression[\"pkg`\", Alternatives[\"a*\", \"b*\"]]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[{\"pkg`a*\", \"pkg`b*\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`[ab].*$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"pkg`alpha|pkg`beta\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`(alpha|beta)$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`(?:alpha|beta)$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`l{1,2}ocal$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`llocal\", \"pkg`local\"]"
    );
}

#[test]
fn names_without_explicit_context_only_sees_visible_contexts() {
    let mut session = Session::new();

    session.eval_text("Hidden`local = 1").unwrap();

    assert_eq!(
        session
            .eval_text("Names[\"local\"]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Names[\"Hidden`local\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Hidden`local\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"Hidden`l.*\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Hidden`local\"]"
    );
}

#[test]
fn names_accepts_current_options_subset() {
    let mut session = Session::new();

    session.eval_text("CasePkg`Alpha = 1").unwrap();

    assert_eq!(
        session
            .eval_text("Names[\"casepkg`alpha\"]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Names[\"casepkg`alpha\", IgnoreCase -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[\"CasePkg`Alpha\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^casepkg`a[a-z]+$\"], IgnoreCase -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[\"CasePkg`Alpha\"]"
    );

    session
        .eval_text("SetOptions[Names, IgnoreCase -> True]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("Names[\"casepkg`alpha\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"CasePkg`Alpha\"]"
    );

    let invalid = session.eval_text("Names[\"*\", Bogus -> True]").unwrap();
    assert_eq!(invalid.value.to_string(), "Names[\"*\", Rule[Bogus, True]]");
    assert!(
        invalid
            .messages
            .iter()
            .any(|message| message.symbol == "Names" && message.tag == "argform")
    );
}
