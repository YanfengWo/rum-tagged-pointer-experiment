use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

impl Session {
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
        mut iteration: usize,
        mut collected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(iteration_expr) =
                build_table_iteration_expr(&args, spec_index, &spec, iteration)
            else {
                return Ok(Expr::call(system_expr("List"), collected));
            };

            match self.evaluate(iteration_expr) {
                Ok(value) => {
                    collected.push(value);
                    iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        session.resume_table_after_iteration(
                            args, spec_index, spec, iteration, collected, outcome,
                        )
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
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
