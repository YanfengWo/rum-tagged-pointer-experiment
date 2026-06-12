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
