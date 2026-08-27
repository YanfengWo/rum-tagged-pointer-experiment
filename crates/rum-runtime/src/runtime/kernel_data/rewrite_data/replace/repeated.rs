use super::*;

impl Session {
    pub(in crate::runtime) fn eval_replace_repeated(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("ReplaceRepeated", args_slice));
        }
        let max_iterations_expr = match args_slice {
            [_, _] => None,
            [_, _, option] if replace_repeated_max_iterations_option_value(option).is_some() => {
                Some(option.clone())
            }
            _ => {
                return Ok(self.invalid_call_with_exact_arg_count(
                    "ReplaceRepeated",
                    args_slice,
                    2,
                ));
            }
        };

        let current_result = self.evaluate(args_slice[0].clone());
        let current = self.suspend_with_frame_lazy(current_result, || {
            let args = args.clone();
            let max_iterations_expr = max_iterations_expr.clone();
            move |session, current| {
                session.resume_replace_repeated_after_current(
                    current,
                    args[1].clone(),
                    max_iterations_expr,
                    args.as_ref().to_vec(),
                )
            }
        })?;

        self.resume_replace_repeated_after_current(
            current,
            args_slice[1].clone(),
            max_iterations_expr,
            args_slice.to_vec(),
        )
    }

    fn resume_replace_repeated_after_current(
        &mut self,
        current: Expr,
        rules_expr: Expr,
        max_iterations_expr: Option<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr.clone(),
            RewriteRuleExprContinuation::ReplaceRepeated {
                current,
                rules_input: rules_expr,
                max_iterations_expr,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_repeated_after_rule_exprs(
        &mut self,
        current: Expr,
        rules_input: Expr,
        max_iterations_expr: Option<Expr>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceRepeated",
                &[current, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");
        let rules = Self::shared_rewrite_rule_plan(rules);

        let limit_result = self.resolve_replace_repeated_limit_owned(max_iterations_expr);
        self.drive_internal_value(limit_result, move |session, effective_limit| {
            let Some(effective_limit_expr) = effective_limit else {
                return Ok(session.invalid_call_with_arg_form(
                    "ReplaceRepeated",
                    &call_args,
                    "expected MaxIterations -> Automatic, Infinity, or a positive integer",
                ));
            };
            let effective_limit = expr_as_non_negative_usize(&effective_limit_expr)
                .expect("replace-repeated limit should be non-negative");

            session.resume_replace_repeated_loop(current, rules, 0, effective_limit)
        })
    }

    fn resolve_replace_repeated_limit_owned(
        &mut self,
        max_iterations_expr: Option<Expr>,
    ) -> Result<Option<Expr>, EvalError> {
        let ceiling = self.resource_limits.replace_repeated_limit;
        let Some(max_iterations_expr) = max_iterations_expr else {
            return Ok(Some(Expr::integer(BigInt::from(ceiling))));
        };

        let resolved_result = self.evaluate(max_iterations_expr);
        self.drive_evaluated_value(resolved_result, move |_, resolved| {
            let Some(value) = replace_repeated_max_iterations_option_value(&resolved) else {
                return Ok(None);
            };
            Ok(parse_replace_repeated_max_iterations_value(value, ceiling)
                .map(|limit| Expr::integer(BigInt::from(limit))))
        })
    }

    fn resume_replace_repeated_loop(
        &mut self,
        current: Expr,
        rules: RewriteRulePlan,
        iteration: usize,
        effective_limit: usize,
    ) -> Result<Expr, EvalError> {
        if iteration >= effective_limit {
            return Err(EvalError::IterationLimitExceeded {
                limit: effective_limit,
            });
        }

        let replaced = self.apply_replace_all_with_plan(
            current.clone(),
            rules.clone(),
            RuleApplicationSource::ReplaceRepeated,
        )?;
        let next_result = self.evaluate(replaced);
        let next = self.suspend_with_frame_lazy(next_result, || {
            let current = current.clone();
            let rules = rules.clone();
            move |session, next| {
                session.resume_replace_repeated_after_next(
                    current,
                    rules,
                    iteration,
                    effective_limit,
                    next,
                )
            }
        })?;

        self.resume_replace_repeated_after_next(current, rules, iteration, effective_limit, next)
    }

    fn resume_replace_repeated_after_next(
        &mut self,
        current: Expr,
        rules: RewriteRulePlan,
        iteration: usize,
        effective_limit: usize,
        next: Expr,
    ) -> Result<Expr, EvalError> {
        if next == current {
            Ok(next)
        } else {
            self.resume_replace_repeated_loop(next, rules, iteration + 1, effective_limit)
        }
    }
}

fn replace_repeated_max_iterations_option_value(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    max_iterations_option_symbol_matches(&args[0]).then_some(&args[1])
}

fn max_iterations_option_symbol_matches(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol.name() == "MaxIterations",
        Expr::Atom(Atom::String(name)) => {
            name == "MaxIterations"
                || resolve_unqualified_symbol(name)
                    .is_some_and(|symbol| symbol.name() == "MaxIterations")
        }
        _ => false,
    }
}

fn parse_replace_repeated_max_iterations_value(value: &Expr, ceiling: usize) -> Option<usize> {
    if is_system_symbol_name(value, "Automatic") || is_system_symbol_name(value, "Infinity") {
        return Some(ceiling);
    }

    expr_as_non_negative_usize(value)
        .filter(|limit| *limit > 0)
        .map(|limit| limit.min(ceiling))
}
