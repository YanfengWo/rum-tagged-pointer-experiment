use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod matching;

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedReplaceListScan {
    pub(super) target: Expr,
    pub(super) normalized_target: Expr,
    pub(super) rules: RewriteRulePlan,
    pub(super) max_results: Option<usize>,
    pub(super) collected: PersistentRewriteExprs,
    pub(super) index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_replace_list(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        match args_slice {
            [rules_expr] => Ok(replace_list_operator_form(rules_expr.clone())),
            [target_expr, rules_expr] | [target_expr, rules_expr, _] => {
                let target_result = self.evaluate(target_expr.clone());
                let target = self.suspend_with_frame_lazy(target_result, || {
                    let args = args.clone();
                    move |session, target| {
                        session.resume_replace_list_after_target(
                            target,
                            args[1].clone(),
                            args.get(2).cloned(),
                            args.as_ref().to_vec(),
                        )
                    }
                })?;

                self.resume_replace_list_after_target(
                    target,
                    rules_expr.clone(),
                    args_slice.get(2).cloned(),
                    args_slice.to_vec(),
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceList", args_slice, 1, 3)),
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
        match max_result {
            Ok(resolved_max) => {
                self.finish_replace_list_limit(target, rules_expr, resolved_max, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                self.push_suspension_frame(move |session, value| {
                    session.finish_replace_list_limit(
                        target,
                        rules_expr,
                        value?,
                        call_args.into_vec(),
                    )
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_replace_list_limit(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        resolved_max: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(max_results) = parse_replace_list_max_results(&resolved_max) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceList",
                &call_args,
                "expected a non-negative integer or Infinity as the optional third argument",
            ));
        };
        self.resume_replace_list_after_limit(target, rules_expr, max_results, call_args)
    }

    fn resume_replace_list_after_limit(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr,
            RewriteRuleExprContinuation::ReplaceList {
                target,
                max_results,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_list_after_rule_exprs(
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
        let rules = Self::shared_rewrite_rule_plan(rules);
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
        rules: RewriteRulePlan,
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
        rules: RewriteRulePlan,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        mut index: usize,
        mut pending: Option<Vec<SymbolExprMap>>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            if max_results.is_some_and(|limit| collected.len() >= limit) {
                return Ok(collected);
            }

            let Some(rule) = rules.get(index) else {
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
                        let state = SuspendedReplaceListScan {
                            target,
                            normalized_target,
                            rules,
                            max_results,
                            collected: PersistentRewriteExprs::freeze(collected),
                            index,
                        };
                        self.push_suspension_frame(move |session, outcome| {
                            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(outcome?);
                            session
                                .collect_replace_list_results_suspended(state, Some(bindings))
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
                bindings,
            );
        }
    }

    pub(super) fn collect_replace_list_results_suspended(
        &mut self,
        mut state: SuspendedReplaceListScan,
        mut pending: Option<Vec<SymbolExprMap>>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            if state
                .max_results
                .is_some_and(|limit| state.collected.len() >= limit)
            {
                return Ok(state.collected.into_vec());
            }

            let Some(rule) = state.rules.get(state.index) else {
                return Ok(state.collected.into_vec());
            };
            let bindings = if let Some(bindings) = pending.take() {
                bindings
            } else {
                let bindings_result = self.collect_replace_list_rule_bindings_owned(
                    rule.matcher.clone(),
                    state.normalized_target.clone(),
                );
                match bindings_result {
                    Ok(bindings) => bindings,
                    Err(EvalError::InputRequired { request }) => {
                        self.push_suspension_frame(move |session, outcome| {
                            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(outcome?);
                            session
                                .collect_replace_list_results_suspended(state, Some(bindings))
                                .map(|results| results.into_continuation_expr())
                        });
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => return Err(error),
                }
            };

            if bindings.is_empty() {
                state.index += 1;
                continue;
            }
            return self.collect_replace_list_rule_results_suspended(state, bindings, 0);
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

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceListScan>();
};
