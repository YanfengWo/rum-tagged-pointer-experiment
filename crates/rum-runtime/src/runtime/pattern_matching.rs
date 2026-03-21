use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledMatchOutcome {
    pub(super) matched: bool,
    pub(super) bindings: BTreeMap<Symbol, Expr>,
}

impl ContinuationValue for CompiledMatchOutcome {
    fn into_continuation_expr(self) -> Expr {
        let bindings = self
            .bindings
            .into_iter()
            .map(|(symbol, value)| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::Atom(Atom::Symbol(symbol)), value],
                )
            })
            .collect::<Vec<_>>();

        Expr::call(
            system_expr("List"),
            vec![
                system_expr(if self.matched { "True" } else { "False" }),
                Expr::call(system_expr("List"), bindings),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "List") && args.len() == 2 =>
            {
                let mut items = args.into_iter();
                let matched_expr = items.next().expect("match outcomes store a boolean slot");
                let bindings_expr = items
                    .next()
                    .expect("match outcomes store a binding-list slot");

                Self {
                    matched: is_system_symbol_name(&matched_expr, "True"),
                    bindings: bindings_from_continuation_expr(bindings_expr),
                }
            }
            other => panic!("unexpected continuation payload for compiled matcher: {other:?}"),
        }
    }
}

fn bindings_from_continuation_expr(expr: Expr) -> BTreeMap<Symbol, Expr> {
    let Expr::Call { head, args } = expr else {
        panic!("binding payload should be a List expression");
    };
    assert!(
        is_system_symbol_name(&head, "List"),
        "binding payload should use List"
    );

    args.into_iter()
        .map(|item| {
            let Expr::Call { head, args } = item else {
                panic!("binding payload items should be Rule expressions");
            };
            assert!(
                is_system_symbol_name(&head, "Rule") && args.len() == 2,
                "binding payload items should be Rule[symbol, value]"
            );
            let mut parts = args.into_iter();
            let name_expr = parts.next().expect("binding rules store a symbol");
            let value = parts.next().expect("binding rules store a value");
            let Expr::Atom(Atom::Symbol(symbol)) = name_expr else {
                panic!("binding rules should store a symbol on the lhs");
            };

            (symbol, value)
        })
        .collect()
}

fn match_outcome(matched: bool, bindings: BTreeMap<Symbol, Expr>) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}

fn optional_default_values(expr: Expr) -> Vec<Expr> {
    sequence_items(&expr)
        .map(|items| items.to_vec())
        .unwrap_or_else(|| vec![expr])
}

impl Session {
    #[allow(dead_code)]
    pub(super) fn match_expr(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let outcome = self.match_expr_owned(pattern.clone(), value.clone(), bindings.clone())?;
        *bindings = outcome.bindings;
        Ok(outcome.matched)
    }

