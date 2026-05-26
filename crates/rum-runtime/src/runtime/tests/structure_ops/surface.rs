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
        session.eval_text("Depth[f[]]").unwrap().value.to_string(),
        "2"
    );
    assert_eq!(
        session.eval_text("Depth[{}]").unwrap().value.to_string(),
        "2"
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
            .eval_text("Depth[Association[f[1] -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[], Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[a -> 1, b -> f[2]]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[Association[a -> Association[b -> 1]], Heads -> True]")
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
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {Heads -> True}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], \"Heads\" -> True]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {Heads -> False, Heads -> True}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[][a], {Heads -> True, Heads -> x}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    let unknown_option = session.eval_text("Depth[f[a], Bad -> 1]").unwrap();
    assert_eq!(
        unknown_option.value.to_string(),
        "Depth[f[a], Rule[Bad, 1]]"
    );
    assert!(
        unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "Depth" && message.tag == "optx")
    );
    let invalid_heads_value = session.eval_text("Depth[f[a], Heads -> x]").unwrap();
    assert_eq!(
        invalid_heads_value.value.to_string(),
        "Depth[f[a], Rule[Heads, x]]"
    );
    assert!(
        invalid_heads_value
            .messages
            .iter()
            .any(|message| message.symbol == "Depth" && message.tag == "opttf")
    );
    let non_option = session.eval_text("Depth[f[a], {1}]").unwrap();
    assert_eq!(non_option.value.to_string(), "Depth[f[a], List[1]]");
    assert!(
        non_option
            .messages
            .iter()
            .any(|message| message.symbol == "Depth" && message.tag == "nonopt")
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
        session
            .eval_text("First[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Last[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Rest[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Most[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {First[Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 1]"
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
fn flatten_reports_current_invalid_boundaries() {
    let mut session = Session::new();

    let atomic = session.eval_text("Flatten[x]").unwrap();
    assert_eq!(atomic.value.to_string(), "Flatten[x]");
    assert!(
        atomic
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "normal")
    );

    let association = session
        .eval_text("Flatten[Association[a -> Association[b -> 1], c -> 2]]")
        .unwrap();
    assert_eq!(
        association.value.to_string(),
        "Flatten[Association[Rule[a, Association[Rule[b, 1]]], Rule[c, 2]]]"
    );
    assert!(
        association
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "normal")
    );

    let invalid_level = session.eval_text("Flatten[{a}, x]").unwrap();
    assert_eq!(invalid_level.value.to_string(), "Flatten[List[a], x]");
    assert!(
        invalid_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flev")
    );

    let invalid_level_with_head = session.eval_text("Flatten[{a}, x, List]").unwrap();
    assert_eq!(
        invalid_level_with_head.value.to_string(),
        "Flatten[List[a], x, List]"
    );
    assert!(
        invalid_level_with_head
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flev")
    );

    let excess_level = session.eval_text("Flatten[{{a, b}, c}, {{2}}]").unwrap();
    assert_eq!(
        excess_level.value.to_string(),
        "Flatten[List[List[a, b], c], List[List[2]]]"
    );
    assert!(
        excess_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "fldep")
    );

    let repeated_level = session.eval_text("Flatten[{{a, b}}, {{1}, {1}}]").unwrap();
    assert_eq!(
        repeated_level.value.to_string(),
        "Flatten[List[List[a, b]], List[List[1], List[1]]]"
    );
    assert!(
        repeated_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flrep")
    );

    let non_positive_level = session.eval_text("Flatten[{{a, b}}, {{0}}]").unwrap();
    assert_eq!(
        non_positive_level.value.to_string(),
        "Flatten[List[List[a, b]], List[List[0]]]"
    );
    assert!(
        non_positive_level
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "flpi")
    );

    let atomic_with_invalid_spec = session.eval_text("Flatten[x, y]").unwrap();
    assert_eq!(atomic_with_invalid_spec.value.to_string(), "Flatten[x, y]");
    assert!(
        atomic_with_invalid_spec
            .messages
            .iter()
            .any(|message| message.symbol == "Flatten" && message.tag == "normal")
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
