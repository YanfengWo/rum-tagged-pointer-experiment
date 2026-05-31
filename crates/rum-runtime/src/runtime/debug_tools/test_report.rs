use super::*;

impl Session {
    pub(in crate::runtime) fn eval_test_create(
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

    pub(in crate::runtime) fn eval_test_report(
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

    pub(in crate::runtime::debug_tools) fn project_evaluation_data_properties(
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
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
                Ok((system_expr("$Failed"), Some("Throw")))
            }
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            Err(EvalError::Exit { code }) => Err(EvalError::Exit { code }),
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
                self.emit_top_level_control_flow_messages(&err);
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
