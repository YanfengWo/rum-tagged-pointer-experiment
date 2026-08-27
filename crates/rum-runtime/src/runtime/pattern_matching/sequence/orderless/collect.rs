use super::candidates::OrderlessCandidateChoice;
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct OrderlessChoiceCollectionState {
    flat_head: Option<Expr>,
    pattern: MatcherArg,
    rest_patterns: Vec<MatcherArg>,
    choices: Vec<OrderlessCandidateChoice>,
    bindings: SymbolExprMap,
    index: usize,
    collected: Vec<SymbolExprMap>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOrderlessChoiceCollection {
    flat_head: Option<Expr>,
    pattern: FrozenMatcherValue<MatcherArg>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    choices: FrozenMatcherSequence<OrderlessCandidateChoice>,
    bindings: FrozenMatchBindings,
    index: usize,
    collected: PersistentMatchResults,
}

impl SuspendedOrderlessChoiceCollection {
    fn freeze(state: OrderlessChoiceCollectionState) -> Self {
        Self {
            flat_head: state.flat_head,
            pattern: FrozenMatcherValue::freeze(state.pattern),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            choices: FrozenMatcherSequence::freeze(state.choices),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            index: state.index,
            collected: PersistentMatchResults::freeze(state.collected),
        }
    }

    fn into_owned(self) -> OrderlessChoiceCollectionState {
        OrderlessChoiceCollectionState {
            flat_head: self.flat_head,
            pattern: self.pattern.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            choices: self.choices.into_vec(),
            bindings: self.bindings.into_owned(),
            index: self.index,
            collected: self.collected.into_vec(),
        }
    }
}

impl Session {
    pub(super) fn collect_compiled_orderless_choices_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.with_match_depth_guard(move |session| {
            session.collect_orderless_choice_matches(OrderlessChoiceCollectionState {
                flat_head: None,
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
                collected,
            })
        })
    }

    pub(super) fn collect_compiled_orderless_flat_choices_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.with_match_depth_guard(move |session| {
            session.collect_orderless_choice_matches(OrderlessChoiceCollectionState {
                flat_head: Some(flat_head),
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
                collected,
            })
        })
    }

    fn collect_orderless_choice_matches(
        &mut self,
        state: OrderlessChoiceCollectionState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let Some(choice) = state.choices.get(state.index).cloned() else {
            return Ok(state.collected);
        };

        let match_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_flat_selected_values_owned(
                flat_head,
                state.pattern.clone(),
                choice.selected_values,
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_orderless_selected_values_owned(
                state.pattern.clone(),
                choice.selected_values,
                state.bindings.clone(),
            )
        };
        match match_result {
            Ok(outcome) => {
                self.finish_orderless_choice_collection(state, choice.rest_values, outcome)
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceCollection::freeze(state);
                let rest_values = SharedArgs::new(choice.rest_values);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_orderless_choice_collection(
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

    fn finish_orderless_choice_collection(
        &mut self,
        mut state: OrderlessChoiceCollectionState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            state.index += 1;
            return self.collect_orderless_choice_matches(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_orderless_arg_matches_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(matches) => self.finish_orderless_choice_rest_collection(state, matches),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceCollection::freeze(state);
                self.push_suspension_frame(move |session, payload| {
                    let matches = Vec::<SymbolExprMap>::from_continuation_expr(payload?);
                    session
                        .finish_orderless_choice_rest_collection(state.into_owned(), matches)
                        .map(|matches| matches.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_rest_collection(
        &mut self,
        mut state: OrderlessChoiceCollectionState,
        mut matches: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        state.collected.append(&mut matches);
        state.index += 1;
        self.collect_orderless_choice_matches(state)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedOrderlessChoiceCollection>();
};
