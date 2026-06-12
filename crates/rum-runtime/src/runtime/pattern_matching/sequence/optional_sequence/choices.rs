use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_sequence_choices(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
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

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_flat_sequence_choices(
        &mut self,
        flat_head: Expr,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
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
        let flat_head_for_retry = flat_head.clone();
        let flat_head_for_match = flat_head.clone();
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

                return session.match_compiled_flat_sequence_choices(
                    flat_head_for_retry,
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
            let flat_head_for_retry = flat_head_for_retry.clone();

            let rest_result = session.match_compiled_args_with_flat_head_owned(
                rest_patterns_for_match,
                rest_values_for_match,
                sequence_outcome.bindings,
                flat_head_for_match,
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

                    session.match_compiled_flat_sequence_choices(
                        flat_head_for_retry,
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
}
