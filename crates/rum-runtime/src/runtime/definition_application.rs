use super::*;

impl Session {
    pub(super) fn try_apply_down_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_head = runtime_path_timing_start("down_values.head_lookup");
        let Some(symbol) = expr_as_symbol(head) else {
            record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::Some);

        let runtime_profile_definition = runtime_path_timing_start("down_values.definition_lookup");
        let Some(definition) = self.definitions.get(symbol) else {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        };

        if definition.down_values.is_empty() {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        }
        record_profile_runtime_path_timing(runtime_profile_definition, RuntimeProfileOutcome::Some);

        self.record_profile_down_value_scan(symbol);

        if is_system_symbol_name(head, heads::patterns::DEFAULT) {
            let runtime_profile_default =
                runtime_path_timing_start("down_values.direct_default_lookup");
            let down_values = definition.down_values.clone();
            let direct_default = self.try_apply_direct_default_value(args, &down_values);
            record_profile_runtime_path_timing(
                runtime_profile_default,
                runtime_profile_outcome_for_option_result(&direct_default),
            );
            if let Some(value) = direct_default? {
                return Ok(Some(value));
            }
        }

        let runtime_profile_candidate = runtime_path_timing_start("down_values.candidate_build");
        let candidate = Expr::call(head.clone(), args.to_vec());
        record_profile_runtime_path_timing(runtime_profile_candidate, RuntimeProfileOutcome::Ok);
        let runtime_profile_normalize =
            runtime_path_timing_start("down_values.normalize_candidate");
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(head, &candidate);
        record_profile_runtime_path_timing(runtime_profile_normalize, RuntimeProfileOutcome::Ok);

