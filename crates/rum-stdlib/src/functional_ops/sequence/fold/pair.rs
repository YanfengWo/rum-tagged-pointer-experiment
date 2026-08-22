use super::*;

struct FoldPairEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    projector: Option<Expr>,
    raw_args: Vec<Expr>,
}

fn resume_fold_pair_from_index(
    session: &mut Session,
    state: FoldPairEvalState,
    mut index: usize,
    mut current_state: Expr,
    mut last_output: Option<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.items.len() {
            return Ok(finish_fold_pair_result(state, last_output));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current_state, state.items[index].clone()],
        );
        let pair = match session.evaluate(call) {
            Ok(pair) => pair,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, pair| {
                        resume_fold_pair_with_pair_result(session, state, index, pair)
                    },
                );
            }
            Err(error) => return Err(error),
        };

        let Some((x_value, y_value)) = parse_pair_result(&pair) else {
            return Ok(fold_pair_unevaluated(state));
        };

        let output = if let Some(projector) = state.projector.as_ref() {
            let call = Expr::call(
                projector.clone(),
                vec![Expr::call(
                    system_expr("List"),
                    vec![x_value.clone(), y_value.clone()],
                )],
            );
            match session.evaluate(call) {
                Ok(output) => output,
                Err(EvalError::InputRequired { request }) => {
                    return session.suspend_with_frame(
                        Err(EvalError::InputRequired { request }),
                        move |session, output| {
                            resume_fold_pair_with_output(session, state, index, y_value, output)
                        },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            x_value
        };

        current_state = y_value;
        last_output = Some(output);
        index += 1;
    }
}

fn resume_fold_pair_with_pair_result(
    session: &mut Session,
    state: FoldPairEvalState,
    index: usize,
    pair: Expr,
) -> Result<Expr, EvalError> {
    let Some((x_value, y_value)) = parse_pair_result(&pair) else {
        return Ok(fold_pair_unevaluated(state));
    };

    let Some(projector) = state.projector.as_ref() else {
        return resume_fold_pair_with_output(session, state, index, y_value, x_value);
    };

    let call = Expr::call(
        projector.clone(),
        vec![Expr::call(
            system_expr("List"),
            vec![x_value.clone(), y_value.clone()],
        )],
    );
    let output = match session.evaluate(call) {
        Ok(output) => output,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame(
                Err(EvalError::InputRequired { request }),
                move |session, output| {
                    resume_fold_pair_with_output(session, state, index, y_value, output)
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_fold_pair_with_output(session, state, index, y_value, output)
}

fn resume_fold_pair_with_output(
    session: &mut Session,
    state: FoldPairEvalState,
    index: usize,
    next_state: Expr,
    output: Expr,
) -> Result<Expr, EvalError> {
    resume_fold_pair_from_index(session, state, index + 1, next_state, Some(output))
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

    resume_fold_pair_from_index(
        session,
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
