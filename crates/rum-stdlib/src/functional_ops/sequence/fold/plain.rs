use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct FoldEvalState {
    operator: Expr,
    items: Vec<Expr>,
    collect_all: bool,
}

fn resume_fold_from_index(
    session: &mut Session,
    state: FoldEvalState,
    mut index: usize,
    mut current: Expr,
    mut history: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.items.len() {
            return Ok(finish_fold_result(state.collect_all, current, history));
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
                        resume_fold_with_value(session, state, index, history, value)
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if state.collect_all {
            history.push(value.clone());
        }
        current = value;
        index += 1;
    }
}

fn resume_fold_with_value(
    session: &mut Session,
    state: FoldEvalState,
    index: usize,
    mut history: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if state.collect_all {
        history.push(value.clone());
    }

    resume_fold_from_index(session, state, index + 1, value, history)
}

pub(super) fn apply_fold(
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

    resume_fold_from_index(
        session,
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

fn finish_fold_result(collect_all: bool, current: Expr, history: Vec<Expr>) -> Expr {
    if collect_all {
        Expr::call(system_expr("List"), history)
    } else {
        current
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FoldEvalState>();
};
