use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IterationConsumer {
    Do,
    Table,
}

impl DoSpec {
    fn iteration_count(&self) -> usize {
        match self {
            Self::Count(count) => *count,
            Self::Iterator { values, .. } => values.len(),
        }
    }
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
        let spec = match parse_do_spec(self, &spec_expr, IterationConsumer::Do)? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "Do",
                    &args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
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
        let spec = match parse_do_spec(self, &spec_expr, IterationConsumer::Table)? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "Table",
                    &args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
        };
        let capacity = spec.iteration_count();

        self.resume_table_iterations(args, spec_index, spec, 0, Vec::with_capacity(capacity))
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
                    Expr::call_with_shared_args(system_expr("List"), args)
                }
            } else {
                Expr::call_with_shared_args(system_expr("List"), args)
            };

            session.evaluate(evaluated)
        }
        other => session.evaluate(other),
    }
}

fn parse_do_spec(
    session: &Session,
    expr: &Expr,
    consumer: IterationConsumer,
) -> Result<Option<DoSpec>, EvalError> {
    if let Some(count) = expr_as_non_negative_bigint(expr) {
        return Ok(Some(DoSpec::Count(checked_iteration_count(
            session, &count, consumer,
        )?)));
    }

    let Expr::Call { head, args } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    Ok(match args.as_slice() {
        [count_expr] => {
            let Some(count) = expr_as_non_negative_bigint(count_expr) else {
                return Ok(None);
            };
            Some(DoSpec::Count(checked_iteration_count(
                session, &count, consumer,
            )?))
        }
        [symbol_expr, values_expr] => {
            let Some(symbol) = parse_held_loop_symbol(symbol_expr) else {
                return Ok(None);
            };

            if let Expr::Call { head, args } = values_expr {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    checked_iteration_count(session, &BigInt::from(args.len()), consumer)?;
                    return Ok(Some(DoSpec::Iterator {
                        symbol,
                        values: args.to_vec(),
                    }));
                }
            }

            let Some(end) = expr_as_signed_bigint(values_expr) else {
                return Ok(None);
            };
            let values = checked_integer_range_values(
                session,
                BigInt::from(1_u8),
                end,
                BigInt::from(1_u8),
                consumer,
            )?;
            Some(DoSpec::Iterator { symbol, values })
        }
        [symbol_expr, start_expr, end_expr] => {
            let (Some(symbol), Some(start), Some(end)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
            ) else {
                return Ok(None);
            };
            let values =
                checked_integer_range_values(session, start, end, BigInt::from(1_u8), consumer)?;
            Some(DoSpec::Iterator { symbol, values })
        }
        [symbol_expr, start_expr, end_expr, step_expr] => {
            let (Some(symbol), Some(start), Some(end), Some(step)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
                expr_as_signed_bigint(step_expr),
            ) else {
                return Ok(None);
            };
            if step == BigInt::from(0_u8) {
                return Ok(None);
            }

            let values = checked_integer_range_values(session, start, end, step, consumer)?;
            Some(DoSpec::Iterator { symbol, values })
        }
        _ => None,
    })
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

fn expr_as_non_negative_bigint(expr: &Expr) -> Option<BigInt> {
    let value = expr_as_exact_integer(expr)?;
    (value >= BigInt::from(0_u8)).then_some(value)
}

fn checked_integer_range_values(
    session: &Session,
    start: BigInt,
    end: BigInt,
    step: BigInt,
    consumer: IterationConsumer,
) -> Result<Vec<Expr>, EvalError> {
    let Some(count) = integer_range_count(&start, &end, &step) else {
        return Ok(Vec::new());
    };
    let count = checked_iteration_count(session, &count, consumer)?;
    Ok(integer_range_values(start, step, count))
}

fn checked_iteration_count(
    session: &Session,
    count: &BigInt,
    consumer: IterationConsumer,
) -> Result<usize, EvalError> {
    match consumer {
        IterationConsumer::Do => session.enforce_iteration_work_limit_for_big_count(count)?,
        IterationConsumer::Table => {
            if let Some(limit) = session.current_memory_limit {
                if count > &BigInt::from(usize::MAX) {
                    return Err(EvalError::MemoryConstraintExceeded { limit });
                }
                let count = bigint_count_to_usize(count)
                    .expect("Table count was checked against usize::MAX before conversion");
                session.enforce_active_memory_limit_for_projected_bytes(
                    generated_list_memory_proxy_bytes(count),
                )?;
            }
            session.enforce_generated_element_limit_for_big_count(count)?;
            session.enforce_iteration_work_limit_for_big_count(count)?;
        }
    }

    Ok(bigint_count_to_usize(count)
        .expect("resource limit prechecks keep iteration counts representable"))
}

fn integer_range_count(start: &BigInt, end: &BigInt, step: &BigInt) -> Option<BigInt> {
    let zero = BigInt::from(0_u8);
    if step == &zero {
        return None;
    }

    let one = BigInt::from(1_u8);
    if step > &zero {
        if start > end {
            return Some(zero);
        }
        return Some(((end - start) / step) + one);
    }

    if start < end {
        return Some(zero);
    }

    Some(((start - end) / -step) + one)
}

fn integer_range_values(start: BigInt, step: BigInt, count: usize) -> Vec<Expr> {
    let mut values = Vec::with_capacity(count);
    let mut current = start;

    for _ in 0..count {
        values.push(Expr::integer(current.clone()));
        current += step.clone();
    }

    values
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn generated_list_memory_proxy_bytes(count: usize) -> usize {
    std::mem::size_of::<Expr>().saturating_add(count.saturating_mul(std::mem::size_of::<Expr>()))
}
