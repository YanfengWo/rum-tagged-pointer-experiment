use super::*;

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
                        self.drive_internal_value(head_result, move |session, outcome| {
                            if !outcome.matched {
                                return Ok(Vec::new());
                            }

                            if orderless {
                                if flat {
                                    session
                                        .collect_compiled_orderless_arg_matches_with_flat_head_owned(
                                            args,
                                            value_args,
                                            outcome.bindings,
                                            flat_head,
                                        )
                                } else {
                                    session.collect_compiled_orderless_arg_matches_owned(
                                        args,
                                        value_args,
                                        outcome.bindings,
                                    )
                                }
                            } else if flat {
                                session.collect_compiled_args_matches_with_flat_head_owned(
                                    args,
                                    value_args,
                                    outcome.bindings,
                                    flat_head,
                                )
                            } else {
                                session.collect_compiled_args_matches_owned(
                                    args,
                                    value_args,
                                    outcome.bindings,
                                )
                            }
                        })
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
                    self.push_suspension_frame(move |session, value| {
                        let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
                        session
                            .filter_replace_list_guard_bindings_owned(
                                guards,
                                bindings,
                                option_scope,
                                accepted,
                                index,
                                Some(outcome),
                            )
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
        rules: Vec<RewriteRule>,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        rule_index: usize,
        rule: RewriteRule,
        bindings: Vec<SymbolExprMap>,
        binding_index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        if max_results.is_some_and(|limit| collected.len() >= limit) {
            return Ok(collected);
        }

        let Some(current_bindings) = bindings.get(binding_index).cloned() else {
            return self.collect_replace_list_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                rule_index + 1,
            );
        };

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
        self.drive_evaluated_value::<Expr, Vec<Expr>, _>(replaced_result, move |session, value| {
            let mut collected = collected;
            collected.push(value);
            session.collect_replace_list_rule_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                rule_index,
                rule,
                bindings,
                binding_index + 1,
            )
        })
    }
}
