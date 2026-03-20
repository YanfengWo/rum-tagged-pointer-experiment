use super::*;

fn interpreted_match_outcome(
    matched: bool,
    bindings: BTreeMap<Symbol, Expr>,
) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}

impl Session {
    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let outcome =
            self.match_expr_interpreted_owned(pattern.clone(), value.clone(), bindings.clone())?;
        *bindings = outcome.bindings;
        Ok(outcome.matched)
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let normalized_pattern = self.normalize_match_expr(&pattern);
        let normalized_value = self.normalize_match_expr(&value);
        let snapshot = bindings.clone();
        let outcome = self.match_expr_interpreted_inner_owned(
            normalized_pattern,
            normalized_value,
            bindings,
        )?;

        if outcome.matched {
            Ok(outcome)
        } else {
            Ok(interpreted_match_outcome(false, snapshot))
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_inner_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(&pattern) {
            let name = name.clone();
            let inner = inner.clone();
            let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
            return self.drive_internal_value(inner_result, move |_, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let mut bindings = outcome.bindings;
                Ok(interpreted_match_outcome(
                    bind_pattern_name(&name, value.clone(), &mut bindings),
                    bindings,
                ))
            });
        }

        if let Some((inner, condition)) = pattern_condition(&pattern) {
            let inner = inner.clone();
            let condition = condition.clone();
            let inner_result = self.match_expr_interpreted_owned(inner, value, bindings);
            return self.drive_internal_value(inner_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let snapshot = outcome.bindings.clone();
                let bindings = outcome.bindings;
                let condition_expr = substitute_bindings(&condition, &bindings);
                let result = session.evaluate(condition_expr);

                session.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
                    result,
                    move |_, result| {
                        Ok(if is_system_symbol_name(&result, "True") {
                            interpreted_match_outcome(true, bindings)
                        } else {
                            interpreted_match_outcome(false, snapshot)
                        })
                    },
                )
            });
        }

        if let Some(blank_head) = pattern_blank_head(&pattern) {
            return Ok(interpreted_match_outcome(
                match_blank(blank_head, &value),
                bindings,
            ));
        }

        match (pattern, value) {
            (Expr::Atom(lhs), Expr::Atom(rhs)) => {
                Ok(interpreted_match_outcome(lhs == rhs, bindings))
            }
            (
                Expr::Call {
                    head: lhs_head,
                    args: lhs_args,
                },
                Expr::Call {
                    head: rhs_head,
                    args: rhs_args,
                },
            ) => {
                let head_result = self.match_expr_interpreted_owned(*lhs_head, *rhs_head, bindings);
                self.drive_internal_value(head_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    session.match_call_args_interpreted_owned(lhs_args, rhs_args, outcome.bindings)
                })
            }
            (Expr::Call { head, args }, other)
                if matcher_head_has_attribute(
                    &MatcherNode::Exact((*head).clone()),
                    self,
                    Attribute::OneIdentity,
                ) =>
            {
                match args.as_slice() {
                    [single] => self.match_expr_interpreted_owned(single.clone(), other, bindings),
                    _ => Ok(interpreted_match_outcome(false, bindings)),
                }
            }
            _ => Ok(interpreted_match_outcome(false, bindings)),
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_call_args_interpreted_owned(
        &mut self,
        pattern_args: Vec<Expr>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(interpreted_match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        if let Some(spec) = decompose_sequence_pattern(&first) {
            let spec = CompiledSequencePattern {
                min_len: spec.min_len,
                head_constraint: spec.head_constraint.cloned(),
                name: spec.name.cloned(),
                condition: spec.condition.cloned(),
            };
            let start = spec.min_len;
            return self.match_sequence_choices_interpreted_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
                start,
            );
        }

        let mut value_iter = value_args.into_iter();
        let Some(value) = value_iter.next() else {
            return Ok(interpreted_match_outcome(false, bindings));
        };
        let rest_values = value_iter.collect::<Vec<_>>();

        let pattern_result = self.match_expr_interpreted_owned(first, value, bindings);
        self.drive_internal_value(pattern_result, move |session, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            session.match_call_args_interpreted_owned(rest_patterns, rest_values, outcome.bindings)
        })
    }

    #[allow(dead_code)]
    pub(super) fn match_sequence_choices_interpreted_owned(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<Expr>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        consume: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let min_rest = min_required_arg_count(&rest_patterns);

        if value_args.len() < spec.min_len + min_rest {
            return Ok(interpreted_match_outcome(false, bindings));
        }

        let max_len = value_args.len() - min_rest;
        if consume > max_len {
            return Ok(interpreted_match_outcome(false, bindings));
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
            self.match_sequence_pattern_interpreted_owned(spec, consumed_values, bindings);
        self.drive_internal_value(sequence_result, move |session, sequence_outcome| {
            if !sequence_outcome.matched {
                return session.match_sequence_choices_interpreted_owned(
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

            let rest_result = session.match_call_args_interpreted_owned(
                rest_patterns_for_match,
                rest_values_for_match,
                sequence_outcome.bindings,
            );
            session.drive_internal_value(rest_result, move |session, rest_outcome| {
                if rest_outcome.matched {
                    Ok(rest_outcome)
                } else {
                    session.match_sequence_choices_interpreted_owned(
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

    #[allow(dead_code)]
    pub(super) fn match_sequence_pattern_interpreted_owned(
        &mut self,
        spec: CompiledSequencePattern,
        values: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if values.len() < spec.min_len {
            return Ok(interpreted_match_outcome(false, snapshot));
        }

        if !values
            .iter()
            .all(|value| match_blank(spec.head_constraint.as_ref(), value))
        {
            return Ok(interpreted_match_outcome(false, snapshot));
        }

        let mut bindings = bindings;

        if let Some(name) = &spec.name {
            let bound_value = sequence_expr(values);

            if !bind_pattern_name(name, bound_value, &mut bindings) {
                return Ok(interpreted_match_outcome(false, snapshot));
            }
        }

        if let Some(condition) = &spec.condition {
            let bindings_for_true = bindings.clone();
            let snapshot_for_false = snapshot.clone();
            let condition_expr = substitute_bindings(condition, &bindings);
            let result = self.evaluate(condition_expr);

            return self.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
                result,
                move |_, result| {
                    Ok(if is_system_symbol_name(&result, "True") {
                        interpreted_match_outcome(true, bindings_for_true)
                    } else {
                        interpreted_match_outcome(false, snapshot_for_false)
                    })
                },
            );
        }

        Ok(interpreted_match_outcome(true, bindings))
    }
}
