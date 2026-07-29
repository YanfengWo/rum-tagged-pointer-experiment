use super::*;

#[test]
fn set_writes_association_function_and_part_keys() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("a = <||>; a[\"=\"] = 1; {a, Lookup[a, \"=\"], a[[\"=\"]]}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[\"=\", 1]], 1, 1]"
    );
    assert_eq!(
        session
            .eval_text("a = Association[x -> 1, y -> 2]; a[x] = 9; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 9], Rule[y, 2]]"
    );
    assert_eq!(
        session
            .eval_text("k = \"x\"; a = <||>; a[k] = 1; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"x\", 1]]"
    );
    assert_eq!(
        session
            .eval_text("k = \"x\"; a = <||>; a[[Key[k]]] = 2; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"x\", 2]]"
    );
    assert_eq!(
        session
            .eval_text("a = Association[x -> Association[y -> 1]]; a[x, y] = 2; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, Association[Rule[y, 2]]]]"
    );
}

#[test]
fn set_association_lvalue_falls_back_or_reports_failed_nested_keys() {
    let mut session = core_session();

    let missing = session.eval_text("a = <||>; a[x, y] = 2").unwrap();
    assert_eq!(missing.value.to_string(), "2");
    assert!(
        missing
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "kval")
    );
    assert_eq!(
        session.eval_text("a").unwrap().value.to_string(),
        "Association[]"
    );

    assert_eq!(
        session
            .eval_text("a = Association[x -> 1]; a[x, y] = 2; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 1]]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[f]; f[1] = old; f[1] = new; {f[1], DownValues[f]}")
            .unwrap()
            .value
            .to_string(),
        "List[new, List[RuleDelayed[HoldPattern[f[1]], new]]]"
    );
}

#[test]
fn set_single_association_keys_preserves_aliases_order_and_rule_heads() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("a = <|1 -> 2|>; b = a; b[3] = 4; {a, b}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, 2]], Association[Rule[1, 2], Rule[3, 4]]]"
    );
    assert_eq!(
        session
            .eval_text("a = <|x -> 1, y -> 2|>; a[y] = 9; a[z] = 3; a[x] = 7; a",)
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 7], Rule[y, 9], Rule[z, 3]]"
    );
    assert_eq!(
        session
            .eval_text("a = Association[x :> delayed, y -> 2]; a[x] = immediate; a",)
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, immediate], Rule[y, 2]]"
    );
    assert_eq!(
        session
            .eval_text("a = <|x -> 1, y -> 2|>; a[[Key[x]]] = 9; a[[Key[z]]] = 3; a",)
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 9], Rule[y, 2], Rule[z, 3]]"
    );
}

#[test]
fn set_single_association_key_keeps_canonicalization_and_invalid_fallbacks() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("a = <|x -> 1, x -> 2|>; a[y] = 3; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 2], Rule[y, 3]]"
    );

    let invalid = session
        .eval_text("a = Association[bad]; a[x] = 2; a")
        .unwrap();
    assert_eq!(invalid.value.to_string(), "Association[bad]");
    assert!(
        invalid
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "wrsym")
    );
}

#[test]
fn set_single_association_key_finishes_rhs_and_selector_evaluation_first() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "a = <|x -> 1|>; rhsSeen = unset; selectorSeen = unset; \
                 a[(selectorSeen = a; x)] = (rhsSeen = a; 2); \
                 {rhsSeen, selectorSeen, a}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[x, 1]], Association[Rule[x, 1]], Association[Rule[x, 2]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "a = <|x -> 1|>; order = {}; \
                 a[(AppendTo[order, selector]; x)] = \
                   (AppendTo[order, rhs]; 2); \
                 {order, a}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[rhs, selector], Association[Rule[x, 2]]]"
    );
}

#[test]
fn set_single_association_key_still_refuses_a_protected_target() {
    let mut session = core_session();
    session.eval_text("a = <|x -> 1|>; Protect[a]").unwrap();

    let outcome = session.eval_text("a[x] = 2").unwrap();

    assert_eq!(outcome.value.to_string(), "2");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "wrsym")
    );
    assert_eq!(
        session.eval_text("a").unwrap().value.to_string(),
        "Association[Rule[x, 1]]"
    );
}
