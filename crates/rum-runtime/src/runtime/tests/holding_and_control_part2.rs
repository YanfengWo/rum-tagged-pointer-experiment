use super::*;

#[test]
fn function_applies_slots_positionally() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Function[Slot[1]][Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Function[Plus[Slot[1], Slot[2]]][1, 2]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Function[x, x][Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Function[{x, y}, Hold[x, y]][Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1, 2]"
    );
}

#[test]
fn binary_function_syntax_applies_explicit_parameter() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("(x |-> x + 1)[5]")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
}

#[test]
fn explicit_parameter_functions_ignore_extra_arguments() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Function[e, e + 1][42, 99]")
            .unwrap()
            .value
            .to_string(),
        "43"
    );
    assert_eq!(
        session
            .eval_text("Function[{x}, x^2][5, 6, 7]")
            .unwrap()
            .value
            .to_string(),
        "25"
    );
    assert_eq!(
        session
            .eval_text("Function[x, x][1, 2, 3]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Catch[Throw[42, foo], foo, Function[e, {\"h\", e}]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"h\", 42]"
    );
}

#[test]
fn pure_function_syntax_uses_slots_and_evaluated_arguments() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("(#1 &)[Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("(#2 &)[10, Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("(f[##] &)[1, Identity[2], 3]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("(f[##2] &)[10, Identity[20], 30]")
            .unwrap()
            .value
            .to_string(),
        "f[20, 30]"
    );
    assert_eq!(
        session
            .eval_text("(Hold[##] &)[1, 2]")
            .unwrap()
            .value
            .to_string(),
        "Hold[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("(f[#x, #y] &)[x -> 1, y -> Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("(f[#x, #1] &)[10, x -> 20]")
            .unwrap()
            .value
            .to_string(),
        "f[20, 10]"
    );
    assert_eq!(
        session
            .eval_text("(Hold[#x] &)[\"x\" -> Identity[3]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[3]"
    );
}

#[test]
fn explicit_parameter_functions_respect_shadowing_and_pattern_locals() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Function[x, Function[x, x]][1]")
            .unwrap()
            .value
            .to_string(),
        "Function[x, x]"
    );
    assert_eq!(
        session
            .eval_text("Function[x, 7 /. x_Integer :> x][1]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Function[{x, y}, y][1, 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn head_evaluation_can_activate_special_forms_and_held_builtins() {
    let mut session = Session::new();

    session.eval_text("assign = Set").unwrap();
    assert_eq!(
        session
            .eval_text("assign[x, Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

    session.eval_text("repl = ReplaceAll").unwrap();
    assert_eq!(
        session
            .eval_text("repl[f[1], f[x_Integer] -> x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    session.eval_text("hold = Hold").unwrap();
    assert_eq!(
        session
            .eval_text("hold[Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1]]"
    );

    session.eval_text("chooser = If").unwrap();
    assert_eq!(
        session
            .eval_text("chooser[True, Identity[1], x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    session.eval_text("threader = Thread").unwrap();
    assert_eq!(
        session
            .eval_text("threader[f[{a, b}, c]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c], f[b, c]]"
    );
}

#[test]
fn dispatch_plan_uses_builtin_metadata_and_head_traits() {
    let mut session = Session::new();

    let set_plan = session.build_call_dispatch_plan(&system_expr("Set"));
    assert_eq!(set_plan.stages(), [CallDispatchStage::SpecialForm]);

    let thread_plan = session.build_call_dispatch_plan(&system_expr("Thread"));
    assert_eq!(
        thread_plan.stages(),
        [
            CallDispatchStage::PrepareArguments,
            CallDispatchStage::OrdinaryBuiltin,
        ]
    );

    let plus_plan = session.build_call_dispatch_plan(&system_expr("Plus"));
    assert_eq!(
        plus_plan.stages(),
        [
            CallDispatchStage::PrepareArguments,
            CallDispatchStage::Listable,
            CallDispatchStage::OrdinaryBuiltin,
        ]
    );

    let function_head = Expr::call(system_expr("Function"), vec![system_expr("Null")]);
    let function_plan = session.build_call_dispatch_plan(&function_head);
    assert_eq!(
        function_plan.stages(),
        [
            CallDispatchStage::PrepareArguments,
            CallDispatchStage::Function,
        ]
    );
}

#[test]
fn cycle_detection_stops_self_referential_definitions_and_recovers_after_error() {
    let mut session = Session::new();

    session.eval_text("loopA := loopB").unwrap();
    session.eval_text("loopB := loopA").unwrap();

    let error = session
        .eval_text("loopA")
        .expect_err("expected cycle detection error");

    assert!(matches!(
        error,
        EvalError::EvaluationCycleDetected { ref expr } if expr == "loopA"
    ));
    assert_eq!(session.eval_text("1").unwrap().value.to_string(), "1");
}

#[test]
fn cycle_detection_stops_mutually_recursive_downvalues() {
    let mut session = Session::new();

    session.eval_text("f[x_Integer] := g[x]").unwrap();
    session.eval_text("g[x_Integer] := f[x]").unwrap();

    let error = session
        .eval_text("f[1]")
        .expect_err("expected cycle detection error");

    assert!(matches!(
        error,
        EvalError::EvaluationCycleDetected { ref expr } if expr == "f[1]"
    ));
}

#[test]
fn cycle_detection_allows_same_call_after_state_progress() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("n = 2; f[] := If[n > 0, n--; f[], 7]; f[]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("n = 2; g[x_] := If[n > 0, n--; g[x], x]; g[7]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn recursion_limit_stops_expanding_recursive_downvalues() {
    let mut session = Session::new();
    session.set_recursion_limit(32);

    session.eval_text("grow[x_] := grow[f[x]]").unwrap();

    let outcome = session.eval_text("grow[1]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );
}

#[test]
fn fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("f[f = Hold, Identity[1]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Hold, 1]"
    );
}

#[test]
fn evaluation_pass_limit_bounds_fixed_point_revisits() {
    let mut session = Session::new();
    session.set_evaluation_pass_limit(1);

    let outcome = session.eval_text("f[f = Hold, Identity[1]]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );
    assert_eq!(session.resource_limits().evaluation_pass_limit, 1);
}

#[test]
fn tracing_records_definition_rule_and_evaluation_events() {
    let mut session = Session::new();
    session.enable_tracing(true);

    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f[1]").unwrap();

    let trace = session.take_trace();

    assert!(trace.iter().any(|event| matches!(
        event,
        TraceEvent::DefinitionStored {
            symbol,
            kind: DefinitionKind::DownValue,
            ..
        } if symbol == "f"
    )));
    assert!(trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input == "f[1]"
    )));
    assert!(trace.iter().any(|event| matches!(
        event,
        TraceEvent::EvaluationPass { before, after }
            if before == "f[1]" && after == "1"
    )));
    assert!(session.take_trace().is_empty());
}
