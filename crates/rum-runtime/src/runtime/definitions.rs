use super::*;

impl Session {
    pub(super) fn eval_normalize_rules(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => self.resume_normalize_rules(expr.clone()),
            _ => Ok(self.invalid_call_with_exact_arg_count("NormalizeRules", args, 1)),
        }
    }

    pub(super) fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Set", args, 2));
        }

        let lhs = args[0].clone();
        let rhs_result = self.evaluate(args[1].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let lhs = lhs.clone();
            move |session, rhs| session.resume_set_with_rhs(lhs, rhs)
        })?;

        self.resume_set_with_rhs(lhs, rhs)
    }

    pub(super) fn eval_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();
        self.finish_definition_assignment("SetDelayed", lhs, rhs, true)
    }

    pub(super) fn eval_rule(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Rule", args, 2));
        }

        let lhs = args[0].clone();
        let rhs_result = self.evaluate(args[1].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let lhs = lhs.clone();
            move |session, rhs| session.resume_rule_with_rhs(lhs, rhs)
        })?;

        self.resume_rule_with_rhs(lhs, rhs)
    }

    pub(super) fn eval_rule_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("RuleDelayed", args, 2));
        }

        Ok(Expr::call(system_expr("RuleDelayed"), args.to_vec()))
    }

    fn resume_normalize_rules(&mut self, original: Expr) -> Result<Expr, EvalError> {
        let normalized_rule_exprs_result = self.resolve_rewrite_rule_exprs_owned(original.clone());
        let normalized_rule_exprs = self
            .drive_internal_value(normalized_rule_exprs_result, |_, rule_exprs| Ok(rule_exprs))?;

        match normalized_rule_exprs {
            Some(rule_exprs) => {
                let rules = self
                    .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
                    .expect("normalized rewrite rule expressions should recompile");
                Ok(Self::normalized_rules_expr(&rules))
            }
            None => Ok(self.invalid_call_with_arg_form(
                "NormalizeRules",
                &[original],
                "expected a rule, a rule list, or a symbol that evaluates to rules",
            )),
        }
    }

    fn resume_set_with_rhs(&mut self, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
        self.finish_definition_assignment("Set", lhs, rhs, false)
    }

    fn resume_rule_with_rhs(&mut self, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
        Ok(Expr::call(system_expr("Rule"), vec![lhs, rhs]))
    }

    fn finish_definition_assignment(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        match self.normalize_definition_assignment_target(&lhs) {
            Some(DefinitionAssignmentTarget::Message(key)) => {
                self.define_message(key, rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            Some(DefinitionAssignmentTarget::Definition(target)) => {
                self.store_definition_target(target, lhs.clone(), rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            None => Ok(Expr::call(system_expr(symbol), vec![lhs, rhs])),
        }
    }
}
