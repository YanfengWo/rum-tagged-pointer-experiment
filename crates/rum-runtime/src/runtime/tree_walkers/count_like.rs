use super::*;

impl Session {
    pub(in crate::runtime) fn eval_count(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("Count", args));
        }
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Count", args, 2, 4));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let pattern_expr = args[1].clone();
            let option_exprs = args[2..].to_vec();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_count_after_target(target, pattern_expr, option_exprs, call_args)
            }
        })?;

        self.resume_count_after_target(target, args[1].clone(), args[2..].to_vec(), call_args)
    }

    pub(in crate::runtime) fn eval_member_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("MemberQ", args));
        }
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("MemberQ", args, 2, 4));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let pattern_expr = args[1].clone();
            let option_exprs = args[2..].to_vec();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_member_q_after_target(target, pattern_expr, option_exprs, call_args)
            }
        })?;

        self.resume_member_q_after_target(target, args[1].clone(), args[2..].to_vec(), call_args)
    }

    pub(in crate::runtime) fn eval_free_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("FreeQ", args));
        }
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("FreeQ", args, 2, 4));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let pattern_expr = args[1].clone();
            let option_exprs = args[2..].to_vec();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_free_q_after_target(target, pattern_expr, option_exprs, call_args)
            }
        })?;

        self.resume_free_q_after_target(target, args[1].clone(), args[2..].to_vec(), call_args)
    }

    fn resume_count_after_target(
        &mut self,
        target: Expr,
        pattern_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame(pattern_result, {
            let target = target.clone();
            let option_exprs = option_exprs.clone();
            let call_args = call_args.clone();
            move |session, pattern| {
                session.resume_count_after_pattern(target, pattern, option_exprs, call_args)
            }
        })?;

        self.resume_count_after_pattern(target, pattern, option_exprs, call_args)
    }

    fn resume_count_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("Count", "Heads", false)?;
        self.finish_count_like_after_pattern(
            "Count",
            target,
            pattern,
            option_exprs,
            call_args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_cases()
            },
            |count| Expr::integer(BigInt::from(count)),
        )
    }

    fn resume_member_q_after_target(
        &mut self,
        target: Expr,
        pattern_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame(pattern_result, {
            let target = target.clone();
            let option_exprs = option_exprs.clone();
            let call_args = call_args.clone();
            move |session, pattern| {
                session.resume_member_q_after_pattern(target, pattern, option_exprs, call_args)
            }
        })?;

        self.resume_member_q_after_pattern(target, pattern, option_exprs, call_args)
    }

    fn resume_member_q_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("MemberQ", "Heads", false)?;
        self.finish_count_like_after_pattern(
            "MemberQ",
            target,
            pattern,
            option_exprs,
            call_args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_member_q()
            },
            |count| system_expr(if count > 0 { "True" } else { "False" }),
        )
    }

    fn resume_free_q_after_target(
        &mut self,
        target: Expr,
        pattern_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame(pattern_result, {
            let target = target.clone();
            let option_exprs = option_exprs.clone();
            let call_args = call_args.clone();
            move |session, pattern| {
                session.resume_free_q_after_pattern(target, pattern, option_exprs, call_args)
            }
        })?;

        self.resume_free_q_after_pattern(target, pattern, option_exprs, call_args)
    }

    fn resume_free_q_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("FreeQ", "Heads", true)?;
        self.finish_count_like_after_pattern(
            "FreeQ",
            target,
            pattern,
            option_exprs,
            call_args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_free_q()
            },
            |count| system_expr(if count == 0 { "True" } else { "False" }),
        )
    }

    fn finish_count_like_after_pattern<F>(
        &mut self,
        builtin_name: &'static str,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
        defaults: CasesTraversalConfig,
        finish: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(usize) -> Expr + Send + 'static,
    {
        let traversal_result = self.resolve_traversal_without_limit_owned(option_exprs, defaults);
        self.drive_internal_value(traversal_result, move |session, traversal| {
            let traversal = match traversal {
                TraversalWithoutLimitResolution::Valid(traversal) => traversal,
                TraversalWithoutLimitResolution::Invalid(error) => {
                    return Ok(session.invalid_traversal_without_limit_call(
                        builtin_name,
                        &call_args,
                        error,
                    ));
                }
            };

            let outcome_result =
                session.count_pattern_matches_owned(target, pattern, traversal, 0, 0);
            session
                .drive_internal_value(outcome_result, move |_, outcome| Ok(finish(outcome.count)))
        })
    }

    fn count_pattern_matches_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
    ) -> Result<TraversalCountOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_count_at_current_node(
                expr,
                pattern,
                traversal,
                level,
                1,
                seen_before,
                0,
            ),
            Expr::Call { head, .. } if is_system_symbol_name(&head, "Association") => {
                if let Some(entries) = association_entries_from_expr(&expr) {
                    let mut children = Vec::new();
                    if traversal.include_heads {
                        children.push((head.as_ref().clone(), false));
                    }
                    children.extend(entries.into_iter().map(|(_, value)| (value, true)));

                    self.count_pattern_children(
                        children,
                        0,
                        expr,
                        pattern,
                        traversal,
                        level + 1,
                        seen_before,
                        0,
                        1,
                    )
                } else {
                    self.count_pattern_matches_in_standard_call(
                        expr,
                        pattern,
                        traversal,
                        level,
                        seen_before,
                    )
                }
            }
            Expr::Call { .. } => self.count_pattern_matches_in_standard_call(
                expr,
                pattern,
                traversal,
                level,
                seen_before,
            ),
        }
    }

    fn count_pattern_matches_in_standard_call(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
    ) -> Result<TraversalCountOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_count_at_current_node(
                expr,
                pattern,
                traversal,
                level,
                1,
                seen_before,
                0,
            ),
            Expr::Call { head, args } => {
                let mut children = Vec::new();
                if traversal.include_heads {
                    children.push((head.into_owned(), false));
                }
                children.extend(args.into_iter().map(|arg| (arg, true)));

                self.count_pattern_children(
                    children,
                    0,
                    expr,
                    pattern,
                    traversal,
                    level + 1,
                    seen_before,
                    0,
                    1,
                )
            }
        }
    }

    fn finish_count_at_current_node(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        depth: usize,
        seen_before: usize,
        count: usize,
    ) -> Result<TraversalCountOutcome, EvalError> {
        if traversal.limit_reached(seen_before + count) {
            return Ok(TraversalCountOutcome::new(count, depth, true));
        }

        let match_result = if traversal.levels.matches(level, depth) {
            self.match_expr_owned(pattern, expr, SymbolExprMap::default())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: SymbolExprMap::default(),
            })
        };

        self.drive_internal_value(match_result, move |_, outcome| {
            let next_count = count + usize::from(outcome.matched);
            let exhausted = traversal.limit_reached(seen_before + next_count);
            Ok(TraversalCountOutcome::new(next_count, depth, exhausted))
        })
    }

    fn count_pattern_children(
        &mut self,
        children: Vec<(Expr, bool)>,
        child_index: usize,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
        count: usize,
        max_child_depth: usize,
    ) -> Result<TraversalCountOutcome, EvalError> {
        let Some((child, contributes_depth)) = children.get(child_index).cloned() else {
            let depth = max_child_depth.saturating_add(1);
            return self.finish_count_at_current_node(
                expr,
                pattern,
                traversal,
                level - 1,
                depth,
                seen_before,
                count,
            );
        };

        let child_result = self.count_pattern_matches_owned(
            child,
            pattern.clone(),
            traversal,
            level,
            seen_before + count,
        );
        self.drive_internal_value(child_result, move |session, child_outcome| {
            let next_count = count + child_outcome.count;
            let next_max_child_depth = if contributes_depth {
                max_child_depth.max(child_outcome.depth)
            } else {
                max_child_depth
            };

            if child_outcome.exhausted {
                return Ok(TraversalCountOutcome::new(
                    next_count,
                    next_max_child_depth + 1,
                    true,
                ));
            }

            session.count_pattern_children(
                children,
                child_index + 1,
                expr,
                pattern,
                traversal,
                level,
                seen_before,
                next_count,
                next_max_child_depth,
            )
        })
    }
}
