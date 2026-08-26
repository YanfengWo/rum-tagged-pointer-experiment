use super::*;

impl Session {
    pub(in crate::runtime) fn resolve_cases_spec_expr_for_matching_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let candidate = strip_hold_pattern(&expr).clone();

        if should_evaluate_single_rule_candidate(&candidate) {
            let resolved_result = self.evaluate(candidate);
            return self.drive_evaluated_value(resolved_result, move |session, resolved| {
                if session
                    .compile_rewrite_rule_for_matching(&resolved)
                    .is_some()
                {
                    Ok(resolved)
                } else {
                    Ok(session.normalize_match_expr(&resolved))
                }
            });
        }

        self.resolve_matcher_pattern_expr_owned(candidate)
    }

    pub(in crate::runtime) fn compile_cases_spec_from_resolved_expr(
        &self,
        resolved: Expr,
    ) -> CasesSpec {
        match self.compile_rewrite_rule_for_matching(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        }
    }

    pub(in crate::runtime) fn resolve_rewrite_rule_exprs_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Option<Vec<Expr>>, EvalError> {
        let resolved_result = self.evaluate(expr);
        self.drive_evaluated_value(resolved_result, |session, resolved| {
            Ok(session.normalize_rewrite_rule_exprs_from_resolved(&resolved))
        })
    }

    pub(in crate::runtime) fn drive_rewrite_rule_exprs_owned<U, F>(
        &mut self,
        expr: Expr,
        cont: F,
    ) -> Result<U, EvalError>
    where
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, Option<Vec<Expr>>) -> Result<U, EvalError> + Send + 'static,
    {
        let resolved_rule_exprs_result = self.resolve_rewrite_rule_exprs_owned(expr);
        self.drive_internal_value(resolved_rule_exprs_result, cont)
    }

    pub(in crate::runtime) fn drive_rewrite_rules_owned<U, F>(
        &mut self,
        expr: Expr,
        cont: F,
    ) -> Result<U, EvalError>
    where
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, Option<Vec<RewriteRule>>) -> Result<U, EvalError> + Send + 'static,
    {
        match self.evaluate(expr) {
            Ok(resolved) => cont(self, self.compile_rewrite_rules_for_matching(&resolved)),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| match outcome {
                    Ok(resolved) => {
                        let rules = session.compile_rewrite_rules_for_matching(&resolved);
                        let resumed = cont(session, rules)?;
                        Ok(resumed.into_continuation_expr())
                    }
                    Err(error) => Err(error),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn normalize_rewrite_rule_exprs_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Option<Vec<Expr>> {
        normalize_rewrite_rules(resolved).map(|rules| {
            rules
                .into_iter()
                .map(|rule| Self::rewrite_rule_expr(&rule))
                .collect()
        })
    }

    pub(in crate::runtime) fn compile_rewrite_rules_from_normalized_exprs(
        &self,
        rule_exprs: Vec<Expr>,
    ) -> Option<Vec<RewriteRule>> {
        self.compile_rewrite_rules_for_matching(&Expr::call(system_expr("List"), rule_exprs))
    }

    pub(in crate::runtime) fn rewrite_rule_expr(rule: &RewriteRule) -> Expr {
        Expr::call(
            system_expr(if rule.delayed { "RuleDelayed" } else { "Rule" }),
            vec![
                rule.lhs.clone(),
                rebuild_definition_rhs_guards(rule.rhs.clone(), &rule.guards),
            ],
        )
    }

    pub(in crate::runtime) fn normalized_rules_expr(rules: &[RewriteRule]) -> Expr {
        Expr::call(
            system_expr("List"),
            rules.iter().map(Self::rewrite_rule_expr).collect(),
        )
    }

    pub(in crate::runtime) fn compile_rewrite_rules_for_matching(
        &self,
        expr: &Expr,
    ) -> Option<Vec<RewriteRule>> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => args
                .iter()
                .map(|item| self.compile_rewrite_rule_for_matching(item))
                .collect(),
            _ => self
                .compile_rewrite_rule_for_matching(expr)
                .map(|rule| vec![rule]),
        }
    }

    pub(in crate::runtime) fn compile_rewrite_rule_for_matching(
        &self,
        expr: &Expr,
    ) -> Option<RewriteRule> {
        let mut rule = normalize_single_rewrite_rule(expr)?;
        rule.matcher = self.compile_down_value_matcher(&rule.lhs, &rule.guards);
        Some(rule)
    }

    pub(in crate::runtime) fn compile_matcher_for_expr(&self, expr: &Expr) -> MatcherNode {
        compile_matcher(&self.normalize_match_expr(expr))
    }
}

fn should_evaluate_single_rule_candidate(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(_)) => true,
        Expr::Call { head, .. } => {
            is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed")
        }
        _ => false,
    }
}
