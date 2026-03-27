use super::*;

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
            .eval_text("f[g[a, b], h[c, d]][[All, 2 ;; ;; 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
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
}
