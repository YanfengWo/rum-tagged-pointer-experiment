use super::*;

impl Session {
    pub(super) fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MatchQ", args, 2));
        }

        let value = self.evaluate(args[0].clone())?;
        let pattern = self.normalize_pattern_expr(&args[1])?;
        let mut bindings = BTreeMap::new();

        Ok(system_expr(
            if self.match_expr(&pattern, &value, &mut bindings)? {
                "True"
            } else {
                "False"
            },
        ))
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

        let target = self.evaluate(args[0].clone())?;
        let rules_expr = self.evaluate(args[1].clone())?;

        let Some(rules) = self.compile_rewrite_rules_for_matching(&rules_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAll",
                &[target, rules_expr],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced = self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAll)?;
        self.evaluate(replaced)
    }

    pub(super) fn eval_replace_repeated(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceRepeated", args, 2));
        }

        let mut current = self.evaluate(args[0].clone())?;
        let rules_expr = self.evaluate(args[1].clone())?;

        let Some(rules) = self.compile_rewrite_rules_for_matching(&rules_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceRepeated",
                &[current, rules_expr],
                "expected a rule or list of rules as the second argument",
            ));
        };

        for _ in 0..self.resource_limits.replace_repeated_limit {
            let replaced = self.apply_replace_all(
                current.clone(),
                &rules,
                RuleApplicationSource::ReplaceRepeated,
            )?;
            let next = self.evaluate(replaced)?;

            if next == current {
                return Ok(next);
            }

            current = next;
        }

        Err(EvalError::IterationLimitExceeded {
            limit: self.resource_limits.replace_repeated_limit,
        })
    }

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
                    self.evaluate(transformed).map(Some)
                } else {
                    Ok(None)
                }
            }
        }
    }
}
