use super::*;

#[test]
fn head_length_and_depth_cover_atoms_calls_and_compound_heads() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("FullForm[1 + 2]")
            .unwrap()
            .value
            .to_string(),
        "\"Plus[1, 2]\""
    );
    assert_eq!(
        session.eval_text("Head[1 + 2]").unwrap().value.to_string(),
        "Integer"
    );
    assert_eq!(
        session
            .eval_text("Head[f[a], Hold]")
            .unwrap()
            .value
            .to_string(),
        "Hold[f]"
    );
    assert_eq!(
        session.eval_text("Length[5]").unwrap().value.to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Length[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session.eval_text("Depth[5]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[a, g[b]]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn first_last_rest_and_most_support_defaults_and_empty_calls() {
    let mut session = Session::new();

    session.eval_text("n = 1").unwrap();
    assert_eq!(
        session
            .eval_text("First[f[a], n = 2]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(session.eval_text("n").unwrap().value.to_string(), "1");
    assert_eq!(
        session
            .eval_text("First[f[], n = 3]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(session.eval_text("n").unwrap().value.to_string(), "3");
    assert_eq!(
        session
            .eval_text("Last[f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session.eval_text("Last[5, x]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Rest[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session
            .eval_text("Most[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session.eval_text("Rest[f[]]").unwrap().value.to_string(),
        "f[]"
    );
    assert_eq!(
        session.eval_text("Most[f[]]").unwrap().value.to_string(),
        "f[]"
    );
}

#[test]
fn first_last_rest_and_most_emit_expected_messages_for_invalid_inputs() {
    let mut session = Session::new();

    let outcome = session.eval_text("First[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "First[f[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "First" && message.tag == "first")
    );

    let outcome = session.eval_text("Last[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Last[f[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Last" && message.tag == "nolast")
    );

    let outcome = session.eval_text("Rest[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "Rest[5]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "normal")
    );

    let outcome = session.eval_text("Most[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "Most[5]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "normal")
    );
}

#[test]
fn append_prepend_symbol_and_construct_cover_current_surface_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Append[f[a, b], x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, x]"
    );
    assert_eq!(
        session
            .eval_text("Prepend[f[a, b], x]")
            .unwrap()
            .value
            .to_string(),
        "f[x, a, b]"
    );
    assert_eq!(
        session
            .eval_text("Append[x][f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, x]"
    );
    assert_eq!(
        session
            .eval_text("Prepend[x][f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "f[x, a, b]"
    );
    assert_eq!(
        session
            .eval_text("Symbol[\"x\"]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Symbol[\"List\"]")
            .unwrap()
            .value
            .to_string(),
        "List"
    );
    assert_eq!(
        session
            .eval_text("Construct[f, a, b]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Construct[Plus, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}
