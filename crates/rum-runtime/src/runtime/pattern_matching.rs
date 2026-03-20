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
                        [MatcherArg::Single(inner)] => {
                            self.match_compiled_owned(inner.clone(), other, bindings)
                        }
                        _ => Ok(match_outcome(false, bindings)),
                    },
                    _ => Ok(match_outcome(false, bindings)),
                }
            }
        }
    }

    fn match_compiled_args_owned(
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

        match first {
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

        let max_len = value_args.len() - min_rest;
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

        if !values
            .iter()
            .all(|value| match_blank(spec.head_constraint.as_ref(), value))
        {
            return Ok(match_outcome(false, snapshot));
        }

        let mut bindings = bindings;

        if let Some(name) = &spec.name {
            let bound_value = sequence_expr(values);

            if !bind_pattern_name(name, bound_value, &mut bindings) {
                return Ok(match_outcome(false, snapshot));
            }
        }

        if let Some(condition) = &spec.condition {
            let bindings_for_true = bindings.clone();
            let snapshot_for_false = snapshot.clone();
            let condition_expr = substitute_bindings(condition, &bindings);

            let condition_result = self.evaluate(condition_expr);
            return self.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
                condition_result,
                move |_, result| {
                    Ok(if is_system_symbol_name(&result, "True") {
                        match_outcome(true, bindings_for_true)
                    } else {
                        match_outcome(false, snapshot_for_false)
                    })
                },
            );
        }

        Ok(match_outcome(true, bindings))
    }
}
