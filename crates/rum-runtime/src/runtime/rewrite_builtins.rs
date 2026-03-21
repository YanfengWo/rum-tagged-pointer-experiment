use super::*;

impl Session {
    pub(super) fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MatchQ", args, 2));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let pattern_expr = args[1].clone();
            move |session, value| session.resume_match_q_after_value(pattern_expr, value)
        })?;

        self.resume_match_q_after_value(args[1].clone(), value)
    }

    pub(super) fn eval_cases(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Cases", args, 2, 4));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let spec_expr = args[1].clone();
            let option_exprs = args[2..].to_vec();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_cases_after_target(target, spec_expr, option_exprs, call_args)
            }
        })?;

        self.resume_cases_after_target(target, args[1].clone(), args[2..].to_vec(), call_args)
    }

    pub(super) fn eval_replace_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceAll", args, 2));
        }

        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let rules_expr = args[1].clone();
            move |session, target| session.resume_replace_all_after_target(target, rules_expr)
        })?;

        self.resume_replace_all_after_target(target, args[1].clone())
    }

    pub(super) fn eval_replace_repeated(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceRepeated", args, 2));
        }

        let current_result = self.evaluate(args[0].clone());
        let current = self.suspend_with_frame(current_result, {
            let rules_expr = args[1].clone();
            move |session, current| {
                session.resume_replace_repeated_after_current(current, rules_expr)
            }
        })?;

        self.resume_replace_repeated_after_current(current, args[1].clone())
    }

    #[allow(dead_code)]
    pub(super) fn normalize_pattern_expr(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        self.resolve_matcher_pattern_expr_owned(expr.clone())
    }

    pub(super) fn resolve_matcher_pattern_expr_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let normalized = strip_hold_pattern(&expr).clone();
        let resolved = match normalized {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(normalized),
            other => Ok(other),
        }?;

        Ok(self.normalize_match_expr(&resolved))
    }

    #[allow(dead_code)]
    pub(super) fn normalize_cases_spec(&mut self, expr: &Expr) -> Result<CasesSpec, EvalError> {
        let resolved = self.resolve_cases_spec_expr_for_matching_owned(expr.clone())?;
        Ok(self.compile_cases_spec_from_resolved_expr(resolved))
    }

    fn resume_cases_after_target(
        &mut self,
        target: Expr,
        spec_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let spec_result = self.resolve_cases_spec_expr_for_matching_owned(spec_expr.clone());
        let resolved_spec = self.suspend_with_frame(spec_result, {
            let target = target.clone();
            let option_exprs = option_exprs.clone();
            let call_args = call_args.clone();
            move |session, resolved_spec| {
                session.resume_cases_after_resolved_spec(
                    target,
                    resolved_spec,
                    option_exprs,
                    call_args,
                )
            }
        })?;

        self.resume_cases_after_resolved_spec(target, resolved_spec, option_exprs, call_args)
    }

    fn resume_cases_after_resolved_spec(
        &mut self,
        target: Expr,
        resolved_spec: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let spec = self.compile_cases_spec_from_resolved_expr(resolved_spec);
        let resolved_option_exprs_result = self.resolve_argument_exprs_owned(option_exprs.clone());
        self.drive_internal_value(resolved_option_exprs_result, {
            let target = target.clone();
            let spec = spec.clone();
            let call_args = call_args.clone();
            move |session, resolved_option_exprs| {
                session.finish_cases_after_resolved_options(
                    target,
                    spec,
                    resolved_option_exprs,
                    call_args,
                )
            }
        })
    }

    fn finish_cases_after_resolved_options(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        resolved_option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = match self
            .compile_cases_traversal_from_resolved_exprs(&resolved_option_exprs)
        {
            Ok(traversal) => traversal,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Cases", &call_args, error.message()));
            }
        };

        self.finish_cases_with_traversal(target, spec, traversal)
    }

    fn finish_cases_with_traversal(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
    ) -> Result<Expr, EvalError> {
        let matches_result = self.collect_cases_owned(target, spec, traversal, 0);
        self.drive_internal_value(matches_result, move |_, matches| {
            Ok(Expr::call(system_expr("List"), matches))
        })
    }

    fn collect_cases_owned(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        let case_result = if traversal.levels.contains(level) {
            self.try_collect_case_owned(expr.clone(), spec.clone())
        } else {
            Ok(None)
        };

        self.drive_internal_value(case_result, move |session, case| {
            let mut matches = Vec::new();
            if let Some(case) = case {
                matches.push(case);
            }

            match expr {
                Expr::Atom(_) => Ok(matches),
                Expr::Call { head, args } => {
                    let mut children = Vec::new();
                    if traversal.include_heads {
                        children.push(*head);
                    }
                    children.extend(args);

                    session.collect_cases_children(children, spec, traversal, level + 1, matches)
                }
            }
        })
    }

    fn collect_cases_children(
        &mut self,
        children: Vec<Expr>,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        mut matches: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        let Some((child, rest)) = children.split_first() else {
            return Ok(matches);
        };

        let child_result = self.collect_cases_owned(child.clone(), spec.clone(), traversal, level);
        let rest = rest.to_vec();

        self.drive_internal_value(child_result, move |session, child_matches| {
            matches.extend(child_matches);
            session.collect_cases_children(rest, spec, traversal, level, matches)
        })
    }

    fn try_collect_case_owned(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);

        match spec {
            CasesSpec::Pattern(pattern) => {
                let match_result = self.match_expr_owned(pattern, expr.clone(), BTreeMap::new());
                self.drive_internal_value(match_result, move |_, outcome| {
                    Ok(if outcome.matched { Some(expr) } else { None })
                })
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    BTreeMap::new(),
                );
                self.drive_internal_value(match_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(None);
                    }

                    let transformed = substitute_rule_bindings(&rule.rhs, &outcome.bindings);
                    let transformed_result = session.evaluate(transformed);
                    session
                        .suspend_with_frame(transformed_result, |_, value| Ok(value))
                        .map(Some)
                })
            }
        }
    }

    fn resume_match_q_after_value(
        &mut self,
        pattern_expr: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame(pattern_result, {
            let value = value.clone();
            move |session, resolved| session.finish_match_q_with_pattern(resolved, value)
        })?;

        self.finish_match_q_with_pattern(pattern, value)
    }

    fn finish_match_q_with_pattern(
        &mut self,
        pattern: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let match_result = self.match_expr_owned(pattern, value, BTreeMap::new());
        self.drive_internal_value(match_result, move |_, outcome| {
            Ok(system_expr(if outcome.matched { "True" } else { "False" }))
        })
    }

    fn resume_replace_all_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let normalized_rule_exprs_result =
            self.resolve_rewrite_rule_exprs_owned(rules_expr.clone());
        self.drive_internal_value(normalized_rule_exprs_result, move |session, rule_exprs| {
            session.resume_replace_all_after_rule_exprs(target, rules_expr, rule_exprs)
        })
    }

    fn resume_replace_all_after_rule_exprs(
        &mut self,
        target: Expr,
        rules_input: Expr,
        normalized_rule_exprs: Option<Vec<Expr>>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAll",
                &[target, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        let replaced = self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAll)?;
        let replaced_result = self.evaluate(replaced);
        self.suspend_with_frame(replaced_result, |_, value| Ok(value))
    }

    fn resume_replace_repeated_after_current(
        &mut self,
        current: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let normalized_rule_exprs_result =
            self.resolve_rewrite_rule_exprs_owned(rules_expr.clone());
        self.drive_internal_value(normalized_rule_exprs_result, move |session, rule_exprs| {
            session.resume_replace_repeated_after_rule_exprs(current, rules_expr, rule_exprs)
        })
    }

    fn resume_replace_repeated_after_rule_exprs(
        &mut self,
        current: Expr,
        rules_input: Expr,
        normalized_rule_exprs: Option<Vec<Expr>>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceRepeated",
                &[current, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        self.resume_replace_repeated_loop(current, rules, 0)
    }

    fn resume_replace_repeated_loop(
        &mut self,
        current: Expr,
        rules: Vec<RewriteRule>,
        iteration: usize,
    ) -> Result<Expr, EvalError> {
        if iteration >= self.resource_limits.replace_repeated_limit {
            return Err(EvalError::IterationLimitExceeded {
                limit: self.resource_limits.replace_repeated_limit,
            });
        }

        let replaced = self.apply_replace_all(
            current.clone(),
            &rules,
            RuleApplicationSource::ReplaceRepeated,
        )?;
        let next_result = self.evaluate(replaced);
        let next = self.suspend_with_frame(next_result, {
            let current = current.clone();
            let rules = rules.clone();
            move |session, next| {
                session.resume_replace_repeated_after_next(current, rules, iteration, next)
            }
        })?;

        self.resume_replace_repeated_after_next(current, rules, iteration, next)
    }

    fn resume_replace_repeated_after_next(
        &mut self,
        current: Expr,
        rules: Vec<RewriteRule>,
        iteration: usize,
        next: Expr,
    ) -> Result<Expr, EvalError> {
        if next == current {
            Ok(next)
        } else {
            self.resume_replace_repeated_loop(next, rules, iteration + 1)
        }
    }
}
