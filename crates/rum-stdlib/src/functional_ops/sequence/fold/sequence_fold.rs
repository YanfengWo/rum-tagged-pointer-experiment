use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct SequenceFoldEvalState {
    head: Expr,
    operator: Expr,
    items: Vec<Expr>,
    seed_len: usize,
    consumed_per_step: usize,
    collect_all: bool,
    raw_args: Vec<Expr>,
}

fn resume_sequence_fold_from_offset(
    session: &mut Session,
    state: SequenceFoldEvalState,
    mut offset: usize,
    mut history: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
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

        let value = match session.evaluate(Expr::call(state.operator.clone(), call_args)) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_sequence_fold_with_value(session, state, offset, history, value)
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        history.push(value);
        offset += state.consumed_per_step;
    }
}

fn resume_sequence_fold_with_value(
    session: &mut Session,
    state: SequenceFoldEvalState,
    offset: usize,
    mut history: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    history.push(value);
    let next_offset = offset + state.consumed_per_step;
    resume_sequence_fold_from_offset(session, state, next_offset, history)
}

pub(super) fn apply_sequence_fold(
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

    resume_sequence_fold_from_offset(
        session,
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SequenceFoldEvalState>();
};
