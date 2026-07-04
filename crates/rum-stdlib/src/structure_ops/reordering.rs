use super::*;

pub(crate) fn eval_reverse(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (expr, levels) = match args {
        [expr] => (expr, BTreeSet::from([1_usize])),
        [expr, level_spec] => {
            let Some(levels) = parse_reverse_levels(level_spec) else {
                return Ok(invalid_reverse_level_call(session, args));
            };
            (expr, levels)
        }
        _ => return Ok(session.invalid_call_with_arg_range("Reverse", args, 1, 2)),
    };

    let Expr::Call { .. } = expr else {
        return Ok(invalid_reverse_normal_call(session, args));
    };

    Ok(reverse_at_levels(expr, &levels, 1))
}

pub(crate) fn eval_transpose(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (expr, permutation) = match args {
        [expr] => (expr, TransposePermutation::SwapFirstTwo),
        [expr, permutation] => {
            let Some(permutation) = parse_transpose_permutation(permutation) else {
                return Ok(invalid_transpose_permutation_call(session, args));
            };
            (expr, permutation)
        }
        _ => return Ok(session.invalid_call_with_arg_range("Transpose", args, 1, 2)),
    };

    let Some(items) = list_items(expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if permutation == TransposePermutation::Identity {
        return Ok(expr.clone());
    }

    match transpose_plain_list_matrix(items) {
        Some(result) => Ok(result),
        None => Ok(invalid_transpose_matrix_call(session, args)),
    }
}

fn invalid_reverse_normal_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Reverse",
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr("Reverse"), args.to_vec())
        ),
    )
}

fn invalid_reverse_level_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Reverse",
        args,
        "ilsmp",
        format!(
            "Single or list of positive machine-sized integers expected at position 2 of {}.",
            Expr::call(system_expr("Reverse"), args.to_vec())
        ),
    )
}

fn invalid_transpose_matrix_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Transpose",
        args,
        "nmtx",
        format!("The first two levels of {} cannot be transposed.", args[0]),
    )
}

fn invalid_transpose_permutation_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Transpose",
        args,
        "permspec",
        format!(
            "Invalid permutation specification found at position 2 in {}.",
            Expr::call(system_expr("Transpose"), args.to_vec())
        ),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransposePermutation {
    Identity,
    SwapFirstTwo,
}

fn parse_reverse_levels(expr: &Expr) -> Option<BTreeSet<usize>> {
    if let Some(level) = expr_as_positive_usize(expr) {
        return Some(BTreeSet::from([level]));
    }

    let items = list_items(expr)?;
    items.iter().map(expr_as_positive_usize).collect()
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn reverse_at_levels(expr: &Expr, levels: &BTreeSet<usize>, depth: usize) -> Expr {
    match expr {
        Expr::Atom(_) => expr.clone(),
        Expr::Call { head, args } => {
            let mut reversed = args
                .iter()
                .map(|arg| reverse_at_levels(arg, levels, depth + 1))
                .collect::<Vec<_>>();
            if levels.contains(&depth) {
                reversed.reverse();
            }
            Expr::call(head.as_ref().clone(), reversed)
        }
    }
}

fn parse_transpose_permutation(expr: &Expr) -> Option<TransposePermutation> {
    let items = list_items(expr)?;
    match items {
        [first, second]
            if expr_as_non_negative_usize(first) == Some(1)
                && expr_as_non_negative_usize(second) == Some(2) =>
        {
            Some(TransposePermutation::Identity)
        }
        [first, second]
            if expr_as_non_negative_usize(first) == Some(2)
                && expr_as_non_negative_usize(second) == Some(1) =>
        {
            Some(TransposePermutation::SwapFirstTwo)
        }
        _ => None,
    }
}

fn transpose_plain_list_matrix(items: &[Expr]) -> Option<Expr> {
    if items.is_empty() {
        return Some(Expr::call(system_expr("List"), Vec::new()));
    }

    if items.iter().all(|item| !is_plain_list_expr(item)) {
        return Some(Expr::call(system_expr("List"), items.to_vec()));
    }

    let rows = items
        .iter()
        .map(list_items)
        .collect::<Option<Vec<&[Expr]>>>()?;
    let width = rows.first().map(|row| row.len()).unwrap_or(0);
    if rows.iter().any(|row| row.len() != width) {
        return None;
    }

    Some(Expr::call(
        system_expr("List"),
        (0..width)
            .map(|column| {
                Expr::call(
                    system_expr("List"),
                    rows.iter().map(|row| row[column].clone()).collect(),
                )
            })
            .collect(),
    ))
}
