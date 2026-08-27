use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct SequenceChoiceCollectionState {
    flat_head: Option<Expr>,
    spec: CompiledSequencePattern,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
    consumes: Vec<usize>,
    index: usize,
    collected: Vec<SymbolExprMap>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSequenceChoiceCollection {
    flat_head: Option<Expr>,
    spec: FrozenMatcherValue<CompiledSequencePattern>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
    consumes: FrozenMatcherSequence<usize>,
    index: usize,
    collected: PersistentMatchResults,
}

impl SuspendedSequenceChoiceCollection {
    fn freeze(state: SequenceChoiceCollectionState) -> Self {
        Self {
            flat_head: state.flat_head,
            spec: FrozenMatcherValue::freeze(state.spec),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            consumes: FrozenMatcherSequence::freeze(state.consumes),
            index: state.index,
            collected: PersistentMatchResults::freeze(state.collected),
        }
    }

    fn into_owned(self) -> SequenceChoiceCollectionState {
        SequenceChoiceCollectionState {
            flat_head: self.flat_head,
            spec: self.spec.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
            consumes: self.consumes.into_vec(),
            index: self.index,
            collected: self.collected.into_vec(),
        }
    }
}

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_sequence_choice_matches_owned(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consumes: Vec<usize>,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_sequence_choice_matches(SequenceChoiceCollectionState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
            consumes,
            index,
            collected,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_flat_sequence_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consumes: Vec<usize>,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_sequence_choice_matches(SequenceChoiceCollectionState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
            consumes,
            index,
            collected,
        })
    }

    fn collect_sequence_choice_matches(
        &mut self,
        state: SequenceChoiceCollectionState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let Some(consume) = state.consumes.get(state.index).copied() else {
            return Ok(state.collected);
        };

        let consumed_values = state
            .value_args
            .iter()
            .take(consume)
            .cloned()
            .collect::<Vec<_>>();
        let rest_values = state
            .value_args
            .iter()
            .skip(consume)
            .cloned()
            .collect::<Vec<_>>();
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            state.spec.clone(),
            consumed_values,
            state.bindings.clone(),
        );
        match sequence_result {
            Ok(outcome) => self.finish_sequence_choice_collection(state, rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceCollection::freeze(state);
                let rest_values = SharedArgs::new(rest_values);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_sequence_choice_collection(
                            state.into_owned(),
                            rest_values.into_vec(),
                            outcome,
                        )
                        .map(|matches| matches.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice_collection(
        &mut self,
        mut state: SequenceChoiceCollectionState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            state.index += 1;
            return self.collect_sequence_choice_matches(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.collect_compiled_args_matches_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_args_matches_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(matches) => self.finish_sequence_choice_rest_collection(state, matches),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceCollection::freeze(state);
                self.push_suspension_frame(move |session, payload| {
                    let matches = Vec::<SymbolExprMap>::from_continuation_expr(payload?);
                    session
                        .finish_sequence_choice_rest_collection(state.into_owned(), matches)
                        .map(|matches| matches.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice_rest_collection(
        &mut self,
        mut state: SequenceChoiceCollectionState,
        mut matches: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        state.collected.append(&mut matches);
        state.index += 1;
        self.collect_sequence_choice_matches(state)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedSequenceChoiceCollection>();
};
