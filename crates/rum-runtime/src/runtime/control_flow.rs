use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConfirmQuietSelectors {
    All,
    Exact(Vec<MessageSelector>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PropagateAbortsMode {
    Automatic,
    False,
    True,
}

impl Session {
    pub(super) fn eval_check(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Check",
                args,
                "argx",
                format!("expected 2 or 3 argument(s), got {}", args.len()),
            ));
        }

        let selectors = match self.normalize_optional_message_selectors(args.get(2)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Check", args, error.message()));
            }
        };

        let fallback = args[1].clone();
        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let primary_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(primary_result, {
            let selectors = selectors.clone();
            let fallback = fallback.clone();
            move |session, outcome| {
                session.resume_check_after_primary(
                    selectors,
                    fallback,
                    message_checkpoint,
                    handled_error_checkpoint,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_check_after_primary(
                selectors,
                fallback,
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            ),
        }
    }

    pub(super) fn eval_quiet(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Quiet",
                args,
                "argx",
                format!("expected 1 or 2 argument(s), got {}", args.len()),
            ));
        }

        let selectors = match self.normalize_optional_message_selectors(args.get(1)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Quiet", args, error.message()));
            }
        };

        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let primary_result = self.evaluate(args[0].clone());
        let result = self.suspend_with_result_frame(primary_result, {
            let selectors = selectors.clone();
            move |session, outcome| {
                session.resume_quiet_after_primary(
                    selectors,
                    message_checkpoint,
                    handled_error_checkpoint,
                    outcome,
                )
            }
        });

        match result {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_quiet_after_primary(
                selectors,
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            ),
        }
    }

    pub(super) fn eval_compound_expression(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.resume_compound_expression_from_index(args.to_vec(), 0)
    }

    pub(super) fn eval_if(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("If", args, 2, 4));
        }

        if let Some(truth) = truthiness_symbol(&args[0]) {
            return self.resume_if_after_condition(
                args.to_vec(),
                system_expr(if truth { "True" } else { "False" }),
            );
        }

        let condition_result = self.evaluate(args[0].clone());
        let condition = self.suspend_with_frame(condition_result, {
            let args = args.to_vec();
            move |session, condition| session.resume_if_after_condition(args, condition)
        })?;

        self.resume_if_after_condition(args.to_vec(), condition)
    }

    pub(super) fn eval_which(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() || args.len() % 2 != 0 {
            return Ok(self.invalid_call_with_arg_form(
                "Which",
                args,
                "expected one or more condition/result pairs",
            ));
        }

        self.resume_which_from_index(args.to_vec(), 0)
    }

    pub(super) fn eval_switch(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 3 || args.len() % 2 == 0 {
            return Ok(self.invalid_call_with_arg_form(
                "Switch",
                args,
                "expected an expression followed by one or more form/value pairs",
            ));
        }

        let subject_result = self.evaluate(args[0].clone());
        let subject = self.suspend_with_frame(subject_result, {
            let args = args.to_vec();
            move |session, subject| session.resume_switch_after_subject(args, subject)
        })?;

        self.resume_switch_after_subject(args.to_vec(), subject)
    }

    pub(super) fn eval_do(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Do",
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_do_after_spec_eval(args.to_vec(), 0)
    }

    pub(super) fn eval_while(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("While", args, 1, 2));
        }

        self.resume_while_test(args.to_vec())
    }

    pub(super) fn eval_until(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Until", args, 1, 2));
        }

        self.resume_until_body(args.to_vec())
    }

    pub(super) fn eval_for(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(3..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("For", args, 3, 4));
        }

        let start_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(start_result, {
            let args = args.to_vec();
            move |session, outcome| session.resume_for_after_start(args, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_for_after_start(args.to_vec(), outcome),
        }
    }

    pub(super) fn eval_table(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Table",
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_table_after_spec_eval(args.to_vec(), 0)
    }

    pub(super) fn eval_abort(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Abort", args, 0));
        }

        Err(EvalError::Abort)
    }

    pub(super) fn eval_check_abort(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("CheckAbort", args, 2, 3));
        }

        let propagate = match parse_check_abort_propagation(args.get(2)) {
            Ok(mode) => mode,
            Err(detail) => return Ok(self.invalid_call_with_arg_form("CheckAbort", args, detail)),
        };

        let primary_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(primary_result, {
            let args = args.to_vec();
            move |session, outcome| {
                session.resume_check_abort_after_primary(args, propagate, outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_check_abort_after_primary(args.to_vec(), propagate, outcome),
        }
    }

    pub(super) fn eval_with_cleanup(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr, cleanup] => {
                let primary_result = self.evaluate(expr.clone());
                self.resume_with_cleanup_after_primary(cleanup.clone(), primary_result)
            }
            [init, expr, cleanup] => {
                let init_result = self.evaluate(init.clone());
                let outcome = self.suspend_with_result_frame(init_result, {
                    let expr = expr.clone();
                    let cleanup = cleanup.clone();
                    move |session, outcome| {
                        session.resume_with_cleanup_after_init(expr, cleanup, outcome)
                    }
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => {
                        self.resume_with_cleanup_after_init(expr.clone(), cleanup.clone(), outcome)
                    }
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("WithCleanup", args, 2, 3)),
        }
    }

    pub(super) fn eval_failsafe(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Failsafe", args, 1, 3));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_enclose(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Enclose", args, 1, 3));
        }

        let handler = args.get(1).cloned();
        let form = args.get(2).cloned();
        let lexical_tag = if form.is_none() {
            Some(fresh_internal_enclose_tag_expr(self))
        } else {
            None
        };
        let body = lexical_tag.as_ref().map_or_else(
            || args[0].clone(),
            |tag| rewrite_untagged_confirms(args[0].clone(), tag),
        );

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let handler = handler.clone();
            let form = form.clone();
            let lexical_tag = lexical_tag.clone();
            move |session, outcome| {
                session.resume_enclose_after_body(handler, form, lexical_tag, outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_enclose_after_body(handler, form, lexical_tag, outcome),
        }
    }

    pub(super) fn eval_confirm(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Confirm", args, 1, 3));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let args = args.to_vec();
            move |session, value| session.resume_confirm_after_value(args, value)
        })?;

        self.resume_confirm_after_value(args.to_vec(), value)
    }

    pub(super) fn eval_confirm_by(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmBy", args, 2, 4));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let args = args.to_vec();
            move |session, value| session.resume_confirm_by_after_value(args, value)
        })?;

        self.resume_confirm_by_after_value(args.to_vec(), value)
    }

    pub(super) fn eval_confirm_match(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmMatch", args, 2, 4));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let args = args.to_vec();
            move |session, value| session.resume_confirm_match_after_value(args, value)
        })?;

        self.resume_confirm_match_after_value(args.to_vec(), value)
    }

    pub(super) fn eval_confirm_quiet(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmQuiet", args, 1, 4));
        }

        let selectors = match normalize_confirm_quiet_selectors(args.get(1)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("ConfirmQuiet", args, error.message()));
            }
        };

        let message_checkpoint = self.messages.len();
        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            let args = args.to_vec();
            let selectors = selectors.clone();
            move |session, outcome| {
                session.resume_confirm_quiet_after_body(
                    args,
                    selectors,
                    message_checkpoint,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_confirm_quiet_after_body(
                args.to_vec(),
                selectors,
                message_checkpoint,
                outcome,
            ),
        }
    }

    pub(super) fn eval_confirm_assert(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmAssert", args, 1, 3));
        }

        let test_result = self.evaluate(args[0].clone());
        let test = self.suspend_with_frame(test_result, {
            let args = args.to_vec();
            move |session, test| session.resume_confirm_assert_after_test(args, test)
        })?;

        self.resume_confirm_assert_after_test(args.to_vec(), test)
    }

    pub(super) fn eval_failure(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [tag, details] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Failure", args, 2));
        };

        let Some(details) = normalize_failure_details_expr(details) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(head.clone(), vec![tag.clone(), details]))
    }

    pub(super) fn eval_failure_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FailureQ", args, 1));
        };

        Ok(system_expr(if is_failure_expr(expr) {
            "True"
        } else {
            "False"
        }))
    }

    pub(super) fn eval_return(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Err(EvalError::Return {
                value: system_expr("Null"),
            }),
            [value] => Err(EvalError::Return {
                value: value.clone(),
            }),
            _ => Ok(self.invalid_call_with_arg_range("Return", args, 0, 1)),
        }
    }

    pub(super) fn eval_throw(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [value] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: system_expr("None"),
            }),
            [value, tag] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: tag.clone(),
            }),
            _ => Ok(self.invalid_call_with_arg_range("Throw", args, 1, 2)),
        }
    }

    pub(super) fn eval_break(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Break", args, 0));
        }

        Err(EvalError::Break)
    }

    pub(super) fn eval_continue(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Continue", args, 0));
        }

        Err(EvalError::Continue)
    }

    pub(super) fn eval_catch(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Catch", args, 1, 3));
        }

        let form = args.get(1).cloned();
        let handler = args.get(2).cloned();
        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            let form = form.clone();
            let handler = handler.clone();
            move |session, outcome| session.resume_catch_after_body(form, handler, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_catch_after_body(form, handler, outcome),
        }
    }

    fn resume_do_after_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = self.suspend_with_frame(spec_result, {
            let args = args.clone();
            move |session, spec_expr| session.finish_do_spec_eval(args, spec_index, spec_expr)
        })?;

        self.finish_do_spec_eval(args, spec_index, spec_expr)
    }

    fn finish_do_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(spec) = parse_do_spec(&spec_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "Do",
                &args,
                "expected a supported repetition count or iterator specification",
            ));
        };

        self.resume_do_iterations(args, spec_index, spec, 0)
    }

    fn resume_do_iterations(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
    ) -> Result<Expr, EvalError> {
        let Some(iteration_expr) = build_do_iteration_expr(&args, spec_index, &spec, iteration)
        else {
            return Ok(system_expr("Null"));
        };

        let iteration_result = self.evaluate(iteration_expr);
        let outcome = self.suspend_with_result_frame(iteration_result, {
            let args = args.clone();
            let spec = spec.clone();
            move |session, outcome| {
                session.resume_do_after_iteration(args, spec_index, spec, iteration, outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_do_after_iteration(args, spec_index, spec, iteration, outcome),
        }
    }

    fn resume_do_after_iteration(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) | Err(EvalError::Continue) => {
                self.resume_do_iterations(args, spec_index, spec, iteration + 1)
            }
            Err(EvalError::Break) => Ok(system_expr("Null")),
            Err(error) => Err(error),
        }
    }

    fn resume_while_test(&mut self, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let test_result = self.evaluate(args[0].clone());
        let test = self.suspend_with_frame(test_result, {
            let args = args.clone();
            move |session, test| session.resume_while_after_test(args, test)
        })?;

        self.resume_while_after_test(args, test)
    }

    fn resume_while_after_test(&mut self, args: Vec<Expr>, test: Expr) -> Result<Expr, EvalError> {
        if !is_system_symbol_name(&test, "True") {
            return Ok(system_expr("Null"));
        }

        let body = args.get(1).cloned().unwrap_or_else(|| system_expr("Null"));
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let args = args.clone();
            move |session, outcome| session.resume_while_after_body(args, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_while_after_body(args, outcome),
        }
    }

    fn resume_while_after_body(
        &mut self,
        args: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) | Err(EvalError::Continue) => self.resume_while_test(args),
            Err(EvalError::Break) => Ok(system_expr("Null")),
            Err(error) => Err(error),
        }
    }

    fn resume_until_body(&mut self, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let body = args.get(1).cloned().unwrap_or_else(|| system_expr("Null"));
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let args = args.clone();
            move |session, outcome| session.resume_until_after_body(args, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_until_after_body(args, outcome),
        }
    }

    fn resume_until_after_body(
        &mut self,
        args: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) | Err(EvalError::Continue) => self.resume_until_test(args),
            Err(EvalError::Break) => Ok(system_expr("Null")),
            Err(error) => Err(error),
        }
    }

    fn resume_until_test(&mut self, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let test_result = self.evaluate(args[0].clone());
        let test = self.suspend_with_frame(test_result, {
            let args = args.clone();
            move |session, test| session.resume_until_after_test(args, test)
        })?;

        self.resume_until_after_test(args, test)
    }

    fn resume_until_after_test(&mut self, args: Vec<Expr>, test: Expr) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            Ok(system_expr("Null"))
        } else {
            self.resume_until_body(args)
        }
    }

    fn resume_for_after_start(
        &mut self,
        args: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) => self.resume_for_test(args),
            Err(error) => Err(error),
        }
    }

    fn resume_for_test(&mut self, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let test_result = self.evaluate(args[1].clone());
        let test = self.suspend_with_frame(test_result, {
            let args = args.clone();
            move |session, test| session.resume_for_after_test(args, test)
        })?;

        self.resume_for_after_test(args, test)
    }

    fn resume_for_after_test(&mut self, args: Vec<Expr>, test: Expr) -> Result<Expr, EvalError> {
        if !is_system_symbol_name(&test, "True") {
            return Ok(system_expr("Null"));
        }

        let body = args.get(3).cloned().unwrap_or_else(|| system_expr("Null"));
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let args = args.clone();
            move |session, outcome| session.resume_for_after_body(args, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_for_after_body(args, outcome),
        }
    }

    fn resume_for_after_body(
        &mut self,
        args: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) | Err(EvalError::Continue) => self.resume_for_increment(args),
            Err(EvalError::Break) => Ok(system_expr("Null")),
            Err(error) => Err(error),
        }
    }

    fn resume_for_increment(&mut self, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let increment_result = self.evaluate(args[2].clone());
        let outcome = self.suspend_with_result_frame(increment_result, {
            let args = args.clone();
            move |session, outcome| session.resume_for_after_increment(args, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_for_after_increment(args, outcome),
        }
    }

    fn resume_for_after_increment(
        &mut self,
        args: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) => self.resume_for_test(args),
            Err(error) => Err(error),
        }
    }

    fn resume_table_after_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = self.suspend_with_frame(spec_result, {
            let args = args.clone();
            move |session, spec_expr| session.finish_table_spec_eval(args, spec_index, spec_expr)
        })?;

        self.finish_table_spec_eval(args, spec_index, spec_expr)
    }

    fn finish_table_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(spec) = parse_do_spec(&spec_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "Table",
                &args,
                "expected a supported repetition count or iterator specification",
            ));
        };

        self.resume_table_iterations(args, spec_index, spec, 0, Vec::new())
    }

    fn resume_table_iterations(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
        collected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(iteration_expr) = build_table_iteration_expr(&args, spec_index, &spec, iteration)
        else {
            return Ok(Expr::call(system_expr("List"), collected));
        };

        let iteration_result = self.evaluate(iteration_expr);
        let outcome = self.suspend_with_result_frame(iteration_result, {
            let args = args.clone();
            let spec = spec.clone();
            let collected = collected.clone();
            move |session, outcome| {
                session.resume_table_after_iteration(
                    args, spec_index, spec, iteration, collected, outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_table_after_iteration(
                args, spec_index, spec, iteration, collected, outcome,
            ),
        }
    }

    fn resume_table_after_iteration(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
        mut collected: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                collected.push(value);
                self.resume_table_iterations(args, spec_index, spec, iteration + 1, collected)
            }
            Err(error) => Err(error),
        }
    }

    fn resume_enclose_after_body(
        &mut self,
        handler: Option<Expr>,
        form: Option<Expr>,
        lexical_tag: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Thrown { value, tag }) if is_failure_expr(&value) => {
                self.resume_enclose_for_failure(handler, form, lexical_tag, value, tag)
            }
            Err(error) => Err(error),
        }
    }

    fn resume_enclose_for_failure(
        &mut self,
        handler: Option<Expr>,
        form: Option<Expr>,
        lexical_tag: Option<Expr>,
        value: Expr,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some(lexical_tag) = lexical_tag {
            if tag == lexical_tag {
                return self.finish_enclose_handler(handler, value);
            }

            return Err(EvalError::Thrown { value, tag });
        }

        let Some(form) = form else {
            return Err(EvalError::Thrown { value, tag });
        };

        let match_result = self.match_expr_owned(form, tag.clone(), BTreeMap::new());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.finish_enclose_handler(handler, value)
            } else {
                Err(EvalError::Thrown { value, tag })
            }
        })
    }

    fn finish_enclose_handler(
        &mut self,
        handler: Option<Expr>,
        failure: Expr,
    ) -> Result<Expr, EvalError> {
        match handler {
            None => Ok(failure),
            Some(Expr::Atom(Atom::String(property))) => {
                Ok(failure_property_value(&failure, &property))
            }
            Some(handler) => self.evaluate(Expr::call(handler, vec![failure])),
        }
    }

    fn resume_confirm_after_value(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if !is_confirmation_failure_like(&value) {
            return Ok(value);
        }

        let raw_expr = args[0].clone();
        let tag = confirmation_tag_from_args(&args, 2);
        let info_result = evaluate_optional_confirmation_info(self, args.get(1).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_failure(args, raw_expr, value, info, tag)
        })
    }

    fn finish_confirm_failure(
        &mut self,
        args: Vec<Expr>,
        raw_expr: Expr,
        value: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = if info.is_none() && is_failure_expr(&value) {
            value
        } else {
            build_confirmation_failure(vec![
                ("Information", info.unwrap_or_else(|| system_expr("Null"))),
                ("Expression", raw_expr),
            ])
        };

        let _ = args;
        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_by_after_value(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let test_call = Expr::call(args[1].clone(), vec![value.clone()]);
        let test_result = self.evaluate(test_call);
        let test = self.suspend_with_frame(test_result, {
            let args = args.clone();
            let value = value.clone();
            move |session, test| session.resume_confirm_by_after_test(args, value, test)
        })?;

        self.resume_confirm_by_after_test(args, value, test)
    }

    fn resume_confirm_by_after_test(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
        test: Expr,
    ) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            return Ok(value);
        }

        let raw_expr = args[0].clone();
        let raw_function = args[1].clone();
        let tag = confirmation_tag_from_args(&args, 3);
        let info_result = evaluate_optional_confirmation_info(self, args.get(2).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_by_failure(raw_expr, raw_function, info, tag)
        })
    }

    fn finish_confirm_by_failure(
        &mut self,
        raw_expr: Expr,
        raw_function: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Expression", raw_expr),
            ("Function", raw_function),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_match_after_value(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let raw_expr = args[0].clone();
        let raw_pattern = args[1].clone();
        let tag = confirmation_tag_from_args(&args, 3);
        let match_result =
            self.match_expr_owned(raw_pattern.clone(), value.clone(), BTreeMap::new());

        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                Ok(value)
            } else {
                let info_result =
                    evaluate_optional_confirmation_info(session, args.get(2).cloned());
                session.drive_internal_value(info_result, move |session, info| {
                    session.finish_confirm_match_failure(raw_expr, raw_pattern, info, tag)
                })
            }
        })
    }

    fn finish_confirm_match_failure(
        &mut self,
        raw_expr: Expr,
        raw_pattern: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Expression", raw_expr),
            ("Pattern", raw_pattern),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_quiet_after_body(
        &mut self,
        args: Vec<Expr>,
        selectors: ConfirmQuietSelectors,
        message_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let matched_message =
            confirm_quiet_matched_message(&self.messages[message_checkpoint..], &selectors);

        let Some(message) = matched_message else {
            return outcome;
        };

        suppress_confirm_quiet_messages(self, message_checkpoint, &selectors);

        let raw_expr = args[0].clone();
        let tag = confirmation_tag_from_args(&args, 3);
        let info_result = evaluate_optional_confirmation_info(self, args.get(2).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_quiet_failure(raw_expr, info, tag, message)
        })
    }

    fn finish_confirm_quiet_failure(
        &mut self,
        raw_expr: Expr,
        info: Option<Expr>,
        tag: Expr,
        message: RuntimeMessage,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Expression", raw_expr),
            (
                "HeldMessage",
                Expr::call(
                    system_expr("Hold"),
                    vec![runtime_message_name_expr(message)],
                ),
            ),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_assert_after_test(
        &mut self,
        args: Vec<Expr>,
        test: Expr,
    ) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            return Ok(system_expr("Null"));
        }

        let raw_test = args[0].clone();
        let tag = confirmation_tag_from_args(&args, 2);
        let info_result = evaluate_optional_confirmation_info(self, args.get(1).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_assert_failure(raw_test, info, tag)
        })
    }

    fn finish_confirm_assert_failure(
        &mut self,
        raw_test: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Test", raw_test),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_check_after_primary(
        &mut self,
        selectors: Option<Vec<MessageSelector>>,
        fallback: Expr,
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let matched = selectors.as_ref().map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
                );

                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);

                if matched {
                    self.evaluate(fallback)
                } else {
                    Ok(value)
                }
            }
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);

                let matched = selectors.as_ref().map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
                );

                if matched {
                    self.evaluate(fallback)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn resume_check_abort_after_primary(
        &mut self,
        args: Vec<Expr>,
        propagate: PropagateAbortsMode,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Abort) => {
                let fallback_result = self.evaluate(args[1].clone());
                let outcome = self.suspend_with_result_frame(fallback_result, {
                    move |session, outcome| {
                        session.resume_check_abort_after_fallback(propagate, outcome)
                    }
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_check_abort_after_fallback(propagate, outcome),
                }
            }
            Err(error) => Err(error),
        }
    }

    fn resume_check_abort_after_fallback(
        &mut self,
        propagate: PropagateAbortsMode,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                if propagate == PropagateAbortsMode::True {
                    Err(EvalError::Abort)
                } else {
                    Ok(value)
                }
            }
            Err(error) => Err(error),
        }
    }

    fn resume_with_cleanup_after_init(
        &mut self,
        expr: Expr,
        cleanup: Expr,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) => {
                let primary_result = self.evaluate(expr);
                self.resume_with_cleanup_after_primary(cleanup, primary_result)
            }
            Err(error) => self.resume_with_cleanup_after_primary(cleanup, Err(error)),
        }
    }

    fn resume_with_cleanup_after_primary(
        &mut self,
        cleanup: Expr,
        primary_outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let cleanup_result = self.evaluate(cleanup.clone());
        match cleanup_result {
            Ok(value) => self.resume_with_cleanup_after_cleanup(primary_outcome, Ok(value)),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, cleanup_outcome| {
                    session.resume_with_cleanup_after_cleanup(primary_outcome, cleanup_outcome)
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => self.resume_with_cleanup_after_cleanup(primary_outcome, Err(error)),
        }
    }

    fn resume_with_cleanup_after_cleanup(
        &mut self,
        primary_outcome: Result<Expr, EvalError>,
        cleanup_outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match cleanup_outcome {
            Ok(_) => primary_outcome,
            Err(error) => Err(error),
        }
    }

    fn resume_quiet_after_primary(
        &mut self,
        selectors: Option<Vec<MessageSelector>>,
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = match outcome {
            Ok(value) => Ok(value),
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                Err(err)
            }
        };

        if let Some(selectors) = selectors.as_ref() {
            self.suppress_messages_since(message_checkpoint, selectors);
        } else {
            self.messages.truncate(message_checkpoint);
        }

        if outcome.is_ok() {
            self.handled_non_loader_eval_errors
                .truncate(handled_error_checkpoint);
        }

        outcome
    }

    fn resume_if_after_condition(
        &mut self,
        args: Vec<Expr>,
        condition: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&condition) {
            Some(true) => self.evaluate(args[1].clone()),
            Some(false) => {
                if args.len() >= 3 {
                    self.evaluate(args[2].clone())
                } else {
                    Ok(system_expr("Null"))
                }
            }
            None if args.len() == 4 => self.evaluate(args[3].clone()),
            None => {
                let mut residual = args;
                residual[0] = condition;
                Ok(Expr::call(system_expr("If"), residual))
            }
        }
    }

    fn resume_compound_expression_from_index(
        &mut self,
        args: Vec<Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        let Some(current) = args.get(index).cloned() else {
            return Ok(system_expr("Null"));
        };

        let current_result = self.evaluate(current);
        let outcome = self.suspend_with_result_frame(current_result, {
            let args = args.clone();
            move |session, outcome| {
                session.resume_compound_expression_after_item(args, index, outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_compound_expression_after_item(args, index, outcome),
        }
    }

    fn resume_compound_expression_after_item(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) if index + 1 >= args.len() => Ok(value),
            Ok(_) => self.resume_compound_expression_from_index(args, index + 1),
            Err(error) => Err(error),
        }
    }

    fn resume_which_from_index(
        &mut self,
        args: Vec<Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= args.len() {
            return Ok(system_expr("Null"));
        }

        let condition_result = self.evaluate(args[index].clone());
        let condition = self.suspend_with_frame(condition_result, {
            let args = args.clone();
            move |session, condition| session.resume_which_after_condition(args, index, condition)
        })?;

        self.resume_which_after_condition(args, index, condition)
    }

    fn resume_which_after_condition(
        &mut self,
        args: Vec<Expr>,
        index: usize,
        condition: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&condition) {
            Some(true) => self.evaluate(args[index + 1].clone()),
            Some(false) => self.resume_which_from_index(args, index + 2),
            None => {
                let mut residual = args;
                residual[index] = condition;
                Ok(Expr::call(system_expr("Which"), residual))
            }
        }
    }

    fn resume_switch_after_subject(
        &mut self,
        args: Vec<Expr>,
        subject: Expr,
    ) -> Result<Expr, EvalError> {
        self.resume_switch_from_index(args, subject, 1)
    }

    fn resume_switch_from_index(
        &mut self,
        args: Vec<Expr>,
        subject: Expr,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= args.len() {
            let mut residual = args;
            residual[0] = subject;
            return Ok(Expr::call(system_expr("Switch"), residual));
        }

        let pattern = args[index].clone();
        let branch = args[index + 1].clone();
        let match_result = self.match_expr_owned(pattern, subject.clone(), BTreeMap::new());

        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.evaluate(branch)
            } else {
                session.resume_switch_from_index(args, subject, index + 2)
            }
        })
    }

    fn resume_catch_after_body(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Thrown { value, tag }) => {
                self.resume_catch_for_throw(form, handler, value, tag)
            }
            Err(error) => Err(error),
        }
    }

    fn resume_catch_for_throw(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return self.finish_catch_match(handler, value, tag);
        };

        let match_result = self.match_expr_owned(form, tag.clone(), BTreeMap::new());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.finish_catch_match(handler, value, tag)
            } else {
                Err(EvalError::Thrown { value, tag })
            }
        })
    }

    fn finish_catch_match(
        &mut self,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        match handler {
            Some(handler) => self.evaluate(Expr::call(handler, vec![value, tag])),
            None => Ok(value),
        }
    }
}

