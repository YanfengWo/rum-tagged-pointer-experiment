use super::*;

const DEFAULT_SHORT_LIMIT: usize = 72;

impl Session {
    pub(in crate::runtime) fn eval_short(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, limit) = match args {
            [expr] => (expr, DEFAULT_SHORT_LIMIT),
            [expr, limit_expr] => {
                let Some(limit) = expr_as_non_negative_usize(limit_expr).filter(|limit| *limit > 0)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Short",
                        args,
                        "expected the optional second argument to be a positive integer",
                    ));
                };
                (expr, limit)
            }
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        Ok(Expr::string(short_canonical_fragment(expr, limit)))
    }

    pub(in crate::runtime) fn eval_echo(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [value] => {
                self.push_print(render_echo_fragment(value));
                Ok(value.clone())
            }
            [value, label] => {
                self.push_print(format!(
                    "{}: {}",
                    render_echo_fragment(label),
                    render_echo_fragment(value)
                ));
                Ok(value.clone())
            }
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(in crate::runtime) fn eval_echo_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let body = match args {
            [] => Expr::call(system_expr("Echo"), vec![debug_slot_expr()]),
            [function] => {
                let slot = debug_slot_expr();
                Expr::call(
                    system_expr("CompoundExpression"),
                    vec![
                        Expr::call(
                            system_expr("Echo"),
                            vec![Expr::call(function.clone(), vec![slot.clone()])],
                        ),
                        slot,
                    ],
                )
            }
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        Ok(Expr::call(system_expr("Function"), vec![body]))
    }

    pub(in crate::runtime) fn eval_echo_label(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [label] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("Echo"),
                vec![debug_slot_expr(), label.clone()],
            )],
        ))
    }

    pub(in crate::runtime) fn eval_stack(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Stack", args, 0));
        }

        let items = self
            .evaluation_stack
            .iter()
            .take(self.evaluation_stack.len().saturating_sub(1))
            .map(|entry| Expr::call(system_expr("Hold"), vec![entry.expr.clone()]))
            .collect();

        Ok(Expr::call(system_expr("List"), items))
    }

    pub(in crate::runtime) fn eval_rum_profile_reset(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("RumProfileReset", args, 0));
        }

        reset_lexical_profile_counters();
        reset_evaluation_cache_profile_counters();
        Ok(system_expr("Null"))
    }

    pub(in crate::runtime) fn eval_rum_profile_snapshot(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let label = match args {
            [] => "snapshot",
            [label] => expr_as_string(label).unwrap_or("snapshot"),
            _ => {
                return Ok(self.invalid_call_with_arg_range("RumProfileSnapshot", args, 0, 1));
            }
        };

        print_lexical_profile_snapshot(label);
        print_evaluation_cache_profile_snapshot(label);
        Ok(system_expr("Null"))
    }
}

fn debug_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}
fn short_canonical_fragment(expr: &Expr, limit: usize) -> String {
    let rendered = render_echo_fragment(expr);
    if rendered.len() <= limit || limit <= 8 {
        return rendered;
    }

    let prefix_len = limit / 2;
    let suffix_len = limit.saturating_sub(prefix_len + 3);
    let prefix = rendered.chars().take(prefix_len).collect::<String>();
    let suffix = rendered
        .chars()
        .rev()
        .take(suffix_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{prefix}...{suffix}")
}
