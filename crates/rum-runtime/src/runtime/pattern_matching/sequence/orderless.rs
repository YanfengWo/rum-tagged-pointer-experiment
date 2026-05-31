use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_arg_choices_owned(
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
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false);
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

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_arg_choices_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.match_compiled_orderless_flat_optional_choices(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, true);
                self.match_compiled_orderless_flat_choices_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_arg_choice_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.collect_compiled_orderless_optional_matches_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false);
                self.collect_compiled_orderless_choices_matches_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                    Vec::new(),
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_flat_arg_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self
                .collect_compiled_orderless_flat_optional_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, true);
                self.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                    Vec::new(),
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_optional_choices(
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

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_optional_choices(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if !value_args.is_empty() {
            let present_result = self.match_compiled_orderless_flat_arg_choices_owned(
                flat_head.clone(),
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings,
            );

            return self.drive_internal_value(present_result, move |session, outcome| {
                if outcome.matched {
                    Ok(outcome)
                } else {
                    session.match_compiled_orderless_flat_optional_omission_owned(
                        flat_head,
                        spec,
                        rest_patterns,
                        value_args,
                        snapshot,
                    )
                }
            });
        }

        self.match_compiled_orderless_flat_optional_omission_owned(
            flat_head,
            spec,
            rest_patterns,
            value_args,
            snapshot,
        )
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_optional_matches_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let present_result = if value_args.is_empty() {
            Ok(Vec::new())
        } else {
            self.collect_compiled_orderless_arg_choice_matches_owned(
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings.clone(),
            )
        };

        self.drive_internal_value(present_result, move |session, mut present| {
            let omission_result = session.match_compiled_orderless_optional_omission_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            );

            session.drive_internal_value(omission_result, move |_, outcome| {
                if outcome.matched {
                    present.push(outcome.bindings);
                }
                Ok(present)
            })
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_flat_optional_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let present_result = if value_args.is_empty() {
            Ok(Vec::new())
        } else {
            self.collect_compiled_orderless_flat_arg_choice_matches_owned(
                flat_head.clone(),
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings.clone(),
            )
        };

        self.drive_internal_value(present_result, move |session, mut present| {
            let omission_result = session.match_compiled_orderless_flat_optional_omission_owned(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            );

            session.drive_internal_value(omission_result, move |_, outcome| {
                if outcome.matched {
                    present.push(outcome.bindings);
                }
                Ok(present)
            })
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_optional_omission_owned(
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

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_optional_omission_owned(
        &mut self,
        flat_head: Expr,
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

            let default_result = session.match_compiled_orderless_flat_selected_values_owned(
                flat_head.clone(),
                *spec.inner,
                optional_default_values(default_expr),
                bindings,
            );

            session.drive_internal_value(default_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                session.match_compiled_orderless_args_with_flat_head_owned(
                    rest_patterns,
                    value_args,
                    outcome.bindings,
                    flat_head,
                )
            })
        })
    }

    fn collect_compiled_orderless_choices_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let Some(choice) = choices.get(index).cloned() else {
            return Ok(collected);
        };

        let selected_values = choice.selected_values;
        let rest_values = choice.rest_values;
        let match_result = self.match_compiled_orderless_selected_values_owned(
            pattern.clone(),
            selected_values,
            bindings.clone(),
        );
        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_orderless_choices_matches_owned(
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                );
            }

            let rest_result = session.collect_compiled_orderless_arg_matches_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            );
            session.drive_internal_value(rest_result, move |session, mut rest_matches| {
                let mut collected = collected;
                collected.append(&mut rest_matches);
                session.collect_compiled_orderless_choices_matches_owned(
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                )
            })
        })
    }

    fn collect_compiled_orderless_flat_choices_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let Some(choice) = choices.get(index).cloned() else {
            return Ok(collected);
        };

        let selected_values = choice.selected_values;
        let rest_values = choice.rest_values;
        let match_result = self.match_compiled_orderless_flat_selected_values_owned(
            flat_head.clone(),
            pattern.clone(),
            selected_values,
            bindings.clone(),
        );
        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                );
            }

            let rest_result = session.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head.clone(),
            );
            session.drive_internal_value(rest_result, move |session, mut rest_matches| {
                let mut collected = collected;
                collected.append(&mut rest_matches);
                session.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
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

    fn match_compiled_orderless_flat_choices_owned(
        &mut self,
        flat_head: Expr,
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

        let match_result = self.match_compiled_orderless_flat_selected_values_owned(
            flat_head.clone(),
            pattern,
            selected_values,
            bindings,
        );
        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.match_compiled_orderless_flat_choices_owned(
                    flat_head,
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

            let rest_result = session.match_compiled_orderless_args_with_flat_head_owned(
                rest_patterns_for_match,
                rest_values_for_match,
                outcome.bindings,
                flat_head.clone(),
            );
            session.drive_internal_value(rest_result, move |session, rest_outcome| {
                if rest_outcome.matched {
                    Ok(rest_outcome)
                } else {
                    session.match_compiled_orderless_flat_choices_owned(
                        flat_head,
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

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_selected_values_owned(
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

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_selected_values_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        selected_values: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Single(pattern) => match selected_values.as_slice() {
                [] => Ok(match_outcome(false, bindings)),
                [value] => self.match_compiled_owned(pattern, value.clone(), bindings),
                _ => self.match_compiled_owned(
                    pattern,
                    flat_match_value(&flat_head, selected_values),
                    bindings,
                ),
            },
            MatcherArg::Sequence(spec) => {
                self.match_compiled_sequence_pattern_owned(spec, selected_values, bindings)
            }
            MatcherArg::Optional(spec) => {
                if selected_values.is_empty() {
                    self.match_compiled_orderless_flat_optional_omission_owned(
                        flat_head,
                        spec,
                        Vec::new(),
                        Vec::new(),
                        bindings,
                    )
                } else {
                    self.match_compiled_orderless_flat_selected_values_owned(
                        flat_head,
                        *spec.inner,
                        selected_values,
                        bindings,
                    )
                }
            }
            MatcherArg::PatternSequenceGroup(items) => self
                .match_compiled_args_with_flat_head_owned(
                    items,
                    selected_values,
                    bindings,
                    flat_head,
                ),
            MatcherArg::OrderlessPatternSequenceGroup(items) => self
                .match_compiled_orderless_args_with_flat_head_owned(
                    items,
                    selected_values,
                    bindings,
                    flat_head,
                ),
        }
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
    allow_flat_single: bool,
) -> Vec<OrderlessCandidateChoice> {
    match pattern {
        MatcherArg::Single(_) if allow_flat_single => {
            let min_rest = min_required_matcher_arg_count(rest_patterns);
            let Some(max_len) = value_args.len().checked_sub(min_rest) else {
                return Vec::new();
            };
            if max_len == 0 {
                return Vec::new();
            }

            orderless_partition_choices_by_lengths(value_args, 1..=max_len, false)
        }
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
