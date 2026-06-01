use super::*;

enum SequenceLikeExpr {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

impl Session {
    pub(in crate::runtime) fn eval_block_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, size] => Ok(unary_operator_form_with_tail(
                "BlockMap",
                operator.clone(),
                size.clone(),
            )),
            [operator, size, offset]
                if expr_as_positive_usize(size).is_some()
                    && expr_as_positive_usize(offset).is_some() =>
            {
                Ok(unary_operator_form_with_tails(
                    "BlockMap",
                    operator.clone(),
                    vec![size.clone(), offset.clone()],
                ))
            }
            [operator, expr, size_expr] => {
                let Some(size) = expr_as_positive_usize(size_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BlockMap",
                        args,
                        "expected a positive integer block size as the third argument",
                    ));
                };

                apply_block_map(self, head, operator, expr, size, size, args)
            }
            [operator, expr, size_expr, offset_expr] => {
                let Some(size) = expr_as_positive_usize(size_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BlockMap",
                        args,
                        "expected a positive integer block size as the third argument",
                    ));
                };
                let Some(offset) = expr_as_positive_usize(offset_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BlockMap",
                        args,
                        "expected a positive integer offset as the fourth argument",
                    ));
                };

                apply_block_map(self, head, operator, expr, size, offset, args)
            }
            _ => Ok(self.invalid_call_with_arg_range("BlockMap", args, 2, 4)),
        }
    }

    pub(in crate::runtime) fn eval_accumulate(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Accumulate", args, 1));
        }

        let Some(sequence) = sequence_like_from_expr(&args[0]) else {
            return Ok(Expr::call(system_expr("Accumulate"), args.to_vec()));
        };

        accumulate_sequence(self, sequence)
    }

    pub(in crate::runtime) fn eval_take_while(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [criterion] => Ok(unary_operator_form_slot_first("TakeWhile", criterion)),
            [expr, criterion] => {
                let Some(sequence) = sequence_like_from_expr(expr) else {
                    return Ok(Expr::call(system_expr("TakeWhile"), args.to_vec()));
                };

                self.take_while_sequence(sequence, criterion)
            }
            _ => Ok(self.invalid_call_with_arg_range("TakeWhile", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_length_while(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [criterion] => Ok(unary_operator_form_slot_first("LengthWhile", criterion)),
            [expr, criterion] => {
                let Some(sequence) = sequence_like_from_expr(expr) else {
                    return Ok(Expr::call(system_expr("LengthWhile"), args.to_vec()));
                };

                let count = self.length_while_sequence(sequence, criterion)?;
                Ok(Expr::integer(BigInt::from(count)))
            }
            _ => Ok(self.invalid_call_with_arg_range("LengthWhile", args, 1, 2)),
        }
    }

    fn take_while_sequence(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: &Expr,
    ) -> Result<Expr, EvalError> {
        match sequence {
            SequenceLikeExpr::Call { head, items } => {
                let mut taken = Vec::new();
                for item in items {
                    if !criterion_accepts(self, criterion, &item)? {
                        break;
                    }
                    taken.push(item);
                }

                Ok(Expr::call(head, taken))
            }
            SequenceLikeExpr::Association(entries) => {
                let mut taken = Vec::new();
                for (key, value) in entries {
                    if !criterion_accepts(self, criterion, &value)? {
                        break;
                    }
                    taken.push((key, value));
                }

                Ok(render_association(taken))
            }
        }
    }

    fn length_while_sequence(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: &Expr,
    ) -> Result<usize, EvalError> {
        match sequence {
            SequenceLikeExpr::Call { items, .. } => {
                let mut count = 0usize;
                for item in items {
                    if !criterion_accepts(self, criterion, &item)? {
                        break;
                    }
                    count += 1;
                }
                Ok(count)
            }
            SequenceLikeExpr::Association(entries) => {
                let mut count = 0usize;
                for (_, value) in entries {
                    if !criterion_accepts(self, criterion, &value)? {
                        break;
                    }
                    count += 1;
                }
                Ok(count)
            }
        }
    }
}

fn sequence_like_from_expr(expr: &Expr) -> Option<SequenceLikeExpr> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(SequenceLikeExpr::Association(entries));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    Some(SequenceLikeExpr::Call {
        head: head.as_ref().clone(),
        items: args.clone(),
    })
}

fn criterion_accepts(
    session: &mut Session,
    criterion: &Expr,
    item: &Expr,
) -> Result<bool, EvalError> {
    let value = session.evaluate(Expr::call(criterion.clone(), vec![item.clone()]))?;
    Ok(truthiness_symbol(&value) == Some(true))
}

fn accumulate_sequence(
    session: &mut Session,
    sequence: SequenceLikeExpr,
) -> Result<Expr, EvalError> {
    match sequence {
        SequenceLikeExpr::Call { head, items } => {
            let accumulated = accumulate_items(session, items)?;
            Ok(Expr::call(head, accumulated))
        }
        SequenceLikeExpr::Association(entries) => {
            let keys = entries
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>();
            let values = entries
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>();
            let accumulated = accumulate_items(session, values)?;
            Ok(render_association(
                keys.into_iter().zip(accumulated).collect(),
            ))
        }
    }
}

fn accumulate_items(session: &mut Session, items: Vec<Expr>) -> Result<Vec<Expr>, EvalError> {
    let Some((first, rest)) = items.split_first() else {
        return Ok(Vec::new());
    };

    let mut accumulated = vec![first.clone()];
    let mut current = first.clone();
    for item in rest {
        current = session.evaluate(Expr::call(system_expr("Plus"), vec![current, item.clone()]))?;
        accumulated.push(current.clone());
    }

    Ok(accumulated)
}

fn apply_block_map(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    expr: &Expr,
    size: usize,
    offset: usize,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: block_head,
        args: items,
    } = expr
    else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let mut results = Vec::new();
    let mut index = 0usize;
    while index + size <= items.len() {
        let block = Expr::call(
            block_head.as_ref().clone(),
            items[index..index + size].to_vec(),
        );
        results.push(session.evaluate(Expr::call(operator.clone(), vec![block]))?);
        index += offset;
    }

    Ok(Expr::call(system_expr("List"), results))
}
