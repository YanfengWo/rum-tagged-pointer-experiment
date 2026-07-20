use super::*;

#[test]
fn function_applies_slots_positionally() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
fn subvalues_hold_all_holds_outer_arguments_and_evaluate_pierces() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("c = 0; op[a_][b_] := HoldComplete[a, b]; {op[c = c + 1][c = c + 1], c}",)
            .unwrap()
            .value
            .to_string(),
        "List[HoldComplete[1, 2], 2]"
    );

    // Set the attribute after the first application to exercise cache invalidation.
    assert_eq!(
        session
            .eval_text("c = 0; SetAttributes[op, SubValuesHoldAll]; {op[c = c + 1][c = c + 1], c}",)
            .unwrap()
            .value
            .to_string(),
        "List[HoldComplete[1, Set[c, Plus[1, c]]], 1]"
    );

    assert_eq!(
        session
            .eval_text("c = 0; {op[c = c + 1][Evaluate[c = c + 1]], c}")
            .unwrap()
            .value
            .to_string(),
        "List[HoldComplete[1, 2], 2]"
    );
}

#[test]
fn subvalues_hold_all_flattens_sequence_like_hold_all() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "SetAttributes[h, HoldAll]; h[x___] := HoldComplete[x]; h[Sequence[1 + 1, 2 + 2]]",
            )
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[Plus[1, 1], Plus[2, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "SetAttributes[op, SubValuesHoldAll]; op[a_][b___] := HoldComplete[a, b]; op[0][Sequence[1 + 1, 2 + 2]]",
            )
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[0, Plus[1, 1], Plus[2, 2]]"
    );
}

#[test]
fn subvalues_hold_all_strips_and_restores_unevaluated() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "SetAttributes[f, SubValuesHoldAll]; SetAttributes[op, SubValuesHoldAll]; op[a_][b_] := HoldComplete[a, b]; {f[0][Unevaluated[1 + 1]], op[0][Unevaluated[1 + 1]]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[f[0][Unevaluated[Plus[1, 1]]], HoldComplete[0, Plus[1, 1]]]"
    );
}

#[test]
fn subvalues_hold_all_applies_at_every_application_level() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "c = 0; SetAttributes[op, SubValuesHoldAll]; op[a_][b_][d_] := HoldComplete[a, b, d]; {op[c = c + 1][c = c + 1][c = c + 1], c}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[HoldComplete[1, Set[c, Plus[1, c]], Set[c, Plus[1, c]]], 1]"
    );
}

#[test]
fn subvalues_hold_all_attribute_round_trips() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text(
                "SetAttributes[f, SubValuesHoldAll]; {Attributes[f], ClearAttributes[f, SubValuesHoldAll], Attributes[f]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[SubValuesHoldAll], Null, List[]]"
    );
}

#[test]
fn cycle_detection_returns_iteration_limit_for_mutual_ownvalues_and_recovers() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("loopA := loopB").unwrap();
    session.eval_text("loopB := loopA").unwrap();

    let outcome = session
        .eval_text("loopA")
        .expect("cycle detection should return a termination value");

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
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "cycle")
    );
    assert_eq!(session.eval_text("1").unwrap().value.to_string(), "1");
}

#[test]
fn cycle_detection_returns_iteration_limit_for_mutual_downvalues() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("f[x_Integer] := g[x]").unwrap();
    session.eval_text("g[x_Integer] := f[x]").unwrap();

    let outcome = session
        .eval_text("f[1]")
        .expect("cycle detection should return a termination value");

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
}

#[test]
fn recursion_limit_stops_expanding_recursive_downvalues() {
    let mut session = session_with_test_control_builtins();
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
fn native_stack_limit_stops_unbounded_definition_recursion_and_session_stays_usable() {
    let mut session = session_with_test_control_builtins();
    session.set_native_stack_limit(512 * 1024);

    let outcome = session
        .eval_text("f[n_] := f[n + 1]; f[0]")
        .expect("stack limit should return a termination value");

    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"StackLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
    assert_eq!(session.eval_text("1 + 1").unwrap().value.to_string(), "2");
}

#[test]
fn bounded_recursive_definition_below_native_stack_limit_succeeds() {
    let mut session = session_with_test_control_builtins();
    let depth = 48;
    let mut nested = "done".to_string();
    for _ in 0..depth {
        nested = format!("wrap[{nested}]");
    }

    session.eval_text("peel[done] := done").unwrap();
    session.eval_text("peel[wrap[x_]] := peel[x]").unwrap();

    let outcome = session
        .eval_text(&format!("peel[{nested}]"))
        .expect("bounded recursion should complete");

    assert_eq!(outcome.value.to_string(), "done");
}

#[test]
fn fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning() {
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();
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
    let mut session = session_with_test_control_builtins();
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
        } if input.to_string() == "f[1]"
    )));
    assert!(trace.iter().any(|event| matches!(
        event,
        TraceEvent::EvaluationPass { before, after }
            if before.to_string() == "f[1]" && after.to_string() == "1"
    )));
    assert!(session.take_trace().is_empty());
}

#[test]
fn sum_and_product_malformed_iterators_emit_argform_messages() {
    let detail = "expected a supported repetition count or iterator specification";

    for (source, symbol, expected) in [
        ("Sum[i, {i, 1, 3, 0}]", "Sum", "Sum[i, List[i, 1, 3, 0]]"),
        (
            "Product[i, {i, 1, 3, 0}]",
            "Product",
            "Product[i, List[i, 1, 3, 0]]",
        ),
        ("Sum[1, bad]", "Sum", "Sum[1, bad]"),
        ("Product[1, bad]", "Product", "Product[1, bad]"),
    ] {
        let mut session = session_with_test_control_builtins();
        let outcome = session.eval_text(source).unwrap();

        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(
            outcome.messages.iter().any(|message| {
                message.symbol == symbol && message.tag == "argform" && message.text == detail
            }),
            "{source} messages {:?}",
            outcome.messages
        );
    }
}

#[test]
fn sum_and_product_valid_iterators_remain_message_free() {
    for (source, expected) in [("Sum[i, {i, 1, 3}]", "6"), ("Product[i, {i, 1, 4}]", "24")] {
        let mut session = session_with_test_control_builtins();
        let outcome = session.eval_text(source).unwrap();

        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(
            outcome.messages.is_empty(),
            "{source} messages {:?}",
            outcome.messages
        );
    }
}
