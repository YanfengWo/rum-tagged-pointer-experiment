use super::parsing::{parse_test_case, parse_test_case_exprs, parse_test_report_objects};
use super::*;

impl Session {
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
