use super::*;

#[test]
fn join_concatenates_top_level_same_head_expressions() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("Join[]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("Join[{a, b}]").unwrap().value.to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Join[{a, b}, {c, d}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("Join[f[a, b], f[c]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Join[{}, {a}]")
            .unwrap()
            .value
            .to_string(),
        "List[a]"
    );
    assert_eq!(
        session
            .eval_text("Join[f[], f[a]]")
            .unwrap()
            .value
            .to_string(),
        "f[a]"
    );
}

#[test]
fn join_concatenates_associations_with_later_keys_winning() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Join[Association[a -> 1], Association[b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Join[Association[a -> 1], Association[a -> 2, b :> 3]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], RuleDelayed[b, 3]]"
    );
}

#[test]
fn join_supports_current_level_argument_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Join[{{a}, {b}}, {{c}, {d}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, c], List[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Join[{{a}}, {{b}, {c}}, {{d}, {e}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b, d], List[c, e]]"
    );
    assert_eq!(
        session
            .eval_text("Join[f[g[a]], f[g[b], g[c]], f[g[d], g[e]], 2]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, b, d], g[c, e]]"
    );
    assert_eq!(
        session
            .eval_text("Join[f[g[h[a]]], f[g[h[b]], g[h[c]]], 3]")
            .unwrap()
            .value
            .to_string(),
        "f[g[h[a, b]], g[h[c]]]"
    );
    assert_eq!(
        session
            .eval_text("Join[{Association[a -> 1]}, {Association[a -> 2, b :> 3]}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 2], RuleDelayed[b, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("Join[{{a}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a]]"
    );
}

#[test]
fn join_reports_head_mismatches_and_leaves_atomic_inputs_inert() {
    let mut session = core_session();

    let single_atom = session.eval_text("Join[a]").unwrap();
    assert_eq!(single_atom.value.to_string(), "Join[a]");
    assert!(
        single_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "normal")
    );

    let mismatched = session.eval_text("Join[f[a], g[b]]").unwrap();
    assert_eq!(mismatched.value.to_string(), "Join[f[a], g[b]]");
    assert!(
        mismatched
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "heads")
    );

    let list_and_call = session.eval_text("Join[{a}, f[b]]").unwrap();
    assert_eq!(list_and_call.value.to_string(), "Join[List[a], f[b]]");
    assert!(
        list_and_call
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "heads")
    );

    let atomic = session.eval_text("Join[a, {b}]").unwrap();
    assert_eq!(atomic.value.to_string(), "Join[a, List[b]]");
    assert!(atomic.messages.is_empty());

    let atom_before_association = session
        .eval_text("Join[f[a], b, Association[c -> 1]]")
        .unwrap();
    assert_eq!(
        atom_before_association.value.to_string(),
        "Join[f[a], b, Association[Rule[c, 1]]]"
    );
    assert!(atom_before_association.messages.is_empty());
}

#[test]
fn join_reports_current_level_argument_boundaries() {
    let mut session = core_session();

    let incompatible = session.eval_text("Join[Association[a -> 1], {b}]").unwrap();
    assert_eq!(
        incompatible.value.to_string(),
        "Join[Association[Rule[a, 1]], List[b]]"
    );
    assert!(
        incompatible
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "incpt")
    );

    let shallow_first = session.eval_text("Join[{{a}}, {{b}}, 3]").unwrap();
    assert_eq!(
        shallow_first.value.to_string(),
        "Join[List[List[a]], List[List[b]], 3]"
    );
    assert!(
        shallow_first
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "normal1")
    );

    let heterogeneous_first = session
        .eval_text("Join[f[g[a], h[b]], f[g[c], h[d]], 2]")
        .unwrap();
    assert_eq!(
        heterogeneous_first.value.to_string(),
        "Join[f[g[a], h[b]], f[g[c], h[d]], 2]"
    );
    assert!(
        heterogeneous_first
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "normal1")
    );

    let mismatched_deep_head = session.eval_text("Join[f[g[a]], f[h[b]], 2]").unwrap();
    assert_eq!(
        mismatched_deep_head.value.to_string(),
        "Join[f[g[a]], f[h[b]], 2]"
    );
    assert!(
        mismatched_deep_head
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "headsd")
    );

    let top_level_association = session
        .eval_text("Join[Association[a -> 1], Association[b -> 2], 2]")
        .unwrap();
    assert_eq!(
        top_level_association.value.to_string(),
        "Join[Association[Rule[a, 1]], Association[Rule[b, 2]], 2]"
    );
    assert!(
        top_level_association
            .messages
            .iter()
            .any(|message| message.symbol == "Join" && message.tag == "normal1")
    );
}
