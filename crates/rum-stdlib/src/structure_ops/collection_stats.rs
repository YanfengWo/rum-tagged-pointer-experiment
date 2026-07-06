use std::cmp::Ordering;

use super::ordering_sets::compare_structural_order;
use super::*;

#[derive(Clone, Debug)]
struct KeyedEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

#[derive(Clone)]
struct DecorateByState {
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    operation: DecoratedOperation,
}

#[derive(Clone, Copy)]
enum DecoratedOperation {
    MaxBy { select_max: bool },
    TakeBy { largest: bool, count: usize },
    ReverseSortBy,
    GatherBy,
}

pub(crate) fn eval_tally(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Tally", args, 1));
    };

    let Some((_expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "Tally", args));
    };

    let mut counts = Vec::<(Expr, usize)>::new();
    for item in items {
        if let Some((_, count)) = counts.iter_mut().find(|(seen, _)| seen == &item) {
            *count += 1;
        } else {
            counts.push((item, 1));
        }
    }

    Ok(list_expr(
        counts
            .into_iter()
            .map(|(item, count)| list_expr(vec![item, Expr::integer(BigInt::from(count))]))
            .collect(),
    ))
}

pub(crate) fn eval_max_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_extreme_by(session, "MaxBy", args, true)
}

pub(crate) fn eval_min_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_extreme_by(session, "MinBy", args, false)
}

pub(crate) fn eval_take_largest_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_take_by(session, head, args, "TakeLargestBy", true)
}

pub(crate) fn eval_take_smallest_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_take_by(session, head, args, "TakeSmallestBy", false)
}

pub(crate) fn eval_reverse_sort(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ReverseSort", args, 1));
    };

    let Some((expr_head, mut items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "ReverseSort", args));
    };

    items.sort_by(compare_structural_order);
    items.reverse();
    Ok(Expr::call(expr_head, items))
}

pub(crate) fn eval_reverse_sort_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr, function] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ReverseSortBy", args, 2));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(
            session,
            "ReverseSortBy",
            args,
        ));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::ReverseSortBy,
        },
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_gather_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr, function] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("GatherBy", args, 2));
    };

    let Some((_expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "GatherBy", args));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head: system_expr("List"),
            function: function.clone(),
            items,
            operation: DecoratedOperation::GatherBy,
        },
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_riffle(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr, separator] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Riffle", args, 2));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "Riffle", args));
    };

    if items.len() <= 1 {
        return Ok(Expr::call(expr_head, items));
    }

    let output_len = items.len().saturating_mul(2).saturating_sub(1);
    session.enforce_generated_element_limit_for_count(output_len)?;

    let mut riffled = Vec::with_capacity(output_len);
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            riffled.push(separator.clone());
        }
        riffled.push(item);
    }

    Ok(Expr::call(expr_head, riffled))
}

fn eval_extreme_by(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    select_max: bool,
) -> Result<Expr, EvalError> {
    let [expr, function] = args else {
        return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 2));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };
    if items.is_empty() {
        return Ok(invalid_empty_collection_call(session, symbol, args));
    }

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::MaxBy { select_max },
        },
        0,
        Vec::new(),
    )
}

fn eval_take_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    largest: bool,
) -> Result<Expr, EvalError> {
    let [expr, function, count] = args else {
        return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 3));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };
    let Some(count) = expr_as_non_negative_usize(count) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::TakeBy { largest, count },
        },
        0,
        Vec::new(),
    )
}

fn resume_decorate_by_from_index(
    session: &mut Session,
    state: DecorateByState,
    index: usize,
    decorated: Vec<KeyedEntry>,
) -> Result<Expr, EvalError> {
    let Some(item) = state.items.get(index).cloned() else {
        return finish_decorated(state, decorated);
    };

    let key_result = session.evaluate(Expr::call(state.function.clone(), vec![item.clone()]));
    let suspended_state = state.clone();
    let suspended_decorated = decorated.clone();
    let suspended_item = item.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_decorate_by_with_key(
            session,
            suspended_state,
            index,
            suspended_decorated,
            suspended_item,
            key,
        )
    })?;

    resume_decorate_by_with_key(session, state, index, decorated, item, key)
}

fn resume_decorate_by_with_key(
    session: &mut Session,
    state: DecorateByState,
    index: usize,
    mut decorated: Vec<KeyedEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    decorated.push(KeyedEntry { item, key, index });
    resume_decorate_by_from_index(session, state, index + 1, decorated)
}

fn finish_decorated(state: DecorateByState, decorated: Vec<KeyedEntry>) -> Result<Expr, EvalError> {
    match state.operation {
        DecoratedOperation::MaxBy { select_max } => Ok(select_extreme_by(decorated, select_max)
            .unwrap_or_else(|| Expr::call(state.expr_head, Vec::new()))),
        DecoratedOperation::TakeBy { largest, count } => Ok(Expr::call(
            state.expr_head,
            take_by(decorated, largest, count),
        )),
        DecoratedOperation::ReverseSortBy => {
            let mut sorted = decorated;
            sorted.sort_by(|lhs, rhs| {
                compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
            });
            sorted.reverse();
            Ok(Expr::call(
                state.expr_head,
                sorted.into_iter().map(|entry| entry.item).collect(),
            ))
        }
        DecoratedOperation::GatherBy => Ok(list_expr(gather_by_key(decorated))),
    }
}

fn select_extreme_by(decorated: Vec<KeyedEntry>, select_max: bool) -> Option<Expr> {
    let mut entries = decorated.into_iter();
    let mut best = entries.next()?;

    for entry in entries {
        let ordering = compare_structural_order(&entry.key, &best.key);
        if (select_max && ordering == Ordering::Greater)
            || (!select_max && ordering == Ordering::Less)
        {
            best = entry;
        }
    }

    Some(best.item)
}

fn take_by(mut decorated: Vec<KeyedEntry>, largest: bool, count: usize) -> Vec<Expr> {
    decorated.sort_by(|lhs, rhs| {
        let key_order = if largest {
            compare_structural_order(&rhs.key, &lhs.key)
        } else {
            compare_structural_order(&lhs.key, &rhs.key)
        };
        key_order.then(lhs.index.cmp(&rhs.index))
    });
    decorated.truncate(count);
    decorated.into_iter().map(|entry| entry.item).collect()
}

fn gather_by_key(decorated: Vec<KeyedEntry>) -> Vec<Expr> {
    let mut groups = Vec::<(Expr, Vec<Expr>)>::new();

    for entry in decorated {
        if let Some((_, group)) = groups.iter_mut().find(|(key, _)| key == &entry.key) {
            group.push(entry.item);
        } else {
            groups.push((entry.key, vec![entry.item]));
        }
    }

    groups
        .into_iter()
        .map(|(_, group)| list_expr(group))
        .collect()
}

fn normal_call_parts(expr: &Expr) -> Option<(Expr, Vec<Expr>)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    Some((head.as_ref().clone(), args.to_vec()))
}

fn invalid_collection_normal_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_empty_collection_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "empty",
        format!("{symbol} cannot select from an empty list."),
    )
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}
