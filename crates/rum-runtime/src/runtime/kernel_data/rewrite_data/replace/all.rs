use super::*;

impl Session {
    pub(in crate::runtime) fn eval_replace_all(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("ReplaceAll", args));
        }
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceAll", args, 2));
        }

        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let rules_expr = args[1].clone();
            move |session, target| session.resume_replace_all_after_target(target, rules_expr)
        })?;

        self.resume_replace_all_after_target(target, args[1].clone())
    }

    #[allow(dead_code)]
    fn resume_replace_all_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rules_owned(rules_expr.clone(), move |session, rules| {
            session.resume_replace_all_after_rules(target, rules_expr, rules)
        })
    }

    fn resume_replace_all_after_rules(
        &mut self,
        target: Expr,
        rules_input: Expr,
        normalized_rules: Option<Vec<RewriteRule>>,
    ) -> Result<Expr, EvalError> {
        let Some(rules) = normalized_rules else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAll",
                &[target, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced = self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAll)?;
        if expr_is_self_evaluating_atom(&replaced) {
            return Ok(replaced);
        }
        let replaced_result = self.evaluate(replaced);
        self.suspend_with_frame(replaced_result, |_, value| Ok(value))
    }

    pub(in crate::runtime) fn eval_replace_all_unheld(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("ReplaceAllUnheld", args));
        }
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceAllUnheld", args, 2));
        }

        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let rules_expr = args[1].clone();
            move |session, target| {
                session.resume_replace_all_unheld_after_target(target, rules_expr)
            }
        })?;

        self.resume_replace_all_unheld_after_target(target, args[1].clone())
    }

    #[allow(dead_code)]
    fn resume_replace_all_unheld_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rules_owned(rules_expr.clone(), move |session, rules| {
            session.resume_replace_all_unheld_after_rules(target, rules_expr, rules)
        })
    }

    fn resume_replace_all_unheld_after_rules(
        &mut self,
        target: Expr,
        rules_input: Expr,
        normalized_rules: Option<Vec<RewriteRule>>,
    ) -> Result<Expr, EvalError> {
        let Some(rules) = normalized_rules else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAllUnheld",
                &[target, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced =
            self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAllUnheld)?;
        if expr_is_self_evaluating_atom(&replaced) {
            return Ok(replaced);
        }
        let replaced_result = self.evaluate(replaced);
        self.suspend_with_frame(replaced_result, |_, value| Ok(value))
    }
}
