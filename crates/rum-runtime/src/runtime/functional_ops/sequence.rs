use super::*;

enum SequenceLikeExpr {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FoldWhileConfig {
    test_arity: NestWhileTestArity,
    tail_adjustment: isize,
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

    pub(in crate::runtime) fn eval_fold(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(variadic_operator_form("Fold", vec![operator.clone()])),
            [operator, list_expr] => apply_fold(self, operator, None, list_expr, false),
            [operator, initial, list_expr] => {
                apply_fold(self, operator, Some(initial.clone()), list_expr, false)
            }
            _ => Ok(self.invalid_call_with_arg_range("Fold", args, 1, 3)),
        }
    }

    pub(in crate::runtime) fn eval_fold_list(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(variadic_operator_form("FoldList", vec![operator.clone()])),
            [operator, list_expr] => apply_fold(self, operator, None, list_expr, true),
            [operator, initial, list_expr] => {
                apply_fold(self, operator, Some(initial.clone()), list_expr, true)
            }
            _ => Ok(self.invalid_call_with_arg_range("FoldList", args, 1, 3)),
        }
    }

    pub(in crate::runtime) fn eval_fold_pair(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, list_expr] => {
                apply_fold_pair(self, head, operator, None, list_expr, None, args)
            }
            [operator, initial, list_expr] => apply_fold_pair(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                None,
                args,
            ),
            [operator, initial, list_expr, projector] => apply_fold_pair(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                Some(projector.clone()),
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("FoldPair", args, 2, 4)),
        }
    }

    pub(in crate::runtime) fn eval_fold_while(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, test] => Ok(variadic_operator_form_with_suffix(
                "FoldWhile",
                vec![operator.clone()],
                vec![test.clone()],
            )),
            [operator, list_expr, test] => apply_fold_while(
                self, head, operator, None, list_expr, test, None, None, args,
            ),
            [operator, initial, list_expr, test] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                None,
                None,
                args,
            ),
            [operator, initial, list_expr, test, m_expr] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                Some(m_expr),
                None,
                args,
            ),
            [operator, initial, list_expr, test, m_expr, n_expr] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                Some(m_expr),
                Some(n_expr),
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("FoldWhile", args, 2, 6)),
        }
    }

    pub(in crate::runtime) fn eval_sequence_fold(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, seeds, items] => {
                apply_sequence_fold(self, head, operator, seeds, items, None, false, args)
            }
            [operator, seeds, items, arity_expr] => apply_sequence_fold(
                self,
                head,
                operator,
                seeds,
                items,
                Some(arity_expr),
                false,
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("SequenceFold", args, 3, 4)),
        }
    }

    pub(in crate::runtime) fn eval_sequence_fold_list(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, seeds, items] => {
                apply_sequence_fold(self, head, operator, seeds, items, None, true, args)
            }
            [operator, seeds, items, arity_expr] => apply_sequence_fold(
                self,
                head,
                operator,
                seeds,
                items,
                Some(arity_expr),
                true,
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("SequenceFoldList", args, 3, 4)),
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

fn iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => Some(
            association_entries_from_expr(expr)?
                .into_iter()
                .map(|(_, value)| value)
                .collect(),
        ),
        Expr::Call { args, .. } => Some(args.clone()),
        Expr::Atom(_) => None,
    }
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

fn apply_fold(
    session: &mut Session,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    collect_all: bool,
) -> Result<Expr, EvalError> {
    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call(
            system_expr(if collect_all { "FoldList" } else { "Fold" }),
            match initial {
                Some(initial) => vec![operator.clone(), initial, list_expr.clone()],
                None => vec![operator.clone(), list_expr.clone()],
            },
        ));
    };

    let (mut current, start_index, mut history) = if let Some(initial) = initial {
        let history = if collect_all {
            vec![initial.clone()]
        } else {
            Vec::new()
        };
        (initial, 0, history)
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(
                system_expr(if collect_all { "FoldList" } else { "Fold" }),
                vec![operator.clone(), list_expr.clone()],
            ));
        };
        let history = if collect_all {
            vec![first.clone()]
        } else {
            Vec::new()
        };
        let start_index = items.len() - rest.len();
        (first.clone(), start_index, history)
    };

    for item in items.into_iter().skip(start_index) {
        current = session.evaluate(Expr::call(operator.clone(), vec![current, item]))?;
        if collect_all {
            history.push(current.clone());
        }
    }

    if collect_all {
        Ok(Expr::call(system_expr("List"), history))
    } else {
        Ok(current)
    }
}

fn apply_fold_pair(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    projector: Option<Expr>,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let (mut state, start_index) = if let Some(initial) = initial {
        (initial, 0usize)
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };
        (first.clone(), items.len() - rest.len())
    };

    let mut last_output = None;
    for item in items.into_iter().skip(start_index) {
        let pair = session.evaluate(Expr::call(operator.clone(), vec![state, item]))?;
        let Some((x_value, y_value)) = parse_pair_result(&pair) else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };

        let output = if let Some(projector) = &projector {
            session.evaluate(Expr::call(
                projector.clone(),
                vec![Expr::call(
                    system_expr("List"),
                    vec![x_value.clone(), y_value.clone()],
                )],
            ))?
        } else {
            x_value.clone()
        };

        last_output = Some(output);
        state = y_value;
    }

    Ok(last_output.unwrap_or_else(|| Expr::call(head.clone(), raw_args.to_vec())))
}

