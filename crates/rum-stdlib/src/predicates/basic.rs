use super::*;

pub(crate) fn eval_atom_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("AtomQ", args, 1));
    };

    Ok(predicate_bool_expr(matches!(expr, Expr::Atom(_))))
}

pub(crate) fn eval_list_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ListQ", args, 1));
    };

    Ok(predicate_bool_expr(
        matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)),
    ))
}

pub(crate) fn eval_string_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("StringQ", args, 1));
    };

    Ok(predicate_bool_expr(matches!(
        expr,
        Expr::Atom(Atom::String(_))
    )))
}

pub(crate) fn eval_date_object_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DateObjectQ", args, 1));
    };

    Ok(predicate_bool_expr(expr_has_system_head(
        expr,
        "DateObject",
    )))
}

pub(crate) fn eval_time_object_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("TimeObjectQ", args, 1));
    };

    Ok(predicate_bool_expr(expr_has_system_head(
        expr,
        "TimeObject",
    )))
}
