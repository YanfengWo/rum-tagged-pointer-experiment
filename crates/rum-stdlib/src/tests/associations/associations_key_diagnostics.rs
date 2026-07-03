use super::*;

#[test]
fn key_transform_diagnostics_cover_invalid_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyTake[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("KeyTake[{}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[List[a -> 1, b -> 2], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("KeyDrop[{}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    let outcome = session.eval_text("KeyTake[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyTake[a, b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyTake"
            && message.tag == "invrl"
            && message.text == "The argument a is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("KeyDrop[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDrop[a, b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyDrop"
            && message.tag == "invrl"
            && message.text == "The argument a is not a valid Association or a list of rules."
    }));
    let outcome = session
        .eval_text("KeyTake[{Association[a -> 1], x}, a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyTake[List[Association[Rule[a, 1]], x], a]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyTake"
            && message.tag == "invru"
            && message.text
                == "The argument List[Association[Rule[a, 1]], x] is not a valid Association or a list of rules."
    }));
    let outcome = session
        .eval_text("KeyDrop[{Association[a -> 1], x}, a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyDrop[List[Association[Rule[a, 1]], x], a]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyDrop"
            && message.tag == "invru"
            && message.text == "The argument x is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("KeyTake[{a -> 1, b}, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyTake[List[Rule[a, 1], b], a]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyTake"
            && message.tag == "invru"
            && message.text
                == "The argument List[Rule[a, 1], b] is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("KeyDrop[{a -> 1, b}, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDrop[List[Rule[a, 1], b], a]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyDrop"
            && message.tag == "invru"
            && message.text == "The argument b is not a valid Association or a list of rules."
    }));
    let outcome = session.eval_text("KeyMap[f, List[a -> 1]]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMap[f, List[Rule[a, 1]]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMap" && message.tag == "invak")
    );
    let outcome = session.eval_text("KeyValueMap[f, List[a -> 1]]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyValueMap[f, List[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyValueMap" && message.tag == "invak")
    );
    assert_eq!(
        session
            .eval_text("KeySelect[List[a -> 1], TrueQ]")
            .unwrap()
            .value
            .to_string(),
        "Association[]"
    );
    assert_eq!(
        session
            .eval_text("KeySort[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    let outcome = session.eval_text("AssociationThread[{a}, {1, 2}]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociationThread[List[a], List[1, 2]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationThread" && message.tag == "idim")
    );
    let outcome = session.eval_text("AssociationMap[f, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationMap[f, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationMap" && message.tag == "invrp")
    );
    assert_eq!(
        session
            .eval_text("AssociationMap[f, List[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[Rule[a, 1], f[Rule[a, 1]]], Rule[Rule[b, 2], f[Rule[b, 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyUnion[Association[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "KeyUnion[Association[Rule[a, 1]], a]"
    );
    assert_eq!(
        session
            .eval_text("Normal[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 1]]"
    );
    assert_eq!(
        session.eval_text("Normal[a]").unwrap().value.to_string(),
        "a"
    );
    let outcome = session.eval_text("Normal[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Normal[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Normal"
            && message.tag == "argb"
            && message.text
                == "Normal called with 0 arguments; between 1 and 3 arguments are expected."
    }));
    let outcome = session
        .eval_text("Normal[Association[a -> 1], Association, Heads -> True]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Normal[Association[Rule[a, 1]], Association, Rule[Heads, True]]"
    );
    assert!(outcome.messages.is_empty());
    let outcome = session.eval_text("Normal[a, b, c, d]").unwrap();
    assert_eq!(outcome.value.to_string(), "Normal[a, b, c, d]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Normal"
            && message.tag == "argb"
            && message.text
                == "Normal called with 4 arguments; between 1 and 3 arguments are expected."
    }));
}
