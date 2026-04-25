use std::time::{Duration, Instant};

use super::*;

const REPEATED_TIMING_SAMPLES: usize = 3;
const DEFAULT_SHORT_LIMIT: usize = 72;

impl Session {
    pub(super) fn eval_trace(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let default_include_heads = self.system_option_bool("Trace", "Heads", false)?;

        if let [expr] = args {
            let (_value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
            return Ok(trace);
        }

        let (expr, pattern, levels, include_heads) = match args {
            [expr, pattern] => (expr, Some(pattern.clone()), None, default_include_heads),
            [expr, pattern, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr, Some(pattern.clone()), None, include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return Ok(self.invalid_call_with_arg_form(
                            "Trace",
                            args,
                            "expected a standard levelspec or Heads -> True|False in the current structured-trace subset",
                        ));
                    };
                    (
                        expr,
                        Some(pattern.clone()),
                        Some(levels),
                        default_include_heads,
                    )
                }
            }
            [expr, pattern, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Trace",
                        args,
                        "expected a standard levelspec as the third argument in the current structured-trace subset",
                    ));
                };
                let Some(include_heads) = parse_heads_option(option) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Trace",
                        args,
                        "expected Heads -> True|False as the optional fourth argument in the current structured-trace subset",
                    ));
                };
                (expr, Some(pattern.clone()), Some(levels), include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range("Trace", args, 2, 4)),
        };

        let (_value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
        self.filter_structured_trace_expr(trace, pattern, levels, include_heads)
    }

    pub(super) fn eval_trace_print(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let default_include_heads = self.system_option_bool("TracePrint", "Heads", false)?;

        if let [expr] = args {
            let (value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
            for line in render_structured_trace_lines(&trace) {
                self.push_print(line);
            }
            return Ok(value);
        }

        let (expr, pattern, levels, include_heads) = match args {
            [expr, pattern] => (expr, Some(pattern.clone()), None, default_include_heads),
            [expr, pattern, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr, Some(pattern.clone()), None, include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return Ok(self.invalid_call_with_arg_form(
                            "TracePrint",
                            args,
                            "expected a standard levelspec or Heads -> True|False in the current structured-trace subset",
                        ));
                    };
                    (
                        expr,
                        Some(pattern.clone()),
                        Some(levels),
                        default_include_heads,
                    )
                }
            }
            [expr, pattern, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "TracePrint",
                        args,
                        "expected a standard levelspec as the third argument in the current structured-trace subset",
                    ));
                };
                let Some(include_heads) = parse_heads_option(option) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "TracePrint",
                        args,
                        "expected Heads -> True|False as the optional fourth argument in the current structured-trace subset",
                    ));
                };
                (expr, Some(pattern.clone()), Some(levels), include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range("TracePrint", args, 2, 4)),
        };

        let (value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
        let trace = self.filter_structured_trace_expr(trace, pattern, levels, include_heads)?;
        for line in render_structured_trace_lines(&trace) {
            self.push_print(line);
        }
        Ok(value)
    }

    pub(super) fn eval_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_timing_like("Timing", args)
    }

    pub(super) fn eval_absolute_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_timing_like("AbsoluteTiming", args)
    }

    pub(super) fn eval_repeated_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RepeatedTiming", args, 1));
        };

        let mut best_elapsed = None;
        let mut last_value = system_expr("Null");

        for _ in 0..REPEATED_TIMING_SAMPLES {
            let started = Instant::now();
            let value = match self.evaluate(expr.clone()) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let elapsed = started.elapsed();
            best_elapsed = Some(best_elapsed.map_or(elapsed, |best: Duration| best.min(elapsed)));
            last_value = value;
        }

        Ok(Expr::call(
            system_expr("List"),
            vec![
                elapsed_duration_expr(best_elapsed.unwrap_or_default()),
                last_value,
            ],
        ))
    }

    pub(super) fn eval_short(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, limit) = match args {
            [expr] => (expr, DEFAULT_SHORT_LIMIT),
            [expr, limit_expr] => {
                let Some(limit) = expr_as_non_negative_usize(limit_expr).filter(|limit| *limit > 0)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Short",
                        args,
                        "expected the optional second argument to be a positive integer",
                    ));
                };
                (expr, limit)
            }
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        Ok(Expr::string(short_canonical_fragment(expr, limit)))
    }

    pub(super) fn eval_echo(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [value] => {
                self.push_print(render_echo_fragment(value));
                Ok(value.clone())
            }
            [value, label] => {
                self.push_print(format!(
                    "{}: {}",
                    render_echo_fragment(label),
                    render_echo_fragment(value)
                ));
                Ok(value.clone())
            }
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(super) fn eval_echo_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let body = match args {
            [] => Expr::call(system_expr("Echo"), vec![debug_slot_expr()]),
            [label] => Expr::call(system_expr("Echo"), vec![debug_slot_expr(), label.clone()]),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        Ok(Expr::call(system_expr("Function"), vec![body]))
    }

    pub(super) fn eval_echo_label(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [label] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("Echo"),
                vec![debug_slot_expr(), label.clone()],
            )],
        ))
    }

    pub(super) fn eval_byte_count(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(BigInt::from(
            expr_memory_proxy_bytes(expr) as u64
        )))
    }

    pub(super) fn eval_on(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_message_switch("On", args, true)
    }

    pub(super) fn eval_off(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_message_switch("Off", args, false)
    }

    pub(super) fn eval_sow(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (value, tag) = match args {
            [value] => (value.clone(), system_expr("None")),
            [value, tag] => (value.clone(), tag.clone()),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        let mut matched = Vec::new();
        for scope_index in 0..self.reap_scopes.len() {
            let pattern_count = self.reap_scopes[scope_index].patterns.len();
            for pattern_index in 0..pattern_count {
                let pattern = self.reap_scopes[scope_index].patterns[pattern_index].clone();
                let outcome = self.match_expr_owned(pattern, tag.clone(), BTreeMap::new())?;
                if outcome.matched {
                    matched.push((scope_index, pattern_index));
                }
            }
        }

        for (scope_index, pattern_index) in matched {
            record_sown_value(
                &mut self.reap_scopes[scope_index].buckets_by_pattern[pattern_index],
                tag.clone(),
                value.clone(),
            );
        }

        Ok(value)
    }

    pub(super) fn eval_reap(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Reap", args, 1, 3));
        }

        let (patterns, pattern_list_form) = normalize_reap_patterns(args.get(1));
        let handler = args.get(2).cloned();
        self.reap_scopes.push(ReapScope {
            buckets_by_pattern: vec![Vec::new(); patterns.len()],
            patterns,
            pattern_list_form,
            handler,
        });

        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            move |session, outcome| session.resume_reap_after_body(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_reap_after_body(outcome),
        }
    }

    pub(super) fn eval_time_constrained(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (body, limit_expr, fallback) = match args {
            [body, limit] => (body.clone(), limit, None),
            [body, limit, fallback] => (body.clone(), limit, Some(fallback.clone())),
            _ => {
                return Ok(self.invalid_call_with_arg_range("TimeConstrained", args, 2, 3));
            }
        };

        let resolved_limit = self.evaluate(limit_expr.clone())?;
        let Some(limit) = parse_time_constraint_duration(&resolved_limit) else {
            return Ok(self.invalid_call_with_arg_form(
                "TimeConstrained",
                args,
                "expected the timeout argument to be a non-negative exact number in the current first-pass subset",
            ));
        };

        let prior_deadline = self.current_time_deadline;
        let next_deadline = combine_time_deadlines(prior_deadline, limit);
        self.current_time_deadline = Some(next_deadline);

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let fallback = fallback.clone();
            move |session, outcome| {
                session.finish_time_constrained(outcome, prior_deadline, fallback)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_time_constrained(outcome, prior_deadline, fallback),
        }
    }

    pub(super) fn eval_memory_constrained(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (body, limit_expr, fallback) = match args {
            [body, limit] => (body.clone(), limit, None),
            [body, limit, fallback] => (body.clone(), limit, Some(fallback.clone())),
            _ => {
                return Ok(self.invalid_call_with_arg_range("MemoryConstrained", args, 2, 3));
            }
        };

        let resolved_limit = self.evaluate(limit_expr.clone())?;
        let Some(limit) = expr_as_non_negative_usize(&resolved_limit) else {
            return Ok(self.invalid_call_with_arg_form(
                "MemoryConstrained",
                args,
                "expected the memory limit argument to be a non-negative exact integer",
            ));
        };

        let prior_limit = self.current_memory_limit;
        self.current_memory_limit = Some(prior_limit.map_or(limit, |current| current.min(limit)));

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let fallback = fallback.clone();
            move |session, outcome| {
                session.finish_memory_constrained(outcome, prior_limit, fallback)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_memory_constrained(outcome, prior_limit, fallback),
        }
    }

    pub(super) fn eval_memory_in_use(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(Expr::integer(BigInt::from(
            self.current_memory_proxy_bytes() as u64,
        )))
    }

    pub(super) fn eval_monitor(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("Monitor", args, 2, usize::MAX));
        }

        let explicit_options = if args.len() > 2 {
            let resolved = self.evaluate(Expr::call(system_expr("List"), args[2..].to_vec()))?;
            let options = match self.compile_assigned_options_from_resolved(&resolved) {
                Ok(options) => options,
                Err(_) => {
                    return Ok(self.invalid_call_with_arg_form(
                        "Monitor",
                        args,
                        "expected the optional third argument to be a rule or list of rules in the current first-pass subset",
                    ));
                }
            };
            Some(options)
        } else {
            None
        };

        if let Some(options) = explicit_options.as_ref() {
            for (name, _, _) in options.iter() {
                if !matches!(name.name(), "EvaluationMonitor" | "StepMonitor") {
                    return Ok(self.invalid_call_with_arg_form(
                        "Monitor",
                        args,
                        "expected only EvaluationMonitor and StepMonitor option rules in the current first-pass subset",
                    ));
                }
            }
        }

        let evaluation_monitor =
            self.monitor_option_expr(explicit_options.as_ref(), "EvaluationMonitor")?;
        let step_monitor = self.monitor_option_expr(explicit_options.as_ref(), "StepMonitor")?;

        self.monitor_scopes.push(MonitorScope {
            display_expr: args[1].clone(),
            evaluation_monitor,
            step_monitor,
            last_display: None,
        });
        let last_index = self.monitor_scopes.len().saturating_sub(1);
        self.refresh_monitor_scope_display(last_index);

        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, |session, outcome| {
            session.finish_monitor(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_monitor(outcome),
        }
    }

    pub(super) fn eval_evaluation_data(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, properties) = match args {
            [expr] => (expr, None),
            [expr, properties] => (expr, Some(properties)),
            _ => return Ok(self.invalid_call_with_arg_range("EvaluationData", args, 1, 2)),
        };

        let checkpoint = DebugCaptureCheckpoint::new(self);
        let prior_tracing = self.tracing_enabled;
        self.tracing_enabled = true;
        let started = Instant::now();
        let input_string = expr.to_string();
        let input_expr = expr.clone();

        let body_result = self.evaluate(expr.clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            let input_string = input_string.clone();
            let input_expr = input_expr.clone();
            move |session, outcome| {
                session.finish_evaluation_data_capture(
                    checkpoint,
                    prior_tracing,
                    started,
                    &input_expr,
                    &input_string,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => {
                let data = self.finish_evaluation_data_capture(
                    checkpoint,
                    prior_tracing,
                    started,
                    &input_expr,
                    &input_string,
                    outcome,
                )?;
                self.project_evaluation_data_properties(data, properties.cloned())
            }
        }
    }

    pub(super) fn eval_test_create(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (input, expected, options_expr) = match args {
            [input, expected] => (input.clone(), expected.clone(), None),
            [input, expected, third] => (input.clone(), expected.clone(), Some(third.clone())),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        let mut test_id = None;
        let mut expected_messages = None;
        let mut same_test = None;

        if let Some(options_expr) = options_expr {
            if let Some(options) = parse_test_create_options(&options_expr) {
                for (name, value) in options {
                    match name.as_str() {
                        "TestID" => test_id = Some(value),
                        "ExpectedMessages" => expected_messages = Some(value),
                        "SameTest" => same_test = Some(value),
                        _ => return Ok(Expr::call(head.clone(), args.to_vec())),
                    }
                }
            } else {
                test_id = Some(options_expr);
            }
        }

        let mut entries = vec![
            (
                Expr::string("Input"),
                Expr::call(system_expr("Hold"), vec![input]),
            ),
            (
                Expr::string("ExpectedOutput"),
                Expr::call(system_expr("Hold"), vec![expected]),
            ),
        ];

        if let Some(test_id) = test_id {
            entries.push((Expr::string("TestID"), test_id));
        }
        if let Some(expected_messages) = expected_messages {
            entries.push((
                Expr::string("ExpectedMessages"),
                Expr::call(system_expr("Hold"), vec![expected_messages]),
            ));
        }
        if let Some(same_test) = same_test {
            entries.push((Expr::string("SameTest"), same_test));
        }

        Ok(render_association(entries))
    }

    pub(super) fn eval_test_report(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (tests_expr, properties, options) = match args {
            [tests_expr] => (tests_expr, None, None),
            [tests_expr, second] => {
                if let Some(options) = self.parse_test_report_options(second)? {
                    (tests_expr, None, Some(options))
                } else {
                    (tests_expr, Some(second.clone()), None)
                }
            }
            [tests_expr, properties, options_expr] => {
                let Some(options) = self.parse_test_report_options(options_expr)? else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                (tests_expr, Some(properties.clone()), Some(options))
            }
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        if let Some(report) = self.merge_test_report_objects(tests_expr) {
            return self.project_association_properties(
                "TestReport",
                &[Expr::string("<omitted>")],
                report,
                properties,
            );
        }

        let tests = match parse_test_case_exprs(tests_expr) {
            Some(tests) => tests,
            None => return Ok(Expr::call(head.clone(), args.to_vec())),
        };
        let options = options.unwrap_or_default();

        let mut successes = 0usize;
        let mut failures = 0usize;
        let mut rendered_results = Vec::with_capacity(tests.len());

        for test_expr in tests {
            let Some(test_case) = parse_test_case(&test_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            let expected_messages =
                self.normalize_expected_message_exprs(test_case.expected_messages.clone())?;

            let evaluation = self.evaluate_test_case_input(test_case.input.clone(), &options)?;
            let actual = evaluation.value;
            let failure_type = evaluation.failure_type.map(str::to_string);
            let messages_exprs: Vec<Expr> = evaluation
                .messages
                .iter()
                .cloned()
                .map(runtime_message_name_expr)
                .collect();
            let messages_text: Vec<Expr> = evaluation
                .messages
                .iter()
                .map(|message| Expr::string(message.text.clone()))
                .collect();
            let output_log: Vec<Expr> = evaluation
                .prints
                .iter()
                .cloned()
                .map(Expr::string)
                .collect();

            let messages_match = expected_messages
                .as_ref()
                .map_or(messages_exprs.is_empty(), |expected| {
                    *expected == messages_exprs
                });
            let output_match = self.test_case_output_matches(
                &test_case,
                options.same_test.clone(),
                actual.clone(),
                test_case.expected.clone(),
            )?;
            let success = output_match && messages_match && failure_type.is_none();
            if success {
                successes += 1;
            } else {
                failures += 1;
            }

            let mut entries = vec![
                (
                    Expr::string("Input"),
                    Expr::call(system_expr("Hold"), vec![test_case.input.clone()]),
                ),
                (Expr::string("Success"), debug_bool_expr(success)),
                (Expr::string("ActualOutput"), actual),
                (
                    Expr::string("ExpectedOutput"),
                    Expr::call(system_expr("Hold"), vec![test_case.expected]),
                ),
                (
                    Expr::string("ActualMessages"),
                    Expr::call(system_expr("List"), messages_exprs.clone()),
                ),
                (
                    Expr::string("ActualMessagesExpressions"),
                    Expr::call(
                        system_expr("List"),
                        evaluation
                            .messages
                            .into_iter()
                            .map(runtime_message_emit_expr)
                            .collect(),
                    ),
                ),
                (
                    Expr::string("ActualMessagesText"),
                    Expr::call(system_expr("List"), messages_text),
                ),
                (
                    Expr::string("ExpectedMessages"),
                    Expr::call(
                        system_expr("List"),
                        expected_messages.clone().unwrap_or_default(),
                    ),
                ),
                (
                    Expr::string("OutputLog"),
                    Expr::call(system_expr("List"), output_log),
                ),
                (Expr::string("Timing"), evaluation.timing),
                (
                    Expr::string("MessagesMatch"),
                    debug_bool_expr(messages_match),
                ),
                (
                    Expr::string("Outcome"),
                    Expr::string(if success { "Success" } else { "Failure" }),
                ),
            ];

            if let Some(failure_type) = failure_type {
                entries.push((Expr::string("FailureType"), Expr::string(failure_type)));
            }
            if let Some(test_id) = test_case.test_id {
                entries.push((Expr::string("TestID"), test_id));
            }

            rendered_results.push(render_association(entries));
        }

        let report = render_association(vec![
            (Expr::string("Success"), debug_bool_expr(failures == 0)),
            (
                Expr::string("TestsRunCount"),
                Expr::integer(BigInt::from(rendered_results.len() as u64)),
            ),
            (
                Expr::string("TestsSucceededCount"),
                Expr::integer(BigInt::from(successes as u64)),
            ),
            (
                Expr::string("TestsFailedCount"),
                Expr::integer(BigInt::from(failures as u64)),
            ),
            (
                Expr::string("TestResults"),
                Expr::call(system_expr("List"), rendered_results),
            ),
        ]);

        self.project_association_properties(
            "TestReport",
            &[Expr::string("<omitted>")],
            report,
            properties,
        )
    }

    pub(super) fn eval_stack(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Stack", args, 0));
        }

        let items = self
            .evaluation_stack
            .iter()
            .take(self.evaluation_stack.len().saturating_sub(1))
            .cloned()
            .map(|expr| Expr::call(system_expr("Hold"), vec![expr]))
            .collect();

        Ok(Expr::call(system_expr("List"), items))
    }

    fn eval_timing_like(&mut self, symbol: &str, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(symbol, args, 1));
        };

        let started = Instant::now();
        let evaluated = self.evaluate(expr.clone());
        let value = self.suspend_with_frame(evaluated, {
            let symbol = symbol.to_string();
            move |session, value| session.finish_timing_like(&symbol, started, value)
        })?;

        self.finish_timing_like(symbol, started, value)
    }

    fn finish_timing_like(
        &mut self,
        _symbol: &str,
        started: Instant,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("List"),
            vec![elapsed_duration_expr(started.elapsed()), value],
        ))
    }

    fn eval_with_structured_trace_capture(
        &mut self,
        expr: Expr,
    ) -> Result<(Expr, Expr), EvalError> {
        let (value, events, depths) = self.capture_trace_events(expr.clone())?;
        Ok((
            value,
            self.structured_trace_expr_from_events(&expr, &events, &depths),
        ))
    }

    fn capture_trace_events(
        &mut self,
        expr: Expr,
    ) -> Result<(Expr, Vec<TraceEvent>, Vec<usize>), EvalError> {
        let prior_enabled = self.tracing_enabled;
        let trace_checkpoint = self.trace_events.len();
        self.tracing_enabled = true;

        let value = match self.evaluate(expr) {
            Ok(value) => value,
            Err(error) => {
                self.restore_trace_capture_on_error(trace_checkpoint, prior_enabled);
                return Err(error);
            }
        };

        self.finish_trace_event_capture(trace_checkpoint, prior_enabled, value)
    }

    fn finish_trace_event_capture(
        &mut self,
        trace_checkpoint: usize,
        prior_enabled: bool,
        value: Expr,
    ) -> Result<(Expr, Vec<TraceEvent>, Vec<usize>), EvalError> {
        let events = self.trace_events[trace_checkpoint..].to_vec();
        let depths = self.trace_event_depths[trace_checkpoint..].to_vec();
        self.tracing_enabled = prior_enabled;

        if !prior_enabled {
            self.truncate_trace_from(trace_checkpoint);
        }

        Ok((value, events, depths))
    }

    fn restore_trace_capture_on_error(&mut self, trace_checkpoint: usize, prior_enabled: bool) {
        self.tracing_enabled = prior_enabled;
        if !prior_enabled {
            self.truncate_trace_from(trace_checkpoint);
        }
    }

    fn truncate_trace_from(&mut self, trace_checkpoint: usize) {
        self.trace_events.truncate(trace_checkpoint);
        self.trace_event_depths.truncate(trace_checkpoint);
    }

    fn structured_trace_expr_from_events(
        &self,
        original: &Expr,
        events: &[TraceEvent],
        depths: &[usize],
    ) -> Expr {
        let mut builder = StructuredTraceBuilder::new();
        builder.push_expr(0, original.clone());
        let mut active_exprs = vec![original.clone()];

        for (event, raw_depth) in events.iter().zip(depths.iter().copied()) {
            match event {
                TraceEvent::EvaluationPass { before, after } => {
                    let before_expr = self.trace_expr_from_text(before);
                    let after_expr = self.trace_expr_from_text(after);
                    let depth =
                        trace_expr_path_depth(&active_exprs, &before_expr).unwrap_or(raw_depth);
                    active_exprs.truncate(depth + 1);
                    builder.push_evaluation_pass(depth, before_expr, after_expr.clone());
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = after_expr;
                }
                TraceEvent::DefinitionStored { value, .. } => {
                    let depth = raw_depth;
                    builder.push_expr(depth, self.trace_expr_from_text(value));
                }
                TraceEvent::RuleApplied { input, result, .. } => {
                    let input_expr = self.trace_expr_from_text(input);
                    let result_expr = self.trace_expr_from_text(result);
                    let depth = raw_depth;
                    active_exprs.truncate(depth + 1);
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = input_expr.clone();
                    builder.push_expr(depth, input_expr);
                    builder.push_expr(depth.saturating_add(1), result_expr.clone());
                    if active_exprs.len() <= depth + 1 {
                        active_exprs.push(result_expr);
                    } else {
                        active_exprs.truncate(depth + 2);
                        active_exprs[depth + 1] = result_expr;
                    }
                }
            }
        }

        builder.finish()
    }

    fn trace_expr_from_text(&self, text: &str) -> Expr {
        self.lower_text(text)
            .unwrap_or_else(|_| Expr::string(text.to_string()))
    }

    fn filter_structured_trace_expr(
        &mut self,
        trace: Expr,
        pattern: Option<Expr>,
        levels: Option<CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = trace else {
            return Ok(trace);
        };
        if !is_system_symbol_name(&head, "List") {
            return Ok(Expr::call(head.as_ref().clone(), args));
        }

        let filtered = self.filter_structured_trace_items(
            args,
            pattern.as_ref(),
            levels.as_ref(),
            include_heads,
        )?;
        Ok(Expr::call(system_expr("List"), filtered))
    }

    fn filter_structured_trace_items(
        &mut self,
        items: Vec<Expr>,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let mut filtered = Vec::new();

        for item in items {
            match item {
                Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                    let nested =
                        self.filter_structured_trace_items(args, pattern, levels, include_heads)?;
                    if !nested.is_empty() {
                        if nested.len() == 1 {
                            if let Expr::Call { head, args } = &nested[0] {
                                if is_system_symbol_name(head, "List") {
                                    filtered.push(Expr::call(system_expr("List"), args.clone()));
                                    continue;
                                }
                            }
                        }
                        filtered.push(Expr::call(system_expr("List"), nested));
                    }
                }
                other => {
                    for held in
                        self.filter_structured_trace_leaf(other, pattern, levels, include_heads)?
                    {
                        push_unique_trace_item(&mut filtered, held);
                    }
                }
            }
        }

        Ok(filtered)
    }

    fn filter_structured_trace_leaf(
        &mut self,
        item: Expr,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let Some(expr) = unwrap_trace_hold(&item) else {
            return Ok(vec![item]);
        };

        match (pattern, levels) {
            (Some(pattern), Some(levels)) => {
                let patterns = normalize_trace_patterns(pattern);
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };
                let mut matched = Vec::new();

                for candidate in collect_level_parts_owned(&expr, traversal, 0) {
                    let mut is_match = false;
                    for pattern in &patterns {
                        let outcome = self.match_expr_owned(
                            pattern.clone(),
                            candidate.clone(),
                            BTreeMap::new(),
                        )?;
                        if outcome.matched {
                            is_match = true;
                            break;
                        }
                    }
                    if is_match {
                        push_unique_trace_expr(&mut matched, candidate);
                    }
                }

                Ok(matched
                    .into_iter()
                    .map(|candidate| Expr::call(system_expr("Hold"), vec![candidate]))
                    .collect())
            }
            (Some(pattern), None) => {
                if include_heads {
                    let all_levels = CasesLevelSpec::range(0, None);
                    self.filter_structured_trace_leaf(
                        Expr::call(system_expr("Hold"), vec![expr]),
                        Some(pattern),
                        Some(&all_levels),
                        true,
                    )
                } else {
                    let mut is_match = false;
                    for pattern in normalize_trace_patterns(pattern) {
                        let outcome =
                            self.match_expr_owned(pattern, expr.clone(), BTreeMap::new())?;
                        if outcome.matched {
                            is_match = true;
                            break;
                        }
                    }
                    Ok(if is_match {
                        vec![Expr::call(system_expr("Hold"), vec![expr])]
                    } else {
                        Vec::new()
                    })
                }
            }
            (None, Some(levels)) => {
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };
                let mut selected = Vec::new();
                for candidate in collect_level_parts_owned(&expr, traversal, 0) {
                    push_unique_trace_expr(&mut selected, candidate);
                }
                Ok(selected
                    .into_iter()
                    .map(|candidate| Expr::call(system_expr("Hold"), vec![candidate]))
                    .collect())
            }
            (None, None) => Ok(vec![Expr::call(system_expr("Hold"), vec![expr])]),
        }
    }

    fn eval_message_switch(
        &mut self,
        symbol: &str,
        args: &[Expr],
        enabled: bool,
    ) -> Result<Expr, EvalError> {
        let Some(selectors) = normalize_message_switch_selectors(args) else {
            return Ok(self.invalid_call_with_arg_form(
                symbol,
                args,
                "expected one or more symbols, MessageName expressions, \"symbol::tag\" strings, or lists of those",
            ));
        };

        if selectors.is_empty() {
            return Ok(self.invalid_call_with_message(
                symbol,
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        if enabled {
            self.disabled_message_selectors
                .retain(|existing| !selectors.iter().any(|selector| selector.covers(existing)));
        } else {
            for selector in selectors {
                if !self.disabled_message_selectors.contains(&selector) {
                    self.disabled_message_selectors.push(selector);
                }
            }
        }

        Ok(system_expr("Null"))
    }

    fn resume_reap_after_body(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let scope = self
            .reap_scopes
            .pop()
            .expect("reap scope should exist while finishing Reap");

        match outcome {
            Ok(value) => Ok(Expr::call(
                system_expr("List"),
                vec![value, self.render_reap_scope(scope)?],
            )),
            Err(error) => Err(error),
        }
    }

    fn render_reap_scope(&mut self, scope: ReapScope) -> Result<Expr, EvalError> {
        if scope.pattern_list_form {
            let mut rendered = Vec::with_capacity(scope.buckets_by_pattern.len());
            for buckets in scope.buckets_by_pattern {
                rendered.push(self.render_reap_buckets(&scope.handler, buckets)?);
            }
            Ok(Expr::call(system_expr("List"), rendered))
        } else {
            let buckets = scope
                .buckets_by_pattern
                .into_iter()
                .next()
                .unwrap_or_default();
            self.render_reap_buckets(&scope.handler, buckets)
        }
    }

    fn render_reap_buckets(
        &mut self,
        handler: &Option<Expr>,
        buckets: Vec<ReapBucket>,
    ) -> Result<Expr, EvalError> {
        let mut rendered = Vec::with_capacity(buckets.len());
        for bucket in buckets {
            let values_expr = Expr::call(system_expr("List"), bucket.values);
            let group = match handler {
                Some(handler) => {
                    let applied = Expr::call(handler.clone(), vec![bucket.tag, values_expr]);
                    self.evaluate(applied)?
                }
                None => values_expr,
            };
            rendered.push(group);
        }
        Ok(Expr::call(system_expr("List"), rendered))
    }

    pub(super) fn enforce_active_resource_limits_for_expr(
        &self,
        expr: &Expr,
    ) -> Result<(), EvalError> {
        if let Some(deadline) = self.current_time_deadline {
            if Instant::now() >= deadline {
                return Err(EvalError::TimeConstraintExceeded);
            }
        }

        if let Some(limit) = self.current_memory_limit {
            let projected = self
                .current_memory_proxy_bytes()
                .saturating_add(expr_memory_proxy_bytes(expr));
            if projected > limit {
                return Err(EvalError::MemoryConstraintExceeded { limit });
            }
        }

        Ok(())
    }

    fn current_memory_proxy_bytes(&self) -> usize {
        let mut total = std::mem::size_of::<Session>();

        total = total.saturating_add(
            self.builtins
                .iter()
                .map(|(name, _)| {
                    name.capacity()
                        .saturating_add(std::mem::size_of::<BuiltinSpec>())
                })
                .sum::<usize>(),
        );

        total = total.saturating_add(self.current_context.capacity());
        total = total.saturating_add(render_string_vec_memory_proxy_bytes(&self.context_path));
        total = total.saturating_add(
            self.context_stack
                .iter()
                .map(render_context_frame_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.context_stack)),
        );
        total = total.saturating_add(render_path_buf_vec_memory_proxy_bytes(
            &self.package_search_paths,
        ));
        total = total.saturating_add(render_string_vec_memory_proxy_bytes(&self.loaded_packages));
        total = total.saturating_add(render_path_buf_vec_memory_proxy_bytes(&self.load_stack));

        total = total.saturating_add(
            self.evaluation_stack
                .iter()
                .map(expr_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.evaluation_stack)),
        );

        total = total.saturating_add(
            self.definitions
                .iter()
                .map(|(symbol, definition)| {
                    symbol.to_string().len() + render_definition_memory_proxy_bytes(definition)
                })
                .sum::<usize>(),
        );

        total = total.saturating_add(
            self.attributes
                .keys()
                .map(|symbol| symbol.to_string().len())
                .sum::<usize>(),
        );

        total = total.saturating_add(
            self.options
                .iter()
                .map(|(symbol, options)| {
                    symbol.to_string().len() + render_symbol_options_memory_proxy_bytes(options)
                })
                .sum::<usize>(),
        );

        total = total.saturating_add(
            self.syntax_information
                .iter()
                .map(|(symbol, expr)| symbol.to_string().len() + expr_memory_proxy_bytes(expr))
                .sum::<usize>(),
        );

        total = total.saturating_add(
            self.declared_stubs
                .iter()
                .map(|symbol| symbol.to_string().len())
                .sum::<usize>(),
        );

        total = total.saturating_add(
            self.once_values
                .iter()
                .map(|(key, value)| key.capacity() + expr_memory_proxy_bytes(value))
                .sum::<usize>(),
        );
        total = total.saturating_add(
            self.option_value_scopes
                .iter()
                .map(render_option_value_scope_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.option_value_scopes)),
        );

        total = total.saturating_add(
            self.disabled_message_selectors
                .iter()
                .map(render_message_selector_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.disabled_message_selectors)),
        );

        total = total.saturating_add(
            self.message_definitions
                .iter()
                .map(|(key, definition)| {
                    key.symbol.to_string().len()
                        + key.tag.len()
                        + definition
                            .immediate
                            .as_ref()
                            .map_or(0, expr_memory_proxy_bytes)
                        + definition
                            .delayed
                            .as_ref()
                            .map_or(0, expr_memory_proxy_bytes)
                })
                .sum::<usize>(),
        );

        total = total.saturating_add(
            self.messages
                .iter()
                .map(render_runtime_message_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.messages)),
        );

        total = total.saturating_add(
            self.last_message_list
                .iter()
                .map(render_runtime_message_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.last_message_list)),
        );

        total = total.saturating_add(render_string_vec_memory_proxy_bytes(&self.prints));

        total = total.saturating_add(
            self.trace_events
                .iter()
                .map(render_trace_event_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.trace_events)),
        );
        total = total.saturating_add(
            self.trace_event_depths
                .capacity()
                .saturating_mul(std::mem::size_of::<usize>()),
        );

        total = total.saturating_add(
            self.handled_non_loader_eval_errors
                .iter()
                .map(render_runtime_message_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(
                    &self.handled_non_loader_eval_errors,
                )),
        );

        total = total.saturating_add(
            self.reap_scopes
                .iter()
                .map(render_reap_scope_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.reap_scopes)),
        );

        total = total.saturating_add(
            self.monitor_scopes
                .iter()
                .map(render_monitor_scope_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.monitor_scopes)),
        );

        total = total.saturating_add(render_string_vecdeque_memory_proxy_bytes(
            &self.queued_input_responses,
        ));
        total = total.saturating_add(
            self.pending_input_request
                .as_ref()
                .map_or(0, render_input_request_memory_proxy_bytes),
        );
        total = total.saturating_add(
            self.pending_eval_request
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<PendingEvalRequest>()),
        );
        total = total.saturating_add(
            self.pending_suspension
                .as_ref()
                .map_or(0, render_suspended_evaluation_memory_proxy_bytes),
        );
        total = total.saturating_add(std::mem::size_of::<ResourceLimits>());

        total
    }

    fn finish_time_constrained(
        &mut self,
        outcome: Result<Expr, EvalError>,
        prior_deadline: Option<Instant>,
        fallback: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.current_time_deadline = prior_deadline;

        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::TimeConstraintExceeded) => match fallback {
                Some(expr) => self.evaluate(expr),
                None => Ok(system_expr("$Aborted")),
            },
            Err(error) => Err(error),
        }
    }

    fn finish_memory_constrained(
        &mut self,
        outcome: Result<Expr, EvalError>,
        prior_limit: Option<usize>,
        fallback: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.current_memory_limit = prior_limit;

        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::MemoryConstraintExceeded { .. }) => match fallback {
                Some(expr) => self.evaluate(expr),
                None => Ok(system_expr("$Aborted")),
            },
            Err(error) => Err(error),
        }
    }

    fn monitor_option_expr(
        &mut self,
        explicit: Option<&SymbolOptions>,
        name: &str,
    ) -> Result<Option<Expr>, EvalError> {
        let resolved =
            self.runtime_option_expr_for_symbol(explicit, &Symbol::system("Monitor"), name)?;

        Ok(match resolved {
            Some(expr) if is_system_symbol_name(&expr, "None") => None,
            other => other,
        })
    }

    fn finish_monitor(&mut self, outcome: Result<Expr, EvalError>) -> Result<Expr, EvalError> {
        let last_index = self.monitor_scopes.len().saturating_sub(1);
        self.refresh_monitor_scope_display(last_index);
        let _ = self.monitor_scopes.pop();
        outcome
    }

    fn finish_evaluation_data_capture(
        &mut self,
        checkpoint: DebugCaptureCheckpoint,
        prior_tracing: bool,
        started: Instant,
        input_expr: &Expr,
        input_string: &str,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.tracing_enabled = prior_tracing;

        if let Err(error) = &outcome {
            if Self::terminated_evaluation_expr_for_error(error).is_some() {
                self.emit_non_loader_eval_message(checkpoint.handled_error_checkpoint, error);
            }
        }

        let messages = self.messages.split_off(checkpoint.message_checkpoint);
        let prints = self.prints.split_off(checkpoint.print_checkpoint);
        let events = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        let depths = self.trace_event_depths[checkpoint.trace_checkpoint..].to_vec();
        let trace_root = self.structured_trace_expr_from_events(input_expr, &events, &depths);

        if !prior_tracing {
            self.truncate_trace_from(checkpoint.trace_checkpoint);
        }
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        let elapsed = elapsed_duration_expr(started.elapsed());
        let (result_expr, success, failure_type) = match outcome {
            Ok(value) => (value, true, system_expr("None")),
            Err(error) if Self::terminated_evaluation_expr_for_error(&error).is_some() => (
                Self::terminated_evaluation_expr_for_error(&error)
                    .expect("termination value should exist after guard"),
                false,
                Expr::string(
                    Self::terminated_evaluation_reason_for_error(&error)
                        .expect("termination reason should exist after guard"),
                ),
            ),
            Err(EvalError::Abort | EvalError::ProtectedAbort) => {
                (system_expr("$Aborted"), false, Expr::string("Abort"))
            }
            Err(EvalError::TimeConstraintExceeded) => (
                system_expr("$Aborted"),
                false,
                Expr::string("TimeConstraint"),
            ),
            Err(EvalError::MemoryConstraintExceeded { .. }) => (
                system_expr("$Aborted"),
                false,
                Expr::string("MemoryConstraint"),
            ),
            Err(EvalError::Break) => (system_expr("$Failed"), false, Expr::string("Break")),
            Err(EvalError::Continue) => (system_expr("$Failed"), false, Expr::string("Continue")),
            Err(EvalError::Return { .. }) => {
                (system_expr("$Failed"), false, Expr::string("Return"))
            }
            Err(EvalError::Thrown { .. }) => (system_expr("$Failed"), false, Expr::string("Throw")),
            Err(EvalError::InputRequired { request }) => {
                return Err(EvalError::InputRequired { request });
            }
            Err(_) => (system_expr("$Failed"), false, Expr::string("Error")),
        };

        Ok(render_association(vec![
            (
                Expr::string("Input"),
                Expr::call(system_expr("Hold"), vec![input_expr.clone()]),
            ),
            (Expr::string("Result"), result_expr),
            (Expr::string("Success"), debug_bool_expr(success)),
            (Expr::string("FailureType"), failure_type),
            (
                Expr::string("Messages"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .cloned()
                        .map(runtime_message_name_expr)
                        .collect(),
                ),
            ),
            (
                Expr::string("MessagesExpressions"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .cloned()
                        .map(runtime_message_emit_expr)
                        .collect(),
                ),
            ),
            (
                Expr::string("MessagesText"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .map(|message| Expr::string(message.text.clone()))
                        .collect(),
                ),
            ),
            (
                Expr::string("OutputLog"),
                Expr::call(
                    system_expr("List"),
                    prints.into_iter().map(Expr::string).collect(),
                ),
            ),
            (Expr::string("Trace"), trace_root),
            (Expr::string("Timing"), elapsed.clone()),
            (Expr::string("AbsoluteTiming"), elapsed),
            (
                Expr::string("InputString"),
                Expr::string(input_string.to_string()),
            ),
        ]))
    }

    pub(super) fn handle_monitor_trace_event(&mut self, event: &TraceEvent) {
        if self.monitor_scopes.is_empty() || !self.monitor_callbacks_enabled {
            return;
        }

        let scopes = self.monitor_scopes.clone();
        self.monitor_callbacks_enabled = false;

        for (index, scope) in scopes.into_iter().enumerate() {
            self.refresh_monitor_scope_display(index);

            if let Some(callback) = scope.step_monitor {
                self.run_monitor_callback_best_effort(callback);
            }

            if matches!(event, TraceEvent::EvaluationPass { .. }) {
                if let Some(callback) = scope.evaluation_monitor {
                    self.run_monitor_callback_best_effort(callback);
                }
            }
        }

        self.monitor_callbacks_enabled = true;
    }

    fn run_monitor_callback_best_effort(&mut self, callback: Expr) {
        match self.evaluate(callback) {
            Ok(_) => {}
            Err(EvalError::InputRequired { .. }) => self.clear_suspension_state(),
            Err(_) => {}
        }
    }

    fn refresh_monitor_scope_display(&mut self, index: usize) {
        let Some(display_expr) = self
            .monitor_scopes
            .get(index)
            .map(|scope| scope.display_expr.clone())
        else {
            return;
        };

        match self.evaluate(display_expr) {
            Ok(value) => {
                let rendered = render_echo_fragment(&value);
                let should_emit = self
                    .monitor_scopes
                    .get(index)
                    .and_then(|current| current.last_display.as_ref())
                    .map_or(true, |current| current != &rendered);

                if should_emit {
                    self.push_print(rendered.clone());
                }

                if let Some(current) = self.monitor_scopes.get_mut(index) {
                    current.last_display = Some(rendered);
                }
            }
            Err(EvalError::InputRequired { .. }) => self.clear_suspension_state(),
            Err(_) => {}
        }
    }

    fn project_evaluation_data_properties(
        &mut self,
        data: Expr,
        properties: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.project_association_properties(
            "EvaluationData",
            &[Expr::string("<omitted>")],
            data,
            properties,
        )
    }

    fn render_test_outcome(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<(Expr, Option<&'static str>), EvalError> {
        match outcome {
            Ok(value) => Ok((value, None)),
            Err(error) if Self::terminated_evaluation_expr_for_error(&error).is_some() => Ok((
                Self::terminated_evaluation_expr_for_error(&error)
                    .expect("termination value should exist after guard"),
                Self::terminated_evaluation_reason_for_error(&error),
            )),
            Err(EvalError::Abort | EvalError::ProtectedAbort) => {
                Ok((system_expr("$Aborted"), Some("Abort")))
            }
            Err(EvalError::TimeConstraintExceeded) => {
                Ok((system_expr("$Aborted"), Some("TimeConstraint")))
            }
            Err(EvalError::MemoryConstraintExceeded { .. }) => {
                Ok((system_expr("$Aborted"), Some("MemoryConstraint")))
            }
            Err(EvalError::Break) => Ok((system_expr("$Failed"), Some("Break"))),
            Err(EvalError::Continue) => Ok((system_expr("$Failed"), Some("Continue"))),
            Err(EvalError::Return { .. }) => Ok((system_expr("$Failed"), Some("Return"))),
            Err(EvalError::Thrown { .. }) => Ok((system_expr("$Failed"), Some("Throw"))),
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            Err(_) => Ok((system_expr("$Failed"), Some("Error"))),
        }
    }

    fn test_case_output_matches(
        &mut self,
        test_case: &TestCaseSpec,
        report_same_test: Option<Expr>,
        actual: Expr,
        expected: Expr,
    ) -> Result<bool, EvalError> {
        let same_test = test_case
            .same_test
            .clone()
            .or(report_same_test)
            .unwrap_or_else(|| system_expr("SameQ"));
        let applied = Expr::call(same_test, vec![actual, expected]);
        let resolved = self.evaluate(applied)?;
        Ok(is_system_symbol_name(&resolved, "True"))
    }

    fn normalize_expected_message_exprs(
        &mut self,
        expected: Option<Vec<Expr>>,
    ) -> Result<Option<Vec<Expr>>, EvalError> {
        let Some(expected) = expected else {
            return Ok(None);
        };

        let mut normalized = Vec::with_capacity(expected.len());
        for expr in expected {
            let key = match self.normalize_message_emit_target(&expr) {
                Ok(key) => key,
                Err(_) => return Ok(None),
            };
            normalized.push(runtime_message_name_expr(RuntimeMessage {
                symbol: key.symbol.to_string(),
                tag: key.tag,
                text: String::new(),
            }));
        }
        Ok(Some(normalized))
    }

    fn project_association_properties(
        &mut self,
        head_name: &str,
        call_prefix: &[Expr],
        data: Expr,
        properties: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(properties) = properties else {
            return Ok(data);
        };

        if is_system_symbol_name(&properties, "All") {
            return Ok(data);
        }

        match property_name_string(&properties) {
            Some(key) if key == "Properties" => Ok(render_association_properties(&data)),
            Some(key) => Ok(
                association_property_by_name(&data, &key).unwrap_or_else(|| {
                    Expr::call(
                        system_expr("Missing"),
                        vec![Expr::string("KeyAbsent"), properties.clone()],
                    )
                }),
            ),
            None => {
                let Some(keys) = list_items(&properties) else {
                    let mut call = call_prefix.to_vec();
                    call.push(properties.clone());
                    return Ok(self.invalid_call_with_arg_form(
                        head_name,
                        &call,
                        "expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols",
                    ));
                };
                let mut entries = Vec::new();
                for key_expr in keys {
                    let Some(key) = property_name_string(key_expr) else {
                        let mut call = call_prefix.to_vec();
                        call.push(properties.clone());
                        return Ok(self.invalid_call_with_arg_form(
                            head_name,
                            &call,
                            "expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols",
                        ));
                    };
                    let value = association_property_by_name(&data, &key).unwrap_or_else(|| {
                        Expr::call(
                            system_expr("Missing"),
                            vec![Expr::string("KeyAbsent"), key_expr.clone()],
                        )
                    });
                    entries.push((Expr::string(key.clone()), value));
                }
                Ok(render_association(entries))
            }
        }
    }

    fn parse_test_report_options(
        &mut self,
        expr: &Expr,
    ) -> Result<Option<TestReportOptions>, EvalError> {
        let options = match self.compile_assigned_options_from_resolved(expr) {
            Ok(options) => options,
            Err(_) => return Ok(None),
        };

        for (name, _, _) in options.iter() {
            if !matches!(
                name.name(),
                "SameTest" | "TimeConstraint" | "MemoryConstraint"
            ) {
                return Ok(None);
            }
        }

        let same_test = if let Some(value) = self.runtime_option_expr_for_symbol(
            Some(&options),
            &Symbol::system("TestReport"),
            "SameTest",
        )? {
            value
        } else {
            system_expr("SameQ")
        };

        let time_constraint = self.test_report_constraint_expr(Some(&options), "TimeConstraint")?;
        let memory_constraint =
            self.test_report_constraint_expr(Some(&options), "MemoryConstraint")?;

        Ok(Some(TestReportOptions {
            same_test: Some(same_test),
            time_constraint,
            memory_constraint,
        }))
    }

    fn test_report_constraint_expr(
        &mut self,
        explicit: Option<&SymbolOptions>,
        option_name: &str,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(resolved) = self.runtime_option_expr_for_symbol(
            explicit,
            &Symbol::system("TestReport"),
            option_name,
        )?
        else {
            return Ok(None);
        };

        Ok((!is_system_symbol_name(&resolved, "Infinity")).then_some(resolved))
    }

    fn evaluate_test_case_input(
        &mut self,
        input: Expr,
        options: &TestReportOptions,
    ) -> Result<EvalCapturedOutcome, EvalError> {
        let wrapped = wrap_test_case_with_constraints(input, options);
        let started = Instant::now();
        let outcome = self.eval_expr_with_stream_capture(wrapped)?;
        let elapsed = elapsed_duration_expr(started.elapsed());

        if outcome.value == test_report_time_constraint_sentinel() {
            return Ok(EvalCapturedOutcome {
                value: system_expr("$Aborted"),
                failure_type: Some("TimeConstraint"),
                messages: outcome.messages,
                prints: outcome.prints,
                timing: elapsed,
            });
        }

        if outcome.value == test_report_memory_constraint_sentinel() {
            return Ok(EvalCapturedOutcome {
                value: system_expr("$Aborted"),
                failure_type: Some("MemoryConstraint"),
                messages: outcome.messages,
                prints: outcome.prints,
                timing: elapsed,
            });
        }

        Ok(EvalCapturedOutcome {
            value: outcome.value,
            failure_type: None,
            messages: outcome.messages,
            prints: outcome.prints,
            timing: elapsed,
        })
    }

    fn eval_expr_with_stream_capture(
        &mut self,
        expr: Expr,
    ) -> Result<EvalCapturedOutcome, EvalError> {
        let started = Instant::now();
        let message_checkpoint = self.messages.len();
        let print_checkpoint = self.prints.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let evaluated = self.evaluate(expr);
        let result = self.finalize_top_level_control_flow(evaluated);

        match result {
            Ok(value) => {
                let messages = self.messages.split_off(message_checkpoint);
                let prints = self.prints.split_off(print_checkpoint);
                self.last_message_list = messages.clone();
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                Ok(EvalCapturedOutcome {
                    value,
                    failure_type: None,
                    messages,
                    prints,
                    timing: elapsed_duration_expr(started.elapsed()),
                })
            }
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                let messages = self.messages[message_checkpoint..].to_vec();
                let prints = self.prints[print_checkpoint..].to_vec();
                self.last_message_list = messages.clone();
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);

                let (value, failure_type) = self.render_test_outcome(Err(err))?;
                Ok(EvalCapturedOutcome {
                    value,
                    failure_type,
                    messages,
                    prints,
                    timing: elapsed_duration_expr(started.elapsed()),
                })
            }
        }
    }

    fn merge_test_report_objects(&self, expr: &Expr) -> Option<Expr> {
        let reports = parse_test_report_objects(expr)?;
        let mut tests_run = 0usize;
        let mut tests_succeeded = 0usize;
        let mut tests_failed = 0usize;
        let mut merged_results = Vec::new();

        for report in reports {
            tests_run = tests_run.saturating_add(report.tests_run);
            tests_succeeded = tests_succeeded.saturating_add(report.tests_succeeded);
            tests_failed = tests_failed.saturating_add(report.tests_failed);
            merged_results.extend(report.test_results);
        }

        Some(render_association(vec![
            (Expr::string("Success"), debug_bool_expr(tests_failed == 0)),
            (
                Expr::string("TestsRunCount"),
                Expr::integer(BigInt::from(tests_run as u64)),
            ),
            (
                Expr::string("TestsSucceededCount"),
                Expr::integer(BigInt::from(tests_succeeded as u64)),
            ),
            (
                Expr::string("TestsFailedCount"),
                Expr::integer(BigInt::from(tests_failed as u64)),
            ),
            (
                Expr::string("TestResults"),
                Expr::call(system_expr("List"), merged_results),
            ),
        ]))
    }
}

