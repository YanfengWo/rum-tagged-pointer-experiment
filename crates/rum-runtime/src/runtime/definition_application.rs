use super::*;

impl Session {
    pub(super) fn try_apply_down_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        let Some(definition) = self.definitions.get(symbol) else {
            return Ok(None);
        };

        if is_system_symbol_name(head, heads::patterns::DEFAULT) {
            let down_values = definition.down_values.clone();
            if let Some(value) = self.try_apply_direct_default_value(args, &down_values)? {
                return Ok(Some(value));
            }
        }

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(head, &candidate);

        self.try_apply_down_values_from_symbol(candidate, normalized_candidate, symbol.clone(), 0)
    }

    fn try_apply_down_values_from_symbol(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        symbol: Symbol,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(down_value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.down_values.get(index))
        else {
            return Ok(None);
        };

        if !Self::definition_lhs_could_match_candidate(&down_value.lhs, &normalized_candidate) {
            return self.try_apply_down_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol,
                index + 1,
            );
        }

        let down_value = down_value.clone();
        let match_result = if let Some(outcome) =
            self.try_fast_definition_match(&down_value.lhs, &normalized_candidate)
        {
            Ok(outcome)
        } else {
            let matcher = if down_value.matcher_attribute_revision == self.attribute_revision {
                down_value.matcher.clone()
            } else {
                self.compile_matcher_for_expr(&down_value.lhs)
            };
            self.match_compiled_owned(matcher, normalized_candidate.clone(), BTreeMap::new())
        };

        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.try_apply_down_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    index + 1,
                );
            }

            let substituted = substitute_rule_bindings(&down_value.rhs, &outcome.bindings);
            let option_scope =
                session.option_value_scope_for_definition(&down_value.lhs, &candidate);
            session.push_rule_applied_trace(
                RuleApplicationSource::DownValue,
                &candidate,
                &down_value.lhs,
                &substituted,
            );
            session
                .evaluate_definition_rhs(option_scope, substituted)
                .map(Some)
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

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(head, &candidate);
        let mut up_values = Vec::new();

        for symbol in collect_upvalue_candidate_symbols(args) {
            let Some(definition) = self.definitions.get(&symbol) else {
                continue;
            };
            up_values.extend(definition.up_values.iter().cloned());
        }

        self.try_apply_up_values_from_list(candidate, normalized_candidate, up_values, 0)
    }

    fn try_apply_up_values_from_list(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        up_values: Vec<UpValue>,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(up_value) = up_values.get(index) else {
            return Ok(None);
        };

        if !Self::definition_lhs_could_match_candidate(&up_value.lhs, &normalized_candidate) {
            return self.try_apply_up_values_from_list(
                candidate,
                normalized_candidate,
                up_values,
                index + 1,
            );
        }

        let up_value = up_value.clone();
        let match_result = if let Some(outcome) =
            self.try_fast_definition_match(&up_value.lhs, &normalized_candidate)
        {
            Ok(outcome)
        } else {
            let matcher = if up_value.matcher_attribute_revision == self.attribute_revision {
                up_value.matcher.clone()
            } else {
                self.compile_matcher_for_expr(&up_value.lhs)
            };
            self.match_compiled_owned(matcher, normalized_candidate.clone(), BTreeMap::new())
        };

        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.try_apply_up_values_from_list(
                    candidate,
                    normalized_candidate,
                    up_values,
                    index + 1,
                );
            }

            let substituted = substitute_rule_bindings(&up_value.rhs, &outcome.bindings);
            let option_scope = session.option_value_scope_for_definition(&up_value.lhs, &candidate);
            session.push_rule_applied_trace(
                RuleApplicationSource::UpValue,
                &candidate,
                &up_value.lhs,
                &substituted,
            );
            session
                .evaluate_definition_rhs(option_scope, substituted)
                .map(Some)
        })
    }

    pub(super) fn try_apply_sub_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = base_head_symbol_of_nested_call(head).cloned() else {
            return Ok(None);
        };

        if !self.definitions.contains_key(&symbol) {
            return Ok(None);
        }

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(head, &candidate);

        self.try_apply_sub_values_from_symbol(candidate, normalized_candidate, symbol, 0)
    }

    fn try_apply_sub_values_from_symbol(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        symbol: Symbol,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(sub_value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.sub_values.get(index))
        else {
            return Ok(None);
        };

        if !Self::definition_lhs_could_match_candidate(&sub_value.lhs, &normalized_candidate) {
            return self.try_apply_sub_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol,
                index + 1,
            );
        }

        let sub_value = sub_value.clone();
        let match_result = if let Some(outcome) =
            self.try_fast_definition_match(&sub_value.lhs, &normalized_candidate)
        {
            Ok(outcome)
        } else {
            let matcher = if sub_value.matcher_attribute_revision == self.attribute_revision {
                sub_value.matcher.clone()
            } else {
                self.compile_matcher_for_expr(&sub_value.lhs)
            };
            self.match_compiled_owned(matcher, normalized_candidate.clone(), BTreeMap::new())
        };

        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.try_apply_sub_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    index + 1,
                );
            }

            let substituted = substitute_rule_bindings(&sub_value.rhs, &outcome.bindings);
            let option_scope =
                session.option_value_scope_for_definition(&sub_value.lhs, &candidate);
            session.push_rule_applied_trace(
                RuleApplicationSource::SubValue,
                &candidate,
                &sub_value.lhs,
                &substituted,
            );
            session
                .evaluate_definition_rhs(option_scope, substituted)
                .map(Some)
        })
    }

    fn normalize_definition_candidate_for_matching(&self, head: &Expr, candidate: &Expr) -> Expr {
        if matches!(
            self.attribute_set_for_head(head).hold_attribute(),
            HoldAttribute::None
        ) {
            candidate.clone()
        } else {
            self.normalize_match_expr(candidate)
        }
    }

    fn definition_lhs_could_match_candidate(lhs: &Expr, candidate: &Expr) -> bool {
        Self::pattern_could_match_value_shape(strip_hold_pattern(lhs), candidate)
    }

    fn pattern_could_match_value_shape(pattern: &Expr, value: &Expr) -> bool {
        let pattern = strip_hold_pattern(pattern);

        if let Some(inner) = pattern_verbatim_inner(pattern) {
            return inner == value;
        }

        if let Some((_, inner)) = pattern_named_pattern(pattern) {
            return Self::pattern_could_match_value_shape(inner, value);
        }

        if let Some((inner, _)) = pattern_condition(pattern) {
            return Self::pattern_could_match_value_shape(inner, value);
        }

        if let Some((inner, _)) = pattern_test(pattern) {
            return Self::pattern_could_match_value_shape(inner, value);
        }

        if pattern_except(pattern).is_some() {
            return true;
        }

        if let Some((inner, _)) = pattern_optional(pattern) {
            return Self::pattern_could_match_value_shape(inner, value);
        }

        if let Some(alternatives) = pattern_alternatives(pattern) {
            return alternatives
                .iter()
                .any(|alternative| Self::pattern_could_match_value_shape(alternative, value));
        }

        if let Some(head_constraint) = pattern_blank_head(pattern) {
            return match_blank(head_constraint, value);
        }

        let Expr::Call {
            head: pattern_head,
            args: pattern_args,
        } = pattern
        else {
            return pattern == value;
        };

        let Expr::Call {
            head: value_head,
            args: value_args,
        } = value
        else {
            return true;
        };

        if !Self::pattern_could_match_value_shape(pattern_head, value_head) {
            return false;
        }

        if Self::pattern_args_may_have_variable_arity(pattern_args) {
            return true;
        }

        pattern_args.len() == value_args.len()
            && pattern_args
                .iter()
                .zip(value_args)
                .all(|(pattern_arg, value_arg)| {
                    Self::pattern_could_match_value_shape(pattern_arg, value_arg)
                })
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
        let mut bindings = BTreeMap::new();
        let matched =
            self.fast_match_definition_pattern(strip_hold_pattern(lhs), candidate, &mut bindings)?;

        Some(CompiledMatchOutcome {
            matched,
            bindings: if matched { bindings } else { BTreeMap::new() },
        })
    }

    fn fast_match_definition_pattern(
        &self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
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
        let rhs_result = self.evaluate_with_option_value_scope(option_scope, rhs);
        let outcome = self.suspend_with_result_frame(rhs_result, |session, outcome| {
            session.finish_definition_rhs_evaluation(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_definition_rhs_evaluation(outcome),
        }
    }

    fn finish_definition_rhs_evaluation(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        }
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
        let Some(rule) = rules.get(index).cloned() else {
            return Ok(None);
        };

        let match_result = self.match_compiled_owned(
            rule.matcher.clone(),
            normalized_expr.clone(),
            BTreeMap::new(),
        );

        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.apply_rules_once_from_list(
                    expr,
                    normalized_expr,
                    rules,
                    source,
                    index + 1,
                );
            }

            let replaced = substitute_rule_bindings(&rule.rhs, &outcome.bindings);
            session.push_rule_applied_trace(source, &expr, &rule.lhs, &replaced);
            Ok(Some(replaced))
        })
    }

    pub(super) fn push_trace(&mut self, event: TraceEvent) {
        let depth = self.evaluation_stack.len().saturating_sub(2);
        self.push_trace_with_depth(event, depth);
    }

    pub(super) fn push_trace_transition(&mut self, event: TraceEvent) {
        let depth = self.evaluation_stack.len();
        self.push_trace_with_depth(event, depth);
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
