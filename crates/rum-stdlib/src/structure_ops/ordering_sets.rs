use std::cmp::Ordering;

use super::*;

#[derive(Clone, Debug)]
struct SortByEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

pub(crate) fn eval_sort(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [expr] => sort_normal_expression(session, "Sort", args, expr),
        [expr, ordering] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "Sort", args, 1));
            };

            let items = expr_args.to_vec();
            let before = vec![vec![false; items.len()]; items.len()];
            resume_sort_comparator_from_pair(
                session,
                args.to_vec(),
                expr_head.as_ref().clone(),
                ordering.clone(),
                items,
                0,
                0,
                before,
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("Sort", args, 1, 2)),
    }
}

pub(crate) fn eval_sort_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [function] => Ok(sort_by_operator_form(function)),
        [expr, function] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "SortBy", args, 1));
            };

            resume_sort_by_from_index(
                session,
                args.to_vec(),
                expr_head.as_ref().clone(),
                function.clone(),
                expr_args.to_vec(),
                0,
                Vec::new(),
            )
        }
        [expr, _function, _ordering] => {
            if matches!(expr, Expr::Atom(_)) {
                Ok(invalid_ordering_normal_call(session, "SortBy", args, 1))
            } else {
                Ok(Expr::call(head.clone(), args.to_vec()))
            }
        }
        _ => Ok(session.invalid_call_with_arg_range("SortBy", args, 1, 3)),
    }
}

pub(crate) fn eval_union(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(Expr::call(system_expr("List"), Vec::new()));
    }

    let (expr_head, groups) = match same_head_argument_groups(session, "Union", args) {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let items = groups.into_iter().flatten().collect::<Vec<_>>();
    Ok(Expr::call(expr_head, sorted_unique_exprs(items)))
}

pub(crate) fn eval_intersection(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Intersection",
            args,
            "argm",
            "Intersection called with 0 arguments; 1 or more arguments are expected.",
        ));
    }

    let (expr_head, mut groups) = match same_head_argument_groups(session, "Intersection", args) {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let first = groups.remove(0);
    let common = first
        .into_iter()
        .filter(|item| {
            groups
                .iter()
                .all(|group| group.iter().any(|candidate| candidate == item))
        })
        .collect::<Vec<_>>();
    Ok(Expr::call(expr_head, sorted_unique_exprs(common)))
}

pub(crate) fn eval_complement(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Complement",
            args,
            "argm",
            "Complement called with 0 arguments; 1 or more arguments are expected.",
        ));
    }

    let (expr_head, mut groups) = match same_head_argument_groups(session, "Complement", args) {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let first = groups.remove(0);
    let excluded = groups.into_iter().flatten().collect::<Vec<_>>();
    let kept = first
        .into_iter()
        .filter(|item| !excluded.iter().any(|excluded| excluded == item))
        .collect::<Vec<_>>();
    Ok(Expr::call(expr_head, sorted_unique_exprs(kept)))
}

fn sort_normal_expression(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    expr: &Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_ordering_normal_call(session, symbol, args, 1));
    };

    let mut sorted = expr_args.to_vec();
    sorted.sort_by(compare_structural_order);
    Ok(Expr::call(expr_head.as_ref().clone(), sorted))
}

