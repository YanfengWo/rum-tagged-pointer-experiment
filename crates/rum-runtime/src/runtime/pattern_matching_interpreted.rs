use super::*;

impl Session {
    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let normalized_pattern = self.normalize_match_expr(pattern);
        let normalized_value = self.normalize_match_expr(value);
        let snapshot = bindings.clone();
        let matched =
            self.match_expr_interpreted_inner(&normalized_pattern, &normalized_value, bindings)?;

        if !matched {
            *bindings = snapshot;
        }

        Ok(matched)
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_inner(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(pattern) {
            if !self.match_expr_interpreted(inner, value, bindings)? {
                return Ok(false);
            }

            return Ok(bind_pattern_name(name, value.clone(), bindings));
        }

        if let Some((inner, condition)) = pattern_condition(pattern) {
            if !self.match_expr_interpreted(inner, value, bindings)? {
                return Ok(false);
            }

            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        if let Some(blank_head) = pattern_blank_head(pattern) {
            return Ok(match_blank(blank_head, value));
        }

        match (pattern, value) {
            (Expr::Atom(lhs), Expr::Atom(rhs)) => Ok(lhs == rhs),
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
                if !self.match_expr_interpreted(lhs_head, rhs_head, bindings)? {
                    return Ok(false);
                }

                self.match_call_args_interpreted(lhs_args, rhs_args, bindings)
            }
            (Expr::Call { head, args }, _)
                if matcher_head_has_attribute(
                    &MatcherNode::Exact((**head).clone()),
                    self,
                    Attribute::OneIdentity,
                ) =>
            {
                match args.as_slice() {
                    [single] => self.match_expr_interpreted(single, value, bindings),
                    _ => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_call_args_interpreted(
        &mut self,
        pattern_args: &[Expr],
        value_args: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if pattern_args.is_empty() {
            return Ok(value_args.is_empty());
        }

        if let Some(spec) = decompose_sequence_pattern(&pattern_args[0]) {
            let rest = &pattern_args[1..];
            let min_rest = min_required_arg_count(rest);

            if value_args.len() < spec.min_len + min_rest {
                return Ok(false);
            }

            let max_len = value_args.len() - min_rest;

            for consume in spec.min_len..=max_len {
                let snapshot = bindings.clone();

                if self.match_sequence_pattern_interpreted(
                    spec,
                    &value_args[..consume],
                    bindings,
                )? && self.match_call_args_interpreted(rest, &value_args[consume..], bindings)?
                {
                    return Ok(true);
                }

                *bindings = snapshot;
            }

            return Ok(false);
        }

        let Some((value, rest)) = value_args.split_first() else {
            return Ok(false);
        };

        if !self.match_expr_interpreted(&pattern_args[0], value, bindings)? {
            return Ok(false);
        }

        self.match_call_args_interpreted(&pattern_args[1..], rest, bindings)
    }

    #[allow(dead_code)]
    pub(super) fn match_sequence_pattern_interpreted(
        &mut self,
        spec: SequencePatternSpec<'_>,
        values: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if values.len() < spec.min_len {
            return Ok(false);
        }

        if !values
            .iter()
            .all(|value| match_blank(spec.head_constraint, value))
        {
            return Ok(false);
        }

        if let Some(name) = spec.name {
            let bound_value = sequence_expr(values.to_vec());

            if !bind_pattern_name(name, bound_value, bindings) {
                return Ok(false);
            }
        }

        if let Some(condition) = spec.condition {
            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        Ok(true)
    }
}
