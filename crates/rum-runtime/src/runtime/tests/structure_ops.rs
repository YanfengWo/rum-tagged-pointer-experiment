use super::*;

#[test]
fn head_length_and_depth_cover_atoms_calls_and_compound_heads() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Head[1 + 2]").unwrap().value.to_string(),
        "Integer"
    );
    assert_eq!(
        session.eval_text("Head[f[a], Hold]").unwrap().value.to_string(),
        "Hold[f]"
    );
    assert_eq!(session.eval_text("Length[5]").unwrap().value.to_string(), "0");
    assert_eq!(
        session.eval_text("Length[f[a, b, c]]").unwrap().value.to_string(),
        "3"
    );
    assert_eq!(session.eval_text("Depth[5]").unwrap().value.to_string(), "1");
    assert_eq!(
        session.eval_text("Depth[f[a, g[b]]]").unwrap().value.to_string(),
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
        session.eval_text("Last[f[a, b]]").unwrap().value.to_string(),
        "b"
    );
    assert_eq!(
        session.eval_text("Last[5, x]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session.eval_text("Rest[f[a, b, c]]").unwrap().value.to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session.eval_text("Most[f[a, b, c]]").unwrap().value.to_string(),
        "f[a, b]"
    );
    assert_eq!(session.eval_text("Rest[f[]]").unwrap().value.to_string(), "f[]");
    assert_eq!(session.eval_text("Most[f[]]").unwrap().value.to_string(), "f[]");
}

#[test]
fn first_last_rest_and_most_emit_expected_messages_for_invalid_inputs() {
    let mut session = Session::new();

    let outcome = session.eval_text("First[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "First[f[]]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "First" && message.tag == "first"));

    let outcome = session.eval_text("Last[f[]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Last[f[]]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "Last" && message.tag == "nolast"));

    let outcome = session.eval_text("Rest[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "Rest[5]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "General" && message.tag == "normal"));

    let outcome = session.eval_text("Most[5]").unwrap();
    assert_eq!(outcome.value.to_string(), "Most[5]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "General" && message.tag == "normal"));
}

#[test]
fn take_drop_append_and_prepend_cover_current_structural_subset() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Take[f[a, b, c, d], 2]").unwrap().value.to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session.eval_text("Take[f[a, b, c, d], -2]").unwrap().value.to_string(),
        "f[c, d]"
    );
    assert_eq!(
        session.eval_text("Take[f[a, b, c, d], {2}]").unwrap().value.to_string(),
        "f[b]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], {2, -1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[g[a, b], h[c, d]], All, {2}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session.eval_text("Take[f[a, b], None]").unwrap().value.to_string(),
        "f[]"
    );

    assert_eq!(
        session.eval_text("Drop[f[a, b, c, d], 2]").unwrap().value.to_string(),
        "f[c, d]"
    );
    assert_eq!(
        session.eval_text("Drop[f[a, b, c, d], -2]").unwrap().value.to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], {2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[g[a, b], h[c, d]], None, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("TakeDrop[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], f[c, d]]"
    );

    assert_eq!(
        session.eval_text("Append[f[a, b], x]").unwrap().value.to_string(),
        "f[a, b, x]"
    );
    assert_eq!(
        session.eval_text("Prepend[f[a, b], x]").unwrap().value.to_string(),
        "f[x, a, b]"
    );
    assert_eq!(
        session.eval_text("Append[x][f[a, b]]").unwrap().value.to_string(),
        "f[a, b, x]"
    );
    assert_eq!(
        session.eval_text("Prepend[x][f[a, b]]").unwrap().value.to_string(),
        "f[x, a, b]"
    );
    assert_eq!(
        session.eval_text("Symbol[\"x\"]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session.eval_text("Symbol[\"List\"]").unwrap().value.to_string(),
        "List"
    );
    assert_eq!(
        session.eval_text("Construct[f, a, b]").unwrap().value.to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session.eval_text("Construct[Plus, 1, 2]").unwrap().value.to_string(),
        "3"
    );
}

#[test]
fn take_drop_append_and_prepend_report_current_invalid_boundaries() {
    let mut session = Session::new();

    let outcome = session.eval_text("Take[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[f[a], 2]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "Take" && message.tag == "take"));

    let outcome = session.eval_text("Drop[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Drop[f[a], 2]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "Drop" && message.tag == "drop"));

    let outcome = session.eval_text("Append[5, x]").unwrap();
    assert_eq!(outcome.value.to_string(), "Append[5, x]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "General" && message.tag == "normal"));

    let outcome = session.eval_text("Prepend[5, x]").unwrap();
    assert_eq!(outcome.value.to_string(), "Prepend[5, x]");
    assert!(outcome
        .messages
        .iter()
        .any(|message| message.symbol == "General" && message.tag == "normal"));
}

#[test]
fn part_supports_exact_indices_nested_paths_and_head_access() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 2]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session
            .eval_text("Part[f[g[a], h[b]], 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {1, -1}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0]")
            .unwrap()
            .value
            .to_string(),
        "f"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], {}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[g[a, b], h[c, d]], All, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c, d], Span[2, All, 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {Span[1, All, 2], -1}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0 ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[f, a, b]"
    );
    assert_eq!(
        session
            .eval_text("f[g[a, b], h[c, d]][[All, 2 ;; ;; 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {{1, 3}, {2}}]")
            .unwrap()
            .value
            .to_string(),
        "f[List[a, c], List[b]]"
    );
}

