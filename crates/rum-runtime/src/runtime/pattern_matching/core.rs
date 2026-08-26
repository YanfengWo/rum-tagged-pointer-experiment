use super::{outcome::*, *};

impl Session {
    #[allow(dead_code)]
    pub(in crate::runtime) fn match_expr(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut SymbolExprMap,
    ) -> Result<bool, EvalError> {
        let outcome = self.match_expr_owned(pattern.clone(), value.clone(), bindings.clone())?;
        *bindings = outcome.bindings;
        Ok(outcome.matched)
    }

    pub(in crate::runtime) fn match_expr_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let matcher = self.compile_matcher_for_expr(&pattern);
        let normalized_value = self.normalize_match_expr(&value);
        self.match_compiled_owned(matcher, normalized_value, bindings)
    }

    pub(in crate::runtime) fn match_compiled_owned(
        &mut self,
        matcher: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = BindingSnapshot::capture(&bindings, &matcher);
        let outcome = self.match_compiled_inner_owned(matcher, value, bindings)?;

        if outcome.matched {
            Ok(outcome)
        } else {
            Ok(match_outcome(false, snapshot.restore(outcome.bindings)))
        }
    }

    pub(in crate::runtime) fn evaluate_pattern_test_owned(
        &mut self,
        test: Expr,
        value: Expr,
        bindings_for_true: SymbolExprMap,
        bindings_for_false: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let test_result = self.evaluate(Expr::call(test, vec![value]));
        self.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
            test_result,
            move |_, result| {
                Ok(if is_system_symbol_name(&result, "True") {
                    match_outcome(true, bindings_for_true)
                } else {
                    match_outcome(false, bindings_for_false)
                })
            },
        )
    }

    fn match_compiled_alternatives_owned(
        &mut self,
        alternatives: Vec<MatcherNode>,
        value: Expr,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(alternative) = alternatives.get(index).cloned() else {
            return Ok(match_outcome(false, bindings));
        };

        let value_for_retry = value.clone();
        let alternatives_for_retry = alternatives.clone();
        let alternative_result = self.match_compiled_owned(alternative, value, bindings);
        self.drive_internal_value(alternative_result, move |session, outcome| {
            if outcome.matched {
                Ok(outcome)
            } else {
                session.match_compiled_alternatives_owned(
                    alternatives_for_retry,
                    value_for_retry,
                    outcome.bindings,
                    index + 1,
                )
            }
        })
    }

    fn match_compiled_named_owned(
        &mut self,
        name: Symbol,
        inner: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value.clone(), bindings);
        self.drive_internal_value(inner_result, move |_, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            let mut bindings = outcome.bindings;
            Ok(match_outcome(
                bind_pattern_name(&name, value.clone(), &mut bindings),
                bindings,
            ))
        })
    }

    fn match_compiled_condition_owned(
        &mut self,
        inner: MatcherNode,
        condition: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value, bindings);
        self.drive_internal_value(inner_result, move |session, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            let snapshot = outcome.bindings.clone();
            let bindings = outcome.bindings;
            session.evaluate_guard_owned(condition, bindings, snapshot, None)
        })
    }

    fn match_compiled_pattern_test_owned(
        &mut self,
        inner: MatcherNode,
        test: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value.clone(), bindings);
        self.drive_internal_value(inner_result, move |session, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            let false_bindings = outcome.bindings.clone();
            session.evaluate_pattern_test_owned(
                test,
                value.clone(),
                outcome.bindings,
                false_bindings,
            )
        })
    }

    fn match_compiled_except_owned(
        &mut self,
        excluded: MatcherNode,
        inner: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value.clone(), bindings);
        self.drive_internal_value(inner_result, move |session, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            let excluded_result = session.match_compiled_owned(excluded, value, outcome.bindings);
            session.drive_internal_value(excluded_result, move |_, excluded_outcome| {
                Ok(if excluded_outcome.matched {
                    match_outcome(false, excluded_outcome.bindings)
                } else {
                    match_outcome(true, excluded_outcome.bindings)
                })
            })
        })
    }

    fn match_compiled_call_owned(
        &mut self,
        head: MatcherNode,
        args: Vec<MatcherArg>,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let one_identity_head = match &head {
            MatcherNode::Exact(Expr::Atom(Atom::Symbol(symbol)))
                if self
                    .attribute_set_for_symbol(symbol)
                    .contains(Attribute::OneIdentity) =>
            {
                Some(*symbol)
            }
            _ => None,
        };
        let orderless = matcher_head_has_attribute(&head, self, Attribute::Orderless);
        let flat = matcher_head_has_attribute(&head, self, Attribute::Flat);

        match value {
            Expr::Call {
                head: value_head,
                args: value_args,
            } => {
                let flat_head = value_head.as_ref().clone();
                let value_head = value_head.into_owned();
                let value_args = value_args.into_vec();
                let head_result = self.match_compiled_owned(head, value_head, bindings);
                self.drive_internal_value(head_result, move |session, outcome| {
                    if !outcome.matched {
                        let Some(one_identity_head) = one_identity_head else {
                            return Ok(outcome);
                        };

                        return session.match_compiled_promoted_singleton_owned(
                            args,
                            Expr::call(flat_head, value_args),
                            outcome.bindings,
                            orderless,
                            flat,
                            Expr::Atom(Atom::Symbol(one_identity_head)),
                        );
                    }

                    if orderless {
                        if flat {
                            session.match_compiled_orderless_args_with_flat_head_owned(
                                args,
                                value_args,
                                outcome.bindings,
                                flat_head,
                            )
                        } else {
                            session.match_compiled_orderless_args_owned(
                                args,
                                value_args,
                                outcome.bindings,
                            )
                        }
                    } else if flat {
                        session.match_compiled_args_with_flat_head_owned(
                            args,
                            value_args,
                            outcome.bindings,
                            flat_head,
                        )
                    } else {
                        session.match_compiled_args_owned(args, value_args, outcome.bindings)
                    }
                })
            }
            other if one_identity_head.is_some() => self.match_compiled_promoted_singleton_owned(
                args,
                other,
                bindings,
                orderless,
                flat,
                Expr::Atom(Atom::Symbol(
                    one_identity_head.expect("guarded OneIdentity head should be present"),
                )),
            ),
            _ => Ok(match_outcome(false, bindings)),
        }
    }

    fn match_compiled_promoted_singleton_owned(
        &mut self,
        args: Vec<MatcherArg>,
        value: Expr,
        bindings: SymbolExprMap,
        orderless: bool,
        flat: bool,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let value_args = vec![value];

        if orderless {
            if flat {
                self.match_compiled_orderless_args_with_flat_head_owned(
                    args, value_args, bindings, flat_head,
                )
            } else {
                self.match_compiled_orderless_args_owned(args, value_args, bindings)
            }
        } else if flat {
            self.match_compiled_args_with_flat_head_owned(args, value_args, bindings, flat_head)
        } else {
            self.match_compiled_args_owned(args, value_args, bindings)
        }
    }

    pub(in crate::runtime) fn match_compiled_key_value_pattern_owned(
        &mut self,
        entries: Vec<(MatcherNode, MatcherNode, bool)>,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(candidates) = keyed_entry_forms_from_expr(&value) else {
            return Ok(match_outcome(false, bindings));
        };

        self.match_compiled_key_value_pattern_entries_owned(entries, candidates, bindings)
    }

    fn match_compiled_key_value_pattern_entries_owned(
        &mut self,
        entries: Vec<(MatcherNode, MatcherNode, bool)>,
        candidates: Vec<AssociationEntryForm>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some((key_pattern, value_pattern, delayed)) = entries.first().cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        self.match_compiled_key_value_pattern_choice_owned(
            key_pattern,
            value_pattern,
            delayed,
            entries.into_iter().skip(1).collect(),
            candidates,
            bindings,
            0,
        )
    }

    fn match_compiled_key_value_pattern_choice_owned(
        &mut self,
        key_pattern: MatcherNode,
        value_pattern: MatcherNode,
        delayed: bool,
        remaining_entries: Vec<(MatcherNode, MatcherNode, bool)>,
        candidates: Vec<AssociationEntryForm>,
        bindings: SymbolExprMap,
        candidate_index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(candidate) = candidates.get(candidate_index).cloned() else {
            return Ok(match_outcome(false, bindings));
        };

        if candidate.delayed != delayed {
            return self.match_compiled_key_value_pattern_choice_owned(
                key_pattern,
                value_pattern,
                delayed,
                remaining_entries,
                candidates,
                bindings,
                candidate_index + 1,
            );
        }

        let candidates_for_retry = candidates.clone();
        let remaining_entries_for_retry = remaining_entries.clone();
        let key_pattern_for_retry = key_pattern.clone();
        let value_pattern_for_retry = value_pattern.clone();
        let candidates_for_success = candidates.clone();
        let remaining_entries_for_success = remaining_entries.clone();
        let candidate_key = candidate.key;
        let candidate_value = candidate.value;

        let key_result = self.match_compiled_owned(key_pattern, candidate_key, bindings);
        self.drive_internal_value(key_result, move |session, key_outcome| {
            if !key_outcome.matched {
                return session.match_compiled_key_value_pattern_choice_owned(
                    key_pattern_for_retry,
                    value_pattern_for_retry,
                    delayed,
                    remaining_entries_for_retry,
                    candidates_for_retry,
                    key_outcome.bindings,
                    candidate_index + 1,
                );
            }

            let candidates_for_value_failure = candidates_for_success.clone();
            let remaining_entries_for_value_failure = remaining_entries_for_success.clone();
            let key_pattern_for_value_failure = key_pattern_for_retry.clone();
            let value_pattern_for_value_failure = value_pattern_for_retry.clone();

            let value_result =
                session.match_compiled_owned(value_pattern, candidate_value, key_outcome.bindings);
            session.drive_internal_value(value_result, move |session, value_outcome| {
                if !value_outcome.matched {
                    return session.match_compiled_key_value_pattern_choice_owned(
                        key_pattern_for_value_failure,
                        value_pattern_for_value_failure,
                        delayed,
                        remaining_entries_for_value_failure,
                        candidates_for_value_failure,
                        value_outcome.bindings,
                        candidate_index + 1,
                    );
                }

                let mut remaining_candidates = candidates_for_success;
                remaining_candidates.remove(candidate_index);
                session.match_compiled_key_value_pattern_entries_owned(
                    remaining_entries_for_success,
                    remaining_candidates,
                    value_outcome.bindings,
                )
            })
        })
    }

    pub(super) fn evaluate_sequence_pattern_checks_owned(
        &mut self,
        checks: Vec<SequencePatternCheck>,
        sequence_value: Expr,
        bindings: SymbolExprMap,
        snapshot: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(check) = checks.get(index).cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        match check {
            SequencePatternCheck::Condition(condition) => {
                let checks_for_next = checks.clone();
                let sequence_for_next = sequence_value.clone();
                let snapshot_for_next = snapshot.clone();
                let condition_result =
                    self.evaluate_guard_owned(condition, bindings, snapshot, None);

                self.drive_internal_value(condition_result, move |session, outcome| {
                    if outcome.matched {
                        session.evaluate_sequence_pattern_checks_owned(
                            checks_for_next,
                            sequence_for_next,
                            outcome.bindings,
                            snapshot_for_next,
                            index + 1,
                        )
                    } else {
                        Ok(outcome)
                    }
                })
            }
            SequencePatternCheck::PatternTest(test) => {
                let checks_for_next = checks.clone();
                let sequence_for_test = sequence_value.clone();
                let sequence_for_next = sequence_value.clone();
                let bindings_for_true = bindings.clone();
                let snapshot_for_false = snapshot.clone();
                let test_result =
                    self.evaluate_pattern_test_owned(test, sequence_for_test, bindings, snapshot);

                self.drive_internal_value(test_result, move |session, outcome| {
                    if outcome.matched {
                        session.evaluate_sequence_pattern_checks_owned(
                            checks_for_next,
                            sequence_for_next,
                            bindings_for_true,
                            snapshot_for_false,
                            index + 1,
                        )
                    } else {
                        Ok(outcome)
                    }
                })
            }
        }
    }

    fn match_compiled_inner_owned(
        &mut self,
        matcher: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match matcher {
            MatcherNode::Exact(pattern) => Ok(match_outcome(pattern == value, bindings)),
            MatcherNode::Blank { head_constraint } => Ok(match_outcome(
                match_blank(head_constraint.as_ref(), &value),
                bindings,
            )),
            MatcherNode::KeyValuePattern(entries) => {
                self.match_compiled_key_value_pattern_owned(entries, value, bindings)
            }
            MatcherNode::Named { name, inner } => {
                self.match_compiled_named_owned(name, *inner, value, bindings)
            }
            MatcherNode::Condition { inner, condition } => {
                self.match_compiled_condition_owned(*inner, condition, value, bindings)
            }
            MatcherNode::DefinitionCondition(matcher) => {
                self.match_compiled_definition_condition_owned(*matcher, value, bindings)
            }
            MatcherNode::PatternTest { inner, test } => {
                self.match_compiled_pattern_test_owned(*inner, test, value, bindings)
            }
            MatcherNode::Except { excluded, inner } => {
                self.match_compiled_except_owned(*excluded, *inner, value, bindings)
            }
            MatcherNode::Alternatives(alternatives) => {
                self.match_compiled_alternatives_owned(alternatives, value, bindings, 0)
            }
            MatcherNode::Call { head, args } => {
                self.match_compiled_call_owned(*head, args, value, bindings)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct BindingSnapshot {
    entries: Vec<(Symbol, Option<Expr>)>,
}

impl BindingSnapshot {
    fn capture(bindings: &SymbolExprMap, matcher: &MatcherNode) -> Self {
        let mut names = BTreeSet::new();
        collect_matcher_binding_names(matcher, &mut names);
        let entries = names
            .into_iter()
            .map(|name| {
                let previous = bindings.get(&name).cloned();
                (name, previous)
            })
            .collect();
        Self { entries }
    }

    fn restore(self, mut bindings: SymbolExprMap) -> SymbolExprMap {
        for (name, previous) in self.entries {
            match previous {
                Some(value) => {
                    bindings.insert(name, value);
                }
                None => {
                    bindings.remove(&name);
                }
            }
        }
        bindings
    }
}

fn collect_matcher_binding_names(matcher: &MatcherNode, names: &mut BTreeSet<Symbol>) {
    match matcher {
        MatcherNode::Exact(_) | MatcherNode::Blank { .. } => {}
        MatcherNode::KeyValuePattern(entries) => {
            for (key, value, _) in entries {
                collect_matcher_binding_names(key, names);
                collect_matcher_binding_names(value, names);
            }
        }
        MatcherNode::Named { name, inner } => {
            names.insert(name.clone());
            collect_matcher_binding_names(inner, names);
        }
        MatcherNode::Condition { inner, .. } | MatcherNode::PatternTest { inner, .. } => {
            collect_matcher_binding_names(inner, names);
        }
        MatcherNode::DefinitionCondition(matcher) => {
            collect_matcher_binding_names(&matcher.inner, names);
        }
        MatcherNode::Except { excluded, inner } => {
            collect_matcher_binding_names(excluded, names);
            collect_matcher_binding_names(inner, names);
        }
        MatcherNode::Alternatives(alternatives) => {
            for alternative in alternatives {
                collect_matcher_binding_names(alternative, names);
            }
        }
        MatcherNode::Call { head, args } => {
            collect_matcher_binding_names(head, names);
            for arg in args {
                collect_matcher_arg_binding_names(arg, names);
            }
        }
    }
}

fn collect_matcher_arg_binding_names(arg: &MatcherArg, names: &mut BTreeSet<Symbol>) {
    match arg {
        MatcherArg::Single(matcher) => collect_matcher_binding_names(matcher, names),
        MatcherArg::Sequence(spec) => {
            if let Some(name) = &spec.name {
                names.insert(name.clone());
            }
            collect_matcher_binding_names(&spec.matcher, names);
        }
        MatcherArg::Optional(spec) => collect_matcher_arg_binding_names(&spec.inner, names),
        MatcherArg::PatternSequenceGroup(items)
        | MatcherArg::OrderlessPatternSequenceGroup(items) => {
            for item in items {
                collect_matcher_arg_binding_names(item, names);
            }
        }
    }
}
