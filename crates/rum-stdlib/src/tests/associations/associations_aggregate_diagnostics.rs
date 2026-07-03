use super::*;

#[test]
fn association_aggregate_diagnostics_cover_invalid_forms() {
    let mut session = core_session();

    let outcome = session
        .eval_text("Merge[Association[a -> 1], Total]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Merge[Association[Rule[a, 1]], Total]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Merge" && message.tag == "list1")
    );
    assert_eq!(
        session
            .eval_text("KeyExistsQ[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyMemberQ[List[a -> 1], a]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("KeyFreeQ[List[a -> 1], b]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    let outcome = session
        .eval_text("Catenate[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Catenate[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Catenate" && message.tag == "invrp")
    );
    let outcome = session.eval_text("Catenate[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Catenate[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "argx"
            && message.text == "Catenate called with 0 arguments; 1 argument is expected."
    }));
    let outcome = session.eval_text("Catenate[{}, {}]").unwrap();
    assert_eq!(outcome.value.to_string(), "Catenate[List[], List[]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "argx"
            && message.text == "Catenate called with 2 arguments; 1 argument is expected."
    }));
    let outcome = session.eval_text("Counts[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Counts[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Counts"
            && message.tag == "argt"
            && message.text == "Counts called with 0 arguments; 1 or 2 arguments are expected."
    }));
    let outcome = session.eval_text("Counts[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "Counts[a]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Counts"
            && message.tag == "invrp"
            && message.text == "The argument a is not a valid Association or a list."
    }));
    let outcome = session.eval_text("Counts[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "Counts[a, b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Counts"
            && message.tag == "invrp"
            && message.text == "The argument a is not a valid Association or a list."
    }));
    let outcome = session.eval_text("Counts[{a}, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "Counts[List[a], f]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Counts"
            && message.tag == "invl"
            && message.text == "The argument f is not a list."
    }));
    let outcome = session.eval_text("Counts[a, b, c]").unwrap();
    assert_eq!(outcome.value.to_string(), "Counts[a, b, c]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Counts"
            && message.tag == "argt"
            && message.text == "Counts called with 3 arguments; 1 or 2 arguments are expected."
    }));
    let outcome = session.eval_text("GroupBy[a, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "GroupBy[a, f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "GroupBy" && message.tag == "list1")
    );
    let outcome = session.eval_text("PositionIndex[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "PositionIndex[a]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "PositionIndex"
            && message.tag == "invrp"
            && message.text == "The argument a is not a valid Association or a list."
    }));
    let outcome = session.eval_text("PositionIndex[]").unwrap();
    assert_eq!(outcome.value.to_string(), "PositionIndex[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "PositionIndex"
            && message.tag == "argx"
            && message.text == "PositionIndex called with 0 arguments; 1 argument is expected."
    }));
    let outcome = session.eval_text("PositionIndex[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "PositionIndex[a, b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "PositionIndex"
            && message.tag == "argx"
            && message.text == "PositionIndex called with 2 arguments; 1 argument is expected."
    }));
}
