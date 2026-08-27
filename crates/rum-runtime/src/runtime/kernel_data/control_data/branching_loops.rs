use super::*;

mod iteration;
mod iteration_aggregate;

#[derive(Clone, Copy)]
enum WhilePhase {
    Test,
    Body,
}

#[derive(Clone, Copy)]
enum UntilPhase {
    Body,
    Test,
}

#[derive(Clone, Copy)]
enum ForPhase {
    Start,
    Test,
    Body,
    Increment,
}

impl Session {
    pub(in crate::runtime) fn eval_compound_expression(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.resume_compound_expression_from_index(args.to_vec(), 0)
    }

    pub(in crate::runtime) fn eval_if(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("If", args_slice, 2, 4));
        }

        if let Some(truth) = truthiness_symbol(&args_slice[0]) {
            return self.resume_if_after_condition(
                args_slice.to_vec(),
                system_expr(if truth { "True" } else { "False" }),
            );
        }

        let condition_result = self.evaluate(args_slice[0].clone());
        let condition = self.suspend_with_frame_lazy(condition_result, || {
            let args = args.clone();
            move |session, condition| {
                session.resume_if_after_condition(args.as_ref().to_vec(), condition)
            }
        })?;

        self.resume_if_after_condition(args_slice.to_vec(), condition)
    }

    pub(in crate::runtime) fn eval_which(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() || args.len() % 2 != 0 {
            return Ok(self.invalid_call_with_arg_form(
                "Which",
                args,
                "expected one or more condition/result pairs",
            ));
        }

        self.resume_which_from_index(args.to_vec(), 0)
    }

    pub(in crate::runtime) fn eval_switch(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() < 3 || args_slice.len() % 2 == 0 {
            return Ok(self.invalid_call_with_arg_form(
                "Switch",
                args_slice,
                "expected an expression followed by one or more form/value pairs",
            ));
        }

        let subject_result = self.evaluate(args_slice[0].clone());
        let subject = self.suspend_with_frame_lazy(subject_result, || {
            let args = args.clone();
            move |session, subject| {
                session.resume_switch_after_subject(args.as_ref().to_vec(), subject)
            }
        })?;

        self.resume_switch_after_subject(args_slice.to_vec(), subject)
    }

    pub(in crate::runtime) fn eval_while(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("While", args, 1, 2));
        }

        self.resume_while_loop(args.to_vec(), WhilePhase::Test, None)
    }

    pub(in crate::runtime) fn eval_until(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Until", args, 1, 2));
        }

        self.resume_until_loop(args.to_vec(), UntilPhase::Body, None)
    }

    pub(in crate::runtime) fn eval_for(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(3..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("For", args, 3, 4));
        }

        self.resume_for_loop(args.to_vec(), ForPhase::Start, None)
    }

    fn resume_while_loop(
        &mut self,
        args: Vec<Expr>,
        mut phase: WhilePhase,
        mut pending: Option<Result<Expr, EvalError>>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                match phase {
                    WhilePhase::Test => match outcome {
                        Ok(test) if is_system_symbol_name(&test, "True") => {
                            phase = WhilePhase::Body;
                        }
                        Ok(_) => return Ok(system_expr("Null")),
                        Err(error) => return Err(error),
                    },
                    WhilePhase::Body => match outcome {
                        Ok(_) | Err(EvalError::Continue) => phase = WhilePhase::Test,
                        Err(EvalError::Break) => return Ok(system_expr("Null")),
                        Err(error) => return Err(error),
                    },
                }
                continue;
            }

            if self.interrupt_armed {
                self.poll_host_interrupt_if_due()?;
            }
            let outcome = match phase {
                WhilePhase::Test => {
                    let runtime_profile_test = runtime_path_timing_start("while.test.evaluate");
                    let result = self.evaluate(args[0].clone());
                    record_profile_runtime_path_timing(
                        runtime_profile_test,
                        runtime_profile_outcome_for_expr_result(&result),
                    );
                    let runtime_profile_suspend = runtime_path_timing_start("while.test.suspend");
                    let outcome = self.suspend_with_result_frame_lazy(result, || {
                        let args = args.clone();
                        move |session, outcome| {
                            session.resume_while_loop(args, WhilePhase::Test, Some(outcome))
                        }
                    });
                    record_profile_runtime_path_timing(
                        runtime_profile_suspend,
                        runtime_profile_outcome_for_expr_result(&outcome),
                    );
                    outcome
                }
                WhilePhase::Body => {
                    let body = args.get(1).cloned().unwrap_or_else(|| system_expr("Null"));
                    let runtime_profile_body = runtime_path_timing_start("while.body.evaluate");
                    let result = self.evaluate(body);
                    record_profile_runtime_path_timing(
                        runtime_profile_body,
                        runtime_profile_outcome_for_expr_result(&result),
                    );
                    let runtime_profile_suspend = runtime_path_timing_start("while.body.suspend");
                    let outcome = self.suspend_with_result_frame_lazy(result, || {
                        let args = args.clone();
                        move |session, outcome| {
                            session.resume_while_loop(args, WhilePhase::Body, Some(outcome))
                        }
                    });
                    record_profile_runtime_path_timing(
                        runtime_profile_suspend,
                        runtime_profile_outcome_for_expr_result(&outcome),
                    );
                    outcome
                }
            };

            match outcome {
                Err(EvalError::InputRequired { request }) => {
                    return Err(EvalError::InputRequired { request });
                }
                outcome => pending = Some(outcome),
            }
        }
    }

    fn resume_until_loop(
        &mut self,
        args: Vec<Expr>,
        mut phase: UntilPhase,
        mut pending: Option<Result<Expr, EvalError>>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                match phase {
                    UntilPhase::Body => match outcome {
                        Ok(_) | Err(EvalError::Continue) => phase = UntilPhase::Test,
                        Err(EvalError::Break) => return Ok(system_expr("Null")),
                        Err(error) => return Err(error),
                    },
                    UntilPhase::Test => match outcome {
                        Ok(test) if is_system_symbol_name(&test, "True") => {
                            return Ok(system_expr("Null"));
                        }
                        Ok(_) => phase = UntilPhase::Body,
                        Err(error) => return Err(error),
                    },
                }
                continue;
            }

            if self.interrupt_armed {
                self.poll_host_interrupt_if_due()?;
            }
            let current = match phase {
                UntilPhase::Body => args.get(1).cloned().unwrap_or_else(|| system_expr("Null")),
                UntilPhase::Test => args[0].clone(),
            };
            let result = self.evaluate(current);
            let outcome = self.suspend_with_result_frame_lazy(result, || {
                let args = args.clone();
                move |session, outcome| session.resume_until_loop(args, phase, Some(outcome))
            });

            match outcome {
                Err(EvalError::InputRequired { request }) => {
                    return Err(EvalError::InputRequired { request });
                }
                outcome => pending = Some(outcome),
            }
        }
    }

    fn resume_for_loop(
        &mut self,
        args: Vec<Expr>,
        mut phase: ForPhase,
        mut pending: Option<Result<Expr, EvalError>>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                match phase {
                    ForPhase::Start => match outcome {
                        Ok(_) => phase = ForPhase::Test,
                        Err(error) => return Err(error),
                    },
                    ForPhase::Test => match outcome {
                        Ok(test) if is_system_symbol_name(&test, "True") => {
                            phase = ForPhase::Body;
                        }
                        Ok(_) => return Ok(system_expr("Null")),
                        Err(error) => return Err(error),
                    },
                    ForPhase::Body => match outcome {
                        Ok(_) | Err(EvalError::Continue) => phase = ForPhase::Increment,
                        Err(EvalError::Break) => return Ok(system_expr("Null")),
                        Err(error) => return Err(error),
                    },
                    ForPhase::Increment => match outcome {
                        Ok(_) => phase = ForPhase::Test,
                        Err(error) => return Err(error),
                    },
                }
                continue;
            }

            if self.interrupt_armed {
                self.poll_host_interrupt_if_due()?;
            }
            let current = match phase {
                ForPhase::Start => args[0].clone(),
                ForPhase::Test => args[1].clone(),
                ForPhase::Body => args.get(3).cloned().unwrap_or_else(|| system_expr("Null")),
                ForPhase::Increment => args[2].clone(),
            };
            let result = self.evaluate(current);
            let outcome = self.suspend_with_result_frame_lazy(result, || {
                let args = args.clone();
                move |session, outcome| session.resume_for_loop(args, phase, Some(outcome))
            });

            match outcome {
                Err(EvalError::InputRequired { request }) => {
                    return Err(EvalError::InputRequired { request });
                }
                outcome => pending = Some(outcome),
            }
        }
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

        let runtime_profile_item = runtime_path_timing_start("compound.item.evaluate");
        let current_result = self.evaluate(current);
        record_profile_runtime_path_timing(
            runtime_profile_item,
            runtime_profile_outcome_for_expr_result(&current_result),
        );
        let runtime_profile_suspend = runtime_path_timing_start("compound.item.suspend");
        let outcome = self.suspend_with_result_frame_lazy(current_result, || {
            let args = args.clone();
            move |session, outcome| {
                session.resume_compound_expression_after_item(args, index, outcome)
            }
        });
        record_profile_runtime_path_timing(
            runtime_profile_suspend,
            runtime_profile_outcome_for_expr_result(&outcome),
        );

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
        let condition = self.suspend_with_frame_lazy(condition_result, || {
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
        let match_result =
            self.match_expr_owned(pattern, subject.clone(), SymbolExprMap::default());

        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.evaluate(branch)
            } else {
                session.resume_switch_from_index(args, subject, index + 2)
            }
        })
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}
