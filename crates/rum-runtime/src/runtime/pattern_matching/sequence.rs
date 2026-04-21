use super::{outcome::*, *};

fn optional_default_values(expr: Expr) -> Vec<Expr> {
    sequence_items(&expr)
        .map(|items| items.to_vec())
        .unwrap_or_else(|| vec![expr])
}

impl Session {
    pub(in crate::runtime) fn match_compiled_args_owned(
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

    pub(in crate::runtime) fn match_compiled_orderless_args_owned(
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

        self.match_compiled_orderless_arg_choices_owned(first, rest_patterns, value_args, bindings)
    }

    pub(super) fn match_compiled_arg_and_rest_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let Some(start) = initial_sequence_consume(&spec, &rest_patterns, &value_args)
                else {
                    return Ok(match_outcome(false, bindings));
                };

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
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.match_compiled_args_owned(expanded, value_args, bindings)
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

    fn match_compiled_orderless_arg_choices_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.match_compiled_orderless_optional_choices(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices = orderless_candidate_choices(&other, &rest_patterns, &value_args);
                self.match_compiled_orderless_choices_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    fn match_compiled_orderless_optional_choices(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if !value_args.is_empty() {
            let present_result = self.match_compiled_orderless_arg_choices_owned(
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings,
            );

            return self.drive_internal_value(present_result, move |session, outcome| {
                if outcome.matched {
                    Ok(outcome)
                } else {
                    session.match_compiled_orderless_optional_omission_owned(
                        spec,
                        rest_patterns,
                        value_args,
                        snapshot,
                    )
                }
            });
        }

        self.match_compiled_orderless_optional_omission_owned(
            spec,
            rest_patterns,
            value_args,
            snapshot,
        )
    }

    fn match_compiled_orderless_optional_omission_owned(
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

            let default_result = session.match_compiled_orderless_selected_values_owned(
                *spec.inner,
                optional_default_values(default_expr),
                bindings,
            );

            session.drive_internal_value(default_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                session.match_compiled_orderless_args_owned(
                    rest_patterns,
                    value_args,
                    outcome.bindings,
                )
            })
        })
    }

    fn match_compiled_orderless_choices_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(choice) = choices.get(index).cloned() else {
            return Ok(match_outcome(false, bindings));
        };

        let snapshot = bindings.clone();
        let choices_for_retry = choices.clone();
        let pattern_for_retry = pattern.clone();
        let rest_patterns_for_retry = rest_patterns.clone();
        let selected_values = choice.selected_values;
        let rest_values = choice.rest_values;
        let rest_patterns_for_match = rest_patterns.clone();
        let rest_values_for_match = rest_values.clone();

        let match_result =
            self.match_compiled_orderless_selected_values_owned(pattern, selected_values, bindings);
        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.match_compiled_orderless_choices_owned(
                    pattern_for_retry,
                    rest_patterns_for_retry,
                    choices_for_retry,
                    snapshot,
                    index + 1,
                );
            }

            let snapshot_for_retry = snapshot.clone();
            let choices_for_retry = choices_for_retry.clone();
            let pattern_for_retry = pattern_for_retry.clone();
            let rest_patterns_for_retry = rest_patterns_for_retry.clone();

            let rest_result = session.match_compiled_orderless_args_owned(
                rest_patterns_for_match,
                rest_values_for_match,
                outcome.bindings,
            );
            session.drive_internal_value(rest_result, move |session, rest_outcome| {
                if rest_outcome.matched {
                    Ok(rest_outcome)
                } else {
                    session.match_compiled_orderless_choices_owned(
                        pattern_for_retry,
                        rest_patterns_for_retry,
                        choices_for_retry,
                        snapshot_for_retry,
                        index + 1,
                    )
                }
            })
        })
    }

    fn match_compiled_orderless_selected_values_owned(
        &mut self,
        pattern: MatcherArg,
        selected_values: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Single(pattern) => match selected_values.as_slice() {
                [value] => self.match_compiled_owned(pattern, value.clone(), bindings),
                _ => Ok(match_outcome(false, bindings)),
            },
            MatcherArg::Sequence(spec) => {
                self.match_compiled_sequence_pattern_owned(spec, selected_values, bindings)
            }
            MatcherArg::Optional(spec) => {
                if selected_values.is_empty() {
                    self.match_compiled_orderless_optional_omission_owned(
                        spec,
                        Vec::new(),
                        Vec::new(),
                        bindings,
                    )
                } else {
                    self.match_compiled_orderless_selected_values_owned(
                        *spec.inner,
                        selected_values,
                        bindings,
                    )
                }
            }
            MatcherArg::PatternSequenceGroup(items) => {
                self.match_compiled_args_owned(items, selected_values, bindings)
            }
            MatcherArg::OrderlessPatternSequenceGroup(items) => {
                self.match_compiled_orderless_args_owned(items, selected_values, bindings)
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
        let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
            return Ok(match_outcome(false, bindings));
        };
        if !(spec.min_len..=max_len).contains(&consume) {
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
                let Some(next_consume) = next_sequence_consume(
                    spec_for_retry.greediness,
                    consume,
                    spec_for_retry.min_len,
                    max_len,
                ) else {
                    return Ok(match_outcome(false, snapshot_for_retry));
                };

                return session.match_compiled_sequence_choices(
                    spec_for_retry,
                    rest_patterns_for_retry,
                    value_args_for_retry,
                    snapshot_for_retry,
                    next_consume,
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
                    let Some(next_consume) = next_sequence_consume(
                        spec_for_retry.greediness,
                        consume,
                        spec_for_retry.min_len,
                        max_len,
                    ) else {
                        return Ok(match_outcome(false, snapshot_for_retry));
                    };

                    session.match_compiled_sequence_choices(
                        spec_for_retry,
                        rest_patterns_for_retry,
                        value_args_for_retry,
                        snapshot_for_retry,
                        next_consume,
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

fn initial_sequence_consume(
    spec: &CompiledSequencePattern,
    rest_patterns: &[MatcherArg],
    value_args: &[Expr],
) -> Option<usize> {
    let min_rest = min_required_matcher_arg_count(rest_patterns);
    let max_len = max_sequence_consume(spec, min_rest, value_args.len())?;

    Some(match spec.greediness {
        SequenceGreediness::Shortest => spec.min_len,
        SequenceGreediness::Longest => max_len,
    })
}

fn max_sequence_consume(
    spec: &CompiledSequencePattern,
    min_rest: usize,
    value_len: usize,
) -> Option<usize> {
    if value_len < spec.min_len + min_rest {
        return None;
    }

    Some(
        spec.max_len
            .map(|limit| limit.min(value_len.saturating_sub(min_rest)))
            .unwrap_or_else(|| value_len.saturating_sub(min_rest)),
    )
}

fn next_sequence_consume(
    greediness: SequenceGreediness,
    current: usize,
    min_len: usize,
    max_len: usize,
) -> Option<usize> {
    match greediness {
        SequenceGreediness::Shortest => (current < max_len).then_some(current + 1),
        SequenceGreediness::Longest => (current > min_len).then_some(current - 1),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OrderlessCandidateChoice {
    selected_values: Vec<Expr>,
    rest_values: Vec<Expr>,
}

fn orderless_candidate_choices(
    pattern: &MatcherArg,
    rest_patterns: &[MatcherArg],
    value_args: &[Expr],
) -> Vec<OrderlessCandidateChoice> {
    match pattern {
        MatcherArg::Single(_) => select_orderless_value_partitions(value_args, 1),
        MatcherArg::Sequence(spec) => {
            let Some(max_len) = max_sequence_consume(
                spec,
                min_required_matcher_arg_count(rest_patterns),
                value_args.len(),
            ) else {
                return Vec::new();
            };

            orderless_partition_choices_by_lengths(
                value_args,
                sequence_candidate_lengths(spec.min_len, max_len, spec.greediness),
                false,
            )
        }
        MatcherArg::Optional(_) => Vec::new(),
        MatcherArg::PatternSequenceGroup(items) => {
            let min_len = min_required_matcher_arg_count(items);
            let max_len = max_possible_matcher_arg_count(items)
                .map(|max_len| max_len.min(value_args.len()))
                .unwrap_or(value_args.len());

            if min_len > max_len {
                return Vec::new();
            }

            orderless_partition_choices_by_lengths(value_args, min_len..=max_len, true)
        }
        MatcherArg::OrderlessPatternSequenceGroup(items) => {
            let min_len = min_required_matcher_arg_count(items);
            let max_len = max_possible_matcher_arg_count(items)
                .map(|max_len| max_len.min(value_args.len()))
                .unwrap_or(value_args.len());

            if min_len > max_len {
                return Vec::new();
            }

            orderless_partition_choices_by_lengths(value_args, min_len..=max_len, false)
        }
    }
}

fn orderless_partition_choices_by_lengths<I>(
    value_args: &[Expr],
    lengths: I,
    reverse_within_len: bool,
) -> Vec<OrderlessCandidateChoice>
where
    I: IntoIterator<Item = usize>,
{
    lengths
        .into_iter()
        .flat_map(|count| {
            let mut choices = select_orderless_value_partitions(value_args, count);
            if reverse_within_len {
                choices.reverse();
            }
            choices
        })
        .collect()
}

fn sequence_candidate_lengths(
    min_len: usize,
    max_len: usize,
    greediness: SequenceGreediness,
) -> Vec<usize> {
    match greediness {
        SequenceGreediness::Shortest => (min_len..=max_len).collect(),
        SequenceGreediness::Longest => (min_len..=max_len).rev().collect(),
    }
}

fn select_orderless_value_partitions(
    value_args: &[Expr],
    choose: usize,
) -> Vec<OrderlessCandidateChoice> {
    if choose > value_args.len() {
        return Vec::new();
    }

    let mut indices = Vec::new();
    let mut partitions = Vec::new();
    collect_orderless_partitions(value_args, choose, 0, &mut indices, &mut partitions);
    partitions
}

fn collect_orderless_partitions(
    value_args: &[Expr],
    choose: usize,
    start: usize,
    indices: &mut Vec<usize>,
    partitions: &mut Vec<OrderlessCandidateChoice>,
) {
    if indices.len() == choose {
        partitions.push(build_orderless_partition(value_args, indices));
        return;
    }

    let remaining = choose - indices.len();
    for index in start..=value_args.len() - remaining {
        indices.push(index);
        collect_orderless_partitions(value_args, choose, index + 1, indices, partitions);
        indices.pop();
    }
}

fn build_orderless_partition(value_args: &[Expr], indices: &[usize]) -> OrderlessCandidateChoice {
    let mut selected_values = Vec::with_capacity(indices.len());
    let mut rest_values = Vec::with_capacity(value_args.len().saturating_sub(indices.len()));
    let mut next_index = 0usize;

    for (index, value) in value_args.iter().enumerate() {
        if indices.get(next_index).copied() == Some(index) {
            selected_values.push(value.clone());
            next_index += 1;
        } else {
            rest_values.push(value.clone());
        }
    }

    OrderlessCandidateChoice {
        selected_values,
        rest_values,
    }
}
