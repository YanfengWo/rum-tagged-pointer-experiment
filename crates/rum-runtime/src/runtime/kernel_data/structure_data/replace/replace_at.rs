use super::*;

impl Session {
    pub(in crate::runtime) fn eval_replace_at(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [rules_expr, position_spec] => Ok(replace_at_operator_form(
                rules_expr.clone(),
                position_spec.clone(),
            )),
            [target_expr, rules_expr, position_spec] => {
                let call_args = args.to_vec();
                let target_result = self.evaluate(target_expr.clone());
                let target = self.suspend_with_frame(target_result, {
                    let rules_expr = rules_expr.clone();
                    let position_spec = position_spec.clone();
                    let call_args = call_args.clone();
                    move |session, target| {
                        session.resume_replace_at_after_target(
                            target,
                            rules_expr,
                            position_spec,
                            call_args,
                        )
                    }
                })?;

                self.resume_replace_at_after_target(
                    target,
                    rules_expr.clone(),
                    position_spec.clone(),
                    call_args,
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
        self.drive_evaluated_value(positions_result, move |session, resolved_positions| {
            let Some(paths) = exact_position_paths_from_expr(&target, &resolved_positions) else {
                return Ok(session.invalid_call_with_arg_form(
                    "ReplaceAt",
                    &call_args,
                    "expected an exact part specification or list of exact part specifications as the third argument",
                ));
            };

            session.resume_replace_at_after_positions(target, rules_expr, paths, call_args)
        })
    }

    fn resume_replace_at_after_positions(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        paths: Vec<Vec<isize>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
            session.resume_replace_at_after_rule_exprs(target, paths, rule_exprs, call_args)
        })
    }

    fn resume_replace_at_after_rule_exprs(
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
        let apply_rules = Self::shared_rewrite_rule_applier(rules);

        self.replace_at_paths_owned(target, paths, apply_rules, 0)
    }

    fn replace_at_paths_owned<F>(
        &mut self,
        expr: Expr,
        paths: Vec<Vec<isize>>,
        apply_rules: F,
        index: usize,
    ) -> Result<Expr, EvalError>
    where
        F: Clone
            + Fn(&mut Session, Expr, RuleApplicationSource) -> Result<Option<Expr>, EvalError>
            + Send
            + 'static,
    {
        let Some(path) = paths.get(index).cloned() else {
            return Ok(expr);
        };

        let Some(part) = exact_expr_at_path(&expr, &path) else {
            return self.replace_at_paths_owned(expr, paths, apply_rules, index + 1);
        };

        let replaced_result = apply_rules(self, part, RuleApplicationSource::ReplaceAt);
        self.drive_internal_value(replaced_result, move |session, replaced| {
            let Some(replaced) = replaced else {
                return session.replace_at_paths_owned(expr, paths, apply_rules, index + 1);
            };

            let updated = replace_exact_path(expr, &path, replaced);
            session.replace_at_paths_owned(updated, paths, apply_rules, index + 1)
        })
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
