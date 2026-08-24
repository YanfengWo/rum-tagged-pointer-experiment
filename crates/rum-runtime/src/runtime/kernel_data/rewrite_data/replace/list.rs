use super::*;

mod matching;

impl Session {
    pub(in crate::runtime) fn eval_replace_list(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [rules_expr] => Ok(replace_list_operator_form(rules_expr.clone())),
            [target_expr, rules_expr] | [target_expr, rules_expr, _] => {
                let call_args = args.to_vec();
                let target_result = self.evaluate(target_expr.clone());
                let target = self.suspend_with_frame(target_result, {
                    let rules_expr = rules_expr.clone();
                    let max_expr = args.get(2).cloned();
                    let call_args = call_args.clone();
                    move |session, target| {
                        session.resume_replace_list_after_target(
                            target, rules_expr, max_expr, call_args,
                        )
                    }
                })?;

                self.resume_replace_list_after_target(
                    target,
                    rules_expr.clone(),
                    args.get(2).cloned(),
                    call_args,
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceList", args, 1, 3)),
        }
    }

    fn resume_replace_list_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_expr: Option<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(max_expr) = max_expr else {
            return self.resume_replace_list_after_limit(target, rules_expr, None, call_args);
        };

        let max_result = self.evaluate(max_expr);
        self.drive_evaluated_value(max_result, move |session, resolved_max| {
            let Some(max_results) = parse_replace_list_max_results(&resolved_max) else {
                return Ok(session.invalid_call_with_arg_form(
                    "ReplaceList",
                    &call_args,
                    "expected a non-negative integer or Infinity as the optional third argument",
                ));
            };

            session.resume_replace_list_after_limit(target, rules_expr, max_results, call_args)
        })
    }

    fn resume_replace_list_after_limit(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
            session.resume_replace_list_after_rule_exprs(target, rule_exprs, max_results, call_args)
        })
    }

    fn resume_replace_list_after_rule_exprs(
        &mut self,
        target: Expr,
        normalized_rule_exprs: Option<Vec<Expr>>,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceList",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        let normalized_target = self.normalize_match_expr(&target);
        let results_result = self.collect_replace_list_results_owned(
            target,
            normalized_target,
            rules,
            max_results,
            Vec::new(),
            0,
        );
        self.drive_internal_value(results_result, |_, results| {
            Ok(Expr::call(system_expr("List"), results))
        })
    }

    fn collect_replace_list_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: Vec<RewriteRule>,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        self.collect_replace_list_results_with_pending(
            target,
            normalized_target,
            rules,
            max_results,
            collected,
            index,
            None,
        )
    }

    /// Walks the rules one per loop iteration instead of recursing per rule,
    /// so a wide rule list no longer grows the native stack. `pending` carries
    /// already-collected bindings for the rule at `index` across a suspension
    /// resume, so a resumed traversal skips re-collecting them and keeps the
    /// accumulated results in rule order.
    fn collect_replace_list_results_with_pending(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: Vec<RewriteRule>,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        mut index: usize,
        mut pending: Option<Vec<SymbolExprMap>>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            if max_results.is_some_and(|limit| collected.len() >= limit) {
                return Ok(collected);
            }

            let Some(rule) = rules.get(index).cloned() else {
                return Ok(collected);
            };

            let bindings = if let Some(bindings) = pending.take() {
                bindings
            } else {
                let bindings_result = self.collect_replace_list_rule_bindings_owned(
                    rule.matcher.clone(),
                    normalized_target.clone(),
                );
                match bindings_result {
                    Ok(bindings) => bindings,
                    Err(EvalError::InputRequired { request }) => {
                        self.push_suspension_frame(move |session, outcome| {
                            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(outcome?);
                            session
                                .collect_replace_list_results_with_pending(
                                    target,
                                    normalized_target,
                                    rules,
                                    max_results,
                                    collected,
                                    index,
                                    Some(bindings),
                                )
                                .map(|results| results.into_continuation_expr())
                        });
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => return Err(error),
                }
            };

            if bindings.is_empty() {
                index += 1;
                continue;
            }

            return self.collect_replace_list_rule_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                index,
                rule,
                bindings,
                0,
            );
        }
    }
}

fn parse_replace_list_max_results(expr: &Expr) -> Option<Option<usize>> {
    if is_system_symbol_name(expr, "Infinity") {
        Some(None)
    } else {
        expr_as_non_negative_usize(expr).map(Some)
    }
}

fn replace_list_operator_form(rules_expr: Expr) -> Expr {
    let parameter = Expr::symbol("rum$replaceListArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr("ReplaceList"), vec![parameter, rules_expr]),
        ],
    )
}
