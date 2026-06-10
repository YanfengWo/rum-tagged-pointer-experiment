use super::*;

impl Session {
    pub(in crate::runtime) fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MatchQ", args, 2));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let pattern_expr = args[1].clone();
            move |session, value| session.resume_match_q_after_value(pattern_expr, value)
        })?;

        self.resume_match_q_after_value(args[1].clone(), value)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_pattern_expr(
        &mut self,
        expr: &Expr,
    ) -> Result<Expr, EvalError> {
        self.resolve_matcher_pattern_expr_owned(expr.clone())
    }

    pub(in crate::runtime) fn resolve_matcher_pattern_expr_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let normalized = strip_hold_pattern(&expr).clone();
        let resolved = if contains_pattern_syntax(&normalized) {
            normalized
        } else {
            self.evaluate(normalized)?
        };

        Ok(self.normalize_match_expr(&resolved))
    }

    #[allow(dead_code)]
    pub(in crate::runtime) fn normalize_cases_spec(
        &mut self,
        expr: &Expr,
    ) -> Result<CasesSpec, EvalError> {
        let resolved = self.resolve_cases_spec_expr_for_matching_owned(expr.clone())?;
        Ok(self.compile_cases_spec_from_resolved_expr(resolved))
    }

    fn resume_match_q_after_value(
        &mut self,
        pattern_expr: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame(pattern_result, {
            let value = value.clone();
            move |session, resolved| session.finish_match_q_with_pattern(resolved, value)
        })?;

        self.finish_match_q_with_pattern(pattern, value)
    }

    fn finish_match_q_with_pattern(
        &mut self,
        pattern: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let match_result = self.match_expr_owned(pattern, value, BTreeMap::new());
        self.drive_internal_value(match_result, move |_, outcome| {
            Ok(system_expr(if outcome.matched { "True" } else { "False" }))
        })
    }
}

fn contains_pattern_syntax(expr: &Expr) -> bool {
    let expr = strip_hold_pattern(expr);

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            if matches_pattern_head(head) {
                true
            } else {
                contains_pattern_syntax(head) || args.iter().any(contains_pattern_syntax)
            }
        }
    }
}

fn matches_pattern_head(head: &Expr) -> bool {
    is_system_symbol_name(head, "Alternatives")
        || is_system_symbol_name(head, "Blank")
        || is_system_symbol_name(head, "BlankSequence")
        || is_system_symbol_name(head, "BlankNullSequence")
        || is_system_symbol_name(head, "Condition")
        || is_system_symbol_name(head, "Except")
        || is_system_symbol_name(head, "HoldPattern")
        || is_system_symbol_name(head, "KeyValuePattern")
        || is_system_symbol_name(head, "Longest")
        || is_system_symbol_name(head, "Optional")
        || is_system_symbol_name(head, "OrderlessPatternSequence")
        || is_system_symbol_name(head, "Pattern")
        || is_system_symbol_name(head, "PatternSequence")
        || is_system_symbol_name(head, "PatternTest")
        || is_system_symbol_name(head, "Repeated")
        || is_system_symbol_name(head, "RepeatedNull")
        || is_system_symbol_name(head, "Shortest")
        || is_system_symbol_name(head, "Verbatim")
}
