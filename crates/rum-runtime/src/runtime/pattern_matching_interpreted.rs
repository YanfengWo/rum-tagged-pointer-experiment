use super::*;

fn interpreted_match_outcome(
    matched: bool,
    bindings: SymbolExprMap,
) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}

impl Session {
    fn match_named_interpreted_owned(
        &mut self,
        name: Symbol,
        inner: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
        self.drive_internal_value(inner_result, move |_, outcome| {
            if !outcome.matched {
                return Ok(outcome);
            }

            let mut bindings = outcome.bindings;
            Ok(interpreted_match_outcome(
                bind_pattern_name(&name, value.clone(), &mut bindings),
                bindings,
            ))
        })
    }

    fn match_condition_interpreted_owned(
        &mut self,
        inner: Expr,
        condition: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_expr_interpreted_owned(inner, value, bindings);
        self.drive_internal_value(inner_result, move |session, outcome| {
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
        })
    }

    fn match_pattern_test_interpreted_owned(
        &mut self,
        inner: Expr,
        test: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
        self.drive_internal_value(inner_result, move |session, outcome| {
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
        })
    }

    fn match_except_interpreted_owned(
        &mut self,
        excluded: Expr,
        inner: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();
        let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
        self.drive_internal_value(inner_result, move |session, outcome| {
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
        })
    }

    fn match_call_interpreted_owned(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match value {
            Expr::Call {
                head: value_head,
                args: value_args,
            } => {
                let head_expr = head.clone();
                let value_head = value_head.into_owned();
                let value_args = value_args.into_vec();
                let head_result = self.match_expr_interpreted_owned(head, value_head, bindings);
                self.drive_internal_value(head_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    session.match_call_args_interpreted_owned(
                        head_expr,
                        args,
                        value_args,
                        outcome.bindings,
                    )
                })
            }
            other
                if matcher_head_has_attribute(
                    &MatcherNode::Exact(head.clone()),
                    self,
                    Attribute::OneIdentity,
                ) =>
            {
                self.match_call_args_interpreted_owned(head, args, vec![other], bindings)
            }
            _ => Ok(interpreted_match_outcome(false, bindings)),
        }
    }

    #[allow(dead_code)]
    fn match_alternatives_interpreted_owned(
        &mut self,
        alternatives: Vec<Expr>,
        value: Expr,
        bindings: SymbolExprMap,
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
        bindings: &mut SymbolExprMap,
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
        bindings: SymbolExprMap,
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
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(&pattern) {
            return self.match_named_interpreted_owned(
                name.clone(),
                inner.clone(),
                value,
                bindings,
            );
        }

        if let Some(entries) = pattern_key_value_pattern_entries(&pattern) {
            return self.match_compiled_key_value_pattern_owned(
                entries
                    .into_iter()
                    .map(|entry| {
                        (
                            compile_matcher(&entry.key),
                            compile_matcher(&entry.value),
                            entry.delayed,
                        )
                    })
                    .collect(),
                value,
                bindings,
            );
        }

        if let Some((inner, condition)) = pattern_condition(&pattern) {
            return self.match_condition_interpreted_owned(
                inner.clone(),
                condition.clone(),
                value,
                bindings,
            );
        }

        if let Some((inner, test)) = pattern_test(&pattern) {
            return self.match_pattern_test_interpreted_owned(
                inner.clone(),
                test.clone(),
                value,
                bindings,
            );
        }

        if let Some((excluded, inner)) = pattern_except(&pattern) {
            let inner = inner
                .cloned()
                .unwrap_or_else(|| Expr::call(system_expr("Blank"), vec![]));
            return self.match_except_interpreted_owned(excluded.clone(), inner, value, bindings);
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
            (Expr::Call { head, args }, value) => self.match_call_interpreted_owned(
                head.into_owned(),
                args.into_vec(),
                value,
                bindings,
            ),
            _ => Ok(interpreted_match_outcome(false, bindings)),
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_call_args_interpreted_owned(
        &mut self,
        pattern_head: Expr,
        pattern_args: Vec<Expr>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let compiled_args = compile_matcher_args(&pattern_head, &pattern_args);
        self.match_compiled_args_owned(compiled_args, value_args, bindings)
    }
}