fn evaluate_do_spec_expr(session: &mut Session, raw_spec: Expr) -> Result<Expr, EvalError> {
    match raw_spec {
        Expr::Call { head, args } if is_system_symbol_name(&head, heads::structure::LIST) => {
            let evaluated = if let Some((first, rest)) = args.split_first() {
                if expr_as_symbol(first).is_some() {
                    let mut normalized = Vec::with_capacity(args.len());
                    normalized.push(Expr::call(system_expr("Hold"), vec![first.clone()]));
                    normalized.extend(rest.iter().cloned());
                    Expr::call(system_expr("List"), normalized)
                } else {
                    Expr::call(system_expr("List"), args)
                }
            } else {
                Expr::call(system_expr("List"), args)
            };

            session.evaluate(evaluated)
        }
        other => session.evaluate(other),
    }
}

fn parse_do_spec(expr: &Expr) -> Option<DoSpec> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some(DoSpec::Count(count));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    match args.as_slice() {
        [count_expr] => Some(DoSpec::Count(expr_as_non_negative_usize(count_expr)?)),
        [symbol_expr, values_expr] => {
            let symbol = parse_held_loop_symbol(symbol_expr)?;

            if let Expr::Call { head, args } = values_expr {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    return Some(DoSpec::Iterator {
                        symbol,
                        values: args.clone(),
                    });
                }
            }

            let end = expr_as_signed_bigint(values_expr)?;
            Some(DoSpec::Iterator {
                symbol,
                values: integer_range_values(BigInt::from(1_u8), end, BigInt::from(1_u8))?,
            })
        }
        [symbol_expr, start_expr, end_expr] => Some(DoSpec::Iterator {
            symbol: parse_held_loop_symbol(symbol_expr)?,
            values: integer_range_values(
                expr_as_signed_bigint(start_expr)?,
                expr_as_signed_bigint(end_expr)?,
                BigInt::from(1_u8),
            )?,
        }),
        [symbol_expr, start_expr, end_expr, step_expr] => {
            let step = expr_as_signed_bigint(step_expr)?;
            if step == BigInt::from(0_u8) {
                return None;
            }

            Some(DoSpec::Iterator {
                symbol: parse_held_loop_symbol(symbol_expr)?,
                values: integer_range_values(
                    expr_as_signed_bigint(start_expr)?,
                    expr_as_signed_bigint(end_expr)?,
                    step,
                )?,
            })
        }
        _ => None,
    }
}

