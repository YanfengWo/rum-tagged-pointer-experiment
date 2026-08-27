use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct SuspendedReplaceAtPaths {
    expr: Expr,
    paths: FrozenReplaceSequence<Vec<isize>>,
    rule_plan: RewriteRulePlan,
    index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_replace_at(
        &mut self,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        match args.as_ref() {
            [rules_expr, position_spec] => Ok(replace_at_operator_form(
                rules_expr.clone(),
                position_spec.clone(),
            )),
            [target_expr, rules_expr, position_spec] => {
                let target_result = self.evaluate(target_expr.clone());
                let target = self.suspend_with_frame_lazy(target_result, || {
                    let args = args.clone();
                    move |session, target| {
                        session.resume_replace_at_after_target(
                            target,
                            args[1].clone(),
                            args[2].clone(),
                            args.as_ref().to_vec(),
                        )
                    }
                })?;

                self.resume_replace_at_after_target(
                    target,
                    rules_expr.clone(),
                    position_spec.clone(),
                    args.as_ref().to_vec(),
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceAt", args, 2, 3)),
        }
    }
    fn resume_replace_at_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        position_spec: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let positions_result = self.evaluate(position_spec);
        match positions_result {
            Ok(resolved_positions) => {
                self.finish_replace_at_positions(target, rules_expr, call_args, resolved_positions)
            }
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                self.push_suspension_frame(move |session, value| {
                    session.finish_replace_at_positions(
                        target,
                        rules_expr,
                        call_args.into_vec(),
                        value?,
                    )
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_replace_at_positions(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        call_args: Vec<Expr>,
        resolved_positions: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(paths) = exact_position_paths_from_expr(&target, &resolved_positions) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAt",
                &call_args,
                "expected an exact part specification or list of exact part specifications as the third argument",
            ));
        };
        self.resume_replace_at_after_positions(target, rules_expr, paths, call_args)
    }

    fn resume_replace_at_after_positions(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        paths: Vec<Vec<isize>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr,
            RewriteRuleExprContinuation::ReplaceAt {
                target,
                paths,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_at_after_rule_exprs(
        &mut self,
        target: Expr,
        paths: Vec<Vec<isize>>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAt",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");
        let rule_plan = Self::shared_rewrite_rule_plan(rules);

        self.replace_at_paths_owned(target, paths, rule_plan, 0)
    }

    fn replace_at_paths_owned(
        &mut self,
        expr: Expr,
        paths: Vec<Vec<isize>>,
        rule_plan: RewriteRulePlan,
        index: usize,
    ) -> Result<Expr, EvalError> {
        let Some(path) = paths.get(index).cloned() else {
            return Ok(expr);
        };

        let Some(part) = exact_expr_at_path(&expr, &path) else {
            return self.replace_at_paths_owned(expr, paths, rule_plan, index + 1);
        };

        let replaced_result = self.apply_rewrite_rule_plan_once(
            part,
            rule_plan.clone(),
            RuleApplicationSource::ReplaceAt,
        );
        match replaced_result {
            Ok(replaced) => {
                let expr = match replaced {
                    Some(replaced) => replace_exact_path(expr, &path, replaced),
                    None => expr,
                };
                self.replace_at_paths_owned(expr, paths, rule_plan, index + 1)
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedReplaceAtPaths {
                    expr,
                    paths: FrozenReplaceSequence::freeze(paths),
                    rule_plan,
                    index,
                };
                self.push_suspension_frame(move |session, value| {
                    let replaced = Option::<Expr>::from_continuation_expr(value?);
                    session.resume_suspended_replace_at_paths(state, path, replaced)
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_replace_at_paths(
        &mut self,
        mut state: SuspendedReplaceAtPaths,
        path: Vec<isize>,
        replaced: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        if let Some(replaced) = replaced {
            state.expr = replace_exact_path(state.expr, &path, replaced);
        }
        state.index += 1;
        loop {
            let Some(path) = state.paths.as_slice().get(state.index).cloned() else {
                return Ok(state.expr);
            };
            let Some(part) = exact_expr_at_path(&state.expr, &path) else {
                state.index += 1;
                continue;
            };
            let replaced_result = self.apply_rewrite_rule_plan_once(
                part,
                state.rule_plan.clone(),
                RuleApplicationSource::ReplaceAt,
            );
            match replaced_result {
                Ok(Some(replaced)) => {
                    state.expr = replace_exact_path(state.expr, &path, replaced);
                    state.index += 1;
                }
                Ok(None) => state.index += 1,
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let replaced = Option::<Expr>::from_continuation_expr(value?);
                        session.resume_suspended_replace_at_paths(state, path, replaced)
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

fn replace_at_operator_form(rules_expr: Expr, position_spec: Expr) -> Expr {
    let parameter = Expr::symbol("rum$replaceAtArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(
                system_expr("ReplaceAt"),
                vec![parameter, rules_expr, position_spec],
            ),
        ],
    )
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceAtPaths>();
};