        self.try_apply_down_values_from_symbol(candidate, normalized_candidate, symbol.clone(), 0)
    }

    fn try_apply_down_values_from_symbol(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        symbol: Symbol,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_value = runtime_path_timing_start("down_values.value_lookup");
        let Some(down_value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.down_values.get(index))
        else {
            record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::Some);

        record_profile_definition_dispatch_candidate("down_value", &symbol);
        let runtime_profile_prefilter = runtime_path_timing_start("down_values.prefilter");
        let prefilter_start = definition_dispatch_timing_start();
        let could_match = Self::definition_lhs_could_match_candidate(
            &down_value.lhs,
            &down_value.matcher,
            &normalized_candidate,
        );
        record_profile_definition_dispatch_phase_timing(
            "down_value",
            &symbol,
            "prefilter",
            prefilter_start,
        );
        record_profile_runtime_path_timing(runtime_profile_prefilter, RuntimeProfileOutcome::Ok);
        if !could_match {
            record_profile_definition_dispatch_prefilter_skip("down_value", &symbol);
            return self.try_apply_down_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol,
                index + 1,
            );
        }

        let runtime_profile_fast_match = runtime_path_timing_start("down_values.fast_match");
        let fast_match_start = definition_dispatch_timing_start();
        let fast_outcome = self.try_fast_definition_match(&down_value.lhs, &normalized_candidate);
        record_profile_definition_dispatch_phase_timing(
            "down_value",
            &symbol,
            "fast_match",
            fast_match_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_fast_match,
            runtime_profile_outcome_for_optional_fast_match(&fast_outcome),
        );
        if let Some(outcome) = fast_outcome {
            record_profile_definition_dispatch_fast_outcome("down_value", &symbol, outcome.matched);
            if !outcome.matched {
                return self.try_apply_down_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    index + 1,
                );
            }

            let (lhs, substituted) = {
                let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
                let runtime_profile_rhs_substitution =
                    runtime_path_timing_start("down_values.rhs_substitution");
                let rhs_substitution_start = definition_dispatch_timing_start();
                let substituted = substitute_rule_bindings_for_rhs(
                    &down_value.rhs,
                    &outcome.bindings,
                    down_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "down_value",
                    &symbol,
                    "rhs_substitution",
                    rhs_substitution_start,
                );
                record_profile_runtime_path_timing(
                    runtime_profile_rhs_substitution,
                    RuntimeProfileOutcome::Ok,
                );
                (down_value.lhs.clone(), substituted)
            };
            let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
            self.push_rule_applied_trace(
                RuleApplicationSource::DownValue,
                &candidate,
                &lhs,
                &substituted,
            );
            self.record_profile_down_value_application(&symbol);
            let _profile_definition_source = evaluation_profile_definition_source(symbol);
            let profile_rule_start = definition_rule_timing_start();
            let result = self.evaluate_definition_rhs(option_scope, substituted);
            record_profile_definition_rule_timing(
                "down_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            return result.map(Some);
        }

        let down_value = down_value.clone();
        let matcher = if down_value.matcher_attribute_revision == self.attribute_revision {
            down_value.matcher.clone()
        } else {
            self.compile_matcher_for_expr(&down_value.lhs)
        };
        let runtime_profile_compiled_matcher =
            runtime_path_timing_start("down_values.compiled_matcher");
        let compiled_matcher_start = definition_dispatch_timing_start();
        let match_result = self.match_compiled_owned(
            matcher,
            normalized_candidate.clone(),
            SymbolExprMap::default(),
        );
        record_profile_definition_dispatch_phase_timing(
            "down_value",
            &symbol,
            "compiled_matcher",
            compiled_matcher_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_compiled_matcher,
            runtime_profile_outcome_for_internal_result(&match_result),
        );

        self.drive_internal_value(match_result, move |session, outcome| {
            record_profile_definition_dispatch_matcher_outcome(
                "down_value",
                &symbol,
                outcome.matched,
            );
            if !outcome.matched {
                return session.try_apply_down_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    index + 1,
                );
            }

            let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
            let runtime_profile_rhs_substitution =
                runtime_path_timing_start("down_values.rhs_substitution");
            let rhs_substitution_start = definition_dispatch_timing_start();
            let substituted = substitute_rule_bindings_for_rhs(
                &down_value.rhs,
                &outcome.bindings,
                down_value.rhs_may_splice_without_bindings,
            );
            record_profile_definition_dispatch_phase_timing(
                "down_value",
                &symbol,
                "rhs_substitution",
                rhs_substitution_start,
            );
            record_profile_runtime_path_timing(
                runtime_profile_rhs_substitution,
                RuntimeProfileOutcome::Ok,
            );
            let option_scope =
                session.option_value_scope_for_definition(&down_value.lhs, &candidate);
            session.push_rule_applied_trace(
                RuleApplicationSource::DownValue,
                &candidate,
                &down_value.lhs,
                &substituted,
            );
            session.record_profile_down_value_application(&symbol);
            let _profile_definition_source = evaluation_profile_definition_source(symbol);
            let profile_rule_start = definition_rule_timing_start();
            let result = session.evaluate_definition_rhs(option_scope, substituted);
            record_profile_definition_rule_timing(
                "down_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            result.map(Some)
        })
    }

    fn try_apply_direct_default_value(
        &mut self,
        args: &[Expr],
        down_values: &[DownValue],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(target) = args.first().cloned() else {
            return Ok(None);
        };

        let lookup_order = match args {
            [_] => vec![
                default_value_lhs(target.clone(), None),
                default_value_lhs(target, Some(1)),
            ],
            [_, index] => {
                let Some(index) = expr_as_non_negative_usize(index).filter(|index| *index > 0)
                else {
                    return Ok(None);
                };
                vec![
                    default_value_lhs(target.clone(), None),
                    default_value_lhs(target, Some(index)),
                ]
            }
            _ => return Ok(None),
        };

        self.evaluate_first_exact_default_value(down_values, lookup_order)
    }

    pub(in crate::runtime) fn resolve_optional_implicit_default(
        &mut self,
        head: Expr,
        position: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let default_symbol = Symbol::system(heads::patterns::DEFAULT);
        let Some(definition) = self.definitions.get(&default_symbol) else {
            return Ok(None);
        };
        let down_values = definition.down_values.clone();

        self.evaluate_first_exact_default_value(
            &down_values,
            vec![
                default_value_lhs(head.clone(), Some(position)),
                default_value_lhs(head, None),
            ],
        )
    }

    fn evaluate_first_exact_default_value(
        &mut self,
        down_values: &[DownValue],
        lookup_order: Vec<Expr>,
    ) -> Result<Option<Expr>, EvalError> {
        for lhs in lookup_order {
            let Some(rhs) = exact_down_value_rhs(down_values, &lhs) else {
                continue;
            };
            return self.evaluate_definition_rhs(None, rhs).map(Some);
        }

        Ok(None)
    }

    pub(super) fn try_apply_up_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        if !self.has_up_value_definitions {
            return Ok(None);
        }

        let mut up_values = Vec::new();

        for symbol in collect_upvalue_candidate_symbols(args) {
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

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(head, &candidate);

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
        let fast_outcome = self.try_fast_definition_match(&up_value.lhs, &normalized_candidate);
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

    pub(super) fn try_apply_sub_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_head = runtime_path_timing_start("sub_values.base_head_lookup");
        let Some(symbol) = base_head_symbol_of_nested_call(head).cloned() else {
            record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::Some);

        let runtime_profile_definition = runtime_path_timing_start("sub_values.definition_lookup");
        let Some(definition) = self.definitions.get(&symbol) else {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        };

        if definition.sub_values.is_empty() {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        }
        record_profile_runtime_path_timing(runtime_profile_definition, RuntimeProfileOutcome::Some);

        let runtime_profile_candidate = runtime_path_timing_start("sub_values.candidate_build");
        let candidate = Expr::call(head.clone(), args.to_vec());
        record_profile_runtime_path_timing(runtime_profile_candidate, RuntimeProfileOutcome::Ok);
        let runtime_profile_normalize = runtime_path_timing_start("sub_values.normalize_candidate");
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(head, &candidate);
        record_profile_runtime_path_timing(runtime_profile_normalize, RuntimeProfileOutcome::Ok);

        self.try_apply_sub_values_from_symbol(candidate, normalized_candidate, symbol, 0)
    }

    fn try_apply_sub_values_from_symbol(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        symbol: Symbol,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_value = runtime_path_timing_start("sub_values.value_lookup");
        let Some(sub_value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.sub_values.get(index))
        else {
            record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::Some);

        record_profile_definition_dispatch_candidate("sub_value", &symbol);
        let runtime_profile_prefilter = runtime_path_timing_start("sub_values.prefilter");
        let prefilter_start = definition_dispatch_timing_start();
        let could_match = Self::definition_lhs_could_match_candidate(
            &sub_value.lhs,
            &sub_value.matcher,
            &normalized_candidate,
        );
        record_profile_definition_dispatch_phase_timing(
            "sub_value",
            &symbol,
            "prefilter",
            prefilter_start,
        );
        record_profile_runtime_path_timing(runtime_profile_prefilter, RuntimeProfileOutcome::Ok);
        if !could_match {
            record_profile_definition_dispatch_prefilter_skip("sub_value", &symbol);
            return self.try_apply_sub_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol,
                index + 1,
            );
        }

        let runtime_profile_fast_match = runtime_path_timing_start("sub_values.fast_match");
        let fast_match_start = definition_dispatch_timing_start();
        let fast_outcome = self.try_fast_definition_match(&sub_value.lhs, &normalized_candidate);
        record_profile_definition_dispatch_phase_timing(
            "sub_value",
            &symbol,
            "fast_match",
            fast_match_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_fast_match,
            runtime_profile_outcome_for_optional_fast_match(&fast_outcome),
        );
        if let Some(outcome) = fast_outcome {
            record_profile_definition_dispatch_fast_outcome("sub_value", &symbol, outcome.matched);
            if !outcome.matched {
                return self.try_apply_sub_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    index + 1,
                );
            }

            let (lhs, substituted) = {
                let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
                let runtime_profile_rhs_substitution =
                    runtime_path_timing_start("sub_values.rhs_substitution");
                let rhs_substitution_start = definition_dispatch_timing_start();
                let substituted = substitute_rule_bindings_for_rhs(
                    &sub_value.rhs,
                    &outcome.bindings,
                    sub_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "sub_value",
                    &symbol,
                    "rhs_substitution",
                    rhs_substitution_start,
                );
                record_profile_runtime_path_timing(
                    runtime_profile_rhs_substitution,
                    RuntimeProfileOutcome::Ok,
                );
                (sub_value.lhs.clone(), substituted)
            };
            let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
            self.push_rule_applied_trace(
                RuleApplicationSource::SubValue,
                &candidate,
                &lhs,
                &substituted,
            );
            self.record_profile_sub_value_application(&symbol);
            let _profile_definition_source = evaluation_profile_definition_source(symbol);
            let profile_rule_start = definition_rule_timing_start();
            let result = self.evaluate_definition_rhs(option_scope, substituted);
            record_profile_definition_rule_timing(
                "sub_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            return result.map(Some);
        }

        let sub_value = sub_value.clone();
        let matcher = if sub_value.matcher_attribute_revision == self.attribute_revision {
            sub_value.matcher.clone()
        } else {
            self.compile_matcher_for_expr(&sub_value.lhs)
        };
        let runtime_profile_compiled_matcher =
            runtime_path_timing_start("sub_values.compiled_matcher");
        let compiled_matcher_start = definition_dispatch_timing_start();
        let match_result = self.match_compiled_owned(
            matcher,
            normalized_candidate.clone(),
            SymbolExprMap::default(),
        );
        record_profile_definition_dispatch_phase_timing(
            "sub_value",
            &symbol,
            "compiled_matcher",
            compiled_matcher_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_compiled_matcher,
            runtime_profile_outcome_for_internal_result(&match_result),
        );

        self.drive_internal_value(match_result, move |session, outcome| {
            record_profile_definition_dispatch_matcher_outcome(
                "sub_value",
                &symbol,
                outcome.matched,
            );
            if !outcome.matched {
                return session.try_apply_sub_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    index + 1,
                );
            }

            let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
            let runtime_profile_rhs_substitution =
                runtime_path_timing_start("sub_values.rhs_substitution");
            let rhs_substitution_start = definition_dispatch_timing_start();
            let substituted = substitute_rule_bindings_for_rhs(
                &sub_value.rhs,
                &outcome.bindings,
                sub_value.rhs_may_splice_without_bindings,
            );
            record_profile_definition_dispatch_phase_timing(
                "sub_value",
                &symbol,
                "rhs_substitution",
                rhs_substitution_start,
            );
            record_profile_runtime_path_timing(
                runtime_profile_rhs_substitution,
                RuntimeProfileOutcome::Ok,
            );
            let option_scope =
                session.option_value_scope_for_definition(&sub_value.lhs, &candidate);
            session.push_rule_applied_trace(
                RuleApplicationSource::SubValue,
                &candidate,
                &sub_value.lhs,
                &substituted,
            );
            session.record_profile_sub_value_application(&symbol);
            let _profile_definition_source = evaluation_profile_definition_source(symbol);
            let profile_rule_start = definition_rule_timing_start();
            let result = session.evaluate_definition_rhs(option_scope, substituted);
            record_profile_definition_rule_timing(
                "sub_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            result.map(Some)
        })
    }

    fn normalize_definition_candidate_for_matching(
        &mut self,
        head: &Expr,
        candidate: &Expr,
    ) -> Expr {
        if matches!(
            self.cached_attribute_set_for_head(head).hold_attribute(),
            HoldAttribute::None
        ) {
            candidate.clone()
        } else {
            self.normalize_match_expr(candidate)
        }
    }

    fn definition_lhs_could_match_candidate(
        _lhs: &Expr,
        matcher: &MatcherNode,
        candidate: &Expr,
    ) -> bool {
        Self::matcher_could_match_candidate_shape(matcher, candidate)
    }

    fn matcher_could_match_candidate_shape(matcher: &MatcherNode, candidate: &Expr) -> bool {
        match matcher {
            MatcherNode::Exact(pattern) => pattern == candidate,
            MatcherNode::Blank { head_constraint } => {
                match_blank(head_constraint.as_ref(), candidate)
            }
            MatcherNode::KeyValuePattern(_) | MatcherNode::Except { .. } => true,
            MatcherNode::Named { inner, .. }
            | MatcherNode::Condition { inner, .. }
            | MatcherNode::PatternTest { inner, .. } => {
                Self::matcher_could_match_candidate_shape(inner, candidate)
            }
            MatcherNode::Alternatives(alternatives) => alternatives.iter().any(|alternative| {
                Self::matcher_could_match_candidate_shape(alternative, candidate)
            }),
            MatcherNode::Call { head, args } => {
                let Expr::Call {
                    head: value_head,
                    args: value_args,
                } = candidate
                else {
                    return true;
                };

                if !Self::matcher_could_match_candidate_shape(head, value_head) {
                    return false;
                }

                if args.iter().any(|arg| !matches!(arg, MatcherArg::Single(_))) {
                    return true;
                }

                args.len() == value_args.len()
                    && args.iter().zip(value_args).all(|(pattern_arg, value_arg)| {
                        let MatcherArg::Single(matcher) = pattern_arg else {
                            return true;
                        };
                        Self::matcher_could_match_candidate_shape(matcher, value_arg)
                    })
            }
        }
    }

    fn pattern_args_may_have_variable_arity(args: &[Expr]) -> bool {
        args.iter().any(|arg| {
            let arg = strip_hold_pattern(arg);
            decompose_sequence_pattern(arg).is_some()
                || pattern_sequence_items(arg).is_some()
                || pattern_orderless_sequence_items(arg).is_some()
                || pattern_optional(arg).is_some()
        })
    }

    fn try_fast_definition_match(
        &self,
        lhs: &Expr,
        candidate: &Expr,
    ) -> Option<CompiledMatchOutcome> {
        let mut bindings = SymbolExprMap::default();
        let matched =
            self.fast_match_definition_pattern(strip_hold_pattern(lhs), candidate, &mut bindings)?;

        Some(CompiledMatchOutcome {
            matched,
            bindings: if matched {
                bindings
            } else {
                SymbolExprMap::default()
            },
        })
    }

    fn fast_match_definition_pattern(
        &self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut SymbolExprMap,
    ) -> Option<bool> {
        let pattern = strip_hold_pattern(pattern);

        if let Some(inner) = pattern_verbatim_inner(pattern) {
            return Some(inner == value);
        }

        if let Some((name, inner)) = pattern_named_pattern(pattern) {
            if !self.fast_match_definition_pattern(inner, value, bindings)? {
                return Some(false);
            }
            return Some(bind_pattern_name(name, value.clone(), bindings));
        }

        if pattern_condition(pattern).is_some()
            || pattern_test(pattern).is_some()
            || pattern_except(pattern).is_some()
            || pattern_optional(pattern).is_some()
            || pattern_alternatives(pattern).is_some()
            || pattern_key_value_pattern_entries(pattern).is_some()
            || decompose_sequence_pattern(pattern).is_some()
            || pattern_sequence_items(pattern).is_some()
            || pattern_orderless_sequence_items(pattern).is_some()
        {
            return None;
        }

        if let Some(head_constraint) = pattern_blank_head(pattern) {
            return Some(match_blank(head_constraint, value));
        }

        match (pattern, value) {
            (Expr::Atom(_), _) => Some(pattern == value),
            (
                Expr::Call {
                    head: pattern_head,
                    args: pattern_args,
                },
                Expr::Call {
                    head: value_head,
                    args: value_args,
                },
            ) => {
                if Self::pattern_args_may_have_variable_arity(pattern_args) {
                    return None;
                }
                if pattern_args.len() != value_args.len() {
                    return Some(false);
                }

                let head_attributes = self.attribute_set_for_head(pattern_head);
                if head_attributes.contains(Attribute::Flat)
                    || head_attributes.contains(Attribute::Orderless)
                    || head_attributes.contains(Attribute::OneIdentity)
                {
                    return None;
                }

                if !self.fast_match_definition_pattern(pattern_head, value_head, bindings)? {
                    return Some(false);
                }

                for (pattern_arg, value_arg) in pattern_args.iter().zip(value_args) {
                    if !self.fast_match_definition_pattern(pattern_arg, value_arg, bindings)? {
                        return Some(false);
                    }
                }

                Some(true)
            }
            (Expr::Call { .. }, Expr::Atom(_)) => None,
        }
    }

    pub(super) fn evaluate_definition_rhs(
        &mut self,
        option_scope: Option<OptionValueScope>,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        let profile_start = lexical_profile_start();
        let eval_cache_profile_start =
            evaluation_cache_profile_source_timing_start("definition_rhs_eval_by_source");
        let runtime_profile_total = runtime_path_timing_start("definition.rhs.total");
        let runtime_profile_evaluate = runtime_path_timing_start("definition.rhs.evaluate");
        let rhs_result = self.evaluate_with_option_value_scope(option_scope, rhs);
        record_profile_runtime_path_timing(
            runtime_profile_evaluate,
            runtime_profile_outcome_for_expr_result(&rhs_result),
        );
        let outcome = self.suspend_with_result_frame(rhs_result, |session, outcome| {
            session.finish_definition_rhs_evaluation(outcome)
        });

        let result = match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_definition_rhs_evaluation(outcome),
        };
        record_profile_definition_rhs_eval(profile_start);
        record_profile_definition_rhs_eval_by_source(eval_cache_profile_start);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    fn finish_definition_rhs_evaluation(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_finish = runtime_path_timing_start("definition.rhs.finish");
        let profile_outcome = runtime_profile_outcome_for_expr_result(&outcome);
        let result = match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        };
        record_profile_runtime_path_timing(runtime_profile_finish, profile_outcome);
        result
    }

    pub(super) fn apply_replace_all(
        &mut self,
        expr: Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        self.apply_replace_all_owned(expr, rules.to_vec(), source)
    }

    pub(super) fn try_apply_rewrite_rules_once_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        self.apply_rules_once_from_list(expr, normalized_expr, rules, source, 0)
    }

    fn apply_replace_all_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        let replace_result = self.apply_rules_once_from_list(
            expr.clone(),
            normalized_expr,
            rules.clone(),
            source,
            0,
        );

        self.drive_internal_value(replace_result, move |session, outcome| match outcome {
            Some(replaced) => Ok(replaced),
            None => session.apply_replace_all_after_miss(expr, rules, source),
        })
    }

    fn apply_replace_all_after_miss(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(_) => Ok(expr),
            Expr::Call { head, args } => {
                let remaining_args = args.into_vec();
                let head_result =
                    self.apply_replace_all_owned(head.into_owned(), rules.clone(), source);
                self.drive_evaluated_value::<Expr, Expr, _>(
                    head_result,
                    move |session, new_head| {
                        session.apply_replace_all_args(
                            new_head,
                            Vec::new(),
                            remaining_args,
                            rules,
                            source,
                        )
                    },
                )
            }
        }
    }

    fn apply_replace_all_args(
        &mut self,
        head: Expr,
        mut rewritten_args: Vec<Expr>,
        remaining_args: Vec<Expr>,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let Some((current, rest)) = remaining_args.split_first() else {
            return Ok(Expr::call(head, rewritten_args));
        };

        let rest = rest.to_vec();
        let arg_result = self.apply_replace_all_owned(current.clone(), rules.clone(), source);

        self.drive_evaluated_value::<Expr, Expr, _>(arg_result, move |session, rewritten| {
            rewritten_args.push(rewritten);
            session.apply_replace_all_args(head, rewritten_args, rest, rules, source)
        })
    }

    fn apply_rules_once_from_list(
        &mut self,
        expr: Expr,
        normalized_expr: Expr,
        rules: Vec<RewriteRule>,
        source: RuleApplicationSource,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(rule) = rules.get(index) else {
            return Ok(None);
        };
        let would_reject =
            !Self::definition_lhs_could_match_candidate(&rule.lhs, &rule.matcher, &normalized_expr);
        record_profile_rewrite_rule_prefilter_candidate(would_reject);
        let matcher = rule.matcher.clone();

        let match_result =
            self.match_compiled_owned(matcher, normalized_expr.clone(), SymbolExprMap::default());

        let outcome = match match_result {
            Ok(outcome) => outcome,
            Err(EvalError::InputRequired { request }) => {
                let rule = rules
                    .get(index)
                    .cloned()
                    .expect("rewrite rule should still exist for suspended match");
                return self.drive_internal_value::<CompiledMatchOutcome, Option<Expr>, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, outcome| {
                        record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);
                        if !outcome.matched {
                            return session.apply_rules_once_from_list(
                                expr,
                                normalized_expr,
                                rules,
                                source,
                                index + 1,
                            );
                        }

                        let replaced = substitute_rule_bindings_for_rhs(
                            &rule.rhs,
                            &outcome.bindings,
                            rule.rhs_may_splice_without_bindings,
                        );
                        session.push_rule_applied_trace(source, &expr, &rule.lhs, &replaced);
                        Ok(Some(replaced))
                    },
                );
            }
            Err(error) => return Err(error),
        };
        record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);

        if !outcome.matched {
            return self.apply_rules_once_from_list(
                expr,
                normalized_expr,
                rules,
                source,
                index + 1,
            );
        }

        let rule = rules
            .get(index)
            .expect("rewrite rule should still exist after match");
        let replaced = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &outcome.bindings,
            rule.rhs_may_splice_without_bindings,
        );
        self.push_rule_applied_trace(source, &expr, &rule.lhs, &replaced);
        Ok(Some(replaced))
    }

    pub(super) fn push_trace(&mut self, event: TraceEvent) {
        let depth = self.evaluation_stack.len().saturating_sub(2);
        self.push_trace_with_depth(event, depth);
    }

    pub(super) fn push_trace_transition(&mut self, event: TraceEvent) {
        let depth = self.evaluation_stack.len();
        self.push_trace_with_depth(event, depth);
    }

    pub(super) fn push_definition_stored_trace<F>(
        &mut self,
        symbol: &Symbol,
        kind: DefinitionKind,
        value: F,
    ) where
        F: FnOnce() -> String,
    {
        if !self.tracing_enabled
            && (self.monitor_scopes.is_empty() || !self.monitor_callbacks_enabled)
        {
            return;
        }

        let event = if self.tracing_enabled {
            TraceEvent::DefinitionStored {
                symbol: symbol.to_string(),
                kind,
                value: value(),
            }
        } else {
            TraceEvent::DefinitionStored {
                symbol: String::new(),
                kind,
                value: String::new(),
            }
        };

        self.push_trace(event);
    }

    pub(super) fn push_rule_applied_trace(
        &mut self,
        source: RuleApplicationSource,
        input: &Expr,
        rule: &Expr,
        result: &Expr,
    ) {
        let event = if self.tracing_enabled {
            TraceEvent::RuleApplied {
                source,
                input: input.to_string(),
                rule: rule.to_string(),
                result: result.to_string(),
            }
        } else {
            TraceEvent::RuleApplied {
                source,
                input: String::new(),
                rule: String::new(),
                result: String::new(),
            }
        };

        self.push_trace(event);
    }

    pub(super) fn push_evaluation_pass_trace(&mut self, before: &Expr, after: &Expr) {
        let event = if self.tracing_enabled {
            TraceEvent::EvaluationPass {
                before: before.to_string(),
                after: after.to_string(),
            }
        } else {
            TraceEvent::EvaluationPass {
                before: String::new(),
                after: String::new(),
            }
        };

        self.push_trace_transition(event);
    }

    fn push_trace_with_depth(&mut self, event: TraceEvent, depth: usize) {
        if self.tracing_enabled {
            self.trace_events.push(event.clone());
            self.trace_event_depths.push(depth);
        }

        self.handle_monitor_trace_event(&event);
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_option_result<T>(
    result: &Result<Option<T>, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(Some(_)) => RuntimeProfileOutcome::Some,
        Ok(None) => RuntimeProfileOutcome::None,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_internal_result<T>(
    result: &Result<T, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_result<T>(result: &Result<T, EvalError>) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_optional_fast_match(
    outcome: &Option<CompiledMatchOutcome>,
) -> RuntimeProfileOutcome {
    if outcome.is_some() {
        RuntimeProfileOutcome::Some
    } else {
        RuntimeProfileOutcome::None
    }
}

fn default_value_lhs(target: Expr, position: Option<usize>) -> Expr {
    let mut args = vec![target];
    if let Some(position) = position {
        args.push(Expr::integer(BigInt::from(position)));
    }
    Expr::call(system_expr(heads::patterns::DEFAULT), args)
}

fn exact_down_value_rhs(down_values: &[DownValue], lhs: &Expr) -> Option<Expr> {
    down_values
        .iter()
        .find(|down_value| down_value.lhs == *lhs)
        .map(|down_value| down_value.rhs.clone())
}
