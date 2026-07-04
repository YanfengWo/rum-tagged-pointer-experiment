use super::*;

impl Session {
    pub(in crate::runtime) fn apply_replace_all(
        &mut self,
        expr: Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        self.apply_replace_all_owned(expr, rules.to_vec(), source)
    }

    pub(in crate::runtime) fn try_apply_rewrite_rules_once_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        self.apply_rules_once_from_list(expr, normalized_expr, rules, source, 0)
    }

    fn apply_replace_all_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        let replace_result = self.apply_rules_once_from_list(
            expr.clone(),
            normalized_expr,
            rules.clone(),
            source,
            0,
        );

        self.drive_internal_value(replace_result, move |session, outcome| match outcome {
            Some(replaced) => Ok(replaced),
            None => session.apply_replace_all_after_miss(expr, rules, source),
        })
    }

    fn apply_replace_all_after_miss(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(_) => Ok(expr),
            Expr::Call { head, args } => {
                let remaining_args = args.into_vec();
                let head_result =
                    self.apply_replace_all_owned(head.into_owned(), rules.clone(), source);
                self.drive_evaluated_value::<Expr, Expr, _>(
                    head_result,
                    move |session, new_head| {
                        session.apply_replace_all_args(
                            new_head,
                            Vec::new(),
                            remaining_args,
                            rules,
                            source,
                        )
                    },
                )
            }
        }
    }

    fn apply_replace_all_args(
        &mut self,
        head: Expr,
        mut rewritten_args: Vec<Expr>,
        remaining_args: Vec<Expr>,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let Some((current, rest)) = remaining_args.split_first() else {
            return Ok(Expr::call(head, rewritten_args));
        };

        let rest = rest.to_vec();
        let arg_result = self.apply_replace_all_owned(current.clone(), rules.clone(), source);

        self.drive_evaluated_value::<Expr, Expr, _>(arg_result, move |session, rewritten| {
            rewritten_args.push(rewritten);
            session.apply_replace_all_args(head, rewritten_args, rest, rules, source)
        })
    }

    fn apply_rules_once_from_list(
        &mut self,
        expr: Expr,
        normalized_expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(rule) = rules.get(index) else {
            return Ok(None);
        };
        let would_reject =
            !Self::definition_lhs_could_match_candidate(&rule.lhs, &rule.matcher, &normalized_expr);
        record_profile_rewrite_rule_prefilter_candidate(would_reject);
        let matcher = rule.matcher.clone();

        let match_result =
            self.match_compiled_owned(matcher, normalized_expr.clone(), SymbolExprMap::default());

        let outcome = match match_result {
            Ok(outcome) => outcome,
            Err(EvalError::InputRequired { request }) => {
                let rule = rules
                    .get(index)
                    .cloned()
                    .expect("rewrite rule should still exist for suspended match");
                return self.drive_internal_value::<CompiledMatchOutcome, Option<Expr>, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, outcome| {
                        record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);
                        if !outcome.matched {
                            return session.apply_rules_once_from_list(
                                expr,
                                normalized_expr,
                                rules,
                                source,
                                index + 1,
                            );
                        }

                        let replaced = substitute_rule_bindings_for_rhs(
                            &rule.rhs,
                            &outcome.bindings,
                            rule.rhs_may_splice_without_bindings,
                        );
                        session.push_rule_applied_trace(source, &expr, &rule.lhs, &replaced);
                        Ok(Some(replaced))
                    },
                );
            }
            Err(error) => return Err(error),
        };
        record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);

        if !outcome.matched {
            return self.apply_rules_once_from_list(
                expr,
                normalized_expr,
                rules,
                source,
                index + 1,
            );
        }

        let rule = rules
            .get(index)
            .expect("rewrite rule should still exist after match");
        let replaced = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &outcome.bindings,
            rule.rhs_may_splice_without_bindings,
        );
        self.push_rule_applied_trace(source, &expr, &rule.lhs, &replaced);
        Ok(Some(replaced))
    }
}
