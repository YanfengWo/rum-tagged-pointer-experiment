use super::*;

#[test]
fn key_function_arity_and_currying_diagnostics_cover_invalid_forms() {
    let mut session = core_session();

    let outcome = session.eval_text("Key[]").unwrap();
    assert_eq!(outcome.value.to_string(), "Key[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Key"
            && message.tag == "argx"
            && message.text == "Key called with 0 arguments; 1 argument is expected."
    }));
    let outcome = session.eval_text("Key[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "Key[a, b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Key"
            && message.tag == "argx"
            && message.text == "Key called with 2 arguments; 1 argument is expected."
    }));

    let outcome = session.eval_text("KeyExistsQ[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyExistsQ[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyExistsQ"
            && message.tag == "argt"
            && message.text == "KeyExistsQ called with 0 arguments; 1 or 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyExistsQ[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyExistsQ[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyExistsQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "False");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyExistsQ" && message.tag == "invrl")
    );

    let outcome = session.eval_text("KeyExistsQ[a, b, c]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyExistsQ[a, b, c]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyExistsQ"
            && message.tag == "argt"
            && message.text == "KeyExistsQ called with 3 arguments; 1 or 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyMemberQ[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMemberQ[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMemberQ"
            && message.tag == "argrx"
            && message.text == "KeyMemberQ called with 0 arguments; 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyMemberQ[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyMemberQ[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyMemberQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMemberQ[a, b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMemberQ" && message.tag == "invas")
    );
    let outcome = session.eval_text("KeyMemberQ[{a -> 1, x}, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "True");
    assert!(outcome.messages.is_empty());
    let outcome = session.eval_text("KeyMemberQ[{x, a -> 1}, a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyMemberQ[List[x, Rule[a, 1]], a]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMemberQ"
            && message.tag == "invrk"
            && message.text == "The argument x is not a valid rule."
    }));
    let outcome = session.eval_text("KeyMemberQ[{a -> 1, x}, z]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyMemberQ[List[Rule[a, 1], x], z]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMemberQ"
            && message.tag == "invrk"
            && message.text == "The argument x is not a valid rule."
    }));
    let outcome = session
        .eval_text("KeyMemberQ[{Association[a -> 1]}, a]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyMemberQ[List[Association[Rule[a, 1]]], a]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMemberQ"
            && message.tag == "invrk"
            && message.text == "The argument Association[Rule[a, 1]] is not a valid rule."
    }));

    let outcome = session.eval_text("KeyMemberQ[a, b, c]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMemberQ[a, b, c]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMemberQ"
            && message.tag == "argrx"
            && message.text == "KeyMemberQ called with 3 arguments; 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyFreeQ[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyFreeQ[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyFreeQ"
            && message.tag == "argrx"
            && message.text == "KeyFreeQ called with 0 arguments; 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyFreeQ[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyFreeQ[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyFreeQ[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyFreeQ[a, b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyFreeQ" && message.tag == "invas")
    );
    let outcome = session.eval_text("KeyFreeQ[{a -> 1, x}, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "False");
    assert!(outcome.messages.is_empty());
    let outcome = session.eval_text("KeyFreeQ[{x, a -> 1}, a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyFreeQ[List[x, Rule[a, 1]], a]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyFreeQ"
            && message.tag == "invrk"
            && message.text == "The argument x is not a valid rule."
    }));
    let outcome = session.eval_text("KeyFreeQ[{a -> 1, x}, z]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyFreeQ[List[Rule[a, 1], x], z]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyFreeQ"
            && message.tag == "invrk"
            && message.text == "The argument x is not a valid rule."
    }));

    let outcome = session.eval_text("KeyFreeQ[a, b, c]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyFreeQ[a, b, c]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyFreeQ"
            && message.tag == "argrx"
            && message.text == "KeyFreeQ called with 3 arguments; 2 arguments are expected."
    }));

    for (symbol, zero_arg_call, three_arg_call) in [
        ("Keys", "Keys[]", "Keys[Association[], x, y]"),
        ("Values", "Values[]", "Values[Association[], x, y]"),
        ("KeyTake", "KeyTake[]", "KeyTake[Association[], a, b]"),
        ("KeyDrop", "KeyDrop[]", "KeyDrop[Association[], a, b]"),
        (
            "KeySelect",
            "KeySelect[]",
            "KeySelect[Association[], TrueQ, x]",
        ),
        ("KeySort", "KeySort[]", "KeySort[Association[], Less, x]"),
    ] {
        let outcome = session.eval_text(zero_arg_call).unwrap();
        assert_eq!(outcome.value.to_string(), zero_arg_call);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "argt"
                && message.text
                    == format!("{symbol} called with 0 arguments; 1 or 2 arguments are expected.")
        }));

        let outcome = session.eval_text(three_arg_call).unwrap();
        assert_eq!(outcome.value.to_string(), three_arg_call);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "argt"
                && message.text
                    == format!("{symbol} called with 3 arguments; 1 or 2 arguments are expected.")
        }));
    }

    let outcome = session.eval_text("KeySortBy[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySortBy[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeySortBy"
            && message.tag == "argt"
            && message.text
                == "KeySortBy called with 0 arguments; 1, 2, or 3 arguments are expected."
    }));

    let outcome = session
        .eval_text("KeySortBy[Association[], f, Less, x]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeySortBy[Association[], f, Less, x]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeySortBy"
            && message.tag == "argt"
            && message.text
                == "KeySortBy called with 4 arguments; 1, 2, or 3 arguments are expected."
    }));

    let outcome = session.eval_text("KeyMap[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyMap[f, rum$keyedArg]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyMap[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMap[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMap"
            && message.tag == "argt"
            && message.text == "KeyMap called with 0 arguments; 1 or 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyMap[f, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyMap[f, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyMap" && message.tag == "invak")
    );

    let outcome = session
        .eval_text("KeyMap[f, Association[a -> 1], x]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyMap[f, Association[Rule[a, 1]], x]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyMap"
            && message.tag == "argt"
            && message.text == "KeyMap called with 3 arguments; 1 or 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyValueMap[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeyValueMap[f, rum$keyedArg]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyValueMap[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyValueMap[]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyValueMap"
            && message.tag == "argt"
            && message.text == "KeyValueMap called with 0 arguments; 1 or 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeyValueMap[f, a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyValueMap[f, a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyValueMap" && message.tag == "invak")
    );

    let outcome = session
        .eval_text("KeyValueMap[f, Association[a -> 1], x]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyValueMap[f, Association[Rule[a, 1]], x]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "KeyValueMap"
            && message.tag == "argt"
            && message.text == "KeyValueMap called with 3 arguments; 1 or 2 arguments are expected."
    }));

    let outcome = session.eval_text("KeySelect[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeySelect[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeySelect[a, TrueQ]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySelect[a, TrueQ]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySelect" && message.tag == "invru")
    );

    let outcome = session.eval_text("AssociationMap[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, AssociationMap[f, rum$keyedArg]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("Merge[Total]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, Merge[rum$keyedArg, Total]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("CountsBy[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, CountsBy[rum$keyedArg, f]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("GroupBy[f]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, GroupBy[rum$keyedArg, f]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("AssociationMap[]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationMap[]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationMap" && message.tag == "argt")
    );

    let outcome = session.eval_text("AssociationMap[f, {a}, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationMap[f, List[a], b]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationMap" && message.tag == "argt")
    );

    let outcome = session.eval_text("KeySortBy[a]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Function[rum$keyedArg, KeySortBy[rum$keyedArg, a]]"
    );
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeySort[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySort[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySort" && message.tag == "invrl")
    );

    let outcome = session.eval_text("KeySortBy[a, f]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeySortBy[a, f]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeySortBy" && message.tag == "invrl")
    );

    let outcome = session.eval_text("AssociationThread[a]").unwrap();
    assert_eq!(outcome.value.to_string(), "AssociationThread[a]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "AssociationThread" && message.tag == "invrk")
    );

    let outcome = session.eval_text("KeyUnion[Association[a -> 1]]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyUnion[Association[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyUnion" && message.tag == "invar")
    );

    let outcome = session
        .eval_text("KeyIntersection[Association[a -> 1]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyIntersection[Association[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "invar")
    );

    let outcome = session
        .eval_text("KeyComplement[Association[a -> 1]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[Association[Rule[a, 1]]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "invar")
    );

    let outcome = session.eval_text("KeyUnion[{}]").unwrap();
    assert_eq!(outcome.value.to_string(), "List[]");
    assert!(outcome.messages.is_empty());

    let outcome = session
        .eval_text("KeyUnion[{Association[a -> 1]}, f, g]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyUnion[List[Association[Rule[a, 1]]], f, g]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyUnion" && message.tag == "argt")
    );

    let outcome = session.eval_text("KeyIntersection[{}]").unwrap();
    assert_eq!(outcome.value.to_string(), "List[]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("KeyIntersection[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyIntersection[]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "argx")
    );

    let outcome = session.eval_text("KeyComplement[{}]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyComplement[List[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "empt")
    );

    let outcome = session
        .eval_text("KeyComplement[{Association[a -> 1]}, b]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[List[Association[Rule[a, 1]]], b]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "nonopt")
    );

    let outcome = session
        .eval_text("KeyComplement[{Association[a -> 1]}, x -> y]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[List[Association[Rule[a, 1]]], Rule[x, y]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "optx")
    );

    let outcome = session
        .eval_text("KeyUnion[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyUnion[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyUnion" && message.tag == "invas")
    );

    let outcome = session
        .eval_text("KeyIntersection[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyIntersection[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyIntersection" && message.tag == "invas")
    );

    let outcome = session
        .eval_text("KeyComplement[{Association[a -> 1], a}]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "KeyComplement[List[Association[Rule[a, 1]], a]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "KeyComplement" && message.tag == "invrl")
    );
}
