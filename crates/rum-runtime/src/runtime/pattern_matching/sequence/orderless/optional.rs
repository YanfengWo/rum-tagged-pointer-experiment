use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_optional_choices(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if !value_args.is_empty() {
            let present_result = self.match_compiled_orderless_arg_choices_owned(
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings,
            );

            return self.drive_internal_value(present_result, move |session, outcome| {
                if outcome.matched {
                    Ok(outcome)
                } else {
                    session.match_compiled_orderless_optional_omission_owned(
                        spec,
                        rest_patterns,
                        value_args,
                        snapshot,
                    )
                }
            });
        }

        self.match_compiled_orderless_optional_omission_owned(
            spec,
            rest_patterns,
            value_args,
            snapshot,
        )
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_optional_choices(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if !value_args.is_empty() {
            let present_result = self.match_compiled_orderless_flat_arg_choices_owned(
                flat_head.clone(),
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings,
            );

            return self.drive_internal_value(present_result, move |session, outcome| {
                if outcome.matched {
                    Ok(outcome)
                } else {
                    session.match_compiled_orderless_flat_optional_omission_owned(
                        flat_head,
                        spec,
                        rest_patterns,
                        value_args,
                        snapshot,
                    )
                }
            });
        }

        self.match_compiled_orderless_flat_optional_omission_owned(
            flat_head,
            spec,
            rest_patterns,
            value_args,
            snapshot,
        )
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_optional_matches_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let present_result = if value_args.is_empty() {
            Ok(Vec::new())
        } else {
            self.collect_compiled_orderless_arg_choice_matches_owned(
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings.clone(),
            )
        };

        self.drive_internal_value(present_result, move |session, mut present| {
            let omission_result = session.match_compiled_orderless_optional_omission_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            );

            session.drive_internal_value(omission_result, move |_, outcome| {
                if outcome.matched {
                    present.push(outcome.bindings);
                }
                Ok(present)
            })
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_flat_optional_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let present_result = if value_args.is_empty() {
            Ok(Vec::new())
        } else {
            self.collect_compiled_orderless_flat_arg_choice_matches_owned(
                flat_head.clone(),
                (*spec.inner).clone(),
                rest_patterns.clone(),
                value_args.clone(),
                bindings.clone(),
            )
        };

        self.drive_internal_value(present_result, move |session, mut present| {
            let omission_result = session.match_compiled_orderless_flat_optional_omission_owned(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            );

            session.drive_internal_value(omission_result, move |_, outcome| {
                if outcome.matched {
                    present.push(outcome.bindings);
                }
                Ok(present)
            })
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_optional_omission_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let default_result = self.resolve_optional_default_expr_owned(spec.default);
        self.drive_internal_value(default_result, move |session, default_expr| {
            let Some(default_expr) = default_expr else {
                return Ok(match_outcome(false, bindings));
            };

            let default_result = session.match_compiled_orderless_selected_values_owned(
                *spec.inner,
                optional_default_values(default_expr),
                bindings,
            );

            session.drive_internal_value(default_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                session.match_compiled_orderless_args_owned(
                    rest_patterns,
                    value_args,
                    outcome.bindings,
                )
            })
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_optional_omission_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let default_result = self.resolve_optional_default_expr_owned(spec.default);
        self.drive_internal_value(default_result, move |session, default_expr| {
            let Some(default_expr) = default_expr else {
                return Ok(match_outcome(false, bindings));
            };

            let default_result = session.match_compiled_orderless_flat_selected_values_owned(
                flat_head.clone(),
                *spec.inner,
                optional_default_values(default_expr),
                bindings,
            );

            session.drive_internal_value(default_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                session.match_compiled_orderless_args_with_flat_head_owned(
                    rest_patterns,
                    value_args,
                    outcome.bindings,
                    flat_head,
                )
            })
        })
    }
}
