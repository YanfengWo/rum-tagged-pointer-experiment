use super::*;

mod comparison;
mod composition;
mod curried;
mod head_ops;
mod key;
mod nearest;
mod reverse;

pub(in crate::runtime) use composition::{apply_composition_to_args, parse_composition};

pub(in crate::runtime) fn is_callable_operator_wrapper(expr: &Expr) -> bool {
    curried::is_curried_operator_wrapper(expr)
        || reverse::is_reverse_applied_wrapper(expr)
        || comparison::is_comparison_operator_wrapper(expr)
        || nearest::is_nearest_to_wrapper(expr)
        || key::parse_key_wrapper(expr).is_some()
}

pub(in crate::runtime) fn apply_callable_operator_wrapper_to_args(
    head: &Expr,
    args: &[Expr],
) -> Option<Expr> {
    if let Some(expr) = curried::apply_curried_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = reverse::apply_reverse_applied_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = comparison::apply_comparison_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = nearest::apply_nearest_to_wrapper_to_args(head, args) {
        return Some(expr);
    }

    key::apply_key_wrapper_to_args(head, args)
}

pub(in crate::runtime) fn invalid_callable_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    comparison::invalid_comparison_operator_wrapper_argx_symbol(head, args)
}
