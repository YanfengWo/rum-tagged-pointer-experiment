use super::model::{EvalCapturedOutcome, TestCaseSpec, TestReportOptions};
use super::parsing::{parse_test_case, parse_test_case_exprs, parse_test_report_objects};
use super::*;

#[derive(Clone)]
struct TestReportEvalState {
    head: Expr,
    raw_args: Vec<Expr>,
    tests: Vec<Expr>,
    properties: Option<Expr>,
    options: TestReportOptions,
}

#[derive(Clone)]
struct TestReportCaseEvalState {
    report: TestReportEvalState,
    index: usize,
    successes: usize,
    failures: usize,
    rendered_results: Vec<Expr>,
    test_case: TestCaseSpec,
    expected_messages: Option<Vec<Expr>>,
}

#[derive(Clone)]
struct TestReportCaseOutputState {
    case: TestReportCaseEvalState,
    evaluation: EvalCapturedOutcome,
    messages_match: bool,
}

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

        self.resume_test_report_from_index(
            TestReportEvalState {
                head: head.clone(),
                raw_args: args.to_vec(),
                tests,
                properties,
                options,
            },
            0,
            0,
            0,
            Vec::new(),
        )
    }

    fn resume_test_report_from_index(
        &mut self,
        state: TestReportEvalState,
        index: usize,
        successes: usize,
        failures: usize,
        rendered_results: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(test_expr) = state.tests.get(index) else {
            return self.finish_test_report(state, successes, failures, rendered_results);
        };

        let Some(test_case) = parse_test_case(test_expr) else {
            return Ok(Expr::call(state.head.clone(), state.raw_args.clone()));
        };
        let expected_messages =
            self.normalize_expected_message_exprs(test_case.expected_messages.clone())?;
        let evaluation_result =
            self.evaluate_test_case_input(test_case.input.clone(), &state.options);
        let case_state = TestReportCaseEvalState {
            report: state,
            index,
            successes,
            failures,
            rendered_results,
            test_case,
            expected_messages,
        };

        self.drive_internal_value::<EvalCapturedOutcome, Expr, _>(
            evaluation_result,
            move |session, evaluation| {
                session.resume_test_report_after_evaluation(case_state, evaluation)
            },
        )
    }

    fn resume_test_report_after_evaluation(
        &mut self,
        case_state: TestReportCaseEvalState,
        evaluation: EvalCapturedOutcome,
    ) -> Result<Expr, EvalError> {
        let messages_exprs: Vec<Expr> = evaluation
            .messages
            .iter()
            .cloned()
            .map(runtime_message_name_expr)
            .collect();
        let messages_match = case_state
            .expected_messages
            .as_ref()
            .map_or(messages_exprs.is_empty(), |expected| {
                *expected == messages_exprs
            });
        let output_match_result = self.test_case_output_matches(
            &case_state.test_case,
            case_state.report.options.same_test.clone(),
            evaluation.value.clone(),
            case_state.test_case.expected.clone(),
        );
        let output_state = TestReportCaseOutputState {
            case: case_state,
            evaluation,
            messages_match,
        };

        self.drive_internal_value::<bool, Expr, _>(
            output_match_result,
            move |session, output_match| {
                session.resume_test_report_after_output_match(output_state, output_match)
            },
        )
    }

    fn resume_test_report_after_output_match(
        &mut self,
        output_state: TestReportCaseOutputState,
        output_match: bool,
    ) -> Result<Expr, EvalError> {
        let TestReportCaseOutputState {
            case,
            evaluation,
            messages_match,
        } = output_state;
        let TestReportCaseEvalState {
            report,
            index,
            mut successes,
            mut failures,
            mut rendered_results,
            test_case,
            expected_messages,
        } = case;

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
                Expr::call(system_expr("List"), messages_exprs),
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
        self.resume_test_report_from_index(report, index + 1, successes, failures, rendered_results)
    }

    fn finish_test_report(
        &mut self,
        state: TestReportEvalState,
        successes: usize,
        failures: usize,
        rendered_results: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
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
            state.properties,
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
