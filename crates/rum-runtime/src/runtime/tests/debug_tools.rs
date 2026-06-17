use super::*;

#[test]
fn trace_returns_a_first_pass_structured_expression_list_without_leaking_session_trace_state() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("Trace[f[1]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Hold[f[1]], List[Hold[Plus[1, 1]], Hold[2]], Hold[2]]"
    );
    assert!(session.take_trace().is_empty());

    let filtered = session.eval_text("Trace[f[1], _Plus]").unwrap();
    assert_eq!(filtered.value.to_string(), "List[List[Hold[Plus[1, 1]]]]");

    let multi = session.eval_text("Trace[f[1], {_Plus, _Integer}]").unwrap();
    assert_eq!(
        multi.value.to_string(),
        "List[List[Hold[Plus[1, 1]], Hold[2]], Hold[2]]"
    );

    let leveled = session.eval_text("Trace[f[1], _Integer, {-1}]").unwrap();
    assert_eq!(
        leveled.value.to_string(),
        "List[Hold[1], List[Hold[1], Hold[2]], Hold[2]]"
    );

    let top_level_only = session.eval_text("Trace[f[1], _Integer, 0]").unwrap();
    assert_eq!(
        top_level_only.value.to_string(),
        "List[List[Hold[2]], Hold[2]]"
    );

    let with_heads = session
        .eval_text("Trace[f[1], _Symbol, Heads -> True]")
        .unwrap();
    assert_eq!(
        with_heads.value.to_string(),
        "List[Hold[f], List[Hold[Plus]]]"
    );

    session
        .eval_text("SetOptions[Trace, Heads -> True]")
        .unwrap();
    let default_heads = session.eval_text("Trace[f[1], _Symbol]").unwrap();
    assert_eq!(
        default_heads.value.to_string(),
        "List[Hold[f], List[Hold[Plus]]]"
    );

    let explicit_no_heads = session
        .eval_text("Trace[f[1], _Symbol, Heads -> False]")
        .unwrap();
    assert_eq!(explicit_no_heads.value.to_string(), "List[]");

    let explicit_no_heads_with_levels = session
        .eval_text("Trace[f[1], _Symbol, {0, Infinity}, {Heads -> False}]")
        .unwrap();
    assert_eq!(explicit_no_heads_with_levels.value.to_string(), "List[]");
}

#[test]
fn trace_uses_recorded_depths_to_keep_nested_branches_stable() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("g[x_] := f[x] + 10").unwrap();

    let traced = session.eval_text("Trace[g[1]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Hold[g[1]], List[Hold[Plus[f[1], 10]], List[Hold[f[1]], List[Hold[Plus[1, 1]], Hold[2]], Hold[2]], Hold[12]], Hold[12]]"
    );

    let filtered = session.eval_text("Trace[g[1], _Plus]").unwrap();
    assert_eq!(
        filtered.value.to_string(),
        "List[List[Hold[Plus[f[1], 10]], List[Hold[Plus[1, 1]]]]]"
    );

    let via_eval_data = session
        .eval_text("EvaluationData[g[1], \"Trace\"]")
        .unwrap();
    assert_eq!(via_eval_data.value.to_string(), traced.value.to_string());
}

