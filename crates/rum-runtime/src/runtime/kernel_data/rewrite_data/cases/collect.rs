use super::*;

impl Session {
    pub(in crate::runtime) fn eval_cases(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=5).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Cases", args, 2, 5));
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
        let mut defaults = CasesTraversalConfig::default_cases();
        defaults.include_heads = self.system_option_bool("Cases", "Heads", false)?;
        let traversal_result = self.resolve_cases_traversal_owned(option_exprs.clone(), defaults);
        self.drive_internal_value(traversal_result, {
            let target = target.clone();
            let spec = spec.clone();
            let call_args = call_args.clone();
            move |session, traversal| {
                session.finish_cases_after_resolved_traversal(target, spec, traversal, call_args)
            }
        })
    }

    fn finish_cases_after_resolved_traversal(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalResolution,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("Cases", &call_args, error));
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
        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let matches_result = self.collect_cases_owned(target, spec, traversal, 0, 0);
        self.drive_internal_value(matches_result, move |_, outcome| {
            Ok(Expr::call(system_expr("List"), outcome.matches))
        })
    }

    fn collect_cases_owned(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_cases_at_current_node(
                expr,
                spec,
                traversal,
                level,
                1,
                seen_before,
                Vec::new(),
            ),
            Expr::Call { head, .. } if is_system_symbol_name(&head, "Association") => {
                if let Some(entries) = association_entries_from_expr(&expr) {
                    let mut children = Vec::new();
                    if traversal.include_heads {
                        children.push((head.as_ref().clone(), false));
                    }
                    children.extend(entries.into_iter().map(|(_, value)| (value, true)));

                    self.collect_cases_children(
                        children,
                        expr,
                        spec,
                        traversal,
                        level + 1,
                        seen_before,
                        Vec::new(),
                        1,
                    )
                } else {
                    self.collect_cases_in_standard_call(expr, spec, traversal, level, seen_before)
                }
            }
            Expr::Call { .. } => {
                self.collect_cases_in_standard_call(expr, spec, traversal, level, seen_before)
            }
        }
    }

    fn collect_cases_in_standard_call(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_cases_at_current_node(
                expr,
                spec,
                traversal,
                level,
                1,
                seen_before,
                Vec::new(),
            ),
            Expr::Call { head, args } => {
                let mut children = Vec::new();
                if traversal.include_heads {
                    children.push((head.into_owned(), false));
                }
                children.extend(args.into_iter().map(|arg| (arg, true)));

                self.collect_cases_children(
                    children,
                    expr,
                    spec,
                    traversal,
                    level + 1,
                    seen_before,
                    Vec::new(),
                    1,
                )
            }
        }
    }

    fn finish_cases_at_current_node(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        depth: usize,
        seen_before: usize,
        mut matches: Vec<Expr>,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        if traversal.limit_reached(seen_before + matches.len()) {
            return Ok(CasesCollectionOutcome::new(matches, depth, true));
        }

        let case_result = if traversal.levels.matches(level, depth) {
            self.try_collect_case_owned(expr, spec)
        } else {
            Ok(None)
        };

        self.drive_internal_value(case_result, move |_, case| {
            if let Some(case) = case {
                matches.push(case);
            }

            let exhausted = traversal.limit_reached(seen_before + matches.len());
            Ok(CasesCollectionOutcome::new(matches, depth, exhausted))
        })
    }

    fn collect_cases_children(
        &mut self,
        children: Vec<(Expr, bool)>,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
        mut matches: Vec<Expr>,
        max_child_depth: usize,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        let Some(((child, contributes_depth), rest)) = children.split_first() else {
            let depth = max_child_depth.saturating_add(1);
            return self.finish_cases_at_current_node(
                expr,
                spec,
                traversal,
                level - 1,
                depth,
                seen_before,
                matches,
            );
        };

        let contributes_depth = *contributes_depth;
        let child_result = self.collect_cases_owned(
            child.clone(),
            spec.clone(),
            traversal,
            level,
            seen_before + matches.len(),
        );
        let rest = rest.to_vec();

        self.drive_internal_value(child_result, move |session, child_outcome| {
            matches.extend(child_outcome.matches);
            let next_max_child_depth = if contributes_depth {
                max_child_depth.max(child_outcome.depth)
            } else {
                max_child_depth
            };

            if child_outcome.exhausted {
                return Ok(CasesCollectionOutcome::new(
                    matches,
                    next_max_child_depth + 1,
                    true,
                ));
            }

            session.collect_cases_children(
                rest,
                expr,
                spec,
                traversal,
                level,
                seen_before,
                matches,
                next_max_child_depth,
            )
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
                let match_result =
                    self.match_expr_owned(pattern, expr.clone(), SymbolExprMap::default());
                self.drive_internal_value(match_result, move |_, outcome| {
                    Ok(if outcome.matched { Some(expr) } else { None })
                })
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    SymbolExprMap::default(),
                );
                self.drive_internal_value(match_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(None);
                    }

                    let transformed = substitute_rule_bindings_for_rhs(
                        &rule.rhs,
                        &outcome.bindings,
                        rule.rhs_may_splice_without_bindings,
                    );
                    let transformed_result = session.evaluate(transformed);
                    session
                        .suspend_with_frame(transformed_result, |_, value| Ok(value))
                        .map(Some)
                })
            }
        }
    }
}
