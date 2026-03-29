use super::*;

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
