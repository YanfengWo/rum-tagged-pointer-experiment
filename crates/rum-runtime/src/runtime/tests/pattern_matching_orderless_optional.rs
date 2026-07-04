use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn assert_wk_cases(session: &mut Session, cases: &[(&str, &str)]) {
    let mut failures = Vec::new();

    for (source, expected) in cases {
        let actual = eval_to_string(session, source);
        if actual != *expected {
            failures.push(format!(
                "{source}\n  expected: {expected}\n    actual: {actual}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "WK-calibrated cases diverged:\n{}",
        failures.join("\n")
    );
}

#[test]
fn orderless_optional_present_matches_and_binds_in_wk_order() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("SetAttributes[f, Orderless]").unwrap();

    assert_wk_cases(
        &mut session,
        &[
            ("MatchQ[f[1, 2], f[a_, b_:0]]", "True"),
            ("MatchQ[f[2, 1], f[a_, b_:0]]", "True"),
            ("f[1, 2] /. f[a_, b_:0] :> {a, b}", "List[2, 1]"),
            ("f[2, 1] /. f[a_, b_:0] :> {a, b}", "List[2, 1]"),
            (
                "ReplaceList[f[1, 2], f[a_, b_:0] :> {a, b}]",
                "List[List[2, 1], List[1, 2]]",
            ),
        ],
    );
}

#[test]
fn orderless_optional_omission_uses_explicit_defaults() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("SetAttributes[f, Orderless]").unwrap();

    assert_wk_cases(
        &mut session,
        &[
            ("MatchQ[f[1], f[a_, b_:0]]", "True"),
            ("f[1] /. f[a_, b_:0] :> {a, b}", "List[1, 0]"),
            (
                "ReplaceList[f[1], f[a_, b_:0] :> {a, b}]",
                "List[List[1, 0]]",
            ),
            ("MatchQ[f[], f[a_:9, b_:0]]", "True"),
            ("f[] /. f[a_:9, b_:0] :> {a, b}", "List[9, 0]"),
        ],
    );
}

#[test]
fn orderless_optional_with_other_args_matches_present_and_omitted() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("SetAttributes[f, Orderless]").unwrap();

    assert_wk_cases(
        &mut session,
        &[
            ("MatchQ[f[x, 1, 2], f[c_, a_, b_:0]]", "True"),
            (
                "f[x, 1, 2] /. f[c_, a_, b_:0] :> {c, a, b}",
                "List[x, 2, 1]",
            ),
            ("MatchQ[f[x, 1], f[c_, a_, b_:0]]", "True"),
            ("f[x, 1] /. f[c_, a_, b_:0] :> {c, a, b}", "List[x, 1, 0]"),
        ],
    );
}

#[test]
fn orderless_optional_dot_uses_head_default() {
    let mut session = session_with_test_rewrite_builtins();

    session
        .eval_text("SetAttributes[f, Orderless]; Default[f] = d")
        .unwrap();

    assert_wk_cases(
        &mut session,
        &[
            ("MatchQ[f[1], f[a_, b_.]]", "True"),
            ("f[1] /. f[a_, b_.] :> {a, b}", "List[1, d]"),
            ("f[1, q] /. f[a_, b_.] :> {a, b}", "List[q, 1]"),
            ("f[1] /. f[a_, b_:v] :> {a, b}", "List[1, v]"),
        ],
    );
}

