use super::*;

#[test]
fn association_lookup_keys_values_diagnostics_cover_invalid_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Association[a]")
            .unwrap()
            .value
            .to_string(),
        "Association[a]"
    );
    let outcome = session.eval_text("AssociationQ[]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationQ[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "AssociationQ"
            && message.tag == "argx"
            && message.text == "AssociationQ called with 0 arguments; 1 argument is expected."
    }));
    let outcome = session.eval_text("AssociationQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationQ[a, b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "AssociationQ"
            && message.tag == "argx"
            && message.text == "AssociationQ called with 2 arguments; 1 argument is expected."
    }));
    let outcome = session.eval_text("Lookup[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "Lookup[a, b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Lookup" && message.tag == "invrl")
    );
    let outcome = session.eval_text("Lookup[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Lookup[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Lookup"
            && message.tag == "argb"
            && message.text
                == "Lookup called with 0 arguments; between 2 and 4 arguments are expected."
    }));
    let outcome = session.eval_text("Lookup[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, Lookup[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());
    let outcome = session.eval_text("Lookup[a, b, c, d, e]").unwrap();
    assert_eq!(outcome.value.to_string(), "Lookup[a, b, c, d, e]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Lookup"
            && message.tag == "argb"
            && message.text
                == "Lookup called with 5 arguments; between 2 and 4 arguments are expected."
    }));
    let outcome = session.eval_text("Lookup[{a -> 1, {b -> 2}}, a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Lookup[List[Rule[a, 1], List[Rule[b, 2]]], a]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Lookup" && message.tag == "invrl")
    );
    assert_eq!(
        session
            .eval_text("Keys[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
    assert_eq!(
        session
            .eval_text("Values[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    let outcome = session.eval_text("Keys[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "Keys[a]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Keys"
            && message.tag == "invrl"
            && message.text == "The argument a is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("Values[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "Values[a]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Values"
            && message.tag == "invrl"
            && message.text == "The argument a is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("Keys[{Association[a -> 1], x}]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Keys[List[Association[Rule[a, 1]], x]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Keys"
            && message.tag == "invrl"
            && message.text == "The argument x is not a valid Association or a list of rules."
    }));
    let outcome = session
        .eval_text("Values[{Association[a -> 1], x}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Values[List[Association[Rule[a, 1]], x]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Values"
            && message.tag == "invrl"
            && message.text
                == "The argument List[Association[Rule[a, 1]], x] is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("Keys[{a -> 1, b}]").unwrap();
    assert_eq!(outcome.value.to_string(), "Keys[List[Rule[a, 1], b]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Keys"
            && message.tag == "invrl"
            && message.text == "The argument b is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("Values[{a -> 1, b}]").unwrap();
    assert_eq!(outcome.value.to_string(), "Values[List[Rule[a, 1], b]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Values"
            && message.tag == "invrl"
            && message.text
                == "The argument List[Rule[a, 1], b] is not a valid Association or a list of rules."
    }));
}
