use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

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

    pub(in crate::runtime) fn eval_do(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_table(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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
