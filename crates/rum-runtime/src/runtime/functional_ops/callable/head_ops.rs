use super::*;

impl Session {
    pub(in crate::runtime) fn eval_through(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Through", args, 1, 2));
        }

        let target = args.get(1);
        self.evaluate(through_expression(&args[0], target))
    }

    pub(in crate::runtime) fn eval_operate(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(unary_operator_form("Operate", operator)),
            [operator, level_expr] if expr_as_non_negative_usize(level_expr).is_some() => Ok(
                unary_operator_form_with_tail("Operate", operator.clone(), level_expr.clone()),
            ),
            [operator, expr] => self.evaluate(operate_on_head_level(expr, operator, 1)),
            [operator, expr, level_expr] => {
                let Some(level) = expr_as_non_negative_usize(level_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Operate",
                        args,
                        "expected a non-negative integer as the third argument",
                    ));
                };

                self.evaluate(operate_on_head_level(expr, operator, level))
            }
            _ => Ok(self.invalid_call_with_arg_range("Operate", args, 1, 3)),
        }
    }
}

fn through_expression(expr: &Expr, target: Option<&Expr>) -> Expr {
    let Expr::Call { head, args } = expr else {
        return expr.clone();
    };

    let Some((transformed, matched)) = through_head_expression(head, args, target) else {
        return expr.clone();
    };

    if matched { transformed } else { expr.clone() }
}

fn through_head_expression(
    head_expr: &Expr,
    call_args: &[Expr],
    target: Option<&Expr>,
) -> Option<(Expr, bool)> {
    let Expr::Call { head, args } = head_expr else {
        return None;
    };

    if target.is_none_or(|target_head| head.as_ref() == target_head) {
        return Some((
            Expr::call(
                head.as_ref().clone(),
                args.iter()
                    .map(|arg| Expr::call(arg.clone(), call_args.to_vec()))
                    .collect(),
            ),
            true,
        ));
    }

    let mut matched = false;
    let transformed_args = args
        .iter()
        .map(|arg| {
            if let Some((transformed, subtree_matched)) =
                through_head_expression(arg, call_args, target)
            {
                matched |= subtree_matched;
                transformed
            } else {
                arg.clone()
            }
        })
        .collect();

    Some((Expr::call(head.as_ref().clone(), transformed_args), matched))
}

fn operate_on_head_level(expr: &Expr, operator: &Expr, level: usize) -> Expr {
    if level == 0 {
        return Expr::call(operator.clone(), vec![expr.clone()]);
    }

    match expr {
        Expr::Call { head, args } => {
            if level == 1 {
                Expr::call_with_shared_args(
                    Expr::call(operator.clone(), vec![head.as_ref().clone()]),
                    args.clone(),
                )
            } else {
                let replaced_head = operate_on_head_level(head, operator, level - 1);
                if replaced_head == **head {
                    expr.clone()
                } else {
                    Expr::call_with_shared_args(replaced_head, args.clone())
                }
            }
        }
        Expr::Atom(_) => expr.clone(),
    }
}
