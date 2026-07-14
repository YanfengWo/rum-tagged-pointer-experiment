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

    session.eval_text("Unprotect[Names]").unwrap();
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

#[test]
fn names_rejects_unsupported_spelling_correction_values() {
    let mut session = Session::new();

    session.eval_text("spellingCorrectionTarget = 1").unwrap();
    for input in [
        "Names[\"Global`spelingCorrectionTarget\"]",
        "Names[\"Global`spelingCorrectionTarget\", SpellingCorrection -> False]",
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), "List[]", "{input}");
        assert!(outcome.messages.is_empty(), "{input}");
    }

    let unsupported = session
        .eval_text("Names[\"Global`spelingCorrectionTarget\", SpellingCorrection -> True]")
        .unwrap();
    assert_eq!(
        unsupported.value.to_string(),
        "Names[\"Global`spelingCorrectionTarget\", Rule[SpellingCorrection, True]]"
    );
    assert!(unsupported.messages.iter().any(|message| {
        message.symbol == "Names"
            && message.tag == "optval"
            && message.text.contains("SpellingCorrection")
            && message.text.contains("True")
    }));

    session.eval_text("Unprotect[Names]").unwrap();
    session
        .eval_text("SetOptions[Names, SpellingCorrection -> True]")
        .unwrap();
    let configured = session
        .eval_text("Names[\"Global`spelingCorrectionTarget\"]")
        .unwrap();
    assert_eq!(
        configured.value.to_string(),
        "Names[\"Global`spelingCorrectionTarget\"]"
    );
    assert!(
        configured
            .messages
            .iter()
            .any(|message| message.symbol == "Names" && message.tag == "optval")
    );
}

#[test]
fn names_rejects_unsupported_resolve_context_alias_values() {
    let mut session = Session::new();

    session.eval_text("Audit`aliasTarget = 1").unwrap();
    for input in [
        "Names[\"Audit`*\"]",
        "Names[\"Audit`*\", ResolveContextAliases -> True]",
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(
            outcome.value.to_string(),
            "List[\"Audit`aliasTarget\"]",
            "{input}"
        );
        assert!(outcome.messages.is_empty(), "{input}");
    }

    let unsupported = session
        .eval_text("Names[\"Audit`*\", ResolveContextAliases -> False]")
        .unwrap();
    assert_eq!(
        unsupported.value.to_string(),
        "Names[\"Audit`*\", Rule[ResolveContextAliases, False]]"
    );
    assert!(unsupported.messages.iter().any(|message| {
        message.symbol == "Names"
            && message.tag == "optval"
            && message.text.contains("ResolveContextAliases")
            && message.text.contains("False")
    }));

    session.eval_text("Unprotect[Names]").unwrap();
    session
        .eval_text("SetOptions[Names, ResolveContextAliases -> False]")
        .unwrap();
    let configured = session.eval_text("Names[\"Audit`*\"]").unwrap();
    assert_eq!(configured.value.to_string(), "Names[\"Audit`*\"]");
    assert!(
        configured
            .messages
            .iter()
            .any(|message| message.symbol == "Names" && message.tag == "optval")
    );
}

#[test]
fn symbol_name_and_name_q_core_anchors() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Attributes[SymbolName]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );
    assert_eq!(
        session
            .eval_text("Attributes[NameQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Protected]"
    );

    assert_eq!(
        session
            .eval_text("SymbolName[Global`foo]")
            .unwrap()
            .value
            .to_string(),
        "\"foo\""
    );
    assert_eq!(
        session
            .eval_text("SymbolName[x]")
            .unwrap()
            .value
            .to_string(),
        "\"x\""
    );
    assert_eq!(
        session
            .eval_text("SymbolName[1 + 1]")
            .unwrap()
            .value
            .to_string(),
        "SymbolName[2]"
    );

    session.eval_text("ExactNameQ`present = 1").unwrap();
    assert_eq!(
        session
            .eval_text("NameQ[\"Plus\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("NameQ[\"ExactNameQ`present\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("NameQ[\"ExactNameQ`missing\"]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("NameQ[\"ZZQ12\"]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session.eval_text("NameQ[Plus]").unwrap().value.to_string(),
        "NameQ[Plus]"
    );
}
