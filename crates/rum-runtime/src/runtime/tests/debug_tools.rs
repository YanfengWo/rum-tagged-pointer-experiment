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
fn byte_count_and_memory_in_use_use_the_current_structural_proxy_subset() {
    let mut session = Session::new();

    let string_small = session.eval_text("ByteCount[\"abc\"]").unwrap().value;
    let string_large = session.eval_text("ByteCount[\"abcdefgh\"]").unwrap().value;
    let atom = session.eval_text("ByteCount[1]").unwrap().value;
    let call = session.eval_text("ByteCount[f[1, 2, 3]]").unwrap().value;
    let byte_array_small = session
        .eval_text("ByteCount[StringToByteArray[\"abc\"]]")
        .unwrap()
        .value;
    let byte_array_large = session
        .eval_text("ByteCount[StringToByteArray[\"abcdefgh\"]]")
        .unwrap()
        .value;

    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![string_small.clone(), string_large]
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![atom, call]))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![byte_array_small, byte_array_large]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let small_int = session.eval_text("ByteCount[1]").unwrap().value;
    let large_int = session.eval_text("ByteCount[10^80]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![small_int, large_int]))
            .unwrap()
            .to_string(),
        "True"
    );

    let before = session.eval_text("MemoryInUse[]").unwrap().value;
    session.eval_text("byteProbe = Table[i, {i, 200}]").unwrap();
    let after = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![before, after]))
            .unwrap()
            .to_string(),
        "True"
    );

    let before_off = session.eval_text("MemoryInUse[]").unwrap().value;
    session.eval_text("Off[FindFile::notfound]").unwrap();
    let after_off = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![before_off.clone(), after_off.clone()]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    session.eval_text("On[FindFile::notfound]").unwrap();
    let after_on = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![after_on, after_off]))
            .unwrap()
            .to_string(),
        "True"
    );

    let before_once = session.eval_text("MemoryInUse[]").unwrap().value;
    session
        .eval_text("Once[Table[i, {i, 50}], \"memory-probe\"]")
        .unwrap();
    let after_once = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![before_once, after_once]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    session
        .eval_text(
            "Options[memoryScoped] = {alpha -> 1, beta -> 2}; memoryScoped[OptionsPattern[]] := {MemoryInUse[], OptionValue[{alpha, beta}]}; memoryBaseline[] := MemoryInUse[]",
        )
        .unwrap();
    let scoped_baseline = session.eval_text("memoryBaseline[]").unwrap().value;
    let scoped_memory = session
        .eval_text("Part[memoryScoped[alpha -> 10, beta -> 20], 1]")
        .unwrap()
        .value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![scoped_baseline, scoped_memory]
            ))
            .unwrap()
            .to_string(),
        "True"
    );
}

