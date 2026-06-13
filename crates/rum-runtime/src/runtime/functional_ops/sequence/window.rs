use super::*;

#[derive(Clone)]
enum SequenceLikeExpr {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

#[derive(Clone, Copy)]
enum WhileSequenceMode {
    Take,
    Length,
}

#[derive(Clone)]
struct BlockMapEvalState {
    operator: Expr,
    block_head: Expr,
    items: Vec<Expr>,
    size: usize,
    offset: usize,
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

                self.length_while_sequence(sequence, criterion)
            }
            _ => Ok(self.invalid_call_with_arg_range("LengthWhile", args, 1, 2)),
        }
    }

    fn take_while_sequence(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: &Expr,
    ) -> Result<Expr, EvalError> {
        self.resume_while_sequence_from_index(
            sequence,
            criterion.clone(),
            0,
            WhileSequenceMode::Take,
        )
    }

    fn length_while_sequence(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: &Expr,
    ) -> Result<Expr, EvalError> {
        self.resume_while_sequence_from_index(
            sequence,
            criterion.clone(),
            0,
            WhileSequenceMode::Length,
        )
    }

    fn resume_while_sequence_from_index(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: Expr,
        index: usize,
        mode: WhileSequenceMode,
    ) -> Result<Expr, EvalError> {
        if index >= sequence.len() {
            return Ok(finish_while_sequence(sequence, index, mode));
        }

        let Some(item) = sequence.value_at(index) else {
            return Ok(finish_while_sequence(sequence, index, mode));
        };
        let suspended_sequence = sequence.clone();
        let suspended_criterion = criterion.clone();
        let value_result = self.evaluate(Expr::call(criterion.clone(), vec![item]));
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_while_sequence_with_value(
                suspended_sequence,
                suspended_criterion,
                index,
                mode,
                value,
            )
        })?;

        self.resume_while_sequence_with_value(sequence, criterion, index, mode, value)
    }

    fn resume_while_sequence_with_value(
        &mut self,
        sequence: SequenceLikeExpr,
        criterion: Expr,
        index: usize,
        mode: WhileSequenceMode,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if truthiness_symbol(&value) == Some(true) {
            self.resume_while_sequence_from_index(sequence, criterion, index + 1, mode)
        } else {
            Ok(finish_while_sequence(sequence, index, mode))
        }
    }

    fn resume_accumulate_from_index(
        &mut self,
        sequence: SequenceLikeExpr,
        index: usize,
        current: Expr,
        accumulated: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index >= sequence.len() {
            return Ok(sequence.render_with_values(accumulated));
        }

        let Some(item) = sequence.value_at(index) else {
            return Ok(sequence.render_with_values(accumulated));
        };
        let suspended_sequence = sequence.clone();
        let suspended_accumulated = accumulated.clone();
        let value_result = self.evaluate(Expr::call(system_expr("Plus"), vec![current, item]));
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_accumulate_with_value(
                suspended_sequence,
                index,
                suspended_accumulated,
                value,
            )
        })?;

        self.resume_accumulate_with_value(sequence, index, accumulated, value)
    }

    fn resume_accumulate_with_value(
        &mut self,
        sequence: SequenceLikeExpr,
        index: usize,
        mut accumulated: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        accumulated.push(value.clone());
        self.resume_accumulate_from_index(sequence, index + 1, value, accumulated)
    }

    fn resume_block_map_from_index(
        &mut self,
        state: BlockMapEvalState,
        index: usize,
        results: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if index + state.size > state.items.len() {
            return Ok(Expr::call(system_expr("List"), results));
        }

        let block = Expr::call(
            state.block_head.clone(),
            state.items[index..index + state.size].to_vec(),
        );
        let suspended_state = state.clone();
        let suspended_results = results.clone();
        let value_result = self.evaluate(Expr::call(state.operator.clone(), vec![block]));
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_block_map_with_value(suspended_state, index, suspended_results, value)
        })?;

        self.resume_block_map_with_value(state, index, results, value)
    }

    fn resume_block_map_with_value(
        &mut self,
        state: BlockMapEvalState,
        index: usize,
        mut results: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        results.push(value);
        let next_index = index + state.offset;
        self.resume_block_map_from_index(state, next_index, results)
    }
}

impl SequenceLikeExpr {
    fn len(&self) -> usize {
        match self {
            Self::Call { items, .. } => items.len(),
            Self::Association(entries) => entries.len(),
        }
    }

    fn value_at(&self, index: usize) -> Option<Expr> {
        match self {
            Self::Call { items, .. } => items.get(index).cloned(),
            Self::Association(entries) => entries.get(index).map(|(_, value)| value.clone()),
        }
    }

    fn render_prefix(&self, count: usize) -> Expr {
        match self {
            Self::Call { head, items } => {
                Expr::call(head.clone(), items.iter().take(count).cloned().collect())
            }
            Self::Association(entries) => {
                render_association(entries.iter().take(count).cloned().collect())
            }
        }
    }

    fn render_with_values(&self, values: Vec<Expr>) -> Expr {
        match self {
            Self::Call { head, .. } => Expr::call(head.clone(), values),
            Self::Association(entries) => render_association(
                entries
                    .iter()
                    .map(|(key, _)| key.clone())
                    .zip(values)
                    .collect(),
            ),
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
        items: args.to_vec(),
    })
}

fn accumulate_sequence(
    session: &mut Session,
    sequence: SequenceLikeExpr,
) -> Result<Expr, EvalError> {
    let Some(first) = sequence.value_at(0) else {
        return Ok(sequence.render_with_values(Vec::new()));
    };

    session.resume_accumulate_from_index(sequence, 1, first.clone(), vec![first])
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

    session.resume_block_map_from_index(
        BlockMapEvalState {
            operator: operator.clone(),
            block_head: block_head.as_ref().clone(),
            items: items.to_vec(),
            size,
            offset,
        },
        0,
        Vec::new(),
    )
}

fn finish_while_sequence(
    sequence: SequenceLikeExpr,
    count: usize,
    mode: WhileSequenceMode,
) -> Expr {
    match mode {
        WhileSequenceMode::Take => sequence.render_prefix(count),
        WhileSequenceMode::Length => Expr::integer(BigInt::from(count)),
    }
}
