use super::*;

impl Session {
    pub(in crate::runtime) fn try_apply_up_values(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
        attributes: AttributeSet,
    ) -> Result<Option<Expr>, EvalError> {
        if !self.has_up_value_definitions {
            return Ok(None);
        }

        let mut up_values = Vec::new();
        let args_slice = args.as_ref();

        for symbol in collect_upvalue_candidate_symbols(args_slice) {
            let Some(definition) = self.definitions.get(&symbol) else {
                continue;
            };
            up_values.extend(
                definition
                    .up_values
                    .iter()
                    .cloned()
                    .map(|up_value| (symbol, up_value)),
            );
        }

        if up_values.is_empty() {
            return Ok(None);
        }

        let candidate = Expr::call_with_shared_args(head.clone(), args.clone());
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(&candidate, attributes);

        self.try_apply_up_values_from_list(candidate, normalized_candidate, up_values, 0)
    }

    fn try_apply_up_values_from_list(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        up_values: Vec<(Symbol, UpValue)>,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some((symbol, up_value)) = up_values.get(index) else {
            return Ok(None);
        };
        let symbol = *symbol;

        record_profile_definition_dispatch_candidate("up_value", &symbol);
        let prefilter_start = definition_dispatch_timing_start();
        let could_match = Self::definition_lhs_could_match_candidate(
            &up_value.lhs,
            &up_value.matcher,
            &normalized_candidate,
        );
        record_profile_definition_dispatch_phase_timing(
            "up_value",
            &symbol,
            "prefilter",
            prefilter_start,
        );
        if !could_match {
            record_profile_definition_dispatch_prefilter_skip("up_value", &symbol);
            return self.try_apply_up_values_from_list(
                candidate,
                normalized_candidate,
                up_values,
                index + 1,
            );
        }

        let fast_match_start = definition_dispatch_timing_start();
        let fast_outcome = if up_value.matcher_attribute_revision == self.attribute_revision {
            self.try_fast_definition_match_compiled(&up_value.matcher, &normalized_candidate)
        } else {
            self.try_fast_definition_match(&up_value.lhs, &normalized_candidate)
        };
        record_profile_definition_dispatch_phase_timing(
            "up_value",
            &symbol,
            "fast_match",
            fast_match_start,
        );
        if let Some(outcome) = fast_outcome {
            record_profile_definition_dispatch_fast_outcome("up_value", &symbol, outcome.matched);
            if !outcome.matched {
                return self.try_apply_up_values_from_list(
                    candidate,
                    normalized_candidate,
                    up_values,
                    index + 1,
                );
            }

            let (lhs, substituted) = {
                let rhs_substitution_start = definition_dispatch_timing_start();
                let substituted = substitute_rule_bindings_for_rhs(
                    &up_value.rhs,
                    &outcome.bindings,
                    up_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "up_value",
                    &symbol,
                    "rhs_substitution",
                    rhs_substitution_start,
                );
                (up_value.lhs.clone(), substituted)
            };
            let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
            self.push_rule_applied_trace(
                RuleApplicationSource::UpValue,
                &candidate,
                &lhs,
                &substituted,
            );
            let profile_rule_start = definition_rule_timing_start();
            let result = self.evaluate_definition_rhs(option_scope, substituted);
            record_profile_definition_rule_timing(
                "up_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            return result.map(Some);
        }

        let up_value = up_value.clone();
        let matcher = if up_value.matcher_attribute_revision == self.attribute_revision {
            up_value.matcher.clone()
        } else {
            self.compile_matcher_for_expr(&up_value.lhs)
        };
        let compiled_matcher_start = definition_dispatch_timing_start();
        let match_result = self.match_compiled_owned(
            matcher,
            normalized_candidate.clone(),
            SymbolExprMap::default(),
        );
        record_profile_definition_dispatch_phase_timing(
            "up_value",
            &symbol,
            "compiled_matcher",
            compiled_matcher_start,
        );

        self.drive_internal_value(match_result, move |session, outcome| {
            record_profile_definition_dispatch_matcher_outcome(
                "up_value",
                &symbol,
                outcome.matched,
            );
            if !outcome.matched {
                return session.try_apply_up_values_from_list(
                    candidate,
                    normalized_candidate,
                    up_values,
                    index + 1,
                );
            }

            let rhs_substitution_start = definition_dispatch_timing_start();
            let substituted = substitute_rule_bindings_for_rhs(
                &up_value.rhs,
                &outcome.bindings,
                up_value.rhs_may_splice_without_bindings,
            );
            record_profile_definition_dispatch_phase_timing(
                "up_value",
                &symbol,
                "rhs_substitution",
                rhs_substitution_start,
            );
            let option_scope = session.option_value_scope_for_definition(&up_value.lhs, &candidate);
            session.push_rule_applied_trace(
                RuleApplicationSource::UpValue,
                &candidate,
                &up_value.lhs,
                &substituted,
            );
            let profile_rule_start = definition_rule_timing_start();
            let result = session.evaluate_definition_rhs(option_scope, substituted);
            record_profile_definition_rule_timing(
                "up_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            result.map(Some)
        })
    }
}
