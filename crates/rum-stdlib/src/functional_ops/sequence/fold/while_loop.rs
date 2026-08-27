use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct FoldWhileEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    test: Expr,
    config: FoldWhileConfig,
    raw_args: Vec<Expr>,
}

fn resume_fold_while_from_index(
    session: &mut Session,
    state: FoldWhileEvalState,
    mut index: usize,
    mut current: Expr,
    mut history: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.items.len() {
            return Ok(finish_fold_while_result(state, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current, state.items[index].clone()],
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_fold_while_with_step_value(session, state, index, history, value)
                    },
                );
            }
            Err(error) => return Err(error),
        };

        history.push(value.clone());
        let next_index = index + 1;
        let call = Expr::call(
            state.test.clone(),
            nest_while_test_args(&history, state.config.test_arity),
        );
        let test_value = match session.evaluate(call) {
            Ok(test_value) => test_value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, test_value| {
                        resume_fold_while_with_test_value(
                            session, state, next_index, value, history, test_value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&test_value) == Some(true) {
            current = value;
            index = next_index;
            continue;
        }

        return finish_fold_while_after_false_test(session, state, next_index, value, history);
    }
}

fn resume_fold_while_with_step_value(
    session: &mut Session,
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
    let test_value = match session.evaluate(call) {
        Ok(test_value) => test_value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame(
                Err(EvalError::InputRequired { request }),
                move |session, test_value| {
                    resume_fold_while_with_test_value(
                        session, state, next_index, value, history, test_value,
                    )
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_fold_while_with_test_value(session, state, next_index, value, history, test_value)
}

fn resume_fold_while_with_test_value(
    session: &mut Session,
    state: FoldWhileEvalState,
    index: usize,
    current: Expr,
    history: Vec<Expr>,
    test_value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&test_value) == Some(true) {
        return resume_fold_while_from_index(session, state, index, current, history);
    }

    finish_fold_while_after_false_test(session, state, index, current, history)
}

fn finish_fold_while_after_false_test(
    session: &mut Session,
    state: FoldWhileEvalState,
    index: usize,
    current: Expr,
    mut history: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if state.config.tail_adjustment > 0 {
        let remaining = usize::try_from(state.config.tail_adjustment).unwrap_or(0);
        return resume_fold_while_tail_from_index(
            session, state, index, current, history, remaining,
        );
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
    session: &mut Session,
    state: FoldWhileEvalState,
    mut index: usize,
    mut current: Expr,
    mut history: Vec<Expr>,
    mut remaining: usize,
) -> Result<Expr, EvalError> {
    loop {
        if remaining == 0 || index >= state.items.len() {
            return Ok(finish_fold_while_result(state, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current, state.items[index].clone()],
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_fold_while_tail_with_value(
                            session, state, index, history, remaining, value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        };

        history.push(value.clone());
        current = value;
        index += 1;
        remaining -= 1;
    }
}

fn resume_fold_while_tail_with_value(
    session: &mut Session,
    state: FoldWhileEvalState,
    index: usize,
    mut history: Vec<Expr>,
    remaining: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    history.push(value.clone());
    resume_fold_while_tail_from_index(session, state, index + 1, value, history, remaining - 1)
}

pub(super) fn apply_fold_while(
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

    resume_fold_while_from_index(
        session,
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

fn finish_fold_while_result(state: FoldWhileEvalState, history: Vec<Expr>) -> Expr {
    history
        .last()
        .cloned()
        .unwrap_or_else(|| Expr::call(state.head, state.raw_args))
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FoldWhileEvalState>();
};
