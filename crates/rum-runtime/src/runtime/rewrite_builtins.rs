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
        let normalized = strip_hold_pattern(expr);
        let normalized = match normalized {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(normalized.clone()),
            other => Ok(other.clone()),
        }?;

        Ok(self.normalize_match_expr(&normalized))
    }

    fn resolve_cases_spec_expr(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr),
            Expr::Call { ref head, .. }
                if is_system_symbol_name(&head, "Rule")
                    || is_system_symbol_name(&head, "RuleDelayed") =>
            {
                self.evaluate(expr)
            }
            other => Ok(other),
        }
    }

    fn compile_cases_spec_from_resolved(&self, resolved: Expr) -> CasesSpec {
        match self.compile_rewrite_rule_for_matching(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        }
    }

    #[allow(dead_code)]
    pub(super) fn normalize_cases_spec(&mut self, expr: &Expr) -> Result<CasesSpec, EvalError> {
        let resolved = self.resolve_cases_spec_expr(expr.clone())?;
        Ok(self.compile_cases_spec_from_resolved(resolved))
    }

    fn resume_cases_after_target(
        &mut self,
        target: Expr,
        spec_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let spec_result = self.resolve_cases_spec_expr(spec_expr.clone());
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
        let spec = self.compile_cases_spec_from_resolved(resolved_spec);

        match option_exprs.as_slice() {
            [] => self.finish_cases_with_traversal(
                target,
                spec,
                CasesTraversalConfig {
                    levels: CasesLevelSpec::all(),
                    include_heads: false,
                },
            ),
            [third] => {
                let third_result = self.evaluate(third.clone());
                let resolved_third = self.suspend_with_frame(third_result, {
                    let target = target.clone();
                    let spec = spec.clone();
                    let call_args = call_args.clone();
                    move |session, resolved_third| {
                        session.resume_cases_after_third(target, spec, resolved_third, call_args)
                    }
                })?;

                self.resume_cases_after_third(target, spec, resolved_third, call_args)
            }
            [third, fourth] => {
                let third_result = self.evaluate(third.clone());
                let resolved_third = self.suspend_with_frame(third_result, {
                    let target = target.clone();
                    let spec = spec.clone();
                    let fourth = fourth.clone();
                    let call_args = call_args.clone();
                    move |session, resolved_third| {
                        session.resume_cases_after_third_and_fourth(
                            target,
                            spec,
                            resolved_third,
                            fourth,
                            call_args,
                        )
                    }
                })?;

                self.resume_cases_after_third_and_fourth(
                    target,
                    spec,
                    resolved_third,
                    fourth.clone(),
                    call_args,
                )
            }
            _ => unreachable!(),
        }
    }

    fn resume_cases_after_third(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        resolved_third: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = if let Some(include_heads) = parse_cases_heads_option(&resolved_third) {
            CasesTraversalConfig {
                levels: CasesLevelSpec::all(),
                include_heads,
            }
        } else if let Some(levels) = parse_cases_level_spec(&resolved_third) {
            CasesTraversalConfig {
                levels,
                include_heads: false,
            }
        } else {
            return Ok(self.invalid_call_with_arg_form(
                "Cases",
                &call_args,
                "expected a non-negative level spec or Heads -> True/False as the third argument",
            ));
        };

        self.finish_cases_with_traversal(target, spec, traversal)
    }

    fn resume_cases_after_third_and_fourth(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        resolved_third: Expr,
        fourth: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(levels) = parse_cases_level_spec(&resolved_third) else {
            return Ok(self.invalid_call_with_arg_form(
                "Cases",
                &call_args,
                "expected a non-negative level spec as the third argument",
            ));
        };

        let fourth_result = self.evaluate(fourth);
        let resolved_fourth = self.suspend_with_frame(fourth_result, {
            let target = target.clone();
            let spec = spec.clone();
            let call_args = call_args.clone();
            move |session, resolved_fourth| {
                session.resume_cases_after_fourth(target, spec, levels, resolved_fourth, call_args)
            }
        })?;

        self.resume_cases_after_fourth(target, spec, levels, resolved_fourth, call_args)
    }

    fn resume_cases_after_fourth(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        levels: CasesLevelSpec,
        resolved_fourth: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(include_heads) = parse_cases_heads_option(&resolved_fourth) else {
            return Ok(self.invalid_call_with_arg_form(
                "Cases",
                &call_args,
                "expected a Heads -> True/False option as the fourth argument",
            ));
        };

        self.finish_cases_with_traversal(
            target,
            spec,
            CasesTraversalConfig {
                levels,
                include_heads,
            },
        )
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
        let normalized = strip_hold_pattern(&pattern_expr);
        let pattern = match normalized {
            Expr::Atom(Atom::Symbol(_)) => {
                let pattern_result = self.evaluate(normalized.clone());
                let resolved = self.suspend_with_frame(pattern_result, {
                    let value = value.clone();
                    move |session, resolved| session.finish_match_q_with_pattern(resolved, value)
                })?;
                return self.finish_match_q_with_pattern(resolved, value);
            }
            other => other.clone(),
        };

        self.finish_match_q_with_pattern(pattern, value)
    }

    fn finish_match_q_with_pattern(
        &mut self,
        pattern: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern = self.normalize_match_expr(&pattern);
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
        let rules_result = self.evaluate(rules_expr.clone());
        let resolved_rules = self.suspend_with_frame(rules_result, {
            let target = target.clone();
            let rules_expr_for_resume = rules_expr.clone();
            move |session, resolved_rules| {
                session.resume_replace_all_after_rules(
                    target,
                    rules_expr_for_resume,
                    resolved_rules,
                )
            }
        })?;

        self.resume_replace_all_after_rules(target, rules_expr, resolved_rules)
    }

    fn resume_replace_all_after_rules(
        &mut self,
        target: Expr,
        _rules_expr: Expr,
        resolved_rules: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(rules) = self.compile_rewrite_rules_for_matching(&resolved_rules) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAll",
                &[target, resolved_rules],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced = self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAll)?;
        let replaced_result = self.evaluate(replaced);
        self.suspend_with_frame(replaced_result, |_, value| Ok(value))
    }

    fn resume_replace_repeated_after_current(
        &mut self,
        current: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let rules_result = self.evaluate(rules_expr.clone());
        let resolved_rules = self.suspend_with_frame(rules_result, {
            let current = current.clone();
            let rules_expr_for_resume = rules_expr.clone();
            move |session, resolved_rules| {
                session.resume_replace_repeated_after_rules(
                    current,
                    rules_expr_for_resume,
                    resolved_rules,
                )
            }
        })?;

        self.resume_replace_repeated_after_rules(current, rules_expr, resolved_rules)
    }

    fn resume_replace_repeated_after_rules(
        &mut self,
        current: Expr,
        resolved_rules_input: Expr,
        resolved_rules: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(rules) = self.compile_rewrite_rules_for_matching(&resolved_rules) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceRepeated",
                &[current, resolved_rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

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
