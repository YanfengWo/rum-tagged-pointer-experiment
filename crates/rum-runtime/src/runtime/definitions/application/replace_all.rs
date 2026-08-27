use super::*;

mod plan;

pub(in crate::runtime) use plan::RewriteRulePlan;
use plan::RewriteRuleScanCursor;

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
            self.apply_replace_all_owned(expr, RewriteRulePlan::from_slice(rules), source);
        self.drive_internal_value(outcome_result, |_, outcome| Ok(outcome.expr))
    }

    pub(in crate::runtime) fn shared_rewrite_rule_plan(rules: Vec<RewriteRule>) -> RewriteRulePlan {
        if rules
            .iter()
            .any(|rule| matches!(rule.matcher, MatcherNode::Exact(_)))
        {
            RewriteRulePlan::from_slice(&rules)
        } else {
            RewriteRulePlan::linear(rules)
        }
    }

    pub(in crate::runtime) fn apply_rewrite_rule_plan_once(
        &mut self,
        expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        // Replace and ReplaceAt build this indexed, Arc-backed plan once per
        // traversal; their cloneable applier carries the same plan across
        // targets and suspension frames.
        let cursor = rules.start_scan(&normalized_expr);
        self.apply_rules_once_from_list(expr, normalized_expr, rules, source, cursor)
    }

    fn apply_replace_all_owned(
        &mut self,
        expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        let cursor = rules.start_scan(&normalized_expr);
        let replace_result = self.apply_rules_once_from_list(
            expr.clone(),
            normalized_expr,
            rules.clone(),
            source,
            cursor,
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
        rules: RewriteRulePlan,
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
                let remaining_args = args;
                let head_result =
                    self.apply_replace_all_owned(head.into_owned(), rules.clone(), source);
                self.drive_internal_value(head_result, move |session, head_outcome| {
                    let rewritten_args = Vec::with_capacity(remaining_args.len());
                    session.apply_replace_all_args(
                        head_outcome.expr,
                        rewritten_args,
                        remaining_args,
                        rules,
                        source,
                        head_outcome.changed,
                        hold_attribute,
                        0,
                        None,
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

    /// Walks the arguments one per loop iteration, keeping the accumulated
    /// `rewritten_args` in place instead of recursing per element (the
    /// continuation self-recursion overflowed the native stack on wide flat
    /// lists). `pending` carries an already-computed argument outcome across a
    /// suspension resume, so the merge step runs exactly once per element on
    /// both the live and the resumed paths.
    fn apply_replace_all_args(
        &mut self,
        head: Expr,
        mut rewritten_args: Vec<Expr>,
        remaining_args: SharedArgs,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
        mut changed: bool,
        hold_attribute: HoldAttribute,
        mut argument_index: usize,
        mut pending: Option<ReplaceAllTraversalOutcome>,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                if outcome.changed {
                    self.append_changed_rewrite_argument(&head, &mut rewritten_args, outcome.expr);
                } else {
                    rewritten_args.push(outcome.expr);
                }
                changed = changed || outcome.changed;
                argument_index += 1;
            }

            let Some(current) = remaining_args.get(argument_index) else {
                return Ok(ReplaceAllTraversalOutcome {
                    expr: Expr::call(head, rewritten_args),
                    changed,
                });
            };

            if source == RuleApplicationSource::ReplaceAllUnheld
                && hold_attribute.holds_argument(argument_index)
            {
                rewritten_args.push(current.clone());
                argument_index += 1;
                continue;
            }

            let current = current.clone();
            let arg_result = self.apply_replace_all_owned(current, rules.clone(), source);

            match arg_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = ReplaceAllTraversalOutcome::from_continuation_expr(value?);
                        session
                            .apply_replace_all_args(
                                head,
                                rewritten_args,
                                remaining_args,
                                rules,
                                source,
                                changed,
                                hold_attribute,
                                argument_index,
                                Some(outcome),
                            )
                            .map(|outcome| outcome.into_continuation_expr())
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
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
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
        mut cursor: RewriteRuleScanCursor,
    ) -> Result<Option<Expr>, EvalError> {
        loop {
            let Some(index) = rules.next_rule_index(&mut cursor) else {
                return Ok(None);
            };
            let Some(rule) = rules.get(index) else {
                return Ok(None);
            };
            let would_reject = !self.definition_lhs_could_match_candidate(
                &rule.lhs,
                &rule.matcher,
                &normalized_expr,
            );
            record_profile_rewrite_rule_prefilter_candidate(would_reject);
            if would_reject {
                continue;
            }
            let matcher = rule.matcher.clone();

            let match_result = self.match_compiled_owned(
                matcher,
                normalized_expr.clone(),
                SymbolExprMap::default(),
            );

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
                            record_profile_rewrite_rule_match_outcome(
                                would_reject,
                                outcome.matched,
                            );
                            if !outcome.matched {
                                return session.apply_rules_once_from_list(
                                    expr,
                                    normalized_expr,
                                    rules,
                                    source,
                                    cursor,
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
                continue;
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
            return Ok(Some(replaced));
        }
    }
}
