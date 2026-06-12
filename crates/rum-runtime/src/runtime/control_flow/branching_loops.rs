use super::*;

mod iteration;

impl Session {
    pub(in crate::runtime) fn eval_compound_expression(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.resume_compound_expression_from_index(args.to_vec(), 0)
    }

    pub(in crate::runtime) fn eval_if(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_switch(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_while(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("While", args, 1, 2));
        }

        self.resume_while_test(args.to_vec())
    }

    pub(in crate::runtime) fn eval_until(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Until", args, 1, 2));
        }

        self.resume_until_body(args.to_vec())
    }

    pub(in crate::runtime) fn eval_for(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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
        let outcome = self.suspend_with_result_frame_lazy(current_result, || {
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
        let match_result = self.match_expr_owned(pattern, subject.clone(), SymbolExprMap::default());

        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.evaluate(branch)
            } else {
                session.resume_switch_from_index(args, subject, index + 2)
            }
        })
    }
}