#[test]
fn trace_print_emits_trace_lines_and_returns_the_underlying_value() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("TracePrint[f[1]]").unwrap();
    assert_eq!(traced.value.to_string(), "2");
    assert_eq!(traced.prints, vec!["f[1]", "  Plus[1, 1]", "  2", "2"]);
    assert!(session.take_trace().is_empty());

    let filtered = session.eval_text("TracePrint[f[1], _Plus]").unwrap();
    assert_eq!(filtered.value.to_string(), "2");
    assert_eq!(filtered.prints, vec!["Plus[1, 1]"]);

    let multi = session
        .eval_text("TracePrint[f[1], {_Plus, _Integer}]")
        .unwrap();
    assert_eq!(multi.value.to_string(), "2");
    assert_eq!(multi.prints, vec!["  Plus[1, 1]", "  2", "2"]);

    let leveled = session
        .eval_text("TracePrint[f[1], _Integer, {-1}]")
        .unwrap();
    assert_eq!(leveled.value.to_string(), "2");
    assert_eq!(leveled.prints, vec!["1", "  1", "  2", "2"]);

    let with_heads = session
        .eval_text("TracePrint[f[1], _Symbol, Heads -> True]")
        .unwrap();
    assert_eq!(with_heads.value.to_string(), "2");
    assert_eq!(with_heads.prints, vec!["f", "  Plus"]);

    session
        .eval_text("SetOptions[TracePrint, Heads -> True]")
        .unwrap();
    let default_heads = session.eval_text("TracePrint[f[1], _Symbol]").unwrap();
    assert_eq!(default_heads.value.to_string(), "2");
    assert_eq!(default_heads.prints, vec!["f", "  Plus"]);

    let explicit_no_heads = session
        .eval_text("TracePrint[f[1], _Symbol, Heads -> False]")
        .unwrap();
    assert_eq!(explicit_no_heads.value.to_string(), "2");
    assert!(explicit_no_heads.prints.is_empty());

    let explicit_no_heads_with_levels = session
        .eval_text("TracePrint[f[1], _Symbol, {0, Infinity}, {Heads -> False}]")
        .unwrap();
    assert_eq!(explicit_no_heads_with_levels.value.to_string(), "2");
    assert!(explicit_no_heads_with_levels.prints.is_empty());
}

#[test]
fn trace_print_uses_the_same_nested_structure_for_deeper_trace_branches() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("g[x_] := f[x] + 10").unwrap();

    let traced = session.eval_text("TracePrint[g[1], _Plus]").unwrap();
    assert_eq!(traced.value.to_string(), "12");
    assert_eq!(traced.prints, vec!["Plus[f[1], 10]", "  Plus[1, 1]"]);
}

#[test]
fn echo_family_prints_and_returns_values_in_the_current_first_pass_subset() {
    let mut session = Session::new();

    let echoed = session.eval_text("Echo[1 + 1]").unwrap();
    assert_eq!(echoed.value.to_string(), "2");
    assert_eq!(echoed.prints, vec!["2"]);

    let labeled = session.eval_text("Echo[1 + 1, \"sum\"]").unwrap();
    assert_eq!(labeled.value.to_string(), "2");
    assert_eq!(labeled.prints, vec!["sum: 2"]);

    let via_function = session.eval_text("EchoFunction[][1 + 1]").unwrap();
    assert_eq!(via_function.value.to_string(), "2");
    assert_eq!(via_function.prints, vec!["2"]);

    let via_label = session.eval_text("EchoLabel[\"sum\"][1 + 1]").unwrap();
    assert_eq!(via_label.value.to_string(), "2");
    assert_eq!(via_label.prints, vec!["sum: 2"]);
}

#[test]
fn short_uses_the_current_canonical_text_truncation_subset() {
    let mut session = Session::new();

    let shortened = session.eval_text("Short[\"abcdef\", 16]").unwrap();
    assert_eq!(shortened.value.to_string(), "\"abcdef\"");

    let long_form = session.eval_text("Short[Table[i, {i, 20}], 18]").unwrap();
    assert!(long_form.value.to_string().contains("..."));
}

#[test]
fn rum_profile_hooks_are_inert_without_profile_env() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("RumProfileReset[]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("RumProfileSnapshot[\"unit\"]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("RumProfileReset[extra]")
            .unwrap()
            .value
            .to_string(),
        "RumProfileReset[extra]"
    );
}

#[test]
fn on_and_off_gate_matching_messages_at_session_scope() {
    let mut session = Session::new();

    session.eval_text("Off[FindFile::notfound]").unwrap();
    let suppressed = session.eval_text("Message[FindFile::notfound]").unwrap();
    assert!(suppressed.messages.is_empty());

    session.eval_text("On[FindFile]").unwrap();
    let restored = session.eval_text("Message[FindFile::notfound]").unwrap();
    assert!(
        restored
            .messages
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );
}