fn debug_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn property_name_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Atom(Atom::String(name)) => Some(name.clone()),
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name().to_string()),
        _ => None,
    }
}

fn render_echo_fragment(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        other => other.to_string(),
    }
}

fn expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(value)) => {
            std::mem::size_of::<Expr>().saturating_add(bigint_memory_proxy_bytes(value))
        }
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(bigint_memory_proxy_bytes(value.numerator()))
            .saturating_add(bigint_memory_proxy_bytes(value.denominator())),
        Expr::Atom(Atom::String(value)) => {
            std::mem::size_of::<Expr>().saturating_add(value.capacity())
        }
        Expr::Atom(Atom::Symbol(symbol)) => {
            std::mem::size_of::<Expr>().saturating_add(symbol.context().len() + symbol.name().len())
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "ByteArray") && args.len() == 1 =>
        {
            let payload_len = args
                .first()
                .and_then(expr_as_string)
                .and_then(|encoded| base64_decoded_len(&encoded))
                .unwrap_or_else(|| args.iter().map(expr_memory_proxy_bytes).sum::<usize>());
            std::mem::size_of::<Expr>()
                .saturating_add(expr_memory_proxy_bytes(head))
                .saturating_add(
                    args.capacity()
                        .saturating_sub(args.len())
                        .saturating_mul(std::mem::size_of::<Expr>()),
                )
                .saturating_add(payload_len)
        }
        Expr::Call { head, args } => {
            let mut total = std::mem::size_of::<Expr>()
                .saturating_add(expr_memory_proxy_bytes(head))
                .saturating_add(
                    args.capacity()
                        .saturating_sub(args.len())
                        .saturating_mul(std::mem::size_of::<Expr>()),
                );
            for arg in args {
                total = total.saturating_add(expr_memory_proxy_bytes(arg));
            }
            total
        }
    }
}

