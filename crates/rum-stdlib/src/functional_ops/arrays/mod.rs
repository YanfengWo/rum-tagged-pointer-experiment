use super::*;

mod filter;
mod generators;
mod spec;
mod tensor;

use self::filter::*;
pub(crate) use self::generators::{eval_array, eval_outer};
use self::spec::*;
use self::tensor::*;

pub(crate) fn eval_inner(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (f, left, right, g) = match args {
        [f, left, right] => (f, left, right, system_expr("Plus")),
        [f, left, right, g] => (f, left, right, g.clone()),
        _ => return Ok(session.invalid_call_with_arg_range("Inner", args, 3, 4)),
    };

    let Some(result) = compute_inner(session, "Inner", f, left, right, &g)? else {
        return Ok(Expr::call(system_expr("Inner"), args.to_vec()));
    };

    Ok(result)
}

pub(crate) fn eval_dot(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some((first, rest)) = args.split_first() else {
        return Ok(Expr::call(head.clone(), Vec::new()));
    };
    if rest.is_empty() {
        return Ok(first.clone());
    }

    let times = system_expr("Times");
    let plus = system_expr("Plus");
    let mut current = first.clone();
    for next in rest {
        let Some(result) = compute_inner(session, "Dot", &times, &current, next, &plus)? else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        current = result;
    }

    Ok(current)
}

pub(crate) fn eval_array_filter(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (operator, array, radius_spec, padding_expr) = match args {
        [operator, array, radius_spec] => (operator, array, radius_spec, None),
        [operator, array, radius_spec, padding_expr] => {
            (operator, array, radius_spec, Some(padding_expr))
        }
        _ => return Ok(session.invalid_call_with_arg_range("ArrayFilter", args, 3, 4)),
    };

    let Some((shape, flat)) = parse_rectangular_list_array(array) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayFilter",
            args,
            "expected a rectangular List array as the second argument",
        ));
    };
    let Some(spec) = parse_array_filter_spec(radius_spec, shape.len()) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayFilter",
            args,
            "expected a non-negative integer radius, a matching list of non-negative integer radii, or a matching rectangular 0/1 template array as the third argument",
        ));
    };
    let effective_padding_expr = match padding_expr.cloned() {
        Some(expr) => Some(expr),
        None => session.option_rule_expr_for_symbol(
            &Symbol::system("ArrayFilter"),
            &Symbol::system("Padding"),
        )?,
    };
    let padding = match effective_padding_expr.as_ref() {
        None => ArrayFilterPadding::Uniform(ArrayFilterPaddingLeaf::Fixed),
        Some(padding_expr) => {
            let Some(parsed) = parse_array_filter_padding(padding_expr, shape.len()) else {
                return Ok(session.invalid_call_with_arg_form(
                    "ArrayFilter",
                    args,
                    "expected Padding -> value, Padding -> None, Padding -> \"Fixed\", Padding -> \"Periodic\", a cyclic padding list, or matching per-axis padding pairs as the fourth argument",
                ));
            };
            parsed
        }
    };

    apply_array_filter(session, operator, &shape, &flat, &spec, &padding)
}

pub(crate) fn eval_array_reduce(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [operator, array, dims_spec] = args else {
        return Ok(session.invalid_call_with_arg_range("ArrayReduce", args, 3, 3));
    };

    let Some((shape, flat)) = parse_rectangular_list_array(array) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayReduce",
            args,
            "expected a rectangular List array as the second argument",
        ));
    };
    let Some(spec) = parse_array_reduce_spec(dims_spec, shape.len()) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayReduce",
            args,
            "expected a positive integer dimension, a dimension Span, a list of dimensions, or a grouped dimension list as the third argument",
        ));
    };

    apply_array_reduce(session, operator, &shape, &flat, &spec)
}
