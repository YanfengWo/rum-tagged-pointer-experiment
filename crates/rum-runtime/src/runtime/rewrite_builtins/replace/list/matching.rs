use super::*;

impl Session {
    pub(super) fn collect_replace_list_rule_bindings_owned(
        &mut self,
        matcher: MatcherNode,
        normalized_target: Expr,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match matcher {
            MatcherNode::Call { head, args } => {
                let one_identity = matcher_head_has_attribute(&head, self, Attribute::OneIdentity);
                let orderless = matcher_head_has_attribute(&head, self, Attribute::Orderless);
                let flat = matcher_head_has_attribute(&head, self, Attribute::Flat);

                match normalized_target {
                    Expr::Call {
                        head: value_head,
                        args: value_args,
                    } => {
                        let flat_head = (*value_head).clone();
                        let head_result =
                            self.match_compiled_owned(*head, *value_head, BTreeMap::new());
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
                                BTreeMap::new(),
                            )
                        } else {
                            self.collect_compiled_args_matches_owned(
                                args,
                                vec![other],
                                BTreeMap::new(),
                            )
                        }
                    }
                    _ => Ok(Vec::new()),
                }
            }
            other => {
                let match_result =
                    self.match_compiled_owned(other, normalized_target, BTreeMap::new());
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

    pub(super) fn collect_replace_list_rule_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: Vec<RewriteRule>,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        rule_index: usize,
        rule: RewriteRule,
        bindings: Vec<BTreeMap<Symbol, Expr>>,
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

        let replaced = substitute_rule_bindings(&rule.rhs, &current_bindings);
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