fn debug_bool_expr(value: bool) -> Expr {
    if value {
        system_expr("True")
    } else {
        system_expr("False")
    }
}

fn short_canonical_fragment(expr: &Expr, limit: usize) -> String {
    let rendered = render_echo_fragment(expr);
    if rendered.len() <= limit || limit <= 8 {
        return rendered;
    }

    let prefix_len = limit / 2;
    let suffix_len = limit.saturating_sub(prefix_len + 3);
    let prefix = rendered.chars().take(prefix_len).collect::<String>();
    let suffix = rendered
        .chars()
        .rev()
        .take(suffix_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{prefix}...{suffix}")
}

fn elapsed_duration_expr(duration: Duration) -> Expr {
    let numerator = BigInt::from(duration.as_nanos());
    let denominator = BigInt::from(1_000_000_000_u64);
    let rational = Rational::new(numerator, denominator)
        .expect("positive denominator should always build a rational");
    Expr::rational(rational)
}

fn push_unique_trace_expr(trace: &mut Vec<Expr>, expr: Expr) {
    if trace.last().is_some_and(|current| current == &expr) {
        return;
    }
    trace.push(expr);
}

fn push_unique_trace_item(items: &mut Vec<Expr>, item: Expr) {
    if items.last().is_some_and(|current| current == &item) {
        return;
    }
    items.push(item);
}

fn unwrap_trace_hold(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Hold") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn render_structured_trace_lines(trace: &Expr) -> Vec<String> {
    let mut lines = Vec::new();
    collect_structured_trace_lines(trace, 0, &mut lines);
    let min_depth = lines.iter().map(|(depth, _)| *depth).min().unwrap_or(0);
    lines
        .into_iter()
        .map(|(depth, line)| format!("{}{}", "  ".repeat(depth.saturating_sub(min_depth)), line))
        .collect()
}

fn collect_structured_trace_lines(trace: &Expr, depth: usize, lines: &mut Vec<(usize, String)>) {
    match trace {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            for item in args {
                match item {
                    Expr::Call { head, args: _ } if is_system_symbol_name(head, "List") => {
                        collect_structured_trace_lines(item, depth + 1, lines);
                    }
                    other => {
                        if let Some(expr) = unwrap_trace_hold(other) {
                            lines.push((depth, render_echo_fragment(&expr)));
                        } else {
                            lines.push((depth, render_echo_fragment(other)));
                        }
                    }
                }
            }
        }
        other => {
            if let Some(expr) = unwrap_trace_hold(other) {
                lines.push((depth, render_echo_fragment(&expr)));
            } else {
                lines.push((depth, render_echo_fragment(other)));
            }
        }
    }
}

