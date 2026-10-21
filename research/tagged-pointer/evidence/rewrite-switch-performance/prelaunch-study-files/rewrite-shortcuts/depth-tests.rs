// Isolated research regression module. Install as runtime/tests/
// rewrite_depth_shortcuts.rs and append `mod rewrite_depth_shortcuts;` to
// runtime/tests/mod.rs in a disposable source copy.
use super::*;

fn assert_replace_value(source: &str, expected: &str) {
    let mut session = session_with_test_control_builtins();
    let outcome = session.eval_text(source).expect(source);
    assert_eq!(outcome.value.to_string(), expected, "{source}");
    assert!(outcome.messages.is_empty(), "unexpected messages: {source}");
}

#[test]
fn depth_shortcut_nonnegative_selection_keeps_root_and_traversal_order() {
    for (spec, expected) in [
        ("0", "f[g[1], 2]"),
        ("{1}", "f[g[1], hit]"),
        ("{2}", "f[g[hit], 2]"),
        ("{1, 2}", "f[g[hit], hit]"),
        ("Infinity", "f[g[hit], hit]"),
        ("All", "f[g[hit], hit]"),
        ("{2, 1}", "f[g[1], 2]"),
        ("{0, -1}", "f[g[1], 2]"),
    ] {
        assert_replace_value(
            &format!("Replace[f[g[1], 2], _Integer -> hit, {spec}]"),
            expected,
        );
    }
    assert_replace_value(
        "Replace[f[1], {f[x_] :> h[x], _Integer -> hit}, All]",
        "h[1]",
    );
}

#[test]
fn depth_shortcut_negative_and_mixed_ranges_use_actual_input_depth() {
    for (source, expected) in [
        ("Replace[f[], _ -> hit, {-1}]", "f[]"),
        ("Replace[f[], _ -> hit, {-2}]", "hit"),
        ("Replace[f[g[]], _ -> hit, {-2}]", "f[hit]"),
        ("Replace[f[g[]], _ -> hit, {-3}]", "hit"),
        (
            "Replace[f[g[1], 2], _Integer -> hit, {-1}]",
            "f[g[hit], hit]",
        ),
        (
            "Replace[f[g[1], 2], _Integer -> hit, {-2, 1}]",
            "f[g[1], hit]",
        ),
        ("Replace[f[g[1], 2], _ -> hit, {-2}]", "f[hit, 2]"),
    ] {
        assert_replace_value(source, expected);
    }
}

#[test]
fn depth_shortcut_heads_and_compound_head_depth_remain_distinct() {
    for (source, expected) in [
        (
            "Replace[f[g][a], f[g] -> h, {1}, Heads -> True]",
            "h[a]",
        ),
        (
            "Replace[f[g][a], f[g] -> h, {1}, Heads -> False]",
            "f[g][a]",
        ),
        ("Replace[f[g][a], _ -> hit, {-3}, Heads -> True]", "hit"),
        (
            "Replace[f[g][a], _ -> hit, {-3}, Heads -> False]",
            "f[g][a]",
        ),
    ] {
        assert_replace_value(source, expected);
    }
}

#[test]
fn depth_shortcut_distinguishes_association_heads_from_ordinary_fixture_calls() {
    // This kernel-only fixture does not install the stdlib Association builtin.
    // Its unqualified Association is therefore a Global ordinary call, whose
    // Rule children (including integer keys) participate in normal traversal.
    assert_replace_value(
        "Replace[Association[1 -> g[2], 3 -> 4], _Integer -> 9, {1, Infinity}]",
        "Association[Rule[9, g[9]], Rule[9, 9]]",
    );
    // Explicit qualification exercises the kernel's association-specific walk
    // without assuming the test fixture installed the full standard library.
    for spec in ["{1, Infinity}", "{-1}"] {
        assert_replace_value(
            &format!(
                "Replace[System`Association[1 -> g[2], 3 -> 4], _Integer -> 9, {spec}]"
            ),
            "Association[Rule[1, g[9]], Rule[3, 9]]",
        );
    }
}

#[test]
fn depth_shortcut_sequence_splicing_keeps_parent_attribute_boundary() {
    for spec in ["{1, Infinity}", "{-1}"] {
        assert_replace_value(
            &format!(
                "SetAttributes[sh, SequenceHold]; \
                 {{Replace[g[a], a -> Sequence[1, 2], {spec}], \
                   Replace[sh[a], a -> Sequence[1, 2], {spec}]}}"
            ),
            "List[g[1, 2], sh[Sequence[1, 2]]]",
        );
    }
}

#[test]
fn depth_shortcut_finishes_guard_walk_before_delayed_rhs_evaluation() {
    assert_replace_value(
        "seen = 0; result = Replace[f[1, g[2], 3], \
         {x_Integer :> (bad /; (seen = seen * 10 + x; False)), \
          x_Integer :> (seen = seen * 10 + 9; x + 10)}, {1, Infinity}]; \
         {result, seen}",
        "List[f[11, g[12], 13], 123999]",
    );
}

fn assert_depth_resume(source: &str, inputs: &[&str], expected: &str) {
    let mut queued = session_with_test_control_builtins();
    for input in inputs {
        queued.queue_input_response(*input);
    }
    let queued_value = queued.eval_request(source).expect("queued completion").value;
    assert_eq!(queued_value.to_string(), expected);

    let mut resumed = session_with_test_control_builtins();
    let mut failure = resumed.eval_request(source).expect_err("first suspension");
    for (index, input) in inputs.iter().enumerate() {
        assert!(matches!(failure.error, SessionRequestError::InputRequired { .. }));
        let outcome = resumed.resume_eval_request_with_input(*input);
        if index + 1 == inputs.len() {
            let value = outcome.expect("final resume").value;
            assert_eq!(value.to_string(), queued_value.to_string());
            return;
        }
        failure = outcome.expect_err("next suspension");
    }
    panic!("at least one input must be supplied");
}

#[test]
fn depth_shortcut_input_required_keeps_mid_walk_state_and_exactly_once_rhs() {
    for spec in ["{1, Infinity}", "{-1}", "{-2, 2}"] {
        assert_depth_resume(
            &format!(
                "seen = 0; result = Replace[f[1, g[2], 3], \
                 x_Integer :> ((seen = seen + 1; h[x]) /; \
                 MatchQ[InputString[\"guard: \"], \"yes\"]), {spec}]; \
                 {{result, seen}}"
            ),
            &["yes", "yes", "yes"],
            "List[f[h[1], g[h[2]], h[3]], 3]",
        );
    }
    assert_depth_resume(
        "Replace[f[1], _Integer -> hit, \
         If[MatchQ[InputString[\"level: \"], \"yes\"], {1}, {0}]]",
        &["yes"],
        "f[hit]",
    );
}

#[test]
fn depth_shortcut_keeps_rhs_recursion_limit_and_session_recovery() {
    let mut session = session_with_test_control_builtins();
    session.set_recursion_limit(32);
    session.eval_text("grow[x_] := grow[f[x]]").unwrap();
    let outcome = session
        .eval_text("Replace[1, x_Integer :> grow[x], {0}]")
        .expect("limit is a language termination value");
    assert_eq!(outcome.value.to_string(), "TerminatedEvaluation[\"RecursionLimit\"]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "General" && message.tag == "reclim"
    }));
    assert_eq!(session.eval_text("1 + 1").unwrap().value.to_string(), "2");
}
