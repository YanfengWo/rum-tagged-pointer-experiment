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

        let Some(definition) = self.definitions.get(symbol).cloned() else {
            return Ok(None);
        };

        if is_system_symbol_name(head, heads::patterns::DEFAULT) {
            if let Some(value) =
                self.try_apply_direct_default_value(args, &definition.down_values)?
            {
                return Ok(Some(value));
            }
        }

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate = self.normalize_match_expr(&candidate);

        self.try_apply_down_values_from_list(
            candidate,
            normalized_candidate,
            definition.down_values,
            0,
        )
    }

    fn try_apply_down_values_from_list(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        down_values: Vec<DownValue>,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(down_value) = down_values.get(index).cloned() else {
            return Ok(None);
        };

        let matcher = self.compile_matcher_for_expr(&down_value.lhs);
        let match_result =
            self.match_compiled_owned(matcher, normalized_candidate.clone(), BTreeMap::new());

        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.try_apply_down_values_from_list(
                    candidate,
                    normalized_candidate,
                    down_values,
                    index + 1,
                );
            }

            let substituted = substitute_rule_bindings(&down_value.rhs, &outcome.bindings);
            let option_scope =
                session.option_value_scope_for_definition(&down_value.lhs, &candidate);
            session.push_trace(TraceEvent::RuleApplied {
                source: RuleApplicationSource::DownValue,
                input: candidate.to_string(),
                rule: down_value.lhs.to_string(),
                result: substituted.to_string(),
            });
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
        let Some(definition) = self.definitions.get(&default_symbol).cloned() else {
            return Ok(None);
        };

        self.evaluate_first_exact_default_value(
            &definition.down_values,
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
        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate = self.normalize_match_expr(&candidate);
        let mut up_values = Vec::new();

        for symbol in collect_upvalue_candidate_symbols(args) {
            let Some(definition) = self.definitions.get(&symbol).cloned() else {
                continue;
            };
            up_values.extend(definition.up_values);
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
        let Some(up_value) = up_values.get(index).cloned() else {
            return Ok(None);
        };

        let matcher = self.compile_matcher_for_expr(&up_value.lhs);
        let match_result =
            self.match_compiled_owned(matcher, normalized_candidate.clone(), BTreeMap::new());

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
            session.push_trace(TraceEvent::RuleApplied {
                source: RuleApplicationSource::UpValue,
                input: candidate.to_string(),
                rule: up_value.lhs.to_string(),
                result: substituted.to_string(),
            });
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

        let Some(definition) = self.definitions.get(&symbol).cloned() else {
            return Ok(None);
        };

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate = self.normalize_match_expr(&candidate);

        self.try_apply_sub_values_from_list(
            candidate,
            normalized_candidate,
            definition.sub_values,
            0,
        )
    }

    fn try_apply_sub_values_from_list(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        sub_values: Vec<SubValue>,
        index: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(sub_value) = sub_values.get(index).cloned() else {
            return Ok(None);
        };

        let matcher = self.compile_matcher_for_expr(&sub_value.lhs);
        let match_result =
            self.match_compiled_owned(matcher, normalized_candidate.clone(), BTreeMap::new());

        self.drive_internal_value(match_result, move |session, outcome| {
            if !outcome.matched {
                return session.try_apply_sub_values_from_list(
                    candidate,
                    normalized_candidate,
                    sub_values,
                    index + 1,
                );
            }

            let substituted = substitute_rule_bindings(&sub_value.rhs, &outcome.bindings);
            let option_scope =
                session.option_value_scope_for_definition(&sub_value.lhs, &candidate);
            session.push_trace(TraceEvent::RuleApplied {
                source: RuleApplicationSource::SubValue,
                input: candidate.to_string(),
                rule: sub_value.lhs.to_string(),
                result: substituted.to_string(),
            });
            session
                .evaluate_definition_rhs(option_scope, substituted)
                .map(Some)
        })
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
                let head_result = self.apply_replace_all_owned(*head, rules.clone(), source);
                self.drive_evaluated_value::<Expr, Expr, _>(
                    head_result,
                    move |session, new_head| {
                        session.apply_replace_all_args(new_head, Vec::new(), args, rules, source)
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
            session.push_trace(TraceEvent::RuleApplied {
                source,
                input: expr.to_string(),
                rule: rule.lhs.to_string(),
                result: replaced.to_string(),
            });
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
