use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct SequenceChoiceState {
    flat_head: Option<Expr>,
    spec: CompiledSequencePattern,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    snapshot: SymbolExprMap,
    consume: usize,
    max_len: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSequenceChoiceState {
    flat_head: Option<Expr>,
    spec: FrozenMatcherValue<CompiledSequencePattern>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    snapshot: FrozenMatchBindings,
    consume: usize,
    max_len: usize,
}

impl SuspendedSequenceChoiceState {
    fn freeze(state: SequenceChoiceState) -> Self {
        Self {
            flat_head: state.flat_head,
            spec: FrozenMatcherValue::freeze(state.spec),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            snapshot: FrozenMatchBindings::freeze(state.snapshot),
            consume: state.consume,
            max_len: state.max_len,
        }
    }

    fn into_owned(self) -> SequenceChoiceState {
        SequenceChoiceState {
            flat_head: self.flat_head,
            spec: self.spec.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            snapshot: self.snapshot.into_owned(),
            consume: self.consume,
            max_len: self.max_len,
        }
    }
}

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

        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let state = SequenceChoiceState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            snapshot: bindings.clone(),
            consume,
            max_len,
        };
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            state.spec.clone(),
            consumed_values,
            bindings,
        );
        self.drive_sequence_choice_result(sequence_result, state, rest_values)
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

        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let state = SequenceChoiceState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            snapshot: bindings.clone(),
            consume,
            max_len,
        };
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            state.spec.clone(),
            consumed_values,
            bindings,
        );
        self.drive_sequence_choice_result(sequence_result, state, rest_values)
    }

    fn drive_sequence_choice_result(
        &mut self,
        result: Result<CompiledMatchOutcome, EvalError>,
        state: SequenceChoiceState,
        rest_values: Vec<Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match result {
            Ok(outcome) => self.finish_sequence_choice(state, rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceState::freeze(state);
                let rest_values = SharedArgs::new(rest_values);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_sequence_choice(state.into_owned(), rest_values.into_vec(), outcome)
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice(
        &mut self,
        state: SequenceChoiceState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return self.retry_sequence_choice(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_args_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_args_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(outcome) => self.finish_sequence_choice_rest(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceState::freeze(state);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_sequence_choice_rest(state.into_owned(), outcome)
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice_rest(
        &mut self,
        state: SequenceChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.retry_sequence_choice(state)
        }
    }

    fn retry_sequence_choice(
        &mut self,
        state: SequenceChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(next_consume) = next_sequence_consume(
            state.spec.greediness,
            state.consume,
            state.spec.min_len,
            state.max_len,
        ) else {
            return Ok(match_outcome(false, state.snapshot));
        };

        if let Some(flat_head) = state.flat_head {
            self.match_compiled_flat_sequence_choices(
                flat_head,
                state.spec,
                state.rest_patterns,
                state.value_args,
                state.snapshot,
                next_consume,
            )
        } else {
            self.match_compiled_sequence_choices(
                state.spec,
                state.rest_patterns,
                state.value_args,
                state.snapshot,
                next_consume,
            )
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedSequenceChoiceState>();
};
