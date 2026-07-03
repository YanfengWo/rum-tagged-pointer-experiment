use super::*;

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
        "f[g[a, b], h[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c, d], Span[2, All, 2] -> y]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], All -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {1 ;; 2} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );

    session.eval_text("delayedAll = {All, 2} :> n").unwrap();
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a, b], h[c, d]], delayedAll]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, b], h[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c, d], 2 ;; ;; 2 -> y]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c, d]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], 0 ;; -1 -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], All -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
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
            .eval_text("ReplacePart[1, 0 -> x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[1, All -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[1, 0 ;; 0 -> x]")
            .unwrap()
            .value
            .to_string(),
        "1"
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
        "f[g[a], h[b]]"
    );
}

#[test]
fn replaceat_supports_exact_positions_operator_form_and_rule_lists() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplaceAt[f[a, b], x_ -> y, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[x_ -> y, 2][f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[f[1], {x_String -> z, x_Integer -> y}, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[y]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[f[a, b], x_ -> y, {}]")
            .unwrap()
            .value
            .to_string(),
        "y"
    );
}
