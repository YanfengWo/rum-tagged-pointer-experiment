use super::*;

impl Session {
    pub(super) fn match_expr(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let matcher = self.compile_matcher_for_expr(pattern);
        let normalized_value = self.normalize_match_expr(value);
        self.match_compiled(&matcher, &normalized_value, bindings)
    }

    pub(super) fn match_compiled(
        &mut self,
        matcher: &MatcherNode,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let snapshot = bindings.clone();
        let matched = self.match_compiled_inner(matcher, value, bindings)?;

        if !matched {
            *bindings = snapshot;
        }

        Ok(matched)
    }

    pub(super) fn match_compiled_inner(
        &mut self,
        matcher: &MatcherNode,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        match matcher {
            MatcherNode::Exact(pattern) => Ok(pattern == value),
            MatcherNode::Blank { head_constraint } => {
                Ok(match_blank(head_constraint.as_ref(), value))
            }
            MatcherNode::Named { name, inner } => {
                if !self.match_compiled(inner, value, bindings)? {
                    return Ok(false);
                }

                Ok(bind_pattern_name(name, value.clone(), bindings))
            }
            MatcherNode::Condition { inner, condition } => {
                if !self.match_compiled(inner, value, bindings)? {
                    return Ok(false);
                }

                let condition_expr = substitute_bindings(condition, bindings);
                let result = self.evaluate(condition_expr)?;
                Ok(is_system_symbol_name(&result, "True"))
            }
            MatcherNode::Call { head, args } => match value {
                Expr::Call {
                    head: value_head,
                    args: value_args,
                } => {
                    if !self.match_compiled(head, value_head, bindings)? {
                        return Ok(false);
                    }

                    self.match_compiled_args(args, value_args, bindings)
                }
                _ if matcher_head_has_attribute(head, self, Attribute::OneIdentity) => {
                    match args.as_slice() {
                        [MatcherArg::Single(inner)] => self.match_compiled(inner, value, bindings),
                        _ => Ok(false),
                    }
                }
                _ => Ok(false),
            },
        }
    }

    pub(super) fn match_compiled_args(
        &mut self,
        pattern_args: &[MatcherArg],
        value_args: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if pattern_args.is_empty() {
            return Ok(value_args.is_empty());
        }

        match &pattern_args[0] {
            MatcherArg::Sequence(spec) => {
                let rest = &pattern_args[1..];
                let min_rest = min_required_matcher_arg_count(rest);

                if value_args.len() < spec.min_len + min_rest {
                    return Ok(false);
                }

                let max_len = value_args.len() - min_rest;

                for consume in spec.min_len..=max_len {
                    let snapshot = bindings.clone();

                    if self.match_compiled_sequence_pattern(
                        spec,
                        &value_args[..consume],
                        bindings,
                    )? && self.match_compiled_args(rest, &value_args[consume..], bindings)?
                    {
                        return Ok(true);
                    }

                    *bindings = snapshot;
                }

                Ok(false)
            }
            MatcherArg::Single(pattern) => {
                let Some((value, rest)) = value_args.split_first() else {
                    return Ok(false);
                };

                if !self.match_compiled(pattern, value, bindings)? {
                    return Ok(false);
                }

                self.match_compiled_args(&pattern_args[1..], rest, bindings)
            }
        }
    }

    pub(super) fn match_compiled_sequence_pattern(
        &mut self,
        spec: &CompiledSequencePattern,
        values: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if values.len() < spec.min_len {
            return Ok(false);
        }

        if !values
            .iter()
            .all(|value| match_blank(spec.head_constraint.as_ref(), value))
        {
            return Ok(false);
        }

        if let Some(name) = &spec.name {
            let bound_value = sequence_expr(values.to_vec());

            if !bind_pattern_name(name, bound_value, bindings) {
                return Ok(false);
            }
        }

        if let Some(condition) = &spec.condition {
            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        Ok(true)
    }
}
