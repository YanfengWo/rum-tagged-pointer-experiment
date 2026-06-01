use super::*;

impl Session {
    pub(in crate::runtime) fn eval_replace(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Replace", args, 2, 3));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let rules_expr = args[1].clone();
            let levels_expr = args.get(2).cloned();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_replace_after_target(target, rules_expr, levels_expr, call_args)
            }
        })?;

        self.resume_replace_after_target(target, args[1].clone(), args.get(2).cloned(), call_args)
    }

    pub(in crate::runtime) fn eval_replace_list(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [rules_expr] => Ok(replace_list_operator_form(rules_expr.clone())),
            [target_expr, rules_expr] | [target_expr, rules_expr, _] => {
                let call_args = args.to_vec();
                let target_result = self.evaluate(target_expr.clone());
                let target = self.suspend_with_frame(target_result, {
                    let rules_expr = rules_expr.clone();
                    let max_expr = args.get(2).cloned();
                    let call_args = call_args.clone();
                    move |session, target| {
                        session.resume_replace_list_after_target(
                            target, rules_expr, max_expr, call_args,
                        )
                    }
                })?;

                self.resume_replace_list_after_target(
                    target,
                    rules_expr.clone(),
                    args.get(2).cloned(),
                    call_args,
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceList", args, 1, 3)),
        }
    }

    pub(in crate::runtime) fn eval_replace_all(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_replace_repeated(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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
    fn resume_replace_all_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
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

    fn resume_replace_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        levels_expr: Option<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
            session.resume_replace_after_rule_exprs(target, levels_expr, rule_exprs, call_args)
        })
    }

    fn resume_replace_after_rule_exprs(
        &mut self,
        target: Expr,
        levels_expr: Option<Expr>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "Replace",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        let levels_result = self.resolve_replace_level_spec_owned(levels_expr);
        self.drive_internal_value(levels_result, move |session, levels| {
            let Some(levels) = levels else {
                return Ok(session.invalid_call_with_arg_form(
                    "Replace",
                    &call_args,
                    "expected a standard levelspec as the optional third argument",
                ));
            };

            let replaced_result = session.replace_owned(target, rules, levels, 0);
            session.drive_internal_value(replaced_result, move |session, outcome| {
                let evaluated_result = session.evaluate(outcome.expr);
                session.suspend_with_frame(evaluated_result, |_, value| Ok(value))
            })
        })
    }

    fn resume_replace_list_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_expr: Option<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(max_expr) = max_expr else {
            return self.resume_replace_list_after_limit(target, rules_expr, None, call_args);
        };

        let max_result = self.evaluate(max_expr);
        self.drive_evaluated_value(max_result, move |session, resolved_max| {
            let Some(max_results) = parse_replace_list_max_results(&resolved_max) else {
                return Ok(session.invalid_call_with_arg_form(
                    "ReplaceList",
                    &call_args,
                    "expected a non-negative integer or Infinity as the optional third argument",
                ));
            };

            session.resume_replace_list_after_limit(target, rules_expr, max_results, call_args)
        })
    }

    fn resume_replace_list_after_limit(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
            session.resume_replace_list_after_rule_exprs(target, rule_exprs, max_results, call_args)
        })
    }

    fn resume_replace_list_after_rule_exprs(
        &mut self,
        target: Expr,
        normalized_rule_exprs: Option<Vec<Expr>>,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceList",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        let normalized_target = self.normalize_match_expr(&target);
        let results_result = self.collect_replace_list_results_owned(
            target,
            normalized_target,
            rules,
            max_results,
            Vec::new(),
            0,
        );
        self.drive_internal_value(results_result, |_, results| {
            Ok(Expr::call(system_expr("List"), results))
        })
    }

    fn collect_replace_list_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: Vec<RewriteRule>,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        if max_results.is_some_and(|limit| collected.len() >= limit) {
            return Ok(collected);
        }

        let Some(rule) = rules.get(index).cloned() else {
            return Ok(collected);
        };

        let bindings_result = self.collect_replace_list_rule_bindings_owned(
            rule.matcher.clone(),
            normalized_target.clone(),
        );
        self.drive_internal_value(bindings_result, move |session, bindings| {
            if bindings.is_empty() {
                return session.collect_replace_list_results_owned(
                    target,
                    normalized_target,
                    rules,
                    max_results,
                    collected,
                    index + 1,
                );
            }

            session.collect_replace_list_rule_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                index,
                rule,
                bindings,
                0,
            )
        })
    }

    fn collect_replace_list_rule_bindings_owned(
        &mut self,
        matcher: MatcherNode,
        normalized_target: Expr,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match matcher {
            MatcherNode::Call { head, args } => {
                let one_identity = matcher_head_has_attribute(&head, self, Attribute::OneIdentity);
                let orderless = matcher_head_has_attribute(&head, self, Attribute::Orderless);
                let flat = matcher_head_has_attribute(&head, self, Attribute::Flat);

                match normalized_target {
                    Expr::Call {
                        head: value_head,
                        args: value_args,
                    } => {
                        let flat_head = (*value_head).clone();
                        let head_result =
                            self.match_compiled_owned(*head, *value_head, BTreeMap::new());
                        self.drive_internal_value(head_result, move |session, outcome| {
                            if !outcome.matched {
                                return Ok(Vec::new());
                            }

                            if orderless {
                                if flat {
                                    session.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                                        args,
                                        value_args,
                                        outcome.bindings,
                                        flat_head,
                                    )
                                } else {
                                    session.collect_compiled_orderless_arg_matches_owned(
                                        args,
                                        value_args,
                                        outcome.bindings,
                                    )
                                }
                            } else if flat {
                                session.collect_compiled_args_matches_with_flat_head_owned(
                                    args,
                                    value_args,
                                    outcome.bindings,
                                    flat_head,
                                )
                            } else {
                                session.collect_compiled_args_matches_owned(args, value_args, outcome.bindings)
                            }
                        })
                    }
                    other if one_identity => {
                        if orderless {
                            self.collect_compiled_orderless_arg_matches_owned(
                                args,
                                vec![other],
                                BTreeMap::new(),
                            )
                        } else {
                            self.collect_compiled_args_matches_owned(
                                args,
                                vec![other],
                                BTreeMap::new(),
                            )
                        }
                    }
                    _ => Ok(Vec::new()),
                }
            }
            other => {
                let match_result =
                    self.match_compiled_owned(other, normalized_target, BTreeMap::new());
                self.drive_internal_value(match_result, move |_, outcome| {
                    Ok(if outcome.matched {
                        vec![outcome.bindings]
                    } else {
                        Vec::new()
                    })
                })
            }
        }
    }

    fn collect_replace_list_rule_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: Vec<RewriteRule>,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        rule_index: usize,
        rule: RewriteRule,
        bindings: Vec<BTreeMap<Symbol, Expr>>,
        binding_index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        if max_results.is_some_and(|limit| collected.len() >= limit) {
            return Ok(collected);
        }

        let Some(current_bindings) = bindings.get(binding_index).cloned() else {
            return self.collect_replace_list_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                rule_index + 1,
            );
        };

        let replaced = substitute_rule_bindings(&rule.rhs, &current_bindings);
        self.push_trace(TraceEvent::RuleApplied {
            source: RuleApplicationSource::ReplaceList,
            input: target.to_string(),
            rule: rule.lhs.to_string(),
            result: replaced.to_string(),
        });
        let replaced_result = self.evaluate(replaced);
        self.drive_evaluated_value::<Expr, Vec<Expr>, _>(replaced_result, move |session, value| {
            let mut collected = collected;
            collected.push(value);
            session.collect_replace_list_rule_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                rule_index,
                rule,
                bindings,
                binding_index + 1,
            )
        })
    }

    fn resume_replace_repeated_after_current(
        &mut self,
        current: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
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

    fn resolve_replace_level_spec_owned(
        &mut self,
        levels_expr: Option<Expr>,
    ) -> Result<Option<CasesLevelSpec>, EvalError> {
        let Some(levels_expr) = levels_expr else {
            return Ok(Some(CasesLevelSpec::exact(0)));
        };

        let resolved_result = self.evaluate(levels_expr);
        self.drive_evaluated_value(resolved_result, |_, resolved| {
            Ok(parse_standard_level_spec(&resolved))
        })
    }

    fn replace_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        levels: CasesLevelSpec,
        level: usize,
    ) -> Result<ReplaceOutcome, EvalError> {
        let depth = replace_expr_depth(&expr);
        if levels.matches(level, depth) {
            let replaced_result = self.try_apply_rewrite_rules_once_owned(
                expr.clone(),
                rules.clone(),
                RuleApplicationSource::Replace,
            );
            return self.drive_internal_value(replaced_result, move |session, replaced| {
                if let Some(replaced) = replaced {
                    Ok(ReplaceOutcome {
                        depth: replace_expr_depth(&replaced),
                        expr: replaced,
                    })
                } else {
                    session.replace_children_owned(expr, rules, levels, level + 1)
                }
            });
        }

        self.replace_children_owned(expr, rules, levels, level + 1)
    }

    fn replace_children_owned(
        &mut self,
        expr: Expr,
        rules: Vec<RewriteRule>,
        levels: CasesLevelSpec,
        child_level: usize,
    ) -> Result<ReplaceOutcome, EvalError> {
        match expr {
            Expr::Atom(_) => Ok(ReplaceOutcome { expr, depth: 1 }),
            Expr::Call { head, args } if is_system_symbol_name(&head, "Association") => self
                .replace_association_children_owned(
                    *head,
                    args,
                    rules,
                    levels,
                    child_level,
                    Vec::new(),
                    1,
                    0,
                ),
            Expr::Call { head, args } => self.replace_call_children_owned(
                *head,
                args,
                rules,
                levels,
                child_level,
                Vec::new(),
                1,
            ),
        }
    }

    fn replace_call_children_owned(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        rules: Vec<RewriteRule>,
        levels: CasesLevelSpec,
        child_level: usize,
        rebuilt_args: Vec<Expr>,
        max_child_depth: usize,
    ) -> Result<ReplaceOutcome, EvalError> {
        let Some((child, rest)) = args.split_first() else {
            return Ok(ReplaceOutcome {
                expr: Expr::call(head, rebuilt_args),
                depth: max_child_depth.saturating_add(1),
            });
        };

        let child_result = self.replace_owned(child.clone(), rules.clone(), levels, child_level);
        let rest = rest.to_vec();
        self.drive_internal_value(child_result, move |session, outcome| {
            let mut rebuilt_args = rebuilt_args;
            rebuilt_args.push(outcome.expr);
            session.replace_call_children_owned(
                head,
                rest,
                rules,
                levels,
                child_level,
                rebuilt_args,
                max_child_depth.max(outcome.depth),
            )
        })
    }

    fn replace_association_children_owned(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        rules: Vec<RewriteRule>,
        levels: CasesLevelSpec,
        child_level: usize,
        rebuilt_entries: Vec<Expr>,
        max_child_depth: usize,
        index: usize,
    ) -> Result<ReplaceOutcome, EvalError> {
        let Some(entry) = args.get(index).cloned() else {
            return Ok(ReplaceOutcome {
                expr: Expr::call(head, rebuilt_entries),
                depth: max_child_depth.saturating_add(1),
            });
        };

        match entry {
            Expr::Call {
                head: rule_head,
                args: rule_args,
            } if is_system_symbol_name(&rule_head, "Rule") && rule_args.len() == 2 => {
                let key = rule_args[0].clone();
                let value = rule_args[1].clone();
                let value_result = self.replace_owned(value, rules.clone(), levels, child_level);
                self.drive_internal_value(value_result, move |session, outcome| {
                    let mut rebuilt_entries = rebuilt_entries;
                    rebuilt_entries.push(Expr::call(system_expr("Rule"), vec![key, outcome.expr]));
                    session.replace_association_children_owned(
                        head,
                        args,
                        rules,
                        levels,
                        child_level,
                        rebuilt_entries,
                        max_child_depth.max(outcome.depth),
                        index + 1,
                    )
                })
            }
            other => {
                let entry_result = self.replace_owned(other, rules.clone(), levels, child_level);
                self.drive_internal_value(entry_result, move |session, outcome| {
                    let mut rebuilt_entries = rebuilt_entries;
                    rebuilt_entries.push(outcome.expr);
                    session.replace_association_children_owned(
                        head,
                        args,
                        rules,
                        levels,
                        child_level,
                        rebuilt_entries,
                        max_child_depth.max(outcome.depth),
                        index + 1,
                    )
                })
            }
        }
    }
}

fn replace_expr_depth(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { .. } if association_entries_from_expr(expr).is_some() => {
            let value_depth = association_entries_from_expr(expr)
                .expect("association shape checked above")
                .into_iter()
                .map(|(_, value)| replace_expr_depth(&value))
                .max()
                .unwrap_or(0);
            1 + 1.max(value_depth)
        }
        Expr::Call { args, .. } => {
            1 + 1.max(args.iter().map(replace_expr_depth).max().unwrap_or(0))
        }
    }
}

fn parse_replace_list_max_results(expr: &Expr) -> Option<Option<usize>> {
    if is_system_symbol_name(expr, "Infinity") {
        Some(None)
    } else {
        expr_as_non_negative_usize(expr).map(Some)
    }
}

fn replace_list_operator_form(rules_expr: Expr) -> Expr {
    let parameter = Expr::symbol("rum$replaceListArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr("ReplaceList"), vec![parameter, rules_expr]),
        ],
    )
}
