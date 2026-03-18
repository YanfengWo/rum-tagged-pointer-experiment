use super::*;

impl Session {
    pub(super) fn eval_normalize_rules(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                let resolved = self.evaluate(expr.clone())?;

                match normalize_rewrite_rules(&resolved) {
                    Some(rules) => Ok(Self::normalized_rules_expr(&rules)),
                    None => Ok(self.invalid_call_with_arg_form(
                        "NormalizeRules",
                        args,
                        "expected a rule, a rule list, or a symbol that evaluates to rules",
                    )),
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("NormalizeRules", args, 1)),
        }
    }

    pub(super) fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Set", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = self.evaluate(args[1].clone())?;

        if let Some(key) = self.message_name_key_from_expr(&lhs) {
            self.define_message(key, rhs.clone(), false);
            return Ok(rhs);
        }

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            self.store_definition_target(
                TaggedDefinitionTarget::OwnValue(symbol),
                lhs.clone(),
                rhs.clone(),
                false,
            );
            Ok(rhs)
        } else if let Some(target) = classify_definition_lhs(&lhs) {
            self.store_definition_target(target, lhs.clone(), rhs.clone(), false);
            Ok(rhs)
        } else {
            Ok(Expr::call(system_expr("Set"), vec![lhs, rhs]))
        }
    }

    pub(super) fn eval_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();

        if let Some(key) = self.message_name_key_from_expr(&lhs) {
            self.define_message(key, rhs, true);
            return Ok(system_expr("Null"));
        }

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            self.store_definition_target(
                TaggedDefinitionTarget::OwnValue(symbol),
                lhs.clone(),
                rhs.clone(),
                true,
            );
            Ok(system_expr("Null"))
        } else if let Some(target) = classify_definition_lhs(&lhs) {
            self.store_definition_target(target, lhs.clone(), rhs.clone(), true);
            Ok(system_expr("Null"))
        } else {
            Ok(Expr::call(system_expr("SetDelayed"), vec![lhs, rhs]))
        }
    }

    pub(super) fn eval_rule(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Rule", args, 2));
        }

        Ok(Expr::call(
            system_expr("Rule"),
            vec![args[0].clone(), self.evaluate(args[1].clone())?],
        ))
    }

    pub(super) fn eval_rule_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("RuleDelayed", args, 2));
        }

        Ok(Expr::call(system_expr("RuleDelayed"), args.to_vec()))
    }
}