fn parse_pair_result(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { args, .. } = expr else {
        return None;
    };

    match args.as_slice() {
        [first, second] => Some((first.clone(), second.clone())),
        _ => None,
    }
}

fn apply_fold_while(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    test: &Expr,
    m_expr: Option<&Expr>,
    n_expr: Option<&Expr>,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(config) = parse_fold_while_config(m_expr, n_expr) else {
        return Ok(session.invalid_call_with_arg_form(
            "FoldWhile",
            raw_args,
            "expected a positive integer or All test arity, with an optional signed integer tail adjustment",
        ));
    };

    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let (mut current, start_index, mut history) = if let Some(initial) = initial {
        (initial.clone(), 0usize, vec![initial])
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };
        (first.clone(), items.len() - rest.len(), vec![first.clone()])
    };

    let mut index = start_index;
    let mut failed = false;
    while index < items.len() {
        current = session.evaluate(Expr::call(
            operator.clone(),
            vec![current, items[index].clone()],
        ))?;
        history.push(current.clone());
        index += 1;

        let test_value = session.evaluate(Expr::call(
            test.clone(),
            nest_while_test_args(&history, config.test_arity),
        ))?;
        if truthiness_symbol(&test_value) != Some(true) {
            failed = true;
            break;
        }
    }

    if failed {
        if config.tail_adjustment > 0 {
            for _ in 0..usize::try_from(config.tail_adjustment).unwrap_or(0) {
                if index >= items.len() {
                    break;
                }
                let previous = history.last().cloned().unwrap_or_else(|| current.clone());
                current = session.evaluate(Expr::call(
                    operator.clone(),
                    vec![previous, items[index].clone()],
                ))?;
                history.push(current.clone());
                index += 1;
            }
        } else if config.tail_adjustment < 0 {
            let drop_count = usize::try_from(-config.tail_adjustment).unwrap_or(usize::MAX);
            let keep_len = history.len().saturating_sub(drop_count).max(1);
            history.truncate(keep_len);
        }
    }

    Ok(history
        .last()
        .cloned()
        .unwrap_or_else(|| Expr::call(head.clone(), raw_args.to_vec())))
}

fn parse_fold_while_config(
    m_expr: Option<&Expr>,
    n_expr: Option<&Expr>,
) -> Option<FoldWhileConfig> {
    let test_arity = match m_expr {
        Some(expr) if is_system_symbol_name(expr, "All") => NestWhileTestArity::All,
        Some(expr) => {
            let count = expr_as_non_negative_usize(expr)?;
            if count == 0 {
                return None;
            }
            NestWhileTestArity::Count(count)
        }
        None => NestWhileTestArity::Count(1),
    };

    let tail_adjustment = match n_expr {
        Some(expr) => expr_as_signed_isize(expr)?,
        None => 0,
    };

    Some(FoldWhileConfig {
        test_arity,
        tail_adjustment,
    })
}

fn apply_sequence_fold(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    seeds_expr: &Expr,
    items_expr: &Expr,
    arity_expr: Option<&Expr>,
    collect_all: bool,
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(seeds) = iterable_items(seeds_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };
    let Some(items) = iterable_items(items_expr) else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let seed_len = seeds.len();
    let arity = match arity_expr {
        Some(expr) => {
            let Some(arity) = expr_as_non_negative_usize(expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    if collect_all {
                        "SequenceFoldList"
                    } else {
                        "SequenceFold"
                    },
                    raw_args,
                    "expected a non-negative integer arity as the fourth argument",
                ));
            };
            arity
        }
        None => seed_len.saturating_add(1),
    };

    if arity <= seed_len {
        return Ok(session.invalid_call_with_arg_form(
            if collect_all {
                "SequenceFoldList"
            } else {
                "SequenceFold"
            },
            raw_args,
            "expected an arity larger than the seed length in the current first-pass subset",
        ));
    }

    let consumed_per_step = arity - seed_len;
    let mut history = seeds.clone();
    let mut offset = 0usize;

    while offset + consumed_per_step <= items.len() {
        let mut call_args = if seed_len == 0 {
            Vec::new()
        } else {
            history
                .iter()
                .rev()
                .take(seed_len)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        };
        call_args.extend(items[offset..offset + consumed_per_step].iter().cloned());

        history.push(session.evaluate(Expr::call(operator.clone(), call_args))?);
        offset += consumed_per_step;
    }

    if collect_all {
        Ok(Expr::call(system_expr("List"), history))
    } else {
        history
            .last()
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| Ok(Expr::call(head.clone(), raw_args.to_vec())))
    }
}
