use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FoldWhileConfig {
    test_arity: NestWhileTestArity,
    tail_adjustment: isize,
}

#[derive(Clone)]
struct FoldEvalState {
    operator: Expr,
    items: Vec<Expr>,
    collect_all: bool,
}

#[derive(Clone)]
struct FoldPairEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    projector: Option<Expr>,
    raw_args: Vec<Expr>,
}

#[derive(Clone)]
struct FoldWhileEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    test: Expr,
    config: FoldWhileConfig,
    raw_args: Vec<Expr>,
}

#[derive(Clone)]
struct SequenceFoldEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    seed_len: usize,
    consumed_per_step: usize,
    collect_all: bool,
    raw_args: Vec<Expr>,
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

    fn resume_fold_from_index(
        &mut self,
        state: FoldEvalState,
        index: usize,
        current: Expr,
        history: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= state.items.len() {
            return Ok(finish_fold_result(state.collect_all, current, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current, state.items[index].clone()],
        );
        let suspended_state = state.clone();
        let suspended_history = history.clone();
        let value_result = self.evaluate(call);
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_fold_with_value(suspended_state, index, suspended_history, value)
        })?;

        self.resume_fold_with_value(state, index, history, value)
    }

    fn resume_fold_with_value(
        &mut self,
        state: FoldEvalState,
        index: usize,
        mut history: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if state.collect_all {
            history.push(value.clone());
        }

        self.resume_fold_from_index(state, index + 1, value, history)
    }

    fn resume_fold_pair_from_index(
        &mut self,
        state: FoldPairEvalState,
        index: usize,
        current_state: Expr,
        last_output: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= state.items.len() {
            return Ok(finish_fold_pair_result(state, last_output));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current_state, state.items[index].clone()],
        );
        let suspended_state = state.clone();
        let suspended_last_output = last_output.clone();
        let value_result = self.evaluate(call);
        let pair = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_fold_pair_with_pair_result(
                suspended_state,
                index,
                suspended_last_output,
                value,
            )
        })?;

        self.resume_fold_pair_with_pair_result(state, index, last_output, pair)
    }

    fn resume_fold_pair_with_pair_result(
        &mut self,
        state: FoldPairEvalState,
        index: usize,
        last_output: Option<Expr>,
        pair: Expr,
    ) -> Result<Expr, EvalError> {
        let Some((x_value, y_value)) = parse_pair_result(&pair) else {
            return Ok(fold_pair_unevaluated(state));
        };

        let Some(projector) = state.projector.clone() else {
            return self.resume_fold_pair_with_output(state, index, y_value, last_output, x_value);
        };

        let call = Expr::call(
            projector,
            vec![Expr::call(
                system_expr("List"),
                vec![x_value.clone(), y_value.clone()],
            )],
        );
        let suspended_state = state.clone();
        let suspended_y_value = y_value.clone();
        let suspended_last_output = last_output.clone();
        let value_result = self.evaluate(call);
        let output = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_fold_pair_with_output(
                suspended_state,
                index,
                suspended_y_value,
                suspended_last_output,
                value,
            )
        })?;

        self.resume_fold_pair_with_output(state, index, y_value, last_output, output)
    }

    fn resume_fold_pair_with_output(
        &mut self,
        state: FoldPairEvalState,
        index: usize,
        next_state: Expr,
        _last_output: Option<Expr>,
        output: Expr,
    ) -> Result<Expr, EvalError> {
        self.resume_fold_pair_from_index(state, index + 1, next_state, Some(output))
    }

    fn resume_fold_while_from_index(
        &mut self,
        state: FoldWhileEvalState,
        index: usize,
        current: Expr,
        history: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= state.items.len() {
            return Ok(finish_fold_while_result(state, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current, state.items[index].clone()],
        );
        let suspended_state = state.clone();
        let suspended_history = history.clone();
        let value_result = self.evaluate(call);
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_fold_while_with_step_value(
                suspended_state,
                index,
                suspended_history,
                value,
            )
        })?;

        self.resume_fold_while_with_step_value(state, index, history, value)
    }

    fn resume_fold_while_with_step_value(
        &mut self,
        state: FoldWhileEvalState,
        index: usize,
        mut history: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        history.push(value.clone());
        let next_index = index + 1;
        let call = Expr::call(
            state.test.clone(),
            nest_while_test_args(&history, state.config.test_arity),
        );
        let suspended_state = state.clone();
        let suspended_history = history.clone();
        let suspended_value = value.clone();
        let value_result = self.evaluate(call);
        let test_value = self.suspend_with_frame(value_result, move |session, test_value| {
            session.resume_fold_while_with_test_value(
                suspended_state,
                next_index,
                suspended_value,
                suspended_history,
                test_value,
            )
        })?;

        self.resume_fold_while_with_test_value(state, next_index, value, history, test_value)
    }

    fn resume_fold_while_with_test_value(
        &mut self,
        state: FoldWhileEvalState,
        index: usize,
        current: Expr,
        mut history: Vec<Expr>,
        test_value: Expr,
    ) -> Result<Expr, EvalError> {
        if truthiness_symbol(&test_value) == Some(true) {
            return self.resume_fold_while_from_index(state, index, current, history);
        }

        if state.config.tail_adjustment > 0 {
            let remaining = usize::try_from(state.config.tail_adjustment).unwrap_or(0);
            return self
                .resume_fold_while_tail_from_index(state, index, current, history, remaining);
        }

        if state.config.tail_adjustment < 0 {
            let drop_count = state
                .config
                .tail_adjustment
                .checked_neg()
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or(usize::MAX);
            let keep_len = history.len().saturating_sub(drop_count).max(1);
            history.truncate(keep_len);
        }

        Ok(finish_fold_while_result(state, history))
    }

    fn resume_fold_while_tail_from_index(
        &mut self,
        state: FoldWhileEvalState,
        index: usize,
        current: Expr,
        history: Vec<Expr>,
        remaining: usize,
    ) -> Result<Expr, EvalError> {
        if remaining == 0 || index >= state.items.len() {
            return Ok(finish_fold_while_result(state, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current, state.items[index].clone()],
        );
        let suspended_state = state.clone();
        let suspended_history = history.clone();
        let value_result = self.evaluate(call);
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_fold_while_tail_with_value(
                suspended_state,
                index,
                suspended_history,
                remaining,
                value,
            )
        })?;

        self.resume_fold_while_tail_with_value(state, index, history, remaining, value)
    }

    fn resume_fold_while_tail_with_value(
        &mut self,
        state: FoldWhileEvalState,
        index: usize,
        mut history: Vec<Expr>,
        remaining: usize,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        history.push(value.clone());
        self.resume_fold_while_tail_from_index(state, index + 1, value, history, remaining - 1)
    }

    fn resume_sequence_fold_from_offset(
        &mut self,
        state: SequenceFoldEvalState,
        offset: usize,
        history: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if offset + state.consumed_per_step > state.items.len() {
            return Ok(finish_sequence_fold_result(state, history));
        }

        let mut call_args = if state.seed_len == 0 {
            Vec::new()
        } else {
            history
                .iter()
                .rev()
                .take(state.seed_len)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        };
        call_args.extend(
            state.items[offset..offset + state.consumed_per_step]
                .iter()
                .cloned(),
        );

        let suspended_state = state.clone();
        let suspended_history = history.clone();
        let value_result = self.evaluate(Expr::call(state.operator.clone(), call_args));
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_sequence_fold_with_value(
                suspended_state,
                offset,
                suspended_history,
                value,
            )
        })?;

        self.resume_sequence_fold_with_value(state, offset, history, value)
    }

    fn resume_sequence_fold_with_value(
        &mut self,
        state: SequenceFoldEvalState,
        offset: usize,
        mut history: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        history.push(value);
        let next_offset = offset + state.consumed_per_step;
        self.resume_sequence_fold_from_offset(state, next_offset, history)
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

    let (current, start_index, history) = if let Some(initial) = initial {
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

    session.resume_fold_from_index(
        FoldEvalState {
            operator: operator.clone(),
            items,
            collect_all,
        },
        start_index,
        current,
        history,
    )
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

    let (state, start_index) = if let Some(initial) = initial {
        (initial, 0usize)
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };
        (first.clone(), items.len() - rest.len())
    };

    session.resume_fold_pair_from_index(
        FoldPairEvalState {
            head: head.clone(),
            operator: operator.clone(),
            items,
            projector,
            raw_args: raw_args.to_vec(),
        },
        start_index,
        state,
        None,
    )
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

    let (current, start_index, history) = if let Some(initial) = initial {
        (initial.clone(), 0usize, vec![initial])
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call(head.clone(), raw_args.to_vec()));
        };
        (first.clone(), items.len() - rest.len(), vec![first.clone()])
    };

    session.resume_fold_while_from_index(
        FoldWhileEvalState {
            head: head.clone(),
            operator: operator.clone(),
            items,
            test: test.clone(),
            config,
            raw_args: raw_args.to_vec(),
        },
        start_index,
        current,
        history,
    )
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

    session.resume_sequence_fold_from_offset(
        SequenceFoldEvalState {
            head: head.clone(),
            operator: operator.clone(),
            items,
            seed_len,
            consumed_per_step: arity - seed_len,
            collect_all,
            raw_args: raw_args.to_vec(),
        },
        0,
        seeds,
    )
}

fn finish_fold_result(collect_all: bool, current: Expr, history: Vec<Expr>) -> Expr {
    if collect_all {
        Expr::call(system_expr("List"), history)
    } else {
        current
    }
}

fn finish_fold_pair_result(state: FoldPairEvalState, last_output: Option<Expr>) -> Expr {
    last_output.unwrap_or_else(|| Expr::call(state.head, state.raw_args))
}

fn fold_pair_unevaluated(state: FoldPairEvalState) -> Expr {
    Expr::call(state.head, state.raw_args)
}

fn finish_fold_while_result(state: FoldWhileEvalState, history: Vec<Expr>) -> Expr {
    history
        .last()
        .cloned()
        .unwrap_or_else(|| Expr::call(state.head, state.raw_args))
}

fn finish_sequence_fold_result(state: SequenceFoldEvalState, history: Vec<Expr>) -> Expr {
    if state.collect_all {
        Expr::call(system_expr("List"), history)
    } else {
        history
            .last()
            .cloned()
            .unwrap_or_else(|| Expr::call(state.head, state.raw_args))
    }
}
