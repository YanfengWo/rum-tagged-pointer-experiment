use super::*;

#[test]
fn association_mutation_diagnostics_cover_invalid_targets_and_arity() {
    let mut session = core_session();

    let outcome = session
        .eval_text("AssociateTo[Association[a -> 1], a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociateTo[Association[Rule[a, 1]], a]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invlb")
    );
    let outcome = session.eval_text("AssociateTo[a -> 1, b -> 2]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociateTo[Rule[a, 1], Rule[b, 2]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let outcome = session.eval_text("AssociateTo[a :> 1, b -> 2]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociateTo[RuleDelayed[a, 1], Rule[b, 2]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let outcome = session
        .eval_text("AssociateTo[Association[a], b -> 2]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociateTo[Association[a], Rule[b, 2]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let outcome = session.eval_text("AssociateTo[f[1], b -> 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociateTo[f[1], Rule[b, 2]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invak")
    );
    let outcome = session.eval_text("AssociateTo[1, b -> 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociateTo[1, Rule[b, 2]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "rvalue")
    );
    let outcome = session.eval_text("AssociateTo[1 + 2, b -> 2]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "AssociateTo[Plus[1, 2], Rule[b, 2]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let outcome = session.eval_text("KeyDropFrom[a -> 1, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDropFrom[Rule[a, 1], a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    let outcome = session.eval_text("KeyDropFrom[a :> 1, a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyDropFrom[RuleDelayed[a, 1], a]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    let outcome = session.eval_text("KeyDropFrom[Association[a], b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDropFrom[Association[a], b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    let outcome = session.eval_text("KeyDropFrom[f[1], a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDropFrom[f[1], a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    let outcome = session.eval_text("KeyDropFrom[1, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDropFrom[1, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "rvalue")
    );
    let outcome = session.eval_text("KeyDropFrom[1 + 2, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyDropFrom[Plus[1, 2], a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    for (symbol, zero_arg_call, one_arg_call, three_arg_call) in [
        (
            "AssociateTo",
            "AssociateTo[]",
            "AssociateTo[a]",
            "AssociateTo[a, b, c]",
        ),
        (
            "KeyDropFrom",
            "KeyDropFrom[]",
            "KeyDropFrom[a]",
            "KeyDropFrom[a, b, c]",
        ),
    ] {
        let outcome = session.eval_text(zero_arg_call).unwrap();
        assert_eq!(outcome.value.to_string(), zero_arg_call);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "argrx"
                && message.text
                    == format!("{symbol} called with 0 arguments; 2 arguments are expected.")
        }));

        let outcome = session.eval_text(one_arg_call).unwrap();
        assert_eq!(outcome.value.to_string(), one_arg_call);
        assert!(outcome.messages.is_empty());

        let outcome = session.eval_text(three_arg_call).unwrap();
        assert_eq!(outcome.value.to_string(), three_arg_call);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "argrx"
                && message.text
                    == format!("{symbol} called with 3 arguments; 2 arguments are expected.")
        }));
    }
}
