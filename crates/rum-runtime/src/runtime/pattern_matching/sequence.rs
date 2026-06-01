use super::{outcome::*, *};

mod entry;
mod optional_sequence;
mod orderless;

fn optional_default_values(expr: Expr) -> Vec<Expr> {
    sequence_items(&expr)
        .map(|items| items.to_vec())
        .unwrap_or_else(|| vec![expr])
}

impl Session {
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

    fn match_compiled_flat_arg_and_rest_owned(
        &mut self,
        flat_head: Expr,
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

                self.match_compiled_flat_sequence_choices(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    start,
                )
            }
            MatcherArg::Optional(spec) => self.match_compiled_flat_optional_choices(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.match_compiled_args_with_flat_head_owned(
                    expanded, value_args, bindings, flat_head,
                )
            }
            MatcherArg::Single(pattern) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_consume) = value_args.len().checked_sub(min_rest) else {
                    return Ok(match_outcome(false, bindings));
                };
                if max_consume == 0 {
                    return Ok(match_outcome(false, bindings));
                }

                self.match_compiled_flat_single_choices_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    value_args,
                    bindings,
                    1,
                    max_consume,
                )
            }
        }
    }

    fn match_compiled_flat_single_choices_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherNode,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        consume: usize,
        max_consume: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let selected_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let grouped_value = flat_match_value(&flat_head, selected_values);
        let pattern_result =
            self.match_compiled_owned(pattern.clone(), grouped_value, bindings.clone());

        self.drive_internal_value(pattern_result, move |session, outcome| {
            if !outcome.matched {
                return if consume < max_consume {
                    session.match_compiled_flat_single_choices_owned(
                        flat_head,
                        pattern,
                        rest_patterns,
                        value_args,
                        outcome.bindings,
                        consume + 1,
                        max_consume,
                    )
                } else {
                    Ok(match_outcome(false, outcome.bindings))
                };
            }

            let rest_result = session.match_compiled_args_with_flat_head_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head.clone(),
            );
            session.drive_internal_value(rest_result, move |session, rest_outcome| {
                if rest_outcome.matched {
                    Ok(rest_outcome)
                } else if consume < max_consume {
                    session.match_compiled_flat_single_choices_owned(
                        flat_head,
                        pattern,
                        rest_patterns,
                        value_args,
                        bindings,
                        consume + 1,
                        max_consume,
                    )
                } else {
                    Ok(match_outcome(false, bindings))
                }
            })
        })
    }

    fn collect_compiled_arg_and_rest_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
                    return Ok(Vec::new());
                };
                let consumes = sequence_candidate_lengths(spec.min_len, max_len, spec.greediness);

                self.collect_compiled_sequence_choice_matches_owned(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    0,
                    Vec::new(),
                )
            }
            MatcherArg::Optional(spec) => self.collect_compiled_optional_matches_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.collect_compiled_args_matches_owned(expanded, value_args, bindings)
            }
            MatcherArg::Single(pattern) => {
                let mut value_iter = value_args.into_iter();
                let Some(value) = value_iter.next() else {
                    return Ok(Vec::new());
                };
                let rest_values = value_iter.collect::<Vec<_>>();

                let pattern_result = self.match_compiled_owned(pattern, value, bindings);
                self.drive_internal_value(pattern_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(Vec::new());
                    }

                    session.collect_compiled_args_matches_owned(
                        rest_patterns,
                        rest_values,
                        outcome.bindings,
                    )
                })
            }
        }
    }

    fn collect_compiled_flat_arg_and_rest_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
                    return Ok(Vec::new());
                };
                let consumes = sequence_candidate_lengths(spec.min_len, max_len, spec.greediness);

                self.collect_compiled_flat_sequence_choice_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    0,
                    Vec::new(),
                )
            }
            MatcherArg::Optional(spec) => self.collect_compiled_flat_optional_matches_owned(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.collect_compiled_args_matches_with_flat_head_owned(
                    expanded, value_args, bindings, flat_head,
                )
            }
            MatcherArg::Single(pattern) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_consume) = value_args.len().checked_sub(min_rest) else {
                    return Ok(Vec::new());
                };
                if max_consume == 0 {
                    return Ok(Vec::new());
                }

                self.collect_compiled_flat_single_choice_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    value_args,
                    bindings,
                    1,
                    max_consume,
                    Vec::new(),
                )
            }
        }
    }

    fn collect_compiled_flat_single_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherNode,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        consume: usize,
        max_consume: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        if consume > max_consume {
            return Ok(collected);
        }

        let selected_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let grouped_value = flat_match_value(&flat_head, selected_values);
        let pattern_result =
            self.match_compiled_owned(pattern.clone(), grouped_value, bindings.clone());

        self.drive_internal_value(pattern_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_flat_single_choice_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    value_args,
                    bindings,
                    consume + 1,
                    max_consume,
                    collected,
                );
            }

            let rest_result = session.collect_compiled_args_matches_with_flat_head_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head.clone(),
            );
            session.drive_internal_value(rest_result, move |session, mut rest_matches| {
                let mut collected = collected;
                collected.append(&mut rest_matches);
                session.collect_compiled_flat_single_choice_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    value_args,
                    bindings,
                    consume + 1,
                    max_consume,
                    collected,
                )
            })
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
fn flat_match_value(head: &Expr, values: Vec<Expr>) -> Expr {
    match values.as_slice() {
        [single] => single.clone(),
        _ => Expr::call(head.clone(), values),
    }
}