fn build_do_iteration_expr(
    args: &[Expr],
    spec_index: usize,
    spec: &DoSpec,
    iteration: usize,
) -> Option<Expr> {
    build_iteration_expr(args, spec_index, spec, iteration, "Do")
}

fn build_table_iteration_expr(
    args: &[Expr],
    spec_index: usize,
    spec: &DoSpec,
    iteration: usize,
) -> Option<Expr> {
    build_iteration_expr(args, spec_index, spec, iteration, "Table")
}

fn build_iteration_expr(
    args: &[Expr],
    spec_index: usize,
    spec: &DoSpec,
    iteration: usize,
    nested_head: &str,
) -> Option<Expr> {
    let body_or_nested = if spec_index + 2 >= args.len() {
        args[0].clone()
    } else {
        let mut nested_args = Vec::with_capacity(args.len() - spec_index - 1);
        nested_args.push(args[0].clone());
        nested_args.extend(args[(spec_index + 2)..].iter().cloned());
        Expr::call(system_expr(nested_head), nested_args)
    };

    match spec {
        DoSpec::Count(count) => (iteration < *count).then_some(body_or_nested),
        DoSpec::Iterator { symbol, values } => {
            let value = values.get(iteration)?.clone();
            let binding = Expr::call(
                system_expr("Set"),
                vec![Expr::Atom(Atom::Symbol(symbol.clone())), value],
            );
            Some(Expr::call(
                system_expr("Block"),
                vec![
                    Expr::call(system_expr("List"), vec![binding]),
                    body_or_nested,
                ],
            ))
        }
    }
}

