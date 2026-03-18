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

        let candidate = Expr::call(head.clone(), args.to_vec());
        let normalized_candidate = self.normalize_match_expr(&candidate);

        for down_value in definition.down_values {
            let mut bindings = BTreeMap::new();
            let matcher = self.compile_matcher_for_expr(&down_value.lhs);

            if self.match_compiled(&matcher, &normalized_candidate, &mut bindings)? {
                let substituted = substitute_rule_bindings(&down_value.rhs, &bindings);
                self.push_trace(TraceEvent::RuleApplied {
                    source: RuleApplicationSource::DownValue,
                    input: candidate.to_string(),
                    rule: down_value.lhs.to_string(),
                    result: substituted.to_string(),
                });
                return self.evaluate(substituted).map(Some);
            }
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

        for symbol in collect_upvalue_candidate_symbols(args) {
            let Some(definition) = self.definitions.get(&symbol).cloned() else {
                continue;
            };

            for up_value in definition.up_values {
                let mut bindings = BTreeMap::new();
                let matcher = self.compile_matcher_for_expr(&up_value.lhs);

                if self.match_compiled(&matcher, &normalized_candidate, &mut bindings)? {
                    let substituted = substitute_rule_bindings(&up_value.rhs, &bindings);
                    self.push_trace(TraceEvent::RuleApplied {
                        source: RuleApplicationSource::UpValue,
                        input: candidate.to_string(),
                        rule: up_value.lhs.to_string(),
                        result: substituted.to_string(),
                    });
                    return self.evaluate(substituted).map(Some);
                }
            }
        }

        Ok(None)
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

        for sub_value in definition.sub_values {
            let mut bindings = BTreeMap::new();
            let matcher = self.compile_matcher_for_expr(&sub_value.lhs);

            if self.match_compiled(&matcher, &normalized_candidate, &mut bindings)? {
                let substituted = substitute_rule_bindings(&sub_value.rhs, &bindings);
                self.push_trace(TraceEvent::RuleApplied {
                    source: RuleApplicationSource::SubValue,
                    input: candidate.to_string(),
                    rule: sub_value.lhs.to_string(),
                    result: substituted.to_string(),
                });
                return self.evaluate(substituted).map(Some);
            }
        }

        Ok(None)
    }

    pub(super) fn apply_replace_all(
        &mut self,
        expr: Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        if let Some(replaced) = self.apply_rules_once(&expr, rules, source)? {
            return Ok(replaced);
        }

        match expr {
            Expr::Atom(_) => Ok(expr),
            Expr::Call { head, args } => Ok(Expr::call(
                self.apply_replace_all(*head, rules, source)?,
                args.into_iter()
                    .map(|arg| self.apply_replace_all(arg, rules, source))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }

    pub(super) fn apply_rules_once(
        &mut self,
        expr: &Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(expr);

        for rule in rules {
            let mut bindings = BTreeMap::new();

            if self.match_compiled(&rule.matcher, &normalized_expr, &mut bindings)? {
                let replaced = substitute_rule_bindings(&rule.rhs, &bindings);
                self.push_trace(TraceEvent::RuleApplied {
                    source,
                    input: expr.to_string(),
                    rule: rule.lhs.to_string(),
                    result: replaced.to_string(),
                });
                return Ok(Some(replaced));
            }
        }

        Ok(None)
    }

    pub(super) fn push_trace(&mut self, event: TraceEvent) {
        if self.tracing_enabled {
            self.trace_events.push(event);
        }
    }
}
