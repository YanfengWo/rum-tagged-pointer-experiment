use super::*;

#[test]
fn association_mutation_operator_forms_apply_to_raw_held_targets() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "a = <|\"k\" -> 1|>; at = {AssociateTo[\"j\" -> 2][a], a}; \
             a2 = <|\"k\" -> 1, \"j\" -> 2|>; kd = {KeyDropFrom[\"k\"][a2], a2}; \
             {at, kd}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[Association[Rule[\"k\", 1], Rule[\"j\", 2]], \
         Association[Rule[\"k\", 1], Rule[\"j\", 2]]], \
         List[Association[Rule[\"j\", 2]], Association[Rule[\"j\", 2]]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn association_mutation_operator_forms_report_application_arity() {
    let mut session = core_session();

    for (source, expected, symbol, head, count) in [
        (
            "AssociateTo[a -> 1][]",
            "AssociateTo[Rule[a, 1]][]",
            "AssociateTo",
            "AssociateTo[Rule[a, 1]]",
            0,
        ),
        (
            "KeyDropFrom[k][a, b]",
            "KeyDropFrom[k][a, b]",
            "KeyDropFrom",
            "KeyDropFrom[k]",
            2,
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected);
        assert!(outcome.messages.iter().any(|message| {
            message.symbol == symbol
                && message.tag == "argx"
                && message.text
                    == format!("{head} called with {count} arguments; 1 argument is expected.")
        }));
    }
}

#[test]
fn association_mutation_heads_advertise_subvalues_hold_all() {
    let mut session = core_session();

    let outcome = session
        .eval_text("{Attributes[AssociateTo], Attributes[KeyDropFrom]}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[HoldFirst, SubValuesHoldAll, Protected], \
         List[HoldFirst, SubValuesHoldAll, Protected]]"
    );
    assert!(outcome.messages.is_empty());
}