#[test]
fn extract_consumes_exact_paths_and_position_output() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Extract[f[g[a], b, h[c]], {1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a], b, h[c]], Position[f[g[a], b, h[c]], _Symbol, Infinity, Heads -> False]]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{1}, {2}}, List]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a, b], h[c, d]], {All, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a, b], h[c, d]], {{All, 2}, {1, Span[1, All]}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[b, d], g[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {Span[0, -1]}]")
            .unwrap()
            .value
            .to_string(),
        "f[f, a, b]"
    );
    assert_eq!(
        session.eval_text("Extract[f[a, b], {}]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b]]"
    );
}

#[test]
fn replacepart_supports_immediate_delayed_and_head_replacements() {
    let mut session = Session::new();

    session.eval_text("n = 1").unwrap();
    session.eval_text("immediate = 2 -> n").unwrap();
    session.eval_text("delayed = 2 :> n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], immediate]")
            .unwrap()
            .value
            .to_string(),
        "f[a, 1]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], delayed]")
            .unwrap()
            .value
            .to_string(),
        "f[a, 2]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {0} -> g]")
            .unwrap()
            .value
            .to_string(),
        "g[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {{1, 1} -> x, {2, 1} -> y}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[x], h[y]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a, b], h[c, d]], {All, 2} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, x], h[c, x]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c, d], Span[2, All, 2] -> y]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y, c, y]"
    );

    session.eval_text("delayedAll = {All, 2} :> n").unwrap();
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a, b], h[c, d]], delayedAll]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, 2], h[c, 2]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c, d], 2 ;; ;; 2 -> y]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y, c, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], 0 ;; -1 -> x]")
            .unwrap()
            .value
            .to_string(),
        "x[x, x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], All -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "x[x, x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], 0 -> x, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {{}} -> x]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
}

#[test]
fn replacepart_supports_pattern_position_specs_and_rhs_bindings() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c], i_ -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[x, x, x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c], {i_} -> i]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {i_, 0} -> i]")
            .unwrap()
            .value
            .to_string(),
        "f[1[a], 2[b]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {i_, 0} -> x, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a], h[b]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], i_ -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "x[x, x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {{i_}, {0}} -> x]")
            .unwrap()
            .value
            .to_string(),
        "x[x, x]"
    );
}

#[test]
fn replacepart_handles_duplicate_and_overlapping_targets_more_stably() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplacePart[f[a], {1 -> x, 1 -> y}]")
            .unwrap()
            .value
            .to_string(),
        "f[y]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a]], {{1} -> x, {1, 1} -> y}]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a]], {{1, 1} -> y, {1} -> x}]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {All, 0} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[x[a], x[b]]"
    );
}