fn parse_held_loop_symbol(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Hold") && args.len() == 1 => {
            expr_as_symbol(&args[0]).cloned()
        }
        _ => expr_as_symbol(expr).cloned(),
    }
}

fn expr_as_signed_bigint(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") && args.len() == 2 => {
            if is_negative_one_expr(&args[0]) {
                expr_as_exact_integer(&args[1]).map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_exact_integer(&args[0]).map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn integer_range_values(start: BigInt, end: BigInt, step: BigInt) -> Option<Vec<Expr>> {
    let zero = BigInt::from(0_u8);
    if step == zero {
        return None;
    }

    let mut values = Vec::new();
    let mut current = start;

    if step > zero {
        while current <= end {
            values.push(Expr::integer(current.clone()));
            current += step.clone();
        }
    } else {
        while current >= end {
            values.push(Expr::integer(current.clone()));
            current += step.clone();
        }
    }

    Some(values)
}

fn normalize_confirm_quiet_selectors(
    selector_expr: Option<&Expr>,
) -> Result<ConfirmQuietSelectors, MessageSelectorNormalizationError> {
    match selector_expr {
        None => Ok(ConfirmQuietSelectors::All),
        Some(expr) if is_system_symbol_name(expr, "All") => Ok(ConfirmQuietSelectors::All),
        Some(expr) => parse_message_selectors(expr)
            .map(ConfirmQuietSelectors::Exact)
            .ok_or(MessageSelectorNormalizationError::InvalidSelectorSpec),
    }
}

fn confirm_quiet_matched_message(
    messages: &[RuntimeMessage],
    selectors: &ConfirmQuietSelectors,
) -> Option<RuntimeMessage> {
    messages
        .iter()
        .find(|message| match selectors {
            ConfirmQuietSelectors::All => true,
            ConfirmQuietSelectors::Exact(selectors) => {
                selectors.iter().any(|selector| selector.matches(message))
            }
        })
        .cloned()
}

fn suppress_confirm_quiet_messages(
    session: &mut Session,
    checkpoint: usize,
    selectors: &ConfirmQuietSelectors,
) {
    match selectors {
        ConfirmQuietSelectors::All => session.messages.truncate(checkpoint),
        ConfirmQuietSelectors::Exact(selectors) => {
            session.suppress_messages_since(checkpoint, selectors);
        }
    }
}

fn evaluate_optional_confirmation_info(
    session: &mut Session,
    info_expr: Option<Expr>,
) -> Result<Option<Expr>, EvalError> {
    match info_expr {
        None => Ok(None),
        Some(expr) if is_internal_confirmation_auto_info(&expr) => Ok(None),
        Some(expr) => session.evaluate(expr).map(Some),
    }
}

fn confirmation_tag_from_args(args: &[Expr], index: usize) -> Expr {
    args.get(index)
        .cloned()
        .unwrap_or_else(default_confirmation_tag_expr)
}

fn default_confirmation_tag_expr() -> Expr {
    system_expr("InternalConfirmTag")
}

fn fresh_internal_enclose_tag_expr(session: &mut Session) -> Expr {
    let seed = Symbol::new(SYSTEM_CONTEXT, "InternalEncloseTag");
    Expr::Atom(Atom::Symbol(session.allocate_fresh_symbol(&seed)))
}

fn is_internal_confirmation_auto_info(expr: &Expr) -> bool {
    is_system_symbol_name(expr, "InternalConfirmAutomaticInfo")
}

fn build_confirmation_failure(fields: Vec<(&str, Expr)>) -> Expr {
    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("ConfirmationFailed"),
            render_association(
                fields
                    .into_iter()
                    .map(|(key, value)| (Expr::string(key), value))
                    .collect(),
            ),
        ],
    )
}

