use super::*;

impl Session {
    pub(super) fn eval_count(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_member_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_free_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_level(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Level", args, 2, 4));
        }

        let default_include_heads = self.system_option_bool("Level", "Heads", false)?;
        let Some(levels) = parse_level_level_spec(&args[1]) else {
            return Ok(self.invalid_call_with_arg_form(
                "Level",
                args,
                "expected a standard levelspec as the second argument",
            ));
        };

        let (apply_head, include_heads) = match args {
            [_, _, third] => {
                if let Some(include_heads) =
                    parse_heads_option_with_default(third, default_include_heads)
                {
                    (None, include_heads)
                } else {
                    (Some(third.clone()), default_include_heads)
                }
            }
            [_, _, third, fourth] => {
                let Some(include_heads) =
                    parse_heads_option_with_default(fourth, default_include_heads)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Level",
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };

                (Some(third.clone()), include_heads)
            }
            _ => (None, default_include_heads),
        };

        let traversal = CasesTraversalConfig {
            levels,
            max_matches: None,
            include_heads,
        };
        let parts = collect_level_parts(&args[0], traversal, 0).parts;
        Ok(Expr::call(
            apply_head.unwrap_or_else(|| system_expr("List")),
            parts,
        ))
    }

    pub(super) fn eval_position(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=5).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Position", args, 2, 5));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let pattern_expr = args[1].clone();
            let option_exprs = args[2..].to_vec();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_position_after_target(target, pattern_expr, option_exprs, call_args)
            }
        })?;

        self.resume_position_after_target(target, args[1].clone(), args[2..].to_vec(), call_args)
    }

    fn resume_position_after_target(
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
                session.resume_position_after_pattern(target, pattern, option_exprs, call_args)
            }
        })?;

        self.resume_position_after_pattern(target, pattern, option_exprs, call_args)
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
        F: FnOnce(usize) -> Expr + 'static,
    {
        let traversal_result = self.resolve_traversal_without_limit_owned(option_exprs, defaults);
        self.drive_internal_value(traversal_result, move |session, traversal| {
            let traversal = match traversal {
                TraversalWithoutLimitResolution::Valid(traversal) => traversal,
                TraversalWithoutLimitResolution::Invalid(error) => {
                    return Ok(session.invalid_call_with_arg_form(
                        builtin_name,
                        &call_args,
                        error.message(),
                    ));
                }
            };

            let outcome_result =
                session.count_pattern_matches_owned(target, pattern, traversal, 0, 0);
            session
                .drive_internal_value(outcome_result, move |_, outcome| Ok(finish(outcome.count)))
        })
    }

    fn resume_position_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut defaults = CasesTraversalConfig::default_position();
        defaults.include_heads = self.system_option_bool("Position", "Heads", true)?;
        let traversal_result = self.resolve_position_traversal_owned(option_exprs, defaults);
        self.drive_internal_value(traversal_result, {
            let target = target.clone();
            let pattern = pattern.clone();
            let call_args = call_args.clone();
            move |session, traversal| {
                session
                    .finish_position_after_resolved_traversal(target, pattern, traversal, call_args)
            }
        })
    }

    fn finish_position_after_resolved_traversal(
        &mut self,
        target: Expr,
        pattern: Expr,
        traversal: CasesTraversalResolution,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Position",
                    &call_args,
                    error.message(),
                ));
            }
        };

        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let positions_result =
            self.collect_positions_owned(target, pattern, traversal, 0, Vec::new(), 0);
        self.drive_internal_value(positions_result, move |_, outcome| {
            Ok(Expr::call(system_expr("List"), outcome.positions))
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
                    children.push((*head, false));
                }
                children.extend(args.into_iter().map(|arg| (arg, true)));

                self.count_pattern_children(
                    children,
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
            self.match_expr_owned(pattern, expr, BTreeMap::new())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: BTreeMap::new(),
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
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
        count: usize,
        max_child_depth: usize,
    ) -> Result<TraversalCountOutcome, EvalError> {
        let Some(((child, contributes_depth), rest)) = children.split_first() else {
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

        let contributes_depth = *contributes_depth;
        let child_result = self.count_pattern_matches_owned(
            child.clone(),
            pattern.clone(),
            traversal,
            level,
            seen_before + count,
        );
        let rest = rest.to_vec();

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
                rest,
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

    fn collect_positions_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        path: Vec<Expr>,
        seen_before: usize,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_positions_at_current_node(
                expr,
                pattern,
                traversal,
                level,
                1,
                path,
                seen_before,
                Vec::new(),
            ),
            Expr::Call { head, .. } if is_system_symbol_name(&head, "Association") => {
                if let Some(entries) = association_entries_from_expr(&expr) {
                    let mut children = Vec::new();
                    if traversal.include_heads {
                        children.push((
                            (*head).clone(),
                            extend_position(&path, position_index(0)),
                            false,
                        ));
                    }
                    for (key, value) in entries {
                        children.push((value, extend_position(&path, position_key(key)), true));
                    }

                    self.collect_position_children(
                        children,
                        expr,
                        pattern,
                        traversal,
                        level + 1,
                        path,
                        seen_before,
                        Vec::new(),
                        1,
                    )
                } else {
                    self.collect_positions_in_standard_call(
                        expr,
                        pattern,
                        traversal,
                        level,
                        path,
                        seen_before,
                    )
                }
            }
            Expr::Call { .. } => self.collect_positions_in_standard_call(
                expr,
                pattern,
                traversal,
                level,
                path,
                seen_before,
            ),
        }
    }

    fn collect_positions_in_standard_call(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        path: Vec<Expr>,
        seen_before: usize,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_positions_at_current_node(
                expr,
                pattern,
                traversal,
                level,
                1,
                path,
                seen_before,
                Vec::new(),
            ),
            Expr::Call { head, args } => {
                let mut children = Vec::new();
                if traversal.include_heads {
                    children.push((
                        (*head).clone(),
                        extend_position(&path, position_index(0)),
                        false,
                    ));
                }
                for (index, arg) in args.into_iter().enumerate() {
                    children.push((arg, extend_position(&path, position_index(index + 1)), true));
                }

                self.collect_position_children(
                    children,
                    expr,
                    pattern,
                    traversal,
                    level + 1,
                    path,
                    seen_before,
                    Vec::new(),
                    1,
                )
            }
        }
    }

    fn finish_positions_at_current_node(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        depth: usize,
        path: Vec<Expr>,
        seen_before: usize,
        mut positions: Vec<Expr>,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        if traversal.limit_reached(seen_before + positions.len()) {
            return Ok(PositionCollectionOutcome::new(positions, depth, true));
        }

        let match_result = if traversal.levels.matches(level, depth) {
            self.match_expr_owned(pattern, expr, BTreeMap::new())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: BTreeMap::new(),
            })
        };

        self.drive_internal_value(match_result, move |_, outcome| {
            if outcome.matched {
                positions.push(position_expr(&path));
            }

            let exhausted = traversal.limit_reached(seen_before + positions.len());
            Ok(PositionCollectionOutcome::new(positions, depth, exhausted))
        })
    }

    fn collect_position_children(
        &mut self,
        children: Vec<(Expr, Vec<Expr>, bool)>,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        path: Vec<Expr>,
        seen_before: usize,
        mut positions: Vec<Expr>,
        max_child_depth: usize,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        let Some(((child, child_path, contributes_depth), rest)) = children.split_first() else {
            let depth = max_child_depth.saturating_add(1);
            return self.finish_positions_at_current_node(
                expr,
                pattern,
                traversal,
                level - 1,
                depth,
                path,
                seen_before,
                positions,
            );
        };

        let contributes_depth = *contributes_depth;
        let child_result = self.collect_positions_owned(
            child.clone(),
            pattern.clone(),
            traversal,
            level,
            child_path.clone(),
            seen_before + positions.len(),
        );
        let rest = rest.to_vec();

        self.drive_internal_value(child_result, move |session, child_outcome| {
            positions.extend(child_outcome.positions);
            let next_max_child_depth = if contributes_depth {
                max_child_depth.max(child_outcome.depth)
            } else {
                max_child_depth
            };

            if child_outcome.exhausted {
                return Ok(PositionCollectionOutcome::new(
                    positions,
                    next_max_child_depth + 1,
                    true,
                ));
            }

            session.collect_position_children(
                rest,
                expr,
                pattern,
                traversal,
                level,
                path,
                seen_before,
                positions,
                next_max_child_depth,
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LevelCollection {
    parts: Vec<Expr>,
    depth: usize,
}

fn collect_level_parts(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> LevelCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if traversal.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            LevelCollection { parts, depth: 1 }
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut parts = Vec::new();
                let mut max_child_depth = 1usize;

                if traversal.include_heads {
                    let head_result = collect_level_parts(head, traversal, level + 1);
                    parts.extend(head_result.parts);
                }

                for (_, value) in entries {
                    let value_result = collect_level_parts(&value, traversal, level + 1);
                    max_child_depth = max_child_depth.max(value_result.depth);
                    parts.extend(value_result.parts);
                }

                let depth = max_child_depth.saturating_add(1);
                if traversal.levels.matches(level, depth) {
                    parts.push(expr.clone());
                }

                LevelCollection { parts, depth }
            } else {
                collect_level_parts_in_standard_call(expr, traversal, level)
            }
        }
        Expr::Call { .. } => collect_level_parts_in_standard_call(expr, traversal, level),
    }
}

fn collect_level_parts_in_standard_call(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> LevelCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if traversal.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            LevelCollection { parts, depth: 1 }
        }
        Expr::Call { head, args } => {
            let mut parts = Vec::new();
            let mut max_child_depth = 1;

            if traversal.include_heads {
                let head_result = collect_level_parts(head, traversal, level + 1);
                parts.extend(head_result.parts);
            }

            for arg in args {
                let arg_result = collect_level_parts(arg, traversal, level + 1);
                max_child_depth = max_child_depth.max(arg_result.depth);
                parts.extend(arg_result.parts);
            }

            let depth = max_child_depth.saturating_add(1);
            if traversal.levels.matches(level, depth) {
                parts.push(expr.clone());
            }

            LevelCollection { parts, depth }
        }
    }
}

pub(super) fn collect_level_parts_owned(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> Vec<Expr> {
    collect_level_parts(expr, traversal, level).parts
}

fn parse_level_level_spec(expr: &Expr) -> Option<CasesLevelSpec> {
    if expr_as_signed_isize(expr) == Some(0) {
        return Some(CasesLevelSpec::range(1, Some(0)));
    }

    parse_standard_level_spec(expr)
}

fn extend_position(path: &[Expr], next: Expr) -> Vec<Expr> {
    let mut extended = path.to_vec();
    extended.push(next);
    extended
}

fn position_expr(path: &[Expr]) -> Expr {
    Expr::call(system_expr("List"), path.to_vec())
}

fn position_index(index: usize) -> Expr {
    Expr::integer(BigInt::from(index))
}

fn position_key(key: Expr) -> Expr {
    Expr::call(system_expr("Key"), vec![key])
}
