use super::*;

pub(crate) fn eval_order(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Order", args, 2));
    };

    Ok(order_result_expr(compare_exact_order_subset(lhs, rhs)))
}

pub(crate) fn eval_ordered_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("OrderedQ", args, 1));
    };

    let items = match expr {
        Expr::Call { args, .. } => args.as_slice(),
        Expr::Atom(_) => {
            return Ok(session.invalid_call_with_message(
                "OrderedQ",
                args,
                "normal",
                format!(
                    "Nonatomic expression expected at position 1 in {}.",
                    Expr::call(head.clone(), args.to_vec())
                ),
            ));
        }
    };

    Ok(predicate_bool_expr(items.windows(2).all(|pair| {
        compare_exact_order_subset(&pair[0], &pair[1]) != std::cmp::Ordering::Greater
    })))
}

pub(crate) fn eval_lexicographic_order(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(if (1..=3).contains(&args.len()) {
            Expr::call(head.clone(), args.to_vec())
        } else {
            session.invalid_call_with_arg_range("LexicographicOrder", args, 1, 3)
        });
    };

    Ok(order_result_expr(compare_lexicographic_order_subset(
        lhs, rhs,
    )))
}

pub(crate) fn eval_numerical_order(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("NumericalOrder", args, 2));
    };

    Ok(order_result_expr(compare_exact_order_subset(lhs, rhs)))
}

pub(crate) fn eval_alphabetic_order(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(lhs) = string_atom_value(lhs) else {
        return Ok(session.invalid_call_with_message(
            "AlphabeticOrder",
            args,
            "nstr",
            format!("argument {} is not a string", lhs),
        ));
    };
    let Some(rhs) = string_atom_value(rhs) else {
        return Ok(session.invalid_call_with_message(
            "AlphabeticOrder",
            args,
            "nstr",
            format!("argument {} is not a string", rhs),
        ));
    };

    Ok(order_result_expr(lhs.cmp(rhs)))
}

pub(crate) fn eval_between(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [value_expr, interval_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Between", args, 2));
    };

    let Some(value) = exact_numeric_value(value_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(interval_items) = list_items(interval_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let [lower_expr, upper_expr] = interval_items else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let (Some(lower), Some(upper)) = (
        exact_numeric_value(lower_expr),
        exact_numeric_value(upper_expr),
    ) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(predicate_bool_expr(
        compare_exact_numeric(&lower, &value) != std::cmp::Ordering::Greater
            && compare_exact_numeric(&value, &upper) != std::cmp::Ordering::Greater,
    ))
}

pub(crate) fn eval_same_q(
    _session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    Ok(predicate_bool_expr(
        args.windows(2).all(|pair| pair[0] == pair[1]),
    ))
}

pub(crate) fn eval_unsame_q(
    _session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    for (index, lhs) in args.iter().enumerate() {
        if args[index + 1..].iter().any(|rhs| lhs == rhs) {
            return Ok(predicate_bool_expr(false));
        }
    }

    Ok(predicate_bool_expr(true))
}
