use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_reverse_applied(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [count] if expr_as_non_negative_usize(count).is_some() => {
            Ok(unary_operator_form_slot_first("ReverseApplied", count))
        }
        [function] => Ok(Expr::call(head.clone(), vec![function.clone()])),
        [function, count] if expr_as_non_negative_usize(count).is_some() => Ok(Expr::call(
            head.clone(),
            vec![function.clone(), count.clone()],
        )),
        [_, _] => Ok(session.invalid_call_with_arg_form(
            "ReverseApplied",
            args,
            "expected a non-negative integer as the second argument",
        )),
        _ => Ok(session.invalid_call_with_arg_range("ReverseApplied", args, 1, 2)),
    }
}
