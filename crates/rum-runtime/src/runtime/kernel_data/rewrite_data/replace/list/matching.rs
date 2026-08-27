use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct SuspendedReplaceListCallMatch {
    matcher_args: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    flat_head: Expr,
    orderless: bool,
    flat: bool,
}

#[derive(FrameRootTrace)]
struct SuspendedReplaceListGuardFilter {
    guards: SharedArgs,
    bindings: PersistentRewriteQueue<SymbolExprMap>,
    option_scope: Option<OptionValueScope>,
    accepted: PersistentMatchResults,
}

#[derive(FrameRootTrace)]
struct SuspendedReplaceListRuleResults {
    scan: SuspendedReplaceListScan,
    bindings: PersistentRewriteQueue<SymbolExprMap>,
}

impl Session {
    pub(super) fn collect_replace_list_rule_bindings_owned(
        &mut self,
        matcher: MatcherNode,
        normalized_target: Expr,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match matcher {
            MatcherNode::DefinitionCondition(matcher) => {
                let DefinitionConditionMatcher { inner, lhs, guards } = *matcher;
                let option_scope = self.option_value_scope_for_definition(&lhs, &normalized_target);
                let bindings_result =
                    self.collect_replace_list_rule_bindings_owned(inner, normalized_target);
                self.drive_internal_value(bindings_result, move |session, bindings| {
                    session.filter_replace_list_guard_bindings_owned(
                        guards,
                        bindings,
                        option_scope,
                        Vec::new(),
                        0,
                        None,
                    )
                })
            }
            MatcherNode::Call { head, args } => {
                let one_identity = matcher_head_has_attribute(&head, self, Attribute::OneIdentity);
                let orderless = matcher_head_has_attribute(&head, self, Attribute::Orderless);
                let flat = matcher_head_has_attribute(&head, self, Attribute::Flat);

                match normalized_target {
                    Expr::Call {
                        head: value_head,
                        args: value_args,
                    } => {
                        let flat_head = value_head.as_ref().clone();
                        let value_head = value_head.into_owned();
                        let value_args = value_args.into_vec();
                        let head_result =
                            self.match_compiled_owned(*head, value_head, SymbolExprMap::default());
                        match head_result {
                            Ok(outcome) => self.finish_replace_list_call_head_match(
                                args, value_args, flat_head, orderless, flat, outcome,
                            ),
                            Err(EvalError::InputRequired { request }) => {
                                let state = SuspendedReplaceListCallMatch {
                                    matcher_args: FrozenMatcherSequence::freeze(args),
                                    value_args: SharedArgs::new(value_args),
                                    flat_head,
                                    orderless,
                                    flat,
                                };
                                self.push_suspension_frame(move |session, value| {
                                    let outcome =
                                        CompiledMatchOutcome::from_continuation_expr(value?);
                                    session
                                        .finish_suspended_replace_list_call_head_match(
                                            state, outcome,
                                        )
                                        .map(|bindings| bindings.into_continuation_expr())
                                });
                                Err(EvalError::InputRequired { request })
                            }
                            Err(error) => Err(error),
                        }
                    }
                    other if one_identity => {
                        if orderless {
                            self.collect_compiled_orderless_arg_matches_owned(
                                args,
                                vec![other],
                                SymbolExprMap::default(),
                            )
                        } else {
                            self.collect_compiled_args_matches_owned(
                                args,
                                vec![other],
                                SymbolExprMap::default(),
                            )
                        }
                    }
                    _ => Ok(Vec::new()),
                }
            }
            other => {
                let match_result =
                    self.match_compiled_owned(other, normalized_target, SymbolExprMap::default());
                self.drive_internal_value(match_result, move |_, outcome| {
                    Ok(if outcome.matched {
                        vec![outcome.bindings]
                    } else {
                        Vec::new()
                    })
                })
            }
        }
    }