    pub(super) fn match_expr_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let matcher = self.compile_matcher_for_expr(&pattern);
        let normalized_value = self.normalize_match_expr(&value);
        self.match_compiled_owned(matcher, normalized_value, bindings)
    }

    pub(super) fn match_compiled_owned(
        &mut self,
        matcher: MatcherNode,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();
        let outcome = self.match_compiled_inner_owned(matcher, value, bindings)?;

        if outcome.matched {
            Ok(outcome)
        } else {
            Ok(match_outcome(false, snapshot))
        }
    }

    pub(super) fn evaluate_pattern_test_owned(
        &mut self,
        test: Expr,
        value: Expr,
        bindings_for_true: BTreeMap<Symbol, Expr>,
        bindings_for_false: BTreeMap<Symbol, Expr>,
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
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(alternative) = alternatives.get(index).cloned() else {
            return Ok(match_outcome(false, bindings));
        };

        let snapshot = bindings.clone();
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
                    snapshot,
                    index + 1,
                )
            }
        })
    }

    pub(super) fn evaluate_sequence_pattern_checks_owned(
        &mut self,
        checks: Vec<SequencePatternCheck>,
        sequence_value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
        snapshot: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(check) = checks.get(index).cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        match check {
            SequencePatternCheck::Condition(condition) => {
                let bindings_for_true = bindings.clone();
                let snapshot_for_false = snapshot.clone();
                let checks_for_next = checks.clone();
                let sequence_for_next = sequence_value.clone();
                let condition_expr = substitute_bindings(&condition, &bindings);
                let condition_result = self.evaluate(condition_expr);

                self.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
                    condition_result,
                    move |session, result| {
                        if is_system_symbol_name(&result, "True") {
                            session.evaluate_sequence_pattern_checks_owned(
                                checks_for_next,
                                sequence_for_next,
                                bindings_for_true,
                                snapshot_for_false,
                                index + 1,
                            )
                        } else {
                            Ok(match_outcome(false, snapshot_for_false))
                        }
                    },
                )
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
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match matcher {
            MatcherNode::Exact(pattern) => Ok(match_outcome(pattern == value, bindings)),
            MatcherNode::Blank { head_constraint } => Ok(match_outcome(
                match_blank(head_constraint.as_ref(), &value),
                bindings,
            )),
            MatcherNode::Named { name, inner } => {
                let inner_result = self.match_compiled_owned(*inner, value.clone(), bindings);
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
            MatcherNode::Condition { inner, condition } => {
                let inner_result = self.match_compiled_owned(*inner, value, bindings);
                self.drive_internal_value(inner_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    let snapshot = outcome.bindings.clone();
                    let bindings = outcome.bindings;
                    let condition_expr = substitute_bindings(&condition, &bindings);
                    let condition_result = session.evaluate(condition_expr);

                    session.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
                        condition_result,
                        move |_, result| {
                            Ok(if is_system_symbol_name(&result, "True") {
                                match_outcome(true, bindings)
                            } else {
                                match_outcome(false, snapshot)
                            })
                        },
                    )
                })
            }
            MatcherNode::PatternTest { inner, test } => {
                let inner_result = self.match_compiled_owned(*inner, value.clone(), bindings);
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
            MatcherNode::Except { excluded, inner } => {
                let snapshot = bindings.clone();
                let inner_result = self.match_compiled_owned(*inner, value.clone(), bindings);
                self.drive_internal_value(inner_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    let accepted_bindings = outcome.bindings.clone();
                    let excluded_result =
                        session.match_compiled_owned(*excluded, value, outcome.bindings);
                    session.drive_internal_value(excluded_result, move |_, excluded_outcome| {
                        Ok(if excluded_outcome.matched {
                            match_outcome(false, snapshot)
                        } else {
                            match_outcome(true, accepted_bindings)
                        })
                    })
                })
            }
            MatcherNode::Alternatives(alternatives) => {
                self.match_compiled_alternatives_owned(alternatives, value, bindings, 0)
            }
            MatcherNode::Call { head, args } => {
                let one_identity =
                    matcher_head_has_attribute(head.as_ref(), self, Attribute::OneIdentity);

                match value {
                    Expr::Call {
                        head: value_head,
                        args: value_args,
                    } => {
                        let head_result = self.match_compiled_owned(*head, *value_head, bindings);
                        self.drive_internal_value(head_result, move |session, outcome| {
                            if !outcome.matched {
                                return Ok(outcome);
                            }

                            session.match_compiled_args_owned(args, value_args, outcome.bindings)
                        })
                    }
                    other if one_identity => match args.as_slice() {
                        [single] => self.match_compiled_arg_and_rest_owned(
                            single.clone(),
                            Vec::new(),
                            vec![other],
                            bindings,
                        ),
                        _ => Ok(match_outcome(false, bindings)),
                    },
                    _ => Ok(match_outcome(false, bindings)),
                }
            }
        }
    }

    pub(super) fn match_compiled_args_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_arg_and_rest_owned(first, rest_patterns, value_args, bindings)
    }

    fn match_compiled_arg_and_rest_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let start = spec.min_len;
                self.match_compiled_sequence_choices(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    start,
                )
            }
            MatcherArg::Optional(spec) => {
                self.match_compiled_optional_choices(spec, rest_patterns, value_args, bindings)
            }
            MatcherArg::Single(pattern) => {
                let mut value_iter = value_args.into_iter();
                let Some(value) = value_iter.next() else {
                    return Ok(match_outcome(false, bindings));
                };
                let rest_values = value_iter.collect::<Vec<_>>();

                let pattern_result = self.match_compiled_owned(pattern, value, bindings);
                self.drive_internal_value(pattern_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    session.match_compiled_args_owned(rest_patterns, rest_values, outcome.bindings)
                })
            }
        }
    }

    fn match_compiled_optional_choices(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if !value_args.is_empty() {
            let present_result = self.match_compiled_arg_and_rest_owned(
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings,
            );

            return self.drive_internal_value(present_result, move |session, outcome| {
                if outcome.matched {
                    Ok(outcome)
                } else {
                    session.match_compiled_optional_omission_owned(
                        spec,
                        rest_patterns,
                        value_args,
                        snapshot,
                    )
                }
            });
        }

        self.match_compiled_optional_omission_owned(spec, rest_patterns, value_args, snapshot)
    }

    fn match_compiled_optional_omission_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let default_result = self.resolve_optional_default_expr_owned(spec.default);
        self.drive_internal_value(default_result, move |session, default_expr| {
            let Some(default_expr) = default_expr else {
                return Ok(match_outcome(false, bindings));
            };

            let default_result = session.match_compiled_arg_and_rest_owned(
                *spec.inner,
                Vec::new(),
                optional_default_values(default_expr),
                bindings,
            );

            session.drive_internal_value(default_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                session.match_compiled_args_owned(rest_patterns, value_args, outcome.bindings)
            })
        })
    }

    fn resolve_optional_default_expr_owned(
        &mut self,
        default: CompiledOptionalDefault,
    ) -> Result<Option<Expr>, EvalError> {
        match default {
            CompiledOptionalDefault::Explicit(expr) => Ok(Some(expr)),
            CompiledOptionalDefault::UnavailableImplicit => Ok(None),
            CompiledOptionalDefault::Implicit { head, position } => {
                let specific_expr = Expr::call(
                    system_expr("Default"),
                    vec![head.clone(), Expr::integer(BigInt::from(position))],
                );
                let specific_result = self.evaluate(specific_expr.clone());

                self.drive_evaluated_value::<Expr, Option<Expr>, _>(
                    specific_result,
                    move |session, resolved| {
                        if resolved != specific_expr {
                            return Ok(Some(resolved));
                        }

                        let general_expr = Expr::call(system_expr("Default"), vec![head]);
                        let general_result = session.evaluate(general_expr.clone());

                        session.drive_evaluated_value::<Expr, Option<Expr>, _>(
                            general_result,
                            move |_, resolved| Ok((resolved != general_expr).then_some(resolved)),
                        )
                    },
                )
            }
        }
    }

    fn match_compiled_sequence_choices(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        consume: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let min_rest = min_required_matcher_arg_count(&rest_patterns);

        if value_args.len() < spec.min_len + min_rest {
            return Ok(match_outcome(false, bindings));
        }

        let max_len = spec
            .max_len
            .map(|limit| limit.min(value_args.len().saturating_sub(min_rest)))
            .unwrap_or_else(|| value_args.len().saturating_sub(min_rest));
        if consume > max_len {
            return Ok(match_outcome(false, bindings));
        }

        let snapshot = bindings.clone();
        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let spec_for_retry = spec.clone();
        let rest_patterns_for_retry = rest_patterns.clone();
        let value_args_for_retry = value_args.clone();
        let snapshot_for_retry = snapshot.clone();
        let rest_patterns_for_match = rest_patterns.clone();
        let rest_values_for_match = rest_values.clone();

        let sequence_result =
            self.match_compiled_sequence_pattern_owned(spec, consumed_values, bindings);
        self.drive_internal_value(sequence_result, move |session, sequence_outcome| {
            if !sequence_outcome.matched {
                return session.match_compiled_sequence_choices(
                    spec_for_retry,
                    rest_patterns_for_retry,
                    value_args_for_retry,
                    snapshot_for_retry,
                    consume + 1,
                );
            }

            let spec_for_retry = spec_for_retry.clone();
            let rest_patterns_for_retry = rest_patterns_for_retry.clone();
            let value_args_for_retry = value_args_for_retry.clone();
            let snapshot_for_retry = snapshot_for_retry.clone();

            let rest_result = session.match_compiled_args_owned(
                rest_patterns_for_match,
                rest_values_for_match,
                sequence_outcome.bindings,
            );
            session.drive_internal_value(rest_result, move |session, rest_outcome| {
                if rest_outcome.matched {
                    Ok(rest_outcome)
                } else {
                    session.match_compiled_sequence_choices(
                        spec_for_retry,
                        rest_patterns_for_retry,
                        value_args_for_retry,
                        snapshot_for_retry,
                        consume + 1,
                    )
                }
            })
        })
    }

    fn match_compiled_sequence_items_owned(
        &mut self,
        matcher: MatcherNode,
        values: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(value) = values.get(index).cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        let bindings_for_next = bindings.clone();
        let matcher_for_next = matcher.clone();
        let values_for_next = values.clone();
        let item_result = self.match_compiled_owned(matcher, value, bindings);
        self.drive_internal_value(item_result, move |session, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            session.match_compiled_sequence_items_owned(
                matcher_for_next,
                values_for_next,
                bindings_for_next,
                index + 1,
            )
        })
    }

    fn match_compiled_sequence_pattern_owned(
        &mut self,
        spec: CompiledSequencePattern,
        values: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if values.len() < spec.min_len {
            return Ok(match_outcome(false, snapshot));
        }

        if spec.max_len.is_some_and(|max_len| values.len() > max_len) {
            return Ok(match_outcome(false, snapshot));
        }

        let matcher_result = self.match_compiled_sequence_items_owned(
            spec.matcher.clone(),
            values.clone(),
            bindings,
            0,
        );
        self.drive_internal_value(matcher_result, move |session, outcome| {
            if !outcome.matched {
                return Ok(match_outcome(false, snapshot));
            }

            let mut bindings = outcome.bindings;
            let sequence_value = sequence_expr(values.clone());

            if let Some(name) = &spec.name {
                if !bind_pattern_name(name, sequence_value.clone(), &mut bindings) {
                    return Ok(match_outcome(false, snapshot));
                }
            }

            if !spec.checks.is_empty() {
                return session.evaluate_sequence_pattern_checks_owned(
                    spec.checks,
                    sequence_value,
                    bindings,
                    snapshot,
                    0,
                );
            }

            Ok(match_outcome(true, bindings))
        })
    }
}
