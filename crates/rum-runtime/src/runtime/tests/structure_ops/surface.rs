use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn power_threads_over_lists_without_disturbing_canonicalization() {
    let mut session = Session::new();

    let cases = [
        ("Power[{2,3},2]", "List[4, 9]"),
        ("Power[{2,3},{3,2}]", "List[8, 9]"),
        ("Power[{a,b},2]", "List[Power[a, 2], Power[b, 2]]"),
        ("Power[2,{2,3}]", "List[4, 8]"),
        ("Power[{2,3},0]", "List[1, 1]"),
        ("Power[{1,2,3},-1]", "List[1, 1/2, 1/3]"),
        ("Power[2,3]", "8"),
        ("Power[5,1]", "5"),
        ("Power[7,0]", "1"),
        ("Power[1,9]", "1"),
        ("Power[a,2]", "Power[a, 2]"),
        ("2^10", "1024"),
        ("x^2 x^3", "Power[x, 5]"),
        ("(2 x)^2", "Times[4, Power[x, 2]]"),
        ("x^2/x", "x"),
        ("Power[0,0]", "Power[0, 0]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
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
            .any(|message| message.symbol == "First" && message.tag == "nofirst")
    );

    let outcome = session.eval_text("Last[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Last[f[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Last" && message.tag == "nolast")
    );

    let outcome = session.eval_text("First[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "First[5]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "First" && message.tag == "normal")
    );

    let outcome = session.eval_text("Rest[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "Rest[5]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Rest" && message.tag == "normal")
    );

    let outcome = session.eval_text("Most[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "Most[5]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Most" && message.tag == "normal")
    );

    let outcome = session.eval_text("Rest[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Rest[f[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Rest" && message.tag == "norest")
    );

    let outcome = session.eval_text("Most[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Most[f[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Most" && message.tag == "nomost")
    );

    let outcome = session.eval_text("Rest[Association[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Rest[Association[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Rest" && message.tag == "norest")
    );

    let outcome = session.eval_text("Most[Association[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Most[Association[]]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Most" && message.tag == "nomost")
    );
}

#[test]
fn flatten_covers_current_same_head_level_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Flatten[{a, {b, c}, {{d}}}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{a, {b, c}, {{d}}}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c, List[d]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{a, {b, c}, {{d}}}, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[a, List[b, c], List[List[d]]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{a, {b, c}, {{d}}}, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[a, f[b, c]], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{a, f[{b}], {c}}, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[a, f[List[b]], c]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[{a, b}, {c}], Infinity, List]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{f[a, b], f[c]}, Infinity, f]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[g[a, g[b]]], 1, g]")
            .unwrap()
            .value
            .to_string(),
        "f[a, g[b]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[g[a, g[b]]], 2, g]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[h[g[a, b]], g[c]], Infinity, g]")
            .unwrap()
            .value
            .to_string(),
        "f[h[g[a, b]], c]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{a, b}, {c, d}}, {{2}, {1}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, c], List[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{a, b}, {c, d}}, {2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, c, b, d]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{a}, {b, c}}, {{2}, {1}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[c]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {{1, 2}, {3}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[c, d], List[e, f], List[g, h]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {{1, 3}, {2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, c], List[b, d], List[e, g], List[f, h]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[f[a, b], f[c, d]], {{2}, {1}}]")
            .unwrap()
            .value
            .to_string(),
        "f[f[a, c], f[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[f[f[a, b], f[c, d]], {{1, 2}}, f]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{a, b}}, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{}}, {{2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Flatten[{{}}, {{1}, {2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
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
    let invalid_symbol_value = session.eval_text("Symbol[1]").unwrap();
    assert_eq!(invalid_symbol_value.value.to_string(), "Symbol[1]");
    assert!(
        invalid_symbol_value
            .messages
            .iter()
            .any(|message| message.symbol == "Symbol"
                && message.tag == "string"
                && message.text == "String expected at position 1 in Symbol[1].")
    );
    let invalid_symbol_arity = session.eval_text("Symbol[]").unwrap();
    assert_eq!(invalid_symbol_arity.value.to_string(), "Symbol[]");
    assert!(
        invalid_symbol_arity
            .messages
            .iter()
            .any(|message| message.symbol == "Symbol"
                && message.tag == "argx"
                && message.text == "Symbol called with 0 arguments; 1 argument is expected.")
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