#[test]
fn dialog_catches_return_and_respects_current_option_subset() {
    let mut session = Session::new();
    session.eval_text("x = global").unwrap();

    assert_eq!(
        session
            .eval_text("1 + Dialog[Return[2]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );

    let localized = session
        .eval_text("Dialog[x, DialogSymbols -> {x = local}, DialogProlog :> (x = start)]")
        .unwrap();
    assert_eq!(localized.value.to_string(), "start");
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "global");
}

#[test]
fn dialog_without_body_uses_a_current_single_input_line_subset() {
    let mut session = Session::new();
    session.queue_input_response("Return[2 + 3]");

    let response = session.eval_text("Dialog[]").unwrap();
    assert_eq!(response.value.to_string(), "5");
}

#[test]
fn dialog_uses_current_default_options_from_setoptions() {
    let mut session = Session::new();
    session
        .eval_text("SetOptions[Dialog, DialogProlog :> Print[\"hi\"]]")
        .unwrap();
    session.queue_input_response("Return[7]");

    let response = session.eval_text("Dialog[]").unwrap();
    assert_eq!(response.value.to_string(), "7");
    assert_eq!(response.prints, vec!["hi"]);
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

    let changing = session.eval_text("x = 0; Monitor[(x = 1; x), x]").unwrap();
    assert_eq!(changing.value.to_string(), "1");
    assert_eq!(changing.prints, vec!["0", "1"]);

    session
        .eval_text("SetOptions[Monitor, StepMonitor :> Sow[stepDefault]]")
        .unwrap();
    let defaulted = session.eval_text("Reap[Monitor[1 + 1, progress]]").unwrap();
    assert_eq!(
        defaulted.value.to_string(),
        "List[2, List[List[stepDefault]]]"
    );

    session
        .eval_text("SetOptions[Monitor, EvaluationMonitor :> Sow[evalDefault]]")
        .unwrap();
    let default_eval = session.eval_text("Reap[Monitor[1 + 1, progress]]").unwrap();
    assert_eq!(
        default_eval.value.to_string(),
        "List[2, List[List[stepDefault, evalDefault]]]"
    );

    let disabled_defaults = session
        .eval_text(
            "Reap[Monitor[1 + 1, progress, {StepMonitor :> None, EvaluationMonitor :> None}]]",
        )
        .unwrap();
    assert_eq!(disabled_defaults.value.to_string(), "List[2, List[]]");
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
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![success.value.clone(), Expr::string("Input")],
            ))
            .unwrap()
            .to_string(),
        "Hold[CompoundExpression[Echo[Plus[1, 1]], 3]]"
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

    let projected = session
        .eval_text("EvaluationData[Echo[1 + 1]; 3, {\"Result\", \"Success\"}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![projected.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![projected.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let symbol_projected = session
        .eval_text("EvaluationData[Echo[1 + 1]; 3, {Result, \"Success\"}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![symbol_projected.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![symbol_projected.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let properties = session
        .eval_text("EvaluationData[3, \"Properties\"]")
        .unwrap();
    assert!(properties.value.to_string().contains("\"Result\""));
    assert!(properties.value.to_string().contains("\"Trace\""));

    let symbol_properties = session.eval_text("EvaluationData[3, Properties]").unwrap();
    assert!(symbol_properties.value.to_string().contains("\"Result\""));
    assert!(symbol_properties.value.to_string().contains("\"Trace\""));

    let all_projection = session.eval_text("EvaluationData[3, All]").unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Head"),
                vec![all_projection.value.clone()]
            ))
            .unwrap()
            .to_string(),
        "Association"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![all_projection.value, Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "3"
    );

    session.eval_text("f[x_] := x + 1").unwrap();
    let trace_projection = session
        .eval_text("EvaluationData[f[1], \"Trace\"]")
        .unwrap();
    assert_eq!(
        trace_projection.value.to_string(),
        "List[Hold[f[1]], List[Hold[Plus[1, 1]], Hold[2]], Hold[2]]"
    );

    let message_exprs = session
        .eval_text("EvaluationData[Message[FindFile::notfound], \"MessagesExpressions\"]")
        .unwrap();
    assert_eq!(
        message_exprs.value.to_string(),
        "List[Hold[Message[FindFile::notfound]]]"
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
            .eval_text("Lookup[TestCreate[1 + 1, 2, {TestID -> \"sum\"}], \"TestID\"]")
            .unwrap()
            .value
            .to_string(),
        "\"sum\""
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
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}], \"Success\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[TestReport[{TestCreate[1 + 1, 1, {SameTest -> Greater}], TestCreate[Message[FindFile::notfound], Null, {ExpectedMessages -> {FindFile::notfound}}]}], \"TestsSucceededCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[Message[FindFile::notfound], Null, {ExpectedMessages -> {FindFile::notfound}}]}], \"TestResults\"], 1], \"MessagesMatch\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[Message[FindFile::notfound], Null, {ExpectedMessages -> {FindFile::notfound}}]}], \"TestResults\"], 1], \"ActualMessagesText\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"message emitted\"]"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[1 + 1, 2]}], \"TestResults\"], 1], \"Input\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "Hold[Plus[1, 1]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[Part[Lookup[TestReport[{TestCreate[Echo[1 + 1], 2]}], \"TestResults\"], 1], \"OutputLog\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"2\"]"
    );
    assert_eq!(
        session
            .eval_text(
                "NumberQ[Lookup[Part[Lookup[TestReport[{TestCreate[Echo[1 + 1], 2]}], \"TestResults\"], 1], \"Timing\"]]",
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(
                "TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}, \"TestsRunCount\"]",
            )
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("TestReport[{TestCreate[1 + 1, 2], TestCreate[1 + 1, 3]}, TestsRunCount]",)
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    let properties = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, \"Properties\"]")
        .unwrap();
    assert!(properties.value.to_string().contains("\"TestsRunCount\""));
    assert!(properties.value.to_string().contains("\"TestResults\""));
    let symbol_properties = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, Properties]")
        .unwrap();
    assert!(
        symbol_properties
            .value
            .to_string()
            .contains("\"TestsRunCount\"")
    );
    assert!(
        symbol_properties
            .value
            .to_string()
            .contains("\"TestResults\"")
    );
    let all_report = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, All]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Head"),
                vec![all_report.value.clone()]
            ))
            .unwrap()
            .to_string(),
        "Association"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![all_report.value, Expr::string("TestsSucceededCount")],
            ))
            .unwrap()
            .to_string(),
        "1"
    );
    let mixed_projection = session
        .eval_text("TestReport[{TestCreate[1 + 1, 2]}, {TestsRunCount, \"Success\"}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![
                    mixed_projection.value.clone(),
                    Expr::string("TestsRunCount")
                ],
            ))
            .unwrap()
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![mixed_projection.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert!(session.take_messages().is_empty());

    let stack = session
        .eval_text("stackFrame[] := Stack[]; stackFrame[]")
        .unwrap();
    let rendered = stack.value.to_string();
    assert!(rendered.contains("Hold[stackFrame[]]"));
}

