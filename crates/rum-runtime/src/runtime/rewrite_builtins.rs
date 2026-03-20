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

        let target = self.evaluate(args[0].clone())?;
        let spec = self.normalize_cases_spec(&args[1])?;
        let traversal = match args {
            [_, _] => CasesTraversalConfig {
                levels: CasesLevelSpec::all(),
                include_heads: false,
            },
            [_, _, third] => {
                let resolved_third = self.evaluate(third.clone())?;

                if let Some(include_heads) = parse_cases_heads_option(&resolved_third) {
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
                        args,
                        "expected a non-negative level spec or Heads -> True/False as the third argument",
                    ));
                }
            }
            [_, _, third, fourth] => {
                let resolved_third = self.evaluate(third.clone())?;
                let Some(levels) = parse_cases_level_spec(&resolved_third) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Cases",
                        args,
                        "expected a non-negative level spec as the third argument",
                    ));
                };
                let resolved_fourth = self.evaluate(fourth.clone())?;
                let Some(include_heads) = parse_cases_heads_option(&resolved_fourth) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Cases",
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };

                CasesTraversalConfig {
                    levels,
                    include_heads,
                }
            }
            _ => unreachable!(),
        };
        let mut matches = Vec::new();

        self.collect_cases(&target, &spec, traversal, 0, &mut matches)?;

        Ok(Expr::call(system_expr("List"), matches))
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

    pub(super) fn normalize_cases_spec(&mut self, expr: &Expr) -> Result<CasesSpec, EvalError> {
        let resolved = match expr {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr.clone())?,
            Expr::Call { head, .. }
                if is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed") =>
            {
                self.evaluate(expr.clone())?
            }
            _ => expr.clone(),
        };

        Ok(match self.compile_rewrite_rule_for_matching(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        })
    }

    pub(super) fn collect_cases(
        &mut self,
        expr: &Expr,
        spec: &CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        matches: &mut Vec<Expr>,
    ) -> Result<(), EvalError> {
        if traversal.levels.contains(level)
            && let Some(case) = self.try_collect_case(expr, spec)?
        {
            matches.push(case);
        }

        if let Expr::Call { head, args } = expr {
            if traversal.include_heads {
                self.collect_cases(head, spec, traversal, level + 1, matches)?;
            }

            for arg in args {
                self.collect_cases(arg, spec, traversal, level + 1, matches)?;
            }
        }

        Ok(())
    }

    pub(super) fn try_collect_case(
        &mut self,
        expr: &Expr,
        spec: &CasesSpec,
    ) -> Result<Option<Expr>, EvalError> {
        let mut bindings = BTreeMap::new();
        let normalized_expr = self.normalize_match_expr(expr);

        match spec {
            CasesSpec::Pattern(pattern) => {
                if self.match_expr(pattern, expr, &mut bindings)? {
                    Ok(Some(expr.clone()))
                } else {
                    Ok(None)
                }
            }
            CasesSpec::Rule(rule) => {
                if self.match_compiled(&rule.matcher, &normalized_expr, &mut bindings)? {
                    let transformed = substitute_rule_bindings(&rule.rhs, &bindings);
                    let transformed_result = self.evaluate(transformed);
                    self.suspend_with_frame(transformed_result, |_, value| Ok(value))
                        .map(Some)
                } else {
                    Ok(None)
                }
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
        let mut bindings = BTreeMap::new();

        Ok(system_expr(
            if self.match_expr(&pattern, &value, &mut bindings)? {
                "True"
            } else {
                "False"
            },
        ))
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
