use super::*;

fn interpreted_match_outcome(matched: bool, bindings: SymbolExprMap) -> CompiledMatchOutcome {
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
            session.evaluate_guard_owned(condition, bindings, snapshot, None)
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
        match inner_result {
            Ok(outcome) => self.finish_interpreted_except_inner(excluded, value, snapshot, outcome),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_interpreted_except_inner(
                            excluded,
                            value,
                            snapshot.into_owned(),
                            outcome,
                        )
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_interpreted_except_inner(
        &mut self,
        excluded: Expr,
        value: Expr,
        snapshot: SymbolExprMap,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        let accepted_bindings = outcome.bindings.clone();
        let excluded_result = self.match_expr_interpreted_owned(excluded, value, outcome.bindings);
        match excluded_result {
            Ok(excluded_outcome) => Ok(if excluded_outcome.matched {
                interpreted_match_outcome(false, snapshot)
            } else {
                interpreted_match_outcome(true, accepted_bindings)
            }),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                let accepted_bindings = FrozenMatchBindings::freeze(accepted_bindings);
                self.push_suspension_frame(move |_, payload| {
                    let excluded_outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    Ok(if excluded_outcome.matched {
                        interpreted_match_outcome(false, snapshot.into_owned())
                    } else {
                        interpreted_match_outcome(true, accepted_bindings.into_owned())
                    }
                    .into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
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
                match head_result {
                    Ok(outcome) => {
                        self.finish_interpreted_call_head(head_expr, args, value_args, outcome)
                    }
                    Err(EvalError::InputRequired { request }) => {
                        let args = SharedArgs::new(args);
                        let value_args = SharedArgs::new(value_args);
                        self.push_suspension_frame(move |session, payload| {
                            let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                            session
                                .finish_interpreted_call_head(
                                    head_expr,
                                    args.into_vec(),
                                    value_args.into_vec(),
                                    outcome,
                                )
                                .map(|outcome| outcome.into_continuation_expr())
                        });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
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

    fn finish_interpreted_call_head(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        value_args: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        self.match_call_args_interpreted_owned(head, args, value_args, outcome.bindings)
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
        match alternative_result {
            Ok(outcome) => self.finish_interpreted_alternative(
                alternatives_for_retry,
                value_for_retry,
                snapshot,
                index,
                outcome,
            ),
            Err(EvalError::InputRequired { request }) => {
                let alternatives = SharedArgs::new(alternatives_for_retry);
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                self.push_suspension_frame(move |session, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    session
                        .finish_interpreted_alternative(
                            alternatives.into_vec(),
                            value_for_retry,
                            snapshot.into_owned(),
                            index,
                            outcome,
                        )
                        .map(|outcome| outcome.into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_interpreted_alternative(
        &mut self,
        alternatives: Vec<Expr>,
        value: Expr,
        snapshot: SymbolExprMap,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.match_alternatives_interpreted_owned(alternatives, value, snapshot, index + 1)
        }
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
        let inner_result =
            self.match_expr_interpreted_inner_owned(normalized_pattern, normalized_value, bindings);
        match inner_result {
            Ok(outcome) => Ok(finish_interpreted_match(snapshot, outcome)),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                self.push_suspension_frame(move |_, payload| {
                    let outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
                    Ok(finish_interpreted_match(snapshot.into_owned(), outcome)
                        .into_continuation_expr())
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
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

fn finish_interpreted_match(
    snapshot: SymbolExprMap,
    outcome: CompiledMatchOutcome,
) -> CompiledMatchOutcome {
    if outcome.matched {
        outcome
    } else {
        interpreted_match_outcome(false, snapshot)
    }
}
