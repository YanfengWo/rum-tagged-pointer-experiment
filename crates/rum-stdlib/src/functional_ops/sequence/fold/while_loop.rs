mod resume_frames;
use resume_frames::*;

use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct FoldWhileEvalState {
    operator: Expr,
    items: Vec<Expr>,
    test: Expr,
    config: FoldWhileConfig,
    fallback: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedFoldWhileEvalState {
    operator: Expr,
    items: FrozenFunctionalSequence<Expr>,
    test: Expr,
    config: FoldWhileConfig,
    fallback: Expr,
}

impl SuspendedFoldWhileEvalState {
    fn freeze(state: FoldWhileEvalState) -> Self {
        Self {
            operator: state.operator,
            items: FrozenFunctionalSequence::freeze(state.items),
            test: state.test,
            config: state.config,
            fallback: state.fallback,
        }
    }
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
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedFoldWhileEvalState::freeze(state);
                                        let history = PersistentFunctionalSequence::freeze(history);
                ResumeFoldWhileFromIndexSuspensionFrame1 { state, index, history }
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
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedFoldWhileEvalState::freeze(state);
                                        let history = PersistentFunctionalSequence::freeze(history);
                ResumeFoldWhileFromIndexSuspensionFrame2 { state, next_index, value, history }
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

fn resume_suspended_fold_while_from_index(
    session: &mut Session,
    state: SuspendedFoldWhileEvalState,
    mut index: usize,
    mut current: Expr,
    mut history: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.items.len() {
            return Ok(finish_suspended_fold_while_result(state, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![
                current,
                state
                    .items
                    .get(index)
                    .expect("fold-while item should exist")
                    .clone(),
            ],
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedFoldWhileFromIndexSuspensionFrame1 { state, index, history },
                );
            }
            Err(error) => return Err(error),
        };

        history = history.appended(value.clone());
        let next_index = index + 1;
        let call = Expr::call(
            state.test.clone(),
            suspended_test_args(&history, state.config.test_arity),
        );
        let test_value = match session.evaluate(call) {
            Ok(test_value) => test_value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedFoldWhileFromIndexSuspensionFrame2 { state, next_index, value, history },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&test_value) == Some(true) {
            current = value;
            index = next_index;
            continue;
        }

        return finish_suspended_fold_while_after_false_test(
            session, state, next_index, value, history,
        );
    }
}

fn resume_suspended_fold_while_with_step_value(
    session: &mut Session,
    state: SuspendedFoldWhileEvalState,
    index: usize,
    history: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let history = history.appended(value.clone());
    let next_index = index + 1;
    let call = Expr::call(
        state.test.clone(),
        suspended_test_args(&history, state.config.test_arity),
    );
    let test_value = match session.evaluate(call) {
        Ok(test_value) => test_value,
        Err(EvalError::InputRequired { request }) => {
            return rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(value) => Ok(value),
                suspend => ResumeSuspendedFoldWhileWithStepValueSuspensionFrame { state, next_index, value, history },
            );
        }
        Err(error) => return Err(error),
    };

    resume_suspended_fold_while_with_test_value(
        session, state, next_index, value, history, test_value,
    )
}

fn resume_suspended_fold_while_with_test_value(
    session: &mut Session,
    state: SuspendedFoldWhileEvalState,
    index: usize,
    current: Expr,
    history: PersistentFunctionalSequence<Expr>,
    test_value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&test_value) == Some(true) {
        return resume_suspended_fold_while_from_index(session, state, index, current, history);
    }

    finish_suspended_fold_while_after_false_test(session, state, index, current, history)
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
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedFoldWhileEvalState::freeze(state);
                                        let history = PersistentFunctionalSequence::freeze(history);
                ResumeFoldWhileTailFromIndexSuspensionFrame { state, index, history, remaining }
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

fn finish_suspended_fold_while_after_false_test(
    session: &mut Session,
    state: SuspendedFoldWhileEvalState,
    index: usize,
    current: Expr,
    history: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    if state.config.tail_adjustment > 0 {
        let remaining = usize::try_from(state.config.tail_adjustment).unwrap_or(0);
        return resume_suspended_fold_while_tail_from_index(
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
        let mut values = history.to_vec();
        let keep_len = values.len().saturating_sub(drop_count).max(1);
        values.truncate(keep_len);
        return Ok(values.last().cloned().unwrap_or(state.fallback));
    }

    Ok(finish_suspended_fold_while_result(state, history))
}

fn resume_suspended_fold_while_tail_from_index(
    session: &mut Session,
    state: SuspendedFoldWhileEvalState,
    mut index: usize,
    mut current: Expr,
    mut history: PersistentFunctionalSequence<Expr>,
    mut remaining: usize,
) -> Result<Expr, EvalError> {
    loop {
        if remaining == 0 || index >= state.items.len() {
            return Ok(finish_suspended_fold_while_result(state, history));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![
                current,
                state
                    .items
                    .get(index)
                    .expect("fold-while tail item should exist")
                    .clone(),
            ],
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedFoldWhileTailFromIndexSuspensionFrame { state, index, history, remaining },
                );
            }
            Err(error) => return Err(error),
        };

        history = history.appended(value.clone());
        current = value;
        index += 1;
        remaining -= 1;
    }
}

fn resume_suspended_fold_while_tail_with_value(
    session: &mut Session,
    state: SuspendedFoldWhileEvalState,
    index: usize,
    history: PersistentFunctionalSequence<Expr>,
    remaining: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    let history = history.appended(value.clone());
    resume_suspended_fold_while_tail_from_index(
        session,
        state,
        index + 1,
        value,
        history,
        remaining - 1,
    )
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
    raw_args: &rum_core::SharedArgs,
) -> Result<Expr, EvalError> {
    let Some(config) = parse_fold_while_config(m_expr, n_expr) else {
        return Ok(session.invalid_call_with_arg_form(
            "FoldWhile",
            raw_args,
            "expected a positive integer or All test arity, with an optional signed integer tail adjustment",
        ));
    };

    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
    };

    let (current, start_index, history) = if let Some(initial) = initial {
        (initial.clone(), 0usize, vec![initial])
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
        };
        (first.clone(), items.len() - rest.len(), vec![first.clone()])
    };

    resume_fold_while_from_index(
        session,
        FoldWhileEvalState {
            operator: operator.clone(),
            items,
            test: test.clone(),
            config,
            fallback: Expr::call_with_shared_args(head.clone(), raw_args.clone()),
        },
        start_index,
        current,
        history,
    )
}

fn finish_fold_while_result(state: FoldWhileEvalState, history: Vec<Expr>) -> Expr {
    history.last().cloned().unwrap_or(state.fallback)
}

fn finish_suspended_fold_while_result(
    state: SuspendedFoldWhileEvalState,
    history: PersistentFunctionalSequence<Expr>,
) -> Expr {
    history.last().cloned().unwrap_or(state.fallback)
}

fn suspended_test_args(
    history: &PersistentFunctionalSequence<Expr>,
    arity: NestWhileTestArity,
) -> Vec<Expr> {
    match arity {
        NestWhileTestArity::Count(count) => history.recent(count),
        NestWhileTestArity::All => history.to_vec(),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FoldWhileEvalState>();
    assert_traces::<SuspendedFoldWhileEvalState>();
};
