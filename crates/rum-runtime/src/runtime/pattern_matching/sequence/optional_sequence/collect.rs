use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_sequence_choice_matches_owned(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        consumes: Vec<usize>,
        index: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let Some(consume) = consumes.get(index).copied() else {
            return Ok(collected);
        };

        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            spec.clone(),
            consumed_values,
            bindings.clone(),
        );
        self.drive_internal_value(sequence_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_sequence_choice_matches_owned(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    index + 1,
                    collected,
                );
            }

            let rest_result = session.collect_compiled_args_matches_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            );
            session.drive_internal_value(rest_result, move |session, mut rest_matches| {
                let mut collected = collected;
                collected.append(&mut rest_matches);
                session.collect_compiled_sequence_choice_matches_owned(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    index + 1,
                    collected,
                )
            })
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_flat_sequence_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        consumes: Vec<usize>,
        index: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let Some(consume) = consumes.get(index).copied() else {
            return Ok(collected);
        };

        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            spec.clone(),
            consumed_values,
            bindings.clone(),
        );
        self.drive_internal_value(sequence_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_flat_sequence_choice_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    index + 1,
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
                session.collect_compiled_flat_sequence_choice_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    index + 1,
                    collected,
                )
            })
        })
    }
}
