use super::*;

#[test]
fn key_value_pattern_matches_association_and_rule_inputs() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyValuePattern[a -> _]")
            .unwrap()
            .value
            .to_string(),
        "KeyValuePattern[List[Rule[a, Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyValuePattern[a :> _]")
            .unwrap()
            .value
            .to_string(),
        "KeyValuePattern[List[RuleDelayed[a, Blank[]]]]"
    );
    let outcome = session.eval_text("KeyValuePattern[]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyValuePattern[]");
    assert!(outcome.messages.is_empty());
    let outcome = session.eval_text("KeyValuePattern[a, b]").unwrap();
    assert_eq!(outcome.value.to_string(), "KeyValuePattern[a, b]");
    assert!(outcome.messages.is_empty());
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1, b -> 2], KeyValuePattern[a -> _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1, b -> 2], KeyValuePattern[{b -> 2, a -> _}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[{a -> 1}, KeyValuePattern[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[{a -> 1, b -> 2}, KeyValuePattern[{b -> 2, a -> 1}]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[{a -> 1, a -> 2}, KeyValuePattern[a -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a :> 1], KeyValuePattern[a :> _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[{a :> 1}, KeyValuePattern[a :> _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1], KeyValuePattern[a :> _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a :> 1], KeyValuePattern[a -> _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[a -> 1, KeyValuePattern[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[Association[a -> 1], KeyValuePattern[b -> _]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}
