mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

mod plan;

pub(in crate::runtime) use plan::RewriteRulePlan;
use plan::RewriteRuleScanCursor;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplaceAllTraversalOutcome {
    expr: Expr,
    changed: bool,
}

#[derive(Clone, FrameRootTrace)]
struct PersistentReplaceAllArgs {
    prefix: SharedArgs,
    tail: Option<Arc<PersistentReplaceAllArgNode>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct PersistentReplaceAllArgNode {
    previous: Option<Arc<PersistentReplaceAllArgNode>>,
    value: Expr,
}

impl PersistentReplaceAllArgs {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: None,
            len,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(PersistentReplaceAllArgNode {
                previous: self.tail.clone(),
                value,
            })),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail;
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed ReplaceAll output must be uniquely owned"));
            suffix.push(current.value);
            node = current.previous;
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceAllArgs {
    head: Expr,
    rewritten_args: PersistentReplaceAllArgs,
    remaining_args: SharedArgs,
    rules: RewriteRulePlan,
    changed: bool,
    argument_index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedRewriteRuleMatch {
    expr: Expr,
    normalized_expr: Expr,
    rules: RewriteRulePlan,
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
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| { Ok(outcome.expr) })() },
            suspend => ApplyReplaceAllSuspensionFrameL130C51,
        )
    }

    pub(in crate::runtime) fn apply_replace_all_with_plan(
        &mut self,
        expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let outcome_result = self.apply_replace_all_owned(expr, rules, source);
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| { Ok(outcome.expr) })() },
            suspend => ApplyReplaceAllWithPlanSuspensionFrameL140C51,
        )
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

        crate::resume_or_suspend!(
            self,
            replace_result,
            ok(outcome) => { let session = self; (|| { match outcome {
                    Some(replaced) => Ok(ReplaceAllTraversalOutcome {
                        expr: replaced,
                        changed: true,
                    }),
                    None => session.apply_replace_all_after_miss(expr, rules, source),
                } })() },
            suspend => ApplyReplaceAllOwnedSuspensionFrameL184C51 { expr, rules, source },
        )
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
                crate::resume_or_suspend!(
                    self,
                    head_result,
                    ok(head_outcome) => { let session = self; (|| {
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
                                })() },
                    suspend => ApplyReplaceAllAfterMissSuspensionFrameL213C56 { remaining_args, rules, source, hold_attribute },
                )
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
                    let state = SuspendedReplaceAllArgs {
                        head,
                        rewritten_args: PersistentReplaceAllArgs::freeze(rewritten_args),
                        remaining_args,
                        rules,
                        changed,
                        argument_index,
                    };
                    crate::resume_or_suspend!(insert self, ApplyReplaceAllArgsSuspensionFrameL306C48 { state, source, hold_attribute });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_replace_all_args(
        &mut self,
        mut state: SuspendedReplaceAllArgs,
        source: RuleApplicationSource,
        hold_attribute: HoldAttribute,
        mut outcome: ReplaceAllTraversalOutcome,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        loop {
            state.rewritten_args = self.append_suspended_replace_all_argument(
                &state.head,
                state.rewritten_args,
                outcome.expr,
                outcome.changed,
            );
            state.changed |= outcome.changed;
            state.argument_index += 1;

            loop {
                let Some(current) = state.remaining_args.get(state.argument_index) else {
                    return Ok(ReplaceAllTraversalOutcome {
                        expr: Expr::call(state.head, state.rewritten_args.into_vec()),
                        changed: state.changed,
                    });
                };

                if source == RuleApplicationSource::ReplaceAllUnheld
                    && hold_attribute.holds_argument(state.argument_index)
                {
                    state.rewritten_args = state.rewritten_args.appended(current.clone());
                    state.argument_index += 1;
                    continue;
                }

                let arg_result =
                    self.apply_replace_all_owned(current.clone(), state.rules.clone(), source);
                match arg_result {
                    Ok(next_outcome) => {
                        outcome = next_outcome;
                        break;
                    }
                    Err(EvalError::InputRequired { request }) => {
                        crate::resume_or_suspend!(insert self, ResumeSuspendedReplaceAllArgsSuspensionFrameL365C52 { state, source, hold_attribute });
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }

    fn append_suspended_replace_all_argument(
        &self,
        parent_head: &Expr,
        mut rebuilt_args: PersistentReplaceAllArgs,
        rewritten: Expr,
        changed: bool,
    ) -> PersistentReplaceAllArgs {
        if changed {
            if let Expr::Call { head, args } = rewritten {
                if is_system_symbol_name(&head, heads::functions::SEQUENCE)
                    && !self.attribute_set_for_head(parent_head).sequence_hold()
                {
                    for arg in args {
                        rebuilt_args = rebuilt_args.appended(arg);
                    }
                    return rebuilt_args;
                }
                return rebuilt_args.appended(Expr::call_with_shared_args(head.into_owned(), args));
            }
        }
        rebuilt_args.appended(rewritten)
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
                    let state = SuspendedRewriteRuleMatch {
                        expr,
                        normalized_expr,
                        rules,
                    };
                    crate::resume_or_suspend!(insert self, ApplyRulesOnceFromListSuspensionFrameL469C48 { state, source, cursor, index, would_reject });
                    return Err(EvalError::InputRequired { request });
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

    fn finish_suspended_rewrite_rule_match(
        &mut self,
        state: SuspendedRewriteRuleMatch,
        source: RuleApplicationSource,
        cursor: RewriteRuleScanCursor,
        index: usize,
        would_reject: bool,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);
        if !outcome.matched {
            return self.apply_rules_once_from_list(
                state.expr,
                state.normalized_expr,
                state.rules,
                source,
                cursor,
            );
        }

        let rule = state
            .rules
            .get(index)
            .expect("rewrite rule should still exist for resumed match");
        let replaced = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &outcome.bindings,
            rule.rhs_may_splice_without_bindings,
        );
        self.push_rule_applied_trace(source, &state.expr, &rule.lhs, &replaced);
        Ok(Some(replaced))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PersistentReplaceAllArgs>();
    assert_traces::<PersistentReplaceAllArgNode>();
    assert_traces::<SuspendedReplaceAllArgs>();
    assert_traces::<SuspendedRewriteRuleMatch>();
};