fn build_failsafe_failure(function: Expr, test: Expr, args: Vec<Expr>) -> Expr {
    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("FailsafeFailure"),
            render_association(vec![
                (Expr::string("Function"), function),
                (Expr::string("Test"), test),
                (
                    Expr::string("Arguments"),
                    Expr::call(system_expr("List"), args),
                ),
            ]),
        ],
    )
}

fn is_failure_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, args } if is_system_symbol_name(head, "Failure") && args.len() == 2)
}

fn is_confirmation_failure_like(expr: &Expr) -> bool {
    is_failure_expr(expr)
        || is_system_symbol_name(expr, "$Failed")
        || is_system_symbol_name(expr, "$Aborted")
        || is_system_symbol_name(expr, "$Canceled")
        || matches!(expr, Expr::Call { head, args } if is_system_symbol_name(head, "Missing") && !args.is_empty())
}

fn is_failsafe_failure_like(expr: &Expr) -> bool {
    is_confirmation_failure_like(expr)
}

fn normalize_failure_details_expr(expr: &Expr) -> Option<Expr> {
    if association_entries_from_expr(expr).is_some() {
        return Some(expr.clone());
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let mut entries = Vec::with_capacity(args.len());
    for item in args {
        entries.push(parse_association_entry(item)?);
    }

    Some(render_association(entries))
}

fn failure_property_value(failure: &Expr, property: &str) -> Expr {
    let Expr::Call { head, args } = failure else {
        return Expr::call(
            system_expr("Missing"),
            vec![Expr::string("KeyAbsent"), Expr::string(property)],
        );
    };

    if !is_system_symbol_name(head, "Failure") || args.len() != 2 {
        return Expr::call(
            system_expr("Missing"),
            vec![Expr::string("KeyAbsent"), Expr::string(property)],
        );
    }

    association_value_by_key(&args[1], &Expr::string(property)).unwrap_or_else(|| {
        Expr::call(
            system_expr("Missing"),
            vec![Expr::string("KeyAbsent"), Expr::string(property)],
        )
    })
}

fn rewrite_untagged_confirms(expr: Expr, tag: &Expr) -> Expr {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, "Enclose") => {
            Expr::call(head.as_ref().clone(), args)
        }
        Expr::Call { head, args } => {
            let rewritten_head = rewrite_untagged_confirms((*head).clone(), tag);
            let rewritten_args = args
                .into_iter()
                .map(|arg| rewrite_untagged_confirms(arg, tag))
                .collect::<Vec<_>>();

            let name = expr_as_symbol(&rewritten_head).map(|symbol| symbol.name().to_string());
            match (name.as_deref(), rewritten_args.as_slice()) {
                (Some("Confirm"), [expr]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("Confirm"), [expr, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmBy"), [expr, function]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        function.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmBy"), [expr, function, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), function.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmMatch"), [expr, pattern]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        pattern.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmMatch"), [expr, pattern, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), pattern.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmQuiet"), [expr]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        system_expr("All"),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmQuiet"), [expr, selectors]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        selectors.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmQuiet"), [expr, selectors, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), selectors.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmAssert"), [test]) => Expr::call(
                    rewritten_head,
                    vec![
                        test.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmAssert"), [test, info]) => Expr::call(
                    rewritten_head,
                    vec![test.clone(), info.clone(), tag.clone()],
                ),
                _ => Expr::call(rewritten_head, rewritten_args),
            }
        }
        other => other,
    }
}

