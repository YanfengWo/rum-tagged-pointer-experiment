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
                    head.into_owned(),
                    args.into_vec(),
                    rules,
                    levels,
                    child_level,
                    Vec::new(),
                    1,
                    0,
                ),
            Expr::Call { head, args } => self.replace_call_children_owned(
                head.into_owned(),
                args.into_vec(),
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
