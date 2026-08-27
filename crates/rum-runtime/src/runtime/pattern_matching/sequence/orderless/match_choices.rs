use super::candidates::OrderlessCandidateChoice;
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const MAX_MATCH_DEPTH: usize = 512;

struct OrderlessChoiceState {
    flat_head: Option<Expr>,
    pattern: MatcherArg,
    rest_patterns: Vec<MatcherArg>,
    choices: Vec<OrderlessCandidateChoice>,
    bindings: SymbolExprMap,
    index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOrderlessChoiceState {
    flat_head: Option<Expr>,
    pattern: FrozenMatcherValue<MatcherArg>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    choices: FrozenMatcherSequence<OrderlessCandidateChoice>,
    bindings: FrozenMatchBindings,
    index: usize,
}

impl SuspendedOrderlessChoiceState {
    fn freeze(state: OrderlessChoiceState) -> Self {
        Self {
            flat_head: state.flat_head,
            pattern: FrozenMatcherValue::freeze(state.pattern),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            choices: FrozenMatcherSequence::freeze(state.choices),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            index: state.index,
        }
    }

    fn into_owned(self) -> OrderlessChoiceState {
        OrderlessChoiceState {
            flat_head: self.flat_head,
            pattern: self.pattern.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            choices: self.choices.into_vec(),
            bindings: self.bindings.into_owned(),
            index: self.index,
        }
    }
}

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
            session.match_orderless_choice(OrderlessChoiceState {
                flat_head: None,
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
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
            session.match_orderless_choice(OrderlessChoiceState {
                flat_head: Some(flat_head),
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
            })
        })
    }

    fn match_orderless_choice(
        &mut self,
        state: OrderlessChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(choice) = state.choices.get(state.index).cloned() else {
            return Ok(match_outcome(false, state.bindings));
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
            Ok(outcome) => self.finish_orderless_choice_pattern(state, choice.rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceState::freeze(state);
                let rest_values = SharedArgs::new(choice.rest_values);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_orderless_choice_pattern(
                            state.into_owned(),
                            rest_values.into_vec(),
                            outcome,
                        )
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_pattern(
        &mut self,
        state: OrderlessChoiceState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return self.retry_orderless_choice(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_args_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_orderless_args_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(outcome) => self.finish_orderless_choice_rest(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceState::freeze(state);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_orderless_choice_rest(state.into_owned(), outcome)
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_rest(
        &mut self,
        state: OrderlessChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.retry_orderless_choice(state)
        }
    }

    fn retry_orderless_choice(
        &mut self,
        state: OrderlessChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some(flat_head) = state.flat_head {
            self.match_compiled_orderless_flat_choices_owned(
                flat_head,
                state.pattern,
                state.rest_patterns,
                state.choices,
                state.bindings,
                state.index + 1,
            )
        } else {
            self.match_compiled_orderless_choices_owned(
                state.pattern,
                state.rest_patterns,
                state.choices,
                state.bindings,
                state.index + 1,
            )
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedOrderlessChoiceState>();
};
