use super::*;

fn shortcut_value(source: &str) -> String {
    session_with_test_control_builtins()
        .eval_text(source)
        .expect("rewrite shortcut regression should evaluate")
        .value
        .to_string()
}

#[test]
fn rewrite_shortcut_blank_bindings_preserve_types_values_and_scope() {
    assert_eq!(
        shortcut_value(
            "{Replace[7, _Integer -> hit], Replace[\"s\", _Integer -> miss], \
             Replace[7, x_Integer :> Hold[x, x]], \
             Replace[widget[1], x_widget :> Hold[x]], \
             Replace[y, body_ :> Module[{y}, Hold[body, y]]]}"
        ),
        "List[hit, \"s\", Hold[7, 7], Hold[widget[1]], Hold[y, y$1]]"
    );
}

#[test]
fn rewrite_shortcut_immediate_delayed_and_traversal_effects_stay_ordered() {
    assert_eq!(
        shortcut_value(
            "counter = 0; immediate = x_Integer -> (counter = counter + 1); \
             first = Replace[{10, 20}, immediate, {1}]; \
             delayed = x_Integer :> (counter = counter + 1); \
             second = Replace[{10, 20}, delayed, {1}]; \
             {first, second, counter}"
        ),
        "List[List[1, 1], List[2, 3], 3]"
    );
}

#[test]
fn rewrite_shortcut_rule_priority_guards_and_trace_stay_on_the_original_path() {
    let mut session = session_with_test_control_builtins();
    session.enable_tracing(true);
    let response = session
        .eval_request(
            "guardRuns = 0; bodyRuns = 0; \
             result = Replace[7, { \
                 x_Integer :> (bodyRuns = bodyRuns + 100; bad) /; \
                     (guardRuns = guardRuns + 1; False), \
                 x_Integer :> (bodyRuns = bodyRuns + 1; Hold[x]), \
                 x_Integer :> (bodyRuns = bodyRuns + 1000; late)}]; \
             {result, guardRuns, bodyRuns}",
        )
        .unwrap();
    assert_eq!(response.value.to_string(), "List[Hold[7], 1, 1]");
    assert!(response.messages.is_empty());
    let applied = response
        .trace
        .iter()
        .filter_map(|event| match event {
            TraceEvent::RuleApplied {
                source: RuleApplicationSource::Replace,
                input,
                result,
                ..
            } => Some((input, result)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].0.to_string(), "7");
    assert!(applied[0].1.to_string().contains("Hold[7]"));
}

#[test]
fn rewrite_shortcut_complex_fallback_restores_bindings_and_head_attributes() {
    assert_eq!(
        shortcut_value(
            "SetAttributes[id, OneIdentity]; \
             SetAttributes[ord, {Flat, Orderless}]; \
             {Replace[f[1, 2], {f[x_, x_] :> bad, x_ :> Hold[x]}], \
              Replace[f[1, 1], f[x_, x_] :> Hold[x]], \
              Replace[a, id[a] -> hit], \
              Replace[ord[b, ord[a]], ord[a, b] -> hit]}"
        ),
        "List[Hold[f[1, 2]], Hold[1], hit, hit]"
    );
}

#[test]
fn rewrite_shortcut_pattern_combinators_keep_user_tests_and_held_heads() {
    assert_eq!(
        shortcut_value(
            "seen = 0; test[z_] := (seen = seen + 1; True); \
             custom[z_] := (seen = seen + 100; z); \
             first = Replace[1, { \
                 (x_Integer /; (seen = seen + 1; False)) :> bad, \
                 x_Integer?test :> Hold[x]}]; \
             second = Replace[2, (x_String | x_Integer) :> Hold[x]]; \
             third = Replace[Hold[custom[7]], x_custom :> Hold[x], {1}]; \
             {first, second, third, seen}"
        ),
        "List[Hold[1], Hold[2], Hold[Hold[custom[7]]], 2]"
    );
}

#[test]
fn rewrite_shortcut_suspension_resumes_after_guard_without_replaying_prefix() {
    let source = "seen = 0; result = Replace[{1, 2}, { \
        x_Integer :> bad /; (seen = seen + 1; \
            MatchQ[InputString[\"guard: \"], \"yes\"]), \
        x_Integer :> Hold[x]} , {1}]; {result, seen}";
    let mut queued = session_with_test_control_builtins();
    queued.queue_input_response("no");
    queued.queue_input_response("no");
    let expected = queued.eval_request(source).unwrap().value;
    assert_eq!(expected.to_string(), "List[List[Hold[1], Hold[2]], 2]");

    let mut resumed = session_with_test_control_builtins();
    let first = resumed.eval_request(source).unwrap_err();
    assert!(matches!(
        first.error,
        SessionRequestError::InputRequired { request } if request.prompt == "guard: "
    ));
    let second = resumed.resume_eval_request_with_input("no").unwrap_err();
    assert!(matches!(
        second.error,
        SessionRequestError::InputRequired { request } if request.prompt == "guard: "
    ));
    let actual = resumed.resume_eval_request_with_input("no").unwrap();
    assert_eq!(actual.value, expected);
    assert!(actual.messages.is_empty());
}

#[test]
fn rewrite_shortcut_keeps_rhs_recursion_and_guard_capability_checks() {
    let mut limited = session_with_test_control_builtins();
    limited.set_recursion_limit(16);
    let result = limited
        .eval_request("loop[z_] := loop[z + 1]; Replace[1, x_Integer :> loop[x]]")
        .unwrap();
    assert_eq!(
        result.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );

    let mut sandboxed = session_with_test_control_builtins();
    sandboxed.set_capability_policy(CapabilityPolicy::allow([Capability::Pure]));
    let denied = sandboxed
        .eval_request(
            "Replace[1, {x_Integer :> bad /; \
                 MatchQ[InputString[\"denied: \"], \"yes\"], \
                 x_Integer :> Hold[x]}]",
        )
        .expect("denied interactive guard should decline, without host suspension");
    assert_eq!(denied.value.to_string(), "Hold[1]");
    assert!(
        denied
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );
}
