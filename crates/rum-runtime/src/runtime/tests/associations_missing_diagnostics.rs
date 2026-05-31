use super::*;

#[test]
fn missing_and_delete_missing_diagnostics_cover_invalid_forms() {
    let mut session = Session::new();

    let outcome = session.eval_text("Missing[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Missing[]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("Missing[a, b, c]").unwrap();
    assert_eq!(outcome.value.to_string(), "Missing[a, b, c]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("DeleteMissing[x]").unwrap();
    assert_eq!(outcome.value.to_string(), "DeleteMissing[x]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "DeleteMissing" && message.tag == "invrp")
    );

    let outcome = session.eval_text("DeleteMissing[{Missing[]}, 0]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "DeleteMissing[List[Missing[]], 0]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "DeleteMissing"
            && message.tag == "arg2"
            && message.text
                == "The second argument 0 is expected to be a positive integer or Infinity."
    }));

    let outcome = session
        .eval_text("DeleteMissing[{Missing[]}, 1, a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "DeleteMissing[List[Missing[]], 1, a]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "DeleteMissing"
            && message.tag == "arg3"
            && message.text
                == "The third argument a is expected to be a non-negative integer or Infinity."
    }));

    let outcome = session
        .eval_text("DeleteMissing[{Missing[]}, 1, 2, 3]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "DeleteMissing[List[Missing[]], 1, 2, 3]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "DeleteMissing"
            && message.tag == "argb"
            && message.text
                == "DeleteMissing called with 4 arguments; between 1 and 3 arguments are expected."
    }));

    let outcome = session.eval_text("CountsBy[a, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "CountsBy[a, f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "CountsBy" && message.tag == "list")
    );

    let outcome = session.eval_text("CountsBy[{a}, f, g]").unwrap();
    assert_eq!(outcome.value.to_string(), "CountsBy[List[a], f, g]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "CountsBy" && message.tag == "argt")
    );

    let outcome = session.eval_text("Merge[{a}, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "Merge[List[a], f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Merge" && message.tag == "list1")
    );
    let outcome = session.eval_text("Merge[{{a}}, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "Merge[List[List[a]], f]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Merge"
            && message.tag == "list1"
            && message.text
                == "The argument a is not a valid list of Associations or rules or lists of rules."
    }));

    let outcome = session
        .eval_text("Merge[{Association[a -> 1]}, f, g]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Merge[List[Association[Rule[a, 1]]], f, g]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Merge" && message.tag == "argt")
    );

    let outcome = session.eval_text("GroupBy[{a}, f, g, h]").unwrap();
    assert_eq!(outcome.value.to_string(), "GroupBy[List[a], f, g, h]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "GroupBy" && message.tag == "argb")
    );
}
