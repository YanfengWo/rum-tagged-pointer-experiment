use super::candidates::OrderlessCandidateChoice;
use super::*;

impl Session {
    pub(super) fn collect_compiled_orderless_choices_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let Some(choice) = choices.get(index).cloned() else {
            return Ok(collected);
        };

        let selected_values = choice.selected_values;
        let rest_values = choice.rest_values;
        let match_result = self.match_compiled_orderless_selected_values_owned(
            pattern.clone(),
            selected_values,
            bindings.clone(),
        );
        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_orderless_choices_matches_owned(
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                );
            }

            let rest_result = session.collect_compiled_orderless_arg_matches_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            );
            session.drive_internal_value(rest_result, move |session, mut rest_matches| {
                let mut collected = collected;
                collected.append(&mut rest_matches);
                session.collect_compiled_orderless_choices_matches_owned(
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                )
            })
        })
    }

    pub(super) fn collect_compiled_orderless_flat_choices_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
        collected: Vec<BTreeMap<Symbol, Expr>>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        let Some(choice) = choices.get(index).cloned() else {
            return Ok(collected);
        };

        let selected_values = choice.selected_values;
        let rest_values = choice.rest_values;
        let match_result = self.match_compiled_orderless_flat_selected_values_owned(
            flat_head.clone(),
            pattern.clone(),
            selected_values,
            bindings.clone(),
        );
        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                );
            }

            let rest_result = session.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head.clone(),
            );
            session.drive_internal_value(rest_result, move |session, mut rest_matches| {
                let mut collected = collected;
                collected.append(&mut rest_matches);
                session.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    pattern,
                    rest_patterns,
                    choices,
                    bindings,
                    index + 1,
                    collected,
                )
            })
        })
    }
}
