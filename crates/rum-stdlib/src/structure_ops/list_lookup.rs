use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::cmp::Ordering;
use std::collections::HashSet;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum SelectPositionsLimit {
    All,
    Count(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct SelectPositionsState {
    items: Vec<Expr>,
    criterion: Expr,
    limit: SelectPositionsLimit,
    index: usize,
    positions: Vec<Expr>,
}

impl SelectPositionsLimit {
    fn reached(self, count: usize) -> bool {
        match self {
            Self::All => false,
            Self::Count(limit) => count >= limit,
        }
    }
}

pub(crate) fn eval_select_positions(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [list, criterion] => {
            let Some(items) = list_items(list) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            resume_select_positions_from_index(
                session,
                SelectPositionsState {
                    items: items.to_vec(),
                    criterion: criterion.clone(),
                    limit: SelectPositionsLimit::All,
                    index: 0,
                    positions: Vec::new(),
                },
            )
        }
        [list, criterion, limit] => {
            let Some(items) = list_items(list) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            let Some(limit) = expr_as_non_negative_usize(limit) else {
                return Ok(session.invalid_call_with_arg_form(
                    "SelectPositions",
                    args,
                    "expected a non-negative integer selection limit",
                ));
            };
            resume_select_positions_from_index(
                session,
                SelectPositionsState {
                    items: items.to_vec(),
                    criterion: criterion.clone(),
                    limit: SelectPositionsLimit::Count(limit),
                    index: 0,
                    positions: Vec::new(),
                },
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("SelectPositions", args, 2, 3)),
    }
}

pub(crate) fn eval_duplicates(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_arg_range("Duplicates", args, 1, 1));
    };
    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    const INDEX_THRESHOLD: usize = 16;

    let mut remaining = items.iter();
    let mut seen = Vec::new();
    let mut duplicates = Vec::new();
    for item in remaining.by_ref() {
        if seen.iter().any(|existing| existing == item) {
            if !duplicates.iter().any(|existing| existing == item) {
                duplicates.push(item.clone());
            }
        } else {
            seen.push(item.clone());
            if seen.len() > INDEX_THRESHOLD {
                return Ok(Expr::call(
                    system_expr("List"),
                    collect_duplicates_indexed(seen, duplicates, remaining),
                ));
            }
        }
    }

    Ok(Expr::call(system_expr("List"), duplicates))
}

#[inline(never)]
fn collect_duplicates_indexed<'a>(
    seen: Vec<Expr>,
    mut duplicates: Vec<Expr>,
    remaining: impl Iterator<Item = &'a Expr>,
) -> Vec<Expr> {
    let mut seen = seen.into_iter().collect::<HashSet<_>>();
    let mut emitted = duplicates.iter().cloned().collect::<HashSet<_>>();
    for item in remaining {
        if !seen.insert(item.clone()) && emitted.insert(item.clone()) {
            duplicates.push(item.clone());
        }
    }
    duplicates
}

pub(crate) fn eval_binary_search(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list, value] = args else {
        return Ok(session.invalid_call_with_arg_range("BinarySearch", args, 2, 2));
    };
    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(binary_search_position(items, value)
        .map(|position| Expr::integer(BigInt::from(position)))
        .unwrap_or_else(missing_not_found_expr))
}

fn resume_select_positions_from_index(
    session: &mut Session,
    mut state: SelectPositionsState,
) -> Result<Expr, EvalError> {
    loop {
        if state.limit.reached(state.positions.len()) || state.index >= state.items.len() {
            return Ok(Expr::call(system_expr("List"), state.positions));
        }
        if state.index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let item = state.items[state.index].clone();
        let value = match session.evaluate(Expr::call(state.criterion.clone(), vec![item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_select_positions_with_value(session, state, value)
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        push_position_if_true(session, &mut state, value)?;
        state.index += 1;
    }
}

fn resume_select_positions_with_value(
    session: &mut Session,
    mut state: SelectPositionsState,
    value: Expr,
) -> Result<Expr, EvalError> {
    push_position_if_true(session, &mut state, value)?;
    state.index += 1;
    resume_select_positions_from_index(session, state)
}

fn push_position_if_true(
    session: &mut Session,
    state: &mut SelectPositionsState,
    value: Expr,
) -> Result<(), EvalError> {
    if truthiness_symbol(&value) == Some(true) {
        let next_len = state.positions.len() + 1;
        session.enforce_generated_element_limit_for_count(next_len)?;
        state.positions.push(position_path_expr(state.index + 1));
    }

    Ok(())
}

fn position_path_expr(one_based_index: usize) -> Expr {
    Expr::call(
        system_expr("List"),
        vec![Expr::integer(BigInt::from(one_based_index))],
    )
}

fn binary_search_position(items: &[Expr], value: &Expr) -> Option<usize> {
    let mut lower = 0;
    let mut upper = items.len();

    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        match compare_expr_canonical(&items[middle], value) {
            Ordering::Less => lower = middle + 1,
            Ordering::Equal | Ordering::Greater => upper = middle,
        }
    }

    (lower < items.len() && compare_expr_canonical(&items[lower], value) == Ordering::Equal)
        .then_some(lower + 1)
}

fn missing_not_found_expr() -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("NotFound")])
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SelectPositionsLimit>();
    assert_traces::<SelectPositionsState>();
};