#[test]
#[ignore = "KNOWN WK DIVERGENCE (brief-080): Flat+Orderless+Optional still lacks WK's flat-wrapped singleton candidates/order; non-Flat Optional sequence omission is fixed"]
fn flat_orderless_optional_matches_collects_and_omits_like_wk() {
    let mut session = session_with_test_rewrite_builtins();

    session
        .eval_text("SetAttributes[g, {Flat, Orderless}]")
        .unwrap();

    assert_wk_cases(
        &mut session,
        &[
            ("MatchQ[g[1, 2], g[a_, b_:0]]", "True"),
            ("MatchQ[g[2, 1], g[a_, b_:0]]", "True"),
            ("g[1, 2] /. g[a_, b_:0] :> {a, b}", "List[1, 2]"),
            ("g[2, 1] /. g[a_, b_:0] :> {a, b}", "List[1, 2]"),
            (
                "ReplaceList[g[1, 2], g[a_, b_:0] :> {a, b}]",
                "List[List[g[2], g[1]], List[g[1], g[2]], List[2, 1], List[1, 2], List[g[1, 2], 0], List[g[1, 2], 0]]",
            ),
            ("MatchQ[g[1], g[a_, b_:0]]", "True"),
            ("g[1] /. g[a_, b_:0] :> {a, b}", "List[g[1], 0]"),
            (
                "ReplaceList[g[1], g[a_, b_:0] :> {a, b}]",
                "List[List[g[1], 0], List[1, 0]]",
            ),
            ("MatchQ[g[], g[a_:9, b_:0]]", "True"),
            ("g[] /. g[a_:9, b_:0] :> {a, b}", "List[9, 0]"),
        ],
    );
}

#[test]
fn orderless_optional_with_sequence_neighbors_uses_wk_backtracking_order() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("SetAttributes[f, Orderless]").unwrap();

    assert_wk_cases(
        &mut session,
        &[
            (
                "f[1, 2, 3] /. f[a_, b___, c_:0] :> {a, {b}, c}",
                "List[2, List[3], 1]",
            ),
            (
                "f[1] /. f[a_, b___, c_:0] :> {a, {b}, c}",
                "List[1, List[], 0]",
            ),
            (
                "ReplaceList[f[1, 2], f[a_, b___, c_:0] :> {a, {b}, c}]",
                "List[List[2, List[], 1], List[1, List[], 2], List[1, List[2], 0], List[2, List[1], 0]]",
            ),
            (
                "f[1, 2, 3] /. f[a_, b__, c_:0] :> {a, {b}, c}",
                "List[2, List[3], 1]",
            ),
            (
                "f[1, 2] /. f[a_, b__, c_:0] :> {a, {b}, c}",
                "List[1, List[2], 0]",
            ),
            (
                "ReplaceList[f[1, 2, 3], f[a_, b__, c_:0] :> {a, {b}, c}]",
                "List[List[2, List[3], 1], List[3, List[2], 1], List[1, List[3], 2], List[3, List[1], 2], List[1, List[2], 3], List[2, List[1], 3], List[1, List[2, 3], 0], List[2, List[1, 3], 0], List[3, List[1, 2], 0]]",
            ),
        ],
    );
}

#[test]
#[ignore = "KNOWN WK DIVERGENCE (brief-080): Flat+Orderless+Optional sequence-neighbor case still compounds flat-wrapped singleton order with grouped-match under-collection"]
fn flat_orderless_optional_with_sequence_neighbors_uses_wk_backtracking_order() {
    let mut session = session_with_test_rewrite_builtins();

    session
        .eval_text("SetAttributes[g, {Flat, Orderless}]")
        .unwrap();

    assert_wk_cases(
        &mut session,
        &[
            (
                "g[1, 2] /. g[a_, b___, c_:0] :> {a, {b}, c}",
                "List[1, List[], 2]",
            ),
            (
                "ReplaceList[g[1, 2], g[a_, b___, c_:0] :> {a, {b}, c}]",
                "List[List[g[2], List[], g[1]], List[g[1], List[], g[2]], List[2, List[], 1], List[1, List[], 2], List[g[1], List[2], 0], List[g[2], List[1], 0], List[g[1, 2], List[], 0], List[1, List[2], 0], List[2, List[1], 0], List[g[1, 2], List[], 0]]",
            ),
        ],
    );
}

#[test]
fn orderless_optional_nonmatch_edges_stay_false() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("SetAttributes[f, Orderless]").unwrap();

    assert_wk_cases(
        &mut session,
        &[
            ("MatchQ[h[1], f[a_, b_:0]]", "False"),
            ("MatchQ[f[x], f[a_Integer, b_:0]]", "False"),
        ],
    );

    let mut flat_session = session_with_test_rewrite_builtins();

    flat_session
        .eval_text("SetAttributes[g, {Flat, Orderless}]")
        .unwrap();

    assert_wk_cases(
        &mut flat_session,
        &[("MatchQ[g[x], g[a_Integer, b_:0]]", "False")],
    );
}
