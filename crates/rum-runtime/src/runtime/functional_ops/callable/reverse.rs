use super::*;

#[derive(Clone, Debug)]
struct ReverseAppliedSpec<'a> {
    function: &'a Expr,
    reversed_count: Option<usize>,
}

impl Session {
    pub(in crate::runtime) fn eval_reverse_applied(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [count] if expr_as_non_negative_usize(count).is_some() => {
                Ok(unary_operator_form_slot_first("ReverseApplied", count))
            }
            [function] => Ok(Expr::call(head.clone(), vec![function.clone()])),
            [function, count] if expr_as_non_negative_usize(count).is_some() => Ok(Expr::call(
                head.clone(),
                vec![function.clone(), count.clone()],
            )),
            [_, _] => Ok(self.invalid_call_with_arg_form(
                "ReverseApplied",
                args,
                "expected a non-negative integer as the second argument",
            )),
            _ => Ok(self.invalid_call_with_arg_range("ReverseApplied", args, 1, 2)),
        }
    }
}

pub(super) fn is_reverse_applied_wrapper(expr: &Expr) -> bool {
    parse_reverse_applied_wrapper(expr).is_some()
}

pub(super) fn apply_reverse_applied_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    parse_reverse_applied_wrapper(head).map(|spec| apply_reverse_applied_to_args(&spec, args))
}

fn parse_reverse_applied_wrapper(expr: &Expr) -> Option<ReverseAppliedSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "ReverseApplied") => {
            match args.as_slice() {
                [function] => Some(ReverseAppliedSpec {
                    function,
                    reversed_count: None,
                }),
                [function, count] => Some(ReverseAppliedSpec {
                    function,
                    reversed_count: Some(expr_as_non_negative_usize(count)?),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

fn apply_reverse_applied_to_args(spec: &ReverseAppliedSpec<'_>, args: &[Expr]) -> Expr {
    let reverse_count = spec.reversed_count.unwrap_or(args.len()).min(args.len());
    let mut reordered = args[..reverse_count].to_vec();
    reordered.reverse();
    reordered.extend_from_slice(&args[reverse_count..]);
    Expr::call(spec.function.clone(), reordered)
}
