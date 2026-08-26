use super::*;

#[test]
fn replacepart_rhs_guards_accept_and_decline_exact_position_forms() {
    let mut session = Session::new();

    for (input, expected) in [
        (
            "ReplacePart[{1, 2, 3}, 1 -> (\"immediate\" /; True)]",
            "List[\"immediate\", 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, 1 -> (\"immediate\" /; False)]",
            "List[1, 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, 1 :> (\"delayed\" /; True)]",
            "List[\"delayed\", 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, 1 :> (\"delayed\" /; False)]",
            "List[1, 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, -1 -> (\"negative\" /; True)]",
            "List[1, 2, \"negative\"]",
        ),
        (
            "ReplacePart[{1, 2, 3}, -1 -> (\"negative\" /; False)]",
            "List[1, 2, 3]",
        ),
        (
            "ReplacePart[{{1, 2}, 3}, {1, 2} -> (\"nested\" /; True)]",
            "List[List[1, \"nested\"], 3]",
        ),
        (
            "ReplacePart[{{1, 2}, 3}, {1, 2} -> (\"nested\" /; False)]",
            "List[List[1, 2], 3]",
        ),
        ("ReplacePart[f[a], 0 -> (g /; True)]", "g[a]"),
        ("ReplacePart[f[a], 0 -> (g /; False)]", "f[a]"),
        (
            "ReplacePart[{1, 2, 3}, {{1}, {3}} -> (\"multi\" /; True)]",
            "List[\"multi\", 2, \"multi\"]",
        ),
        (
            "ReplacePart[{1, 2, 3}, {{1}, {3}} -> (\"multi\" /; False)]",
            "List[1, 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, {2 ;; 3} -> (\"span\" /; True)]",
            "List[1, \"span\", \"span\"]",
        ),
        (
            "ReplacePart[{1, 2, 3}, {2 ;; 3} -> (\"span\" /; False)]",
            "List[1, 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, Span[All] -> (\"all\" /; True)]",
            "List[\"all\", \"all\", \"all\"]",
        ),
        (
            "ReplacePart[{1, 2, 3}, Span[All] -> (\"all\" /; False)]",
            "List[1, 2, 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, Span[1, UpTo[2]] -> (\"upto\" /; True)]",
            "List[\"upto\", \"upto\", 3]",
        ),
        (
            "ReplacePart[{1, 2, 3}, Span[1, UpTo[2]] -> (\"upto\" /; False)]",
            "List[1, 2, 3]",
        ),
        ("ReplacePart[f[a], {{}} -> (root /; True)]", "root"),
        ("ReplacePart[f[a], {{}} -> (root /; False)]", "f[a]"),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{input}");
        assert!(outcome.messages.is_empty(), "{input}");
    }
}

#[test]
fn replacepart_pattern_guards_receive_bindings_and_option_scope() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("acceptPosition[2] = True; Options[positionGuard] = {pick -> True}")
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "{ReplacePart[{a, b, c}, {i_} -> (immediate[i] /; acceptPosition[i])], \
                  ReplacePart[{a, b, c}, {i_} :> (delayed[i] /; acceptPosition[i])], \
                  ReplacePart[{a, b}, {i_, OptionsPattern[positionGuard]} :> \
                    (scoped[i] /; OptionValue[pick])], \
                  ReplacePart[{a, b}, {i_} :> (bad[i] /; 1)]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[a, immediate[2], c], List[a, delayed[2], c], List[scoped[1], scoped[2]], List[a, b]]"
    );
}

#[test]
fn replacepart_declining_later_guard_hands_position_to_earlier_rule() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplacePart[{1, 2, 3}, {1 -> \"x\", 1 -> (\"y\" /; False)}]",)
            .unwrap()
            .value
            .to_string(),
        "List[\"x\", 2, 3]"
    );
}

#[test]
fn replacepart_contested_guards_run_last_to_first_and_stop_after_acceptance() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "order = 0; firstRuns = 0; lastRuns = 0; \
                 declined = ReplacePart[{1, 2, 3}, { \
                   1 :> (\"first\" /; (firstRuns = firstRuns + 1; order = order * 10 + 1; True)), \
                   1 :> (\"last\" /; (lastRuns = lastRuns + 1; order = order * 10 + 2; False))}]; \
                 declinedEvidence = {declined, firstRuns, lastRuns, order}; \
                 order = 0; firstRuns = 0; lastRuns = 0; \
                 accepted = ReplacePart[{1, 2, 3}, { \
                   1 :> (\"first\" /; (firstRuns = firstRuns + 1; order = order * 10 + 1; True)), \
                   1 :> (\"last\" /; (lastRuns = lastRuns + 1; order = order * 10 + 2; True))}]; \
                 {declinedEvidence, {accepted, firstRuns, lastRuns, order}}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[List[\"first\", 2, 3], 1, 1, 21], List[List[\"last\", 2, 3], 0, 1, 2]]"
    );
}

#[test]
fn replacepart_evaluates_each_rule_position_guard_at_most_once() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "duplicateRuns = 0; \
                 duplicate = ReplacePart[{a, b}, {{1}, {1}} :> \
                   (\"once\" /; (duplicateRuns = duplicateRuns + 1; True))]; \
                 spanRuns = 0; \
                 span = ReplacePart[{a, b, c}, {1 ;; 3} :> \
                   (\"never\" /; (spanRuns = spanRuns + 1; False))]; \
                 {duplicate, duplicateRuns, span, spanRuns}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[\"once\", b], 1, List[a, b, c], 3]"
    );
}

#[test]
fn replacepart_guard_acceptance_preserves_raw_delayed_bodies() {
    let mut session = Session::new();
    session.eval_text("delayedBodyRuns = 0").unwrap();

    assert_eq!(
        session
            .eval_text(
                "ReplacePart[Hold[a, b], 1 :> \
                   ((delayedBodyRuns = delayedBodyRuns + 1; raw) /; True)]",
            )
            .unwrap()
            .value
            .to_string(),
        "Hold[CompoundExpression[Set[delayedBodyRuns, Plus[delayedBodyRuns, 1]], raw], b]"
    );
    assert_eq!(
        session
            .eval_text("delayedBodyRuns")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
}

#[test]
fn replacepart_guard_suspension_resumes_same_rule_position_and_plan() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("guardResumeRuns = 0; fallbackRuns = 0")
        .unwrap();

    let failure = session
        .eval_request(
            "{ReplacePart[{1, 2}, { \
               1 :> (\"fallback\" /; (fallbackRuns = fallbackRuns + 1; True)), \
               2 -> \"planned\", \
               1 :> (\"guarded\" /; (guardResumeRuns = guardResumeRuns + 1; \
                 MatchQ[InputString[\"replace part guard: \"], \"yes\"]))}], \
             guardResumeRuns, fallbackRuns}",
        )
        .expect_err("ReplacePart rhs guard should suspend for host input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "replace part guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("a declining resumed guard should hand off to the earlier candidate");
    assert_eq!(
        response.value.to_string(),
        "List[List[\"fallback\", \"planned\"], 1, 1]"
    );
}

#[test]
fn replacepart_unguarded_rules_keep_explicit_later_rule_wins_behavior() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplacePart[{1, 2, 3}, {1 -> \"x\", 1 -> \"y\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[\"y\", 2, 3]"
    );
}
