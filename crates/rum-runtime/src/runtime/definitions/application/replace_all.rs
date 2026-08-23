use super::*;
use std::sync::Arc;

type SharedRewriteRules = Arc<[RewriteRule]>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplaceAllTraversalOutcome {
    expr: Expr,
    changed: bool,
}

impl ContinuationValue for ReplaceAllTraversalOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplaceAllTraversalOutcome"),
            vec![
                self.expr,
                system_expr(if self.changed { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationReplaceAllTraversalOutcome",
                ) && args.len() == 2 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let changed_expr = args
                    .pop()
                    .expect("replace-all traversal outcome stores changed");
                let expr = args
                    .pop()
                    .expect("replace-all traversal outcome stores expr");
                Self {
                    expr,
                    changed: is_system_symbol_name(&changed_expr, "True"),
                }
            }
            other => panic!("unexpected replace-all traversal outcome payload: {other:?}"),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn apply_replace_all(
        &mut self,
        expr: Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let outcome_result =
            self.apply_replace_all_owned(expr, SharedRewriteRules::from(rules), source);
        self.drive_internal_value(outcome_result, |_, outcome| Ok(outcome.expr))
    }

    pub(in crate::runtime) fn try_apply_rewrite_rules_once_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        self.apply_rules_once_from_list(expr, normalized_expr, rules.into(), source, 0)
    }

    fn apply_replace_all_owned(
        &mut self,
        expr: Expr,
        rules: SharedRewriteRules,
        source: RuleApplicationSource,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        let replace_result = self.apply_rules_once_from_list(
            expr.clone(),
            normalized_expr,
            rules.clone(),
            source,
            0,
        );

        self.drive_internal_value(replace_result, move |session, outcome| match outcome {
            Some(replaced) => Ok(ReplaceAllTraversalOutcome {
                expr: replaced,
                changed: true,
            }),
            None => session.apply_replace_all_after_miss(expr, rules, source),
        })
    }

    fn apply_replace_all_after_miss(
        &mut self,
        expr: Expr,
        rules: SharedRewriteRules,
        source: RuleApplicationSource,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        match expr {
            Expr::Atom(_) => Ok(ReplaceAllTraversalOutcome {
                expr,
                changed: false,
            }),
            Expr::Call { head, args } => {
                let hold_attribute = if source == RuleApplicationSource::ReplaceAllUnheld {
                    self.replace_all_unheld_hold_attribute_for_head(&head)
                } else {
                    HoldAttribute::None
                };
                let remaining_args = args.into_vec();
                let head_result =
                    self.apply_replace_all_owned(head.into_owned(), rules.clone(), source);
                self.drive_internal_value(head_result, move |session, head_outcome| {
                    session.apply_replace_all_args(
                        head_outcome.expr,
                        Vec::new(),
                        remaining_args,
                        rules,
                        source,
                        head_outcome.changed,
                        hold_attribute,
                        0,
                    )
                })
            }
        }
    }

    fn replace_all_unheld_hold_attribute_for_head(&mut self, head: &Expr) -> HoldAttribute {
        let mut attributes = self.cached_attribute_set_for_head(head);

        if expr_as_symbol(head).is_none() {
            if let Some(symbol) = base_head_symbol_of_nested_call(head) {
                if self
                    .cached_attribute_set_for_symbol(symbol)
                    .contains(Attribute::SubValuesHoldAll)
                {
                    attributes = attributes.with(Attribute::HoldAll);
                }
            }
        }

        attributes.hold_attribute()
    }

    fn apply_replace_all_args(
        &mut self,
        head: Expr,
        mut rewritten_args: Vec<Expr>,
        remaining_args: Vec<Expr>,
        rules: SharedRewriteRules,
        source: RuleApplicationSource,
        changed: bool,
        hold_attribute: HoldAttribute,
        argument_index: usize,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        let Some((current, rest)) = remaining_args.split_first() else {
            return Ok(ReplaceAllTraversalOutcome {
                expr: Expr::call(head, rewritten_args),
                changed,
            });
        };

        let rest = rest.to_vec();
        if source == RuleApplicationSource::ReplaceAllUnheld
            && hold_attribute.holds_argument(argument_index)
        {
            rewritten_args.push(current.clone());
            return self.apply_replace_all_args(
                head,
                rewritten_args,
                rest,
                rules,
                source,
                changed,
                hold_attribute,
                argument_index + 1,
            );
        }

        let arg_result = self.apply_replace_all_owned(current.clone(), rules.clone(), source);

        self.drive_internal_value(arg_result, move |session, outcome| {
            if outcome.changed {
                session.append_changed_rewrite_argument(&head, &mut rewritten_args, outcome.expr);
            } else {
                rewritten_args.push(outcome.expr);
            }
            session.apply_replace_all_args(
                head,
                rewritten_args,
                rest,
                rules,
                source,
                changed || outcome.changed,
                hold_attribute,
                argument_index + 1,
            )
        })
    }

    pub(in crate::runtime) fn append_changed_rewrite_argument(
        &self,
        parent_head: &Expr,
        rebuilt_args: &mut Vec<Expr>,
        rewritten: Expr,
    ) -> bool {
        match rewritten {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, heads::functions::SEQUENCE)
                    && !self.attribute_set_for_head(parent_head).sequence_hold() =>
            {
                rebuilt_args.extend(args);
                true
            }
            other => {
                rebuilt_args.push(other);
                false
            }
        }
    }

    fn apply_rules_once_from_list(
        &mut self,
        expr: Expr,
        normalized_expr: Expr,
        rules: SharedRewriteRules,
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