#[test]
fn testreport_merges_existing_reports_and_accepts_report_level_options() {
    let mut session = Session::new();

    let merged = session
        .eval_text(
            "TestReport[{TestReport[{TestCreate[1 + 1, 2]}], TestReport[{TestCreate[2 + 2, 5]}]}]",
        )
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![merged.value.clone(), Expr::string("TestsRunCount")],
            ))
            .unwrap()
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![merged.value, Expr::string("TestsSucceededCount")],
            ))
            .unwrap()
            .to_string(),
        "1"
    );

    let same_test = session
        .eval_text("TestReport[{TestCreate[1 + 1, 1]}, {SameTest -> Greater}]")
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![same_test.value, Expr::string("TestsSucceededCount")],
            ))
            .unwrap()
            .to_string(),
        "1"
    );

    let projected = session
        .eval_text(
            "TestReport[{TestCreate[1 + 1, 1]}, \"TestsSucceededCount\", {SameTest -> Greater}]",
        )
        .unwrap();
    assert_eq!(projected.value.to_string(), "1");

    let time_limited = session
        .eval_text(
            "Part[Lookup[TestReport[{TestCreate[Pause[1/50], 1]}, {TimeConstraint -> 1/100}], \"TestResults\"], 1]",
        )
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![time_limited.value.clone(), Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"TimeConstraint\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![time_limited.value, Expr::string("Outcome")],
            ))
            .unwrap()
            .to_string(),
        "\"Failure\""
    );

    let memory_limited = session
        .eval_text(
            "Part[Lookup[TestReport[{TestCreate[Range[100], Range[100]]}, {MemoryConstraint -> 20}], \"TestResults\"], 1]",
        )
        .unwrap();
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![memory_limited.value.clone(), Expr::string("FailureType")],
            ))
            .unwrap()
            .to_string(),
        "\"MemoryConstraint\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![memory_limited.value, Expr::string("Outcome")],
            ))
            .unwrap()
            .to_string(),
        "\"Failure\""
    );
}
