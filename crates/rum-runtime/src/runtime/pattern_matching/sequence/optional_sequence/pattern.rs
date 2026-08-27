use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_sequence_items_owned(
        &mut self,
        matcher: MatcherNode,
        values: Vec<Expr>,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(value) = values.get(index).cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        let bindings_for_next = bindings.clone();
        let matcher_for_next = matcher.clone();
        let values_for_next = values.clone();
        let item_result = self.match_compiled_owned(matcher, value, bindings);
        match item_result {
            Ok(outcome) => self.finish_compiled_sequence_item(
                matcher_for_next,
                values_for_next,
                bindings_for_next,
                index,
                outcome,
            ),
            Err(EvalError::InputRequired { request }) => {
                let matcher = FrozenMatcherValue::freeze(matcher_for_next);
                let values = SharedArgs::new(values_for_next);
                let bindings = FrozenMatchBindings::freeze(bindings_for_next);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_compiled_sequence_item(
                            matcher.into_inner(),
                            values.into_vec(),
                            bindings.into_owned(),
                            index,
                            outcome,
                        )
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_compiled_sequence_item(
        &mut self,
        matcher: MatcherNode,
        values: Vec<Expr>,
        bindings: SymbolExprMap,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        self.match_compiled_sequence_items_owned(matcher, values, bindings, index + 1)
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_sequence_pattern_owned(
        &mut self,
        spec: CompiledSequencePattern,
        values: Vec<Expr>,
        bindings: SymbolExprMap,
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
        match matcher_result {
            Ok(outcome) => self.finish_compiled_sequence_pattern(spec, values, snapshot, outcome),
            Err(EvalError::InputRequired { request }) => {
                let spec = FrozenMatcherValue::freeze(spec);
                let values = SharedArgs::new(values);
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_compiled_sequence_pattern(
                            spec.into_inner(),
                            values.into_vec(),
                            snapshot.into_owned(),
                            outcome,
                        )
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_compiled_sequence_pattern(
        &mut self,
        spec: CompiledSequencePattern,
        values: Vec<Expr>,
        snapshot: SymbolExprMap,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(match_outcome(false, snapshot));
        }

        let mut bindings = outcome.bindings;
        let sequence_value = sequence_expr(values);

        if let Some(name) = &spec.name {
            if !bind_pattern_name(name, sequence_value.clone(), &mut bindings) {
                return Ok(match_outcome(false, snapshot));
            }
        }

        if !spec.checks.is_empty() {
            return self.evaluate_sequence_pattern_checks_owned(
                spec.checks,
                sequence_value,
                bindings,
                snapshot,
                0,
            );
        }

        Ok(match_outcome(true, bindings))
    }
}