fn parse_check_abort_propagation(
    option_expr: Option<&Expr>,
) -> Result<PropagateAbortsMode, &'static str> {
    let Some(option_expr) = option_expr else {
        return Ok(PropagateAbortsMode::Automatic);
    };

    let Expr::Call { head, args } = option_expr else {
        return Err(
            "expected PropagateAborts -> True|False|Automatic as the optional third argument",
        );
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return Err(
            "expected PropagateAborts -> True|False|Automatic as the optional third argument",
        );
    }

    if !is_system_symbol_name(&args[0], "PropagateAborts") {
        return Err(
            "expected PropagateAborts -> True|False|Automatic as the optional third argument",
        );
    }

    if is_system_symbol_name(&args[1], "True") {
        Ok(PropagateAbortsMode::True)
    } else if is_system_symbol_name(&args[1], "False") {
        Ok(PropagateAbortsMode::False)
    } else if is_system_symbol_name(&args[1], "Automatic") {
        Ok(PropagateAbortsMode::Automatic)
    } else {
        Err("expected PropagateAborts -> True|False|Automatic as the optional third argument")
    }
}

pub(super) fn apply_callable_failsafe_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_failsafe_wrapper(head)?;

    if spec.test.is_none() {
        return Some(
            args.iter()
                .find(|arg| is_failsafe_failure_like(arg))
                .cloned()
                .unwrap_or_else(|| Expr::call(spec.function.clone(), args.to_vec())),
        );
    }

    let test = spec.test?.clone();
    let success = Expr::call(spec.function.clone(), args.to_vec());
    let fallback = match spec.fail_function {
        Some(fail_function) => Expr::call(fail_function.clone(), args.to_vec()),
        None => build_failsafe_failure(spec.function.clone(), test.clone(), args.to_vec()),
    };

    Some(Expr::call(
        system_expr("If"),
        vec![Expr::call(test, args.to_vec()), success, fallback],
    ))
}

pub(super) fn is_callable_failsafe_wrapper(expr: &Expr) -> bool {
    parse_failsafe_wrapper(expr).is_some()
}

struct FailsafeWrapperSpec<'a> {
    function: &'a Expr,
    test: Option<&'a Expr>,
    fail_function: Option<&'a Expr>,
}

fn parse_failsafe_wrapper(expr: &Expr) -> Option<FailsafeWrapperSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Failsafe") {
        return None;
    }

    match args.as_slice() {
        [function] => Some(FailsafeWrapperSpec {
            function,
            test: None,
            fail_function: None,
        }),
        [function, test] => Some(FailsafeWrapperSpec {
            function,
            test: Some(test),
            fail_function: None,
        }),
        [function, test, fail_function] => Some(FailsafeWrapperSpec {
            function,
            test: Some(test),
            fail_function: Some(fail_function),
        }),
        _ => None,
    }
}
