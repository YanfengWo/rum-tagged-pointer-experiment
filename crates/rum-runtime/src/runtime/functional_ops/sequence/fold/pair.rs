use super::*;

#[derive(Clone)]
struct FoldPairEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    projector: Option<Expr>,
    raw_args: Vec<Expr>,
}

impl Session {
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
}

pub(super) fn apply_fold_pair(
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

fn finish_fold_pair_result(state: FoldPairEvalState, last_output: Option<Expr>) -> Expr {
    last_output.unwrap_or_else(|| Expr::call(state.head, state.raw_args))
}

fn fold_pair_unevaluated(state: FoldPairEvalState) -> Expr {
    Expr::call(state.head, state.raw_args)
}
