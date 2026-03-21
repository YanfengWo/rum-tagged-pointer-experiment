use super::*;

fn interpreted_match_outcome(
    matched: bool,
    bindings: BTreeMap<Symbol, Expr>,
) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}

impl Session {
    #[allow(dead_code)]
    fn match_alternatives_interpreted_owned(
        &mut self,
        alternatives: Vec<Expr>,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(alternative) = alternatives.get(index).cloned() else {
            return Ok(interpreted_match_outcome(false, bindings));
        };

        let snapshot = bindings.clone();
        let value_for_retry = value.clone();
        let alternatives_for_retry = alternatives.clone();
        let alternative_result = self.match_expr_interpreted_owned(alternative, value, bindings);
        self.drive_internal_value(alternative_result, move |session, outcome| {
            if outcome.matched {
                Ok(outcome)
            } else {
                session.match_alternatives_interpreted_owned(
                    alternatives_for_retry,
                    value_for_retry,
                    snapshot,
                    index + 1,
                )
            }
        })
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let outcome =
            self.match_expr_interpreted_owned(pattern.clone(), value.clone(), bindings.clone())?;
        *bindings = outcome.bindings;
        Ok(outcome.matched)
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let normalized_pattern = self.normalize_match_expr(&pattern);
        let normalized_value = self.normalize_match_expr(&value);
        let snapshot = bindings.clone();
        let outcome = self.match_expr_interpreted_inner_owned(
            normalized_pattern,
            normalized_value,
            bindings,
        )?;

        if outcome.matched {
            Ok(outcome)
        } else {
            Ok(interpreted_match_outcome(false, snapshot))
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_inner_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(&pattern) {
            let name = name.clone();
            let inner = inner.clone();
            let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
            return self.drive_internal_value(inner_result, move |_, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let mut bindings = outcome.bindings;
                Ok(interpreted_match_outcome(
                    bind_pattern_name(&name, value.clone(), &mut bindings),
                    bindings,
                ))
            });
        }

        if let Some((inner, condition)) = pattern_condition(&pattern) {
            let inner = inner.clone();
            let condition = condition.clone();
            let inner_result = self.match_expr_interpreted_owned(inner, value, bindings);
            return self.drive_internal_value(inner_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let snapshot = outcome.bindings.clone();
                let bindings = outcome.bindings;
                let condition_expr = substitute_bindings(&condition, &bindings);
                let result = session.evaluate(condition_expr);

                session.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
                    result,
                    move |_, result| {
                        Ok(if is_system_symbol_name(&result, "True") {
                            interpreted_match_outcome(true, bindings)
                        } else {
                            interpreted_match_outcome(false, snapshot)
                        })
                    },
                )
            });
        }

        if let Some((inner, test)) = pattern_test(&pattern) {
            let inner = inner.clone();
            let test = test.clone();
            let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
            return self.drive_internal_value(inner_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let false_bindings = outcome.bindings.clone();
                session.evaluate_pattern_test_owned(
                    test,
                    value.clone(),
                    outcome.bindings,
                    false_bindings,
                )
            });
        }

        if let Some((excluded, inner)) = pattern_except(&pattern) {
            let excluded = excluded.clone();
            let inner = inner
                .cloned()
                .unwrap_or_else(|| Expr::call(system_expr("Blank"), vec![]));
            let snapshot = bindings.clone();
            let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
            return self.drive_internal_value(inner_result, move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let accepted_bindings = outcome.bindings.clone();
                let excluded_result =
                    session.match_expr_interpreted_owned(excluded, value, outcome.bindings);
                session.drive_internal_value(excluded_result, move |_, excluded_outcome| {
                    Ok(if excluded_outcome.matched {
                        interpreted_match_outcome(false, snapshot)
                    } else {
                        interpreted_match_outcome(true, accepted_bindings)
                    })
                })
            });
        }

        if let Some((inner, _)) = pattern_optional(&pattern) {
            return self.match_expr_interpreted_owned(inner.clone(), value, bindings);
        }

        if let Some(alternatives) = pattern_alternatives(&pattern) {
            return self.match_alternatives_interpreted_owned(
                alternatives.to_vec(),
                value,
                bindings,
                0,
            );
        }

        if let Some(blank_head) = pattern_blank_head(&pattern) {
            return Ok(interpreted_match_outcome(
                match_blank(blank_head, &value),
                bindings,
            ));
        }

        match (pattern, value) {
            (Expr::Atom(lhs), Expr::Atom(rhs)) => {
                Ok(interpreted_match_outcome(lhs == rhs, bindings))
            }
            (
                Expr::Call {
                    head: lhs_head,
                    args: lhs_args,
                },
                Expr::Call {
                    head: rhs_head,
                    args: rhs_args,
                },
            ) => {
                let lhs_head_expr = (*lhs_head).clone();
                let head_result = self.match_expr_interpreted_owned(*lhs_head, *rhs_head, bindings);
                self.drive_internal_value(head_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    session.match_call_args_interpreted_owned(
                        lhs_head_expr,
                        lhs_args,
                        rhs_args,
                        outcome.bindings,
                    )
                })
            }
            (Expr::Call { head, args }, other)
                if matcher_head_has_attribute(
                    &MatcherNode::Exact((*head).clone()),
                    self,
                    Attribute::OneIdentity,
                ) =>
            {
                self.match_call_args_interpreted_owned((*head).clone(), args, vec![other], bindings)
            }
            _ => Ok(interpreted_match_outcome(false, bindings)),
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_call_args_interpreted_owned(
        &mut self,
        pattern_head: Expr,
        pattern_args: Vec<Expr>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let compiled_args = compile_matcher_args(&pattern_head, &pattern_args);
        self.match_compiled_args_owned(compiled_args, value_args, bindings)
    }
}
