use super::candidates::OrderlessCandidateChoice;
use super::*;

const MAX_MATCH_DEPTH: usize = 512;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence::orderless) fn with_match_depth_guard<T>(
        &mut self,
        body: impl FnOnce(&mut Session) -> Result<T, EvalError>,
    ) -> Result<T, EvalError> {
        if self.match_depth >= MAX_MATCH_DEPTH {
            return Err(EvalError::MatchRecursionLimitExceeded {
                limit: MAX_MATCH_DEPTH,
            });
        }

        self.match_depth += 1;
        let result = body(self);
        self.match_depth -= 1;
        result
    }

    pub(super) fn match_compiled_orderless_choices_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.with_match_depth_guard(move |session| {
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

            let match_result = session.match_compiled_orderless_selected_values_owned(
                pattern,
                selected_values,
                bindings,
            );
            session.drive_internal_value(match_result, move |session, outcome| {
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
        })
    }

    pub(super) fn match_compiled_orderless_flat_choices_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.with_match_depth_guard(move |session| {
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

            let match_result = session.match_compiled_orderless_flat_selected_values_owned(
                flat_head.clone(),
                pattern,
                selected_values,
                bindings,
            );
            session.drive_internal_value(match_result, move |session, outcome| {
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
        })
    }
}
