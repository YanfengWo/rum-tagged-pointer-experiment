use super::model::{EvalCapturedOutcome, TestCaseSpec, TestReportOptions};
use super::*;

impl Session {
    pub(in crate::runtime::debug_tools::test_report) fn test_case_output_matches(
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

    pub(in crate::runtime::debug_tools::test_report) fn normalize_expected_message_exprs(
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

    pub(in crate::runtime::debug_tools::test_report) fn parse_test_report_options(
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

    pub(in crate::runtime::debug_tools::test_report) fn evaluate_test_case_input(
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