fn normalize_message_switch_selectors(args: &[Expr]) -> Option<Vec<MessageSelector>> {
    let mut selectors = Vec::new();

    for arg in args {
        selectors.extend(parse_message_selectors(arg)?);
    }

    Some(selectors)
}

fn normalize_trace_patterns(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => args.clone(),
        other => vec![other.clone()],
    }
}

fn normalize_reap_patterns(expr: Option<&Expr>) -> (Vec<Expr>, bool) {
    match expr {
        None => (vec![Expr::call(system_expr("Blank"), vec![])], false),
        Some(Expr::Call { head, args }) if is_system_symbol_name(head, "List") => {
            (args.clone(), true)
        }
        Some(pattern) => (vec![pattern.clone()], false),
    }
}

fn record_sown_value(buckets: &mut Vec<ReapBucket>, tag: Expr, value: Expr) {
    if let Some(bucket) = buckets.iter_mut().find(|bucket| bucket.tag == tag) {
        bucket.values.push(value);
        return;
    }

    buckets.push(ReapBucket {
        tag,
        values: vec![value],
    });
}

fn parse_time_constraint_duration(expr: &Expr) -> Option<Duration> {
    let rational = expr_as_exact_rational(expr)?;
    if rational.numerator() < &BigInt::from(0) {
        return None;
    }

    let nanos_numerator = rational.numerator().clone() * BigInt::from(1_000_000_000_u64);
    let nanos = nanos_numerator / rational.denominator().clone();
    let nanos = nanos
        .to_string()
        .parse::<u64>()
        .unwrap_or(u64::MAX.saturating_sub(1));

    Some(Duration::from_nanos(nanos))
}

