use super::*;

#[test]
fn trace_returns_a_first_pass_flat_expression_list_without_leaking_session_trace_state() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("Trace[f[1]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Hold[f[1]], Hold[Plus[1, 1]], Hold[2], Hold[f[1]], Hold[2]]"
    );
    assert!(session.take_trace().is_empty());
}

#[test]
fn trace_print_emits_trace_lines_and_returns_the_underlying_value() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("TracePrint[f[1]]").unwrap();
    assert_eq!(traced.value.to_string(), "2");
    assert_eq!(traced.prints, vec!["f[1]", "Plus[1, 1]", "2", "f[1]", "2"]);
    assert!(session.take_trace().is_empty());
}

#[test]
fn timing_and_absolute_timing_return_elapsed_and_value() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[Timing[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[Timing[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[AbsoluteTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[AbsoluteTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[RepeatedTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[RepeatedTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
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
fn byte_count_uses_the_current_canonical_text_proxy_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ByteCount[\"abc\"]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("ByteCount[1 + 1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
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

#[test]
fn sow_and_reap_collect_by_tag_and_optional_handler() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Reap[Sow[1]]").unwrap().value.to_string(),
        "List[1, List[List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], a]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], b]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], _, List]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[a, List[1]]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], {_Symbol}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[List[1]]]]"
    );
}

#[test]
fn time_and_memory_constraints_cover_current_first_pass_resource_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("TimeConstrained[Pause[1/100]; 7, 1/1000, timeout]")
            .unwrap()
            .value
            .to_string(),
        "timeout"
    );
    assert_eq!(
        session
            .eval_text("MemoryConstrained[Range[100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
    assert_eq!(
        session
            .eval_text("IntegerQ[MemoryInUse[]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn monitor_runs_current_display_and_callback_subset() {
    let mut session = Session::new();

    let outcome = session
        .eval_text(
            "Reap[Monitor[1 + 1, progress, {StepMonitor :> Sow[step], EvaluationMonitor :> Sow[eval]}]]",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[2, List[List[step, eval]]]");
    assert_eq!(outcome.prints, vec!["progress"]);
}

#[test]
fn evaluation_data_captures_result_messages_prints_and_failure_type_without_leaking_streams() {
    let mut session = Session::new();

    let success = session.eval_text("EvaluationData[Echo[1 + 1]; 3]").unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("OutputLog")],
            ))
            .unwrap()
            .to_string(),
        "List[\"2\"]"
    );
    assert!(session.take_messages().is_empty());
    assert!(session.take_prints().is_empty());

    let failure = session.eval_text("EvaluationData[Abort[]]").unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![failure.value.clone(), Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"Abort\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![failure.value, Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "$Aborted"
    );
}

#[test]
fn testcreate_testreport_and_stack_cover_current_first_pass_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Head[TestCreate[1 + 1, 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"TestsRunCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"TestsSucceededCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"TestsFailedCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    let stack = session
        .eval_text("stackFrame[] := Stack[]; stackFrame[]")
        .unwrap();
    let rendered = stack.value.to_string();
    assert!(rendered.contains("Hold[stackFrame[]]"));
}
