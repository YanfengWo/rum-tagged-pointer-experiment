use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FoldWhileConfig {
    test_arity: NestWhileTestArity,
    tail_adjustment: isize,
}

impl Session {
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
}

fn iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::Call { head, args: _ } if is_system_symbol_name(head, "Association") => Some(
            association_entries_from_expr(expr)?
                .into_iter()
                .map(|(_, value)| value)
                .collect(),
        ),
        Expr::Call { args, .. } => Some(args.to_vec()),
        Expr::Atom(_) => None,
    }
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