    fn finish_suspended_replace_list_call_head_match(
        &mut self,
        state: SuspendedReplaceListCallMatch,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.finish_replace_list_call_head_match(
            state.matcher_args.into_vec(),
            state.value_args.into_vec(),
            state.flat_head,
            state.orderless,
            state.flat,
            outcome,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_replace_list_call_head_match(
        &mut self,
        args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        flat_head: Expr,
        orderless: bool,
        flat: bool,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            return Ok(Vec::new());
        }

        if orderless {
            if flat {
                self.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                    args,
                    value_args,
                    outcome.bindings,
                    flat_head,
                )
            } else {
                self.collect_compiled_orderless_arg_matches_owned(
                    args,
                    value_args,
                    outcome.bindings,
                )
            }
        } else if flat {
            self.collect_compiled_args_matches_with_flat_head_owned(
                args,
                value_args,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_args_matches_owned(args, value_args, outcome.bindings)
        }
    }

    fn filter_replace_list_guard_bindings_owned(
        &mut self,
        guards: Box<[Expr]>,
        bindings: Vec<SymbolExprMap>,
        option_scope: Option<OptionValueScope>,
        mut accepted: Vec<SymbolExprMap>,
        mut index: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                if outcome.matched {
                    accepted.push(outcome.bindings);
                }
                index += 1;
            }

            let Some(current) = bindings.get(index).cloned() else {
                return Ok(accepted);
            };
            let false_bindings = current.clone();
            let guard_result = self.evaluate_guard_sequence_owned(
                guards.clone(),
                current,
                false_bindings,
                option_scope.clone(),
            );

            match guard_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let remaining = bindings.into_iter().skip(index + 1).collect();
                    let state = SuspendedReplaceListGuardFilter {
                        guards: SharedArgs::new(guards.into_vec()),
                        bindings: PersistentRewriteQueue::freeze(remaining),
                        option_scope,
                        accepted: PersistentMatchResults::freeze(accepted),
                    };
                    self.push_suspension_frame(move |session, value| {
                        let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
                        session
                            .resume_suspended_replace_list_guard_filter(state, outcome)
                            .map(|bindings| bindings.into_continuation_expr())
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_replace_list_guard_filter(
        &mut self,
        mut state: SuspendedReplaceListGuardFilter,
        mut outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        loop {
            if outcome.matched {
                state.accepted = state.accepted.appended_values(vec![outcome.bindings]);
            }

            let Some((current, remaining)) = state.bindings.pop_front() else {
                return Ok(state.accepted.into_vec());
            };
            state.bindings = remaining;
            let guard_result = self.evaluate_guard_sequence_same_bindings_suspended(
                state.guards.clone(),
                current,
                state.option_scope.clone(),
            );
            match guard_result {
                Ok(next_outcome) => outcome = next_outcome,
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
                        session
                            .resume_suspended_replace_list_guard_filter(state, outcome)
                            .map(|bindings| bindings.into_continuation_expr())
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn collect_replace_list_rule_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: RewriteRulePlan,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        rule_index: usize,
        bindings: Vec<SymbolExprMap>,
    ) -> Result<Vec<Expr>, EvalError> {
        if max_results.is_some_and(|limit| collected.len() >= limit) {
            return Ok(collected);
        }

        let mut bindings = bindings.into_iter();
        let Some(current_bindings) = bindings.next() else {
            return self.collect_replace_list_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                rule_index + 1,
            );
        };
        let remaining_bindings = bindings.collect::<Vec<_>>();
        let rule = rules
            .get(rule_index)
            .expect("ReplaceList rule remains in its immutable plan");

        let replaced = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &current_bindings,
            rule.rhs_may_splice_without_bindings,
        );
        self.push_rule_applied_trace(
            RuleApplicationSource::ReplaceList,
            &target,
            &rule.lhs,
            &replaced,
        );
        let replaced_result = self.evaluate(replaced);
        match replaced_result {
            Ok(value) => {
                let mut collected = collected;
                collected.push(value);
                self.collect_replace_list_rule_results_owned(
                    target,
                    normalized_target,
                    rules,
                    max_results,
                    collected,
                    rule_index,
                    remaining_bindings,
                )
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedReplaceListRuleResults {
                    scan: SuspendedReplaceListScan {
                        target,
                        normalized_target,
                        rules,
                        max_results,
                        collected: PersistentRewriteExprs::freeze(collected),
                        index: rule_index,
                    },
                    bindings: PersistentRewriteQueue::freeze(remaining_bindings),
                };
                self.push_suspension_frame(move |session, value| {
                    session
                        .resume_suspended_replace_list_rule_results(state, value?)
                        .map(|results| results.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn collect_replace_list_rule_results_suspended(
        &mut self,
        scan: SuspendedReplaceListScan,
        bindings: Vec<SymbolExprMap>,
        binding_index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        let remaining = bindings.into_iter().skip(binding_index).collect();
        self.drive_suspended_replace_list_rule_results(
            SuspendedReplaceListRuleResults {
                scan,
                bindings: PersistentRewriteQueue::freeze(remaining),
            },
            None,
        )
    }

    fn resume_suspended_replace_list_rule_results(
        &mut self,
        state: SuspendedReplaceListRuleResults,
        value: Expr,
    ) -> Result<Vec<Expr>, EvalError> {
        self.drive_suspended_replace_list_rule_results(state, Some(value))
    }

    fn drive_suspended_replace_list_rule_results(
        &mut self,
        mut state: SuspendedReplaceListRuleResults,
        mut pending: Option<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            if let Some(value) = pending.take() {
                state.scan.collected = state.scan.collected.appended(value);
            }
            if state
                .scan
                .max_results
                .is_some_and(|limit| state.scan.collected.len() >= limit)
            {
                return Ok(state.scan.collected.into_vec());
            }

            let Some((current_bindings, remaining)) = state.bindings.pop_front() else {
                state.scan.index += 1;
                return self.collect_replace_list_results_suspended(state.scan, None);
            };
            state.bindings = remaining;
            let rule = state
                .scan
                .rules
                .get(state.scan.index)
                .expect("ReplaceList rule remains in its suspended plan");
            let replaced = substitute_rule_bindings_for_rhs(
                &rule.rhs,
                &current_bindings,
                rule.rhs_may_splice_without_bindings,
            );
            self.push_rule_applied_trace(
                RuleApplicationSource::ReplaceList,
                &state.scan.target,
                &rule.lhs,
                &replaced,
            );
            let replaced_result = self.evaluate(replaced);
            match replaced_result {
                Ok(value) => pending = Some(value),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        session
                            .resume_suspended_replace_list_rule_results(state, value?)
                            .map(|results| results.into_continuation_expr())
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceListCallMatch>();
    assert_traces::<SuspendedReplaceListGuardFilter>();
    assert_traces::<SuspendedReplaceListRuleResults>();
};