fn same_head_argument_groups(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Result<(Expr, Vec<Vec<Expr>>), Expr> {
    let Some((first_head, first_args)) = call_head_and_args(&args[0]) else {
        return Err(invalid_ordering_normal_call(session, symbol, args, 1));
    };

    let mut groups = vec![first_args.to_vec()];
    for (index, arg) in args.iter().enumerate().skip(1) {
        let Some((head, items)) = call_head_and_args(arg) else {
            return Err(invalid_ordering_normal_call(
                session,
                symbol,
                args,
                index + 1,
            ));
        };
        if head != first_head {
            return Err(invalid_ordering_heads_call(
                session,
                symbol,
                args,
                index + 1,
                head,
                first_head,
            ));
        }
        groups.push(items.to_vec());
    }

    Ok((first_head.clone(), groups))
}

fn resume_sort_by_from_index(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    decorated: Vec<SortByEntry>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        let mut sorted = decorated;
        sorted.sort_by(|lhs, rhs| {
            compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
        });
        return Ok(Expr::call(
            expr_head,
            sorted.into_iter().map(|entry| entry.item).collect(),
        ));
    };

    let key_result = session.evaluate(Expr::call(function.clone(), vec![item.clone()]));
    let suspended_call_args = call_args.clone();
    let suspended_expr_head = expr_head.clone();
    let suspended_function = function.clone();
    let suspended_items = items.clone();
    let suspended_decorated = decorated.clone();
    let suspended_item = item.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_sort_by_with_key(
            session,
            suspended_call_args,
            suspended_expr_head,
            suspended_function,
            suspended_items,
            index,
            suspended_decorated,
            suspended_item,
            key,
        )
    })?;

    resume_sort_by_with_key(
        session, call_args, expr_head, function, items, index, decorated, item, key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_by_with_key(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    mut decorated: Vec<SortByEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    decorated.push(SortByEntry { item, key, index });
    resume_sort_by_from_index(
        session,
        call_args,
        expr_head,
        function,
        items,
        index + 1,
        decorated,
    )
}

fn resume_sort_comparator_from_pair(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    mut lhs_index: usize,
    mut rhs_index: usize,
    before: Vec<Vec<bool>>,
) -> Result<Expr, EvalError> {
    loop {
        if lhs_index >= items.len() {
            return Ok(sort_by_comparator_table(expr_head, items, &before));
        }
        if rhs_index >= items.len() {
            lhs_index += 1;
            rhs_index = 0;
            continue;
        }
        if lhs_index != rhs_index {
            break;
        }
        rhs_index += 1;
    }

    let comparison_result = session.evaluate(Expr::call(
        ordering.clone(),
        vec![items[lhs_index].clone(), items[rhs_index].clone()],
    ));
    let suspended_call_args = call_args.clone();
    let suspended_expr_head = expr_head.clone();
    let suspended_ordering = ordering.clone();
    let suspended_items = items.clone();
    let suspended_before = before.clone();
    let value = session.suspend_with_frame(comparison_result, move |session, value| {
        resume_sort_comparator_with_value(
            session,
            suspended_call_args,
            suspended_expr_head,
            suspended_ordering,
            suspended_items,
            lhs_index,
            rhs_index,
            suspended_before,
            value,
        )
    })?;

    resume_sort_comparator_with_value(
        session, call_args, expr_head, ordering, items, lhs_index, rhs_index, before, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_comparator_with_value(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
    mut before: Vec<Vec<bool>>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_system_symbol_name(&value, "True") {
        before[lhs_index][rhs_index] = true;
    }

    resume_sort_comparator_from_pair(
        session,
        call_args,
        expr_head,
        ordering,
        items,
        lhs_index,
        rhs_index + 1,
        before,
    )
}

fn invalid_ordering_normal_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    position: usize,
) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position {position} in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_ordering_heads_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    position: usize,
    actual_head: &Expr,
    expected_head: &Expr,
) -> Expr {
    session.invalid_call_with_message(
            symbol,
            args,
            "heads",
            format!(
                "Heads {actual_head} and {expected_head} at positions {position} and 1 are expected to be the same."
            ),
        )
}

fn sort_by_operator_form(function: &Expr) -> Expr {
    let arg = Expr::symbol("rum$sortArg");
    Expr::call(
        system_expr("Function"),
        vec![
            arg.clone(),
            Expr::call(system_expr("SortBy"), vec![arg, function.clone()]),
        ],
    )
}

fn call_head_and_args(expr: &Expr) -> Option<(&Expr, &[Expr])> {
    match expr {
        Expr::Call { head, args } => Some((head.as_ref(), args)),
        Expr::Atom(_) => None,
    }
}

fn sorted_unique_exprs(mut items: Vec<Expr>) -> Vec<Expr> {
    items.sort_by(compare_structural_order);
    let mut unique = Vec::new();
    for item in items {
        if unique.iter().any(|existing| existing == &item) {
            continue;
        }
        unique.push(item);
    }
    unique
}

fn sort_by_comparator_table(expr_head: Expr, items: Vec<Expr>, before: &[Vec<bool>]) -> Expr {
    let mut indices = (0..items.len()).collect::<Vec<_>>();
    indices.sort_by(|lhs, rhs| match (before[*lhs][*rhs], before[*rhs][*lhs]) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => lhs.cmp(rhs),
    });

    Expr::call(
        expr_head,
        indices
            .into_iter()
            .map(|index| items[index].clone())
            .collect(),
    )
}

pub(super) fn compare_structural_order(lhs: &Expr, rhs: &Expr) -> Ordering {
    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => compare_atom_order(lhs, rhs),
        (Expr::Atom(_), Expr::Call { .. }) => Ordering::Less,
        (Expr::Call { .. }, Expr::Atom(_)) => Ordering::Greater,
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
            },
        ) => compare_structural_order(lhs_head, rhs_head)
            .then_with(|| compare_expr_slices(lhs_args, rhs_args)),
    }
}

fn compare_atom_order(lhs: &Atom, rhs: &Atom) -> Ordering {
    atom_order_rank(lhs)
        .cmp(&atom_order_rank(rhs))
        .then_with(|| match (lhs, rhs) {
            (Atom::Integer(lhs), Atom::Integer(rhs)) => lhs.cmp(rhs),
            (Atom::String(lhs), Atom::String(rhs)) => lhs.cmp(rhs),
            (Atom::Symbol(lhs), Atom::Symbol(rhs)) => lhs.to_string().cmp(&rhs.to_string()),
            (Atom::Rational(lhs), Atom::Rational(rhs)) => lhs.cmp(rhs),
            _ => lhs.to_string().cmp(&rhs.to_string()),
        })
}

fn atom_order_rank(atom: &Atom) -> u8 {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) => 0,
        Atom::String(_) => 1,
        Atom::Symbol(_) => 2,
    }
}

fn compare_expr_slices(lhs: &[Expr], rhs: &[Expr]) -> Ordering {
    for (lhs, rhs) in lhs.iter().zip(rhs) {
        let ordering = compare_structural_order(lhs, rhs);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    lhs.len().cmp(&rhs.len())
}