fn combine_time_deadlines(prior: Option<Instant>, limit: Duration) -> Instant {
    let candidate = Instant::now()
        .checked_add(limit)
        .unwrap_or_else(Instant::now);
    prior.map_or(candidate, |current| current.min(candidate))
}

fn render_definition_memory_proxy_bytes(definition: &SymbolDefinition) -> usize {
    std::mem::size_of::<SymbolDefinition>()
        .saturating_add(
            definition
                .own_value
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(
            definition
                .delayed_own_value
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(
            definition
                .up_values
                .iter()
                .map(render_upvalue_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(
            definition
                .down_values
                .iter()
                .map(render_downvalue_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(
            definition
                .sub_values
                .iter()
                .map(render_subvalue_memory_proxy_bytes)
                .sum::<usize>(),
        )
}

fn render_upvalue_memory_proxy_bytes(value: &UpValue) -> usize {
    std::mem::size_of::<UpValue>()
        .saturating_add(expr_memory_proxy_bytes(&value.lhs))
        .saturating_add(expr_memory_proxy_bytes(&value.rhs))
}

fn render_downvalue_memory_proxy_bytes(value: &DownValue) -> usize {
    std::mem::size_of::<DownValue>()
        .saturating_add(expr_memory_proxy_bytes(&value.lhs))
        .saturating_add(expr_memory_proxy_bytes(&value.rhs))
}

fn render_subvalue_memory_proxy_bytes(value: &SubValue) -> usize {
    std::mem::size_of::<SubValue>()
        .saturating_add(expr_memory_proxy_bytes(&value.lhs))
        .saturating_add(expr_memory_proxy_bytes(&value.rhs))
}

fn render_symbol_options_memory_proxy_bytes(options: &SymbolOptions) -> usize {
    std::mem::size_of::<SymbolOptions>().saturating_add(
        options
            .iter()
            .map(|(name, value, _)| {
                std::mem::size_of::<SymbolOptionEntry>().saturating_add(
                    name.to_string()
                        .len()
                        .saturating_add(expr_memory_proxy_bytes(value)),
                )
            })
            .sum::<usize>()
            .saturating_add(
                options
                    .capacity()
                    .saturating_sub(options.len())
                    .saturating_mul(std::mem::size_of::<SymbolOptionEntry>()),
            ),
    )
}

fn render_context_frame_memory_proxy_bytes(frame: &ContextFrame) -> usize {
    std::mem::size_of::<ContextFrame>()
        .saturating_add(frame.previous_context.capacity().saturating_add(
            render_string_vec_memory_proxy_bytes(&frame.previous_context_path),
        ))
        .saturating_add(frame.opened_context.capacity())
}

fn render_message_selector_memory_proxy_bytes(selector: &MessageSelector) -> usize {
    std::mem::size_of::<MessageSelector>().saturating_add(
        selector
            .symbol
            .capacity()
            .saturating_add(selector.tag.as_ref().map_or(0, String::capacity)),
    )
}

fn render_runtime_message_memory_proxy_bytes(message: &RuntimeMessage) -> usize {
    std::mem::size_of::<RuntimeMessage>().saturating_add(
        message
            .symbol
            .capacity()
            .saturating_add(message.tag.capacity())
            .saturating_add(message.text.capacity()),
    )
}

fn render_reap_scope_memory_proxy_bytes(scope: &ReapScope) -> usize {
    let patterns = scope
        .patterns
        .iter()
        .map(expr_memory_proxy_bytes)
        .sum::<usize>();
    let handler = scope.handler.as_ref().map_or(0, expr_memory_proxy_bytes);
    let buckets = scope
        .buckets_by_pattern
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|bucket| {
                    expr_memory_proxy_bytes(&bucket.tag).saturating_add(
                        bucket
                            .values
                            .iter()
                            .map(expr_memory_proxy_bytes)
                            .sum::<usize>(),
                    )
                })
                .sum::<usize>()
        })
        .sum::<usize>();

    std::mem::size_of::<ReapScope>()
        .saturating_add(patterns)
        .saturating_add(handler)
        .saturating_add(buckets)
}

fn render_monitor_scope_memory_proxy_bytes(scope: &MonitorScope) -> usize {
    std::mem::size_of::<MonitorScope>()
        .saturating_add(expr_memory_proxy_bytes(&scope.display_expr))
        .saturating_add(
            scope
                .evaluation_monitor
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(
            scope
                .step_monitor
                .as_ref()
                .map_or(0, expr_memory_proxy_bytes),
        )
        .saturating_add(scope.last_display.as_ref().map_or(0, String::capacity))
}

fn render_trace_event_memory_proxy_bytes(event: &TraceEvent) -> usize {
    match event {
        TraceEvent::EvaluationPass { before, after } => std::mem::size_of::<TraceEvent>()
            .saturating_add(before.capacity())
            .saturating_add(after.capacity()),
        TraceEvent::DefinitionStored { symbol, value, .. } => std::mem::size_of::<TraceEvent>()
            .saturating_add(symbol.capacity())
            .saturating_add(value.capacity()),
        TraceEvent::RuleApplied {
            input,
            rule,
            result,
            ..
        } => std::mem::size_of::<TraceEvent>().saturating_add(
            input
                .capacity()
                .saturating_add(rule.capacity())
                .saturating_add(result.capacity()),
        ),
    }
}

fn bigint_memory_proxy_bytes(value: &BigInt) -> usize {
    value.to_signed_bytes_le().len().max(1)
}

fn base64_decoded_len(encoded: &str) -> Option<usize> {
    if encoded.is_empty() {
        return Some(0);
    }
    if encoded.len() % 4 != 0 {
        return None;
    }

    let mut padding = 0usize;
    for (index, byte) in encoded.as_bytes().iter().copied().enumerate() {
        let is_padding = byte == b'=';
        let is_base64 = matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/'
        );
        if !is_padding && !is_base64 {
            return None;
        }
        if is_padding {
            padding += 1;
            if index < encoded.len().saturating_sub(2) {
                return None;
            }
        } else if padding != 0 {
            return None;
        }
    }

    if padding > 2 {
        return None;
    }

    Some(encoded.len() / 4 * 3usize - padding)
}

fn render_input_request_memory_proxy_bytes(request: &RuntimeInputRequest) -> usize {
    std::mem::size_of::<RuntimeInputRequest>()
        .saturating_add(request.prompt.capacity())
        .saturating_add(1)
}

fn render_suspended_evaluation_memory_proxy_bytes(suspended: &SuspendedEvaluation) -> usize {
    std::mem::size_of::<SuspendedEvaluation>()
        .saturating_add(
            suspended
                .active_stack
                .iter()
                .map(expr_memory_proxy_bytes)
                .sum::<usize>(),
        )
        .saturating_add(suspended.frames.len().saturating_mul(16))
}

fn render_option_value_scope_memory_proxy_bytes(scope: &OptionValueScope) -> usize {
    std::mem::size_of::<OptionValueScope>()
        .saturating_add(scope.target().to_string().len())
        .saturating_add(render_symbol_options_memory_proxy_bytes(scope.explicit()))
}

fn vec_spare_capacity_bytes<T>(values: &Vec<T>) -> usize {
    values
        .capacity()
        .saturating_sub(values.len())
        .saturating_mul(std::mem::size_of::<T>())
}

fn render_string_vec_memory_proxy_bytes(values: &Vec<String>) -> usize {
    values
        .iter()
        .map(String::capacity)
        .sum::<usize>()
        .saturating_add(
            values
                .capacity()
                .saturating_mul(std::mem::size_of::<String>()),
        )
}

fn render_path_buf_vec_memory_proxy_bytes(values: &Vec<PathBuf>) -> usize {
    values
        .iter()
        .map(|path| path.as_os_str().len())
        .sum::<usize>()
        .saturating_add(
            values
                .capacity()
                .saturating_mul(std::mem::size_of::<PathBuf>()),
        )
}

fn render_string_vecdeque_memory_proxy_bytes(values: &VecDeque<String>) -> usize {
    values
        .iter()
        .map(String::capacity)
        .sum::<usize>()
        .saturating_add(
            values
                .capacity()
                .saturating_mul(std::mem::size_of::<String>()),
        )
}

fn render_association_properties(expr: &Expr) -> Expr {
    let properties = association_entries_from_expr(expr)
        .map(|entries| {
            entries
                .iter()
                .map(|(key, _)| match key {
                    Expr::Atom(Atom::String(text)) => Expr::string(text.clone()),
                    other => Expr::string(other.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();
    Expr::call(system_expr("List"), properties)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DebugCaptureCheckpoint {
    message_checkpoint: usize,
    print_checkpoint: usize,
    trace_checkpoint: usize,
    handled_error_checkpoint: usize,
}

impl DebugCaptureCheckpoint {
    fn new(session: &Session) -> Self {
        Self {
            message_checkpoint: session.messages.len(),
            print_checkpoint: session.prints.len(),
            trace_checkpoint: session.trace_events.len(),
            handled_error_checkpoint: session.handled_non_loader_eval_errors.len(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct StructuredTraceBuilder {
    containers: Vec<Vec<Expr>>,
    current_exprs: Vec<Option<Expr>>,
}

impl StructuredTraceBuilder {
    fn new() -> Self {
        Self {
            containers: vec![Vec::new()],
            current_exprs: Vec::new(),
        }
    }

    fn push_expr(&mut self, depth: usize, expr: Expr) {
        self.close_to_depth(depth);

        while self.containers.len() <= depth {
            self.containers.push(Vec::new());
        }
        if self.current_exprs.len() <= depth {
            self.current_exprs.resize(depth + 1, None);
        }

        let duplicate = self.current_exprs[depth]
            .as_ref()
            .is_some_and(|current| current == &expr);
        self.current_exprs.truncate(depth + 1);
        if duplicate {
            return;
        }

        self.containers[depth].push(Expr::call(system_expr("Hold"), vec![expr.clone()]));
        self.current_exprs[depth] = Some(expr);
    }

    fn push_evaluation_pass(&mut self, depth: usize, before: Expr, after: Expr) {
        self.close_to_depth(depth);

        if !self.container_root_matches(depth, &before) {
            self.push_expr(depth, before);
        }
        self.push_expr(depth, after);
    }

    fn finish(mut self) -> Expr {
        self.close_to_depth(0);
        Expr::call(
            system_expr("List"),
            self.containers.pop().unwrap_or_default(),
        )
    }

    fn close_to_depth(&mut self, depth: usize) {
        while self.containers.len() > depth + 1 {
            let child = self.containers.pop().unwrap_or_default();
            if let Some(parent) = self.containers.last_mut() {
                parent.push(Expr::call(system_expr("List"), child));
            }
        }
    }

    fn container_root_matches(&self, depth: usize, target: &Expr) -> bool {
        self.containers.get(depth).and_then(|items| {
            items.iter().find_map(|item| match item {
                Expr::Call { head, args } if is_system_symbol_name(head, "Hold") => {
                    args.first().cloned()
                }
                _ => None,
            })
        }) == Some(target.clone())
    }
}

fn trace_expr_path_depth(active_exprs: &[Expr], target: &Expr) -> Option<usize> {
    active_exprs
        .iter()
        .enumerate()
        .rev()
        .find_map(|(depth, expr)| (expr == target).then_some(depth))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TestCaseSpec {
    input: Expr,
    expected: Expr,
    test_id: Option<Expr>,
    expected_messages: Option<Vec<Expr>>,
    same_test: Option<Expr>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TestReportOptions {
    same_test: Option<Expr>,
    time_constraint: Option<Expr>,
    memory_constraint: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedTestReportObject {
    tests_run: usize,
    tests_succeeded: usize,
    tests_failed: usize,
    test_results: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EvalCapturedOutcome {
    value: Expr,
    failure_type: Option<&'static str>,
    messages: Vec<RuntimeMessage>,
    prints: Vec<String>,
    timing: Expr,
}

fn parse_test_case_exprs(expr: &Expr) -> Option<Vec<Expr>> {
    if association_entries_from_expr(expr).is_some() {
        return Some(vec![expr.clone()]);
    }

    list_items(expr).map(|items| items.to_vec())
}

fn parse_test_case(expr: &Expr) -> Option<TestCaseSpec> {
    let entries = association_entries_from_expr(expr)?;
    let mut input = None;
    let mut expected = None;
    let mut test_id = None;
    let mut expected_messages = None;
    let mut same_test = None;

    for (key, value) in entries {
        match key {
            Expr::Atom(Atom::String(name)) if name == "Input" => {
                input = unwrap_hold_complete(&value);
            }
            Expr::Atom(Atom::String(name)) if name == "ExpectedOutput" => {
                expected = unwrap_hold_complete(&value);
            }
            Expr::Atom(Atom::String(name)) if name == "TestID" => {
                test_id = Some(value);
            }
            Expr::Atom(Atom::String(name)) if name == "ExpectedMessages" => {
                expected_messages = unwrap_hold_complete(&value).map(|held| {
                    list_items(&held)
                        .map(|items| items.to_vec())
                        .unwrap_or_else(|| vec![held])
                });
            }
            Expr::Atom(Atom::String(name)) if name == "SameTest" => {
                same_test = Some(value);
            }
            _ => {}
        }
    }

    Some(TestCaseSpec {
        input: input?,
        expected: expected?,
        test_id,
        expected_messages,
        same_test,
    })
}

fn parse_test_report_objects(expr: &Expr) -> Option<Vec<ParsedTestReportObject>> {
    if let Some(report) = parse_test_report_object(expr) {
        return Some(vec![report]);
    }

    let items = list_items(expr)?;
    items.iter().map(parse_test_report_object).collect()
}

fn parse_test_report_object(expr: &Expr) -> Option<ParsedTestReportObject> {
    let tests_run = association_property_by_name(expr, "TestsRunCount")
        .and_then(|value| expr_as_non_negative_usize(&value))?;
    let tests_succeeded = association_property_by_name(expr, "TestsSucceededCount")
        .and_then(|value| expr_as_non_negative_usize(&value))?;
    let tests_failed = association_property_by_name(expr, "TestsFailedCount")
        .and_then(|value| expr_as_non_negative_usize(&value))?;
    let test_results = association_property_by_name(expr, "TestResults")
        .and_then(|value| list_items(&value).map(|items| items.to_vec()))?;

    Some(ParsedTestReportObject {
        tests_run,
        tests_succeeded,
        tests_failed,
        test_results,
    })
}

fn unwrap_hold_complete(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Hold")
                || is_system_symbol_name(head, "HoldComplete")
                || is_system_symbol_name(head, "HoldForm"))
                && args.len() == 1 =>
        {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn parse_test_create_options(expr: &Expr) -> Option<Vec<(String, Expr)>> {
    let items = if let Some(items) = list_items(expr) {
        items.to_vec()
    } else {
        vec![expr.clone()]
    };

    let mut options = Vec::new();
    for item in items {
        match item {
            Expr::Call { head, args }
                if (is_system_symbol_name(&head, "Rule")
                    || is_system_symbol_name(&head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                let name = match &args[0] {
                    Expr::Atom(Atom::Symbol(symbol)) => symbol.name().to_string(),
                    Expr::Atom(Atom::String(name)) => name.clone(),
                    _ => return None,
                };
                options.push((name, args[1].clone()));
            }
            _ => return None,
        }
    }
    Some(options)
}

fn association_property_by_name(expr: &Expr, name: &str) -> Option<Expr> {
    association_entries_from_expr(expr)?
        .into_iter()
        .find_map(|(key, value)| match key {
            Expr::Atom(Atom::String(current)) if current == name => Some(value),
            _ => None,
        })
}

fn runtime_message_emit_expr(message: RuntimeMessage) -> Expr {
    Expr::call(
        system_expr("Hold"),
        vec![Expr::call(
            system_expr("Message"),
            vec![runtime_message_name_expr(message)],
        )],
    )
}

fn wrap_test_case_with_constraints(input: Expr, options: &TestReportOptions) -> Expr {
    let mut wrapped = input;

    if let Some(limit) = options.time_constraint.clone() {
        wrapped = Expr::call(
            system_expr("TimeConstrained"),
            vec![wrapped, limit, test_report_time_constraint_sentinel()],
        );
    }

    if let Some(limit) = options.memory_constraint.clone() {
        wrapped = Expr::call(
            system_expr("MemoryConstrained"),
            vec![wrapped, limit, test_report_memory_constraint_sentinel()],
        );
    }

    wrapped
}

fn test_report_time_constraint_sentinel() -> Expr {
    Expr::call(
        system_expr("HoldComplete"),
        vec![Expr::string("__RumTestReportTimeConstraint__")],
    )
}

fn test_report_memory_constraint_sentinel() -> Expr {
    Expr::call(
        system_expr("HoldComplete"),
        vec![Expr::string("__RumTestReportMemoryConstraint__")],
    )
}
