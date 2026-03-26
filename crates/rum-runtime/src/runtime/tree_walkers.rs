use super::*;

impl Session {
    pub(super) fn eval_level(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Level", args, 2, 4));
        }

        let Some(levels) = parse_standard_level_spec(&args[1]) else {
            return Ok(self.invalid_call_with_arg_form(
                "Level",
                args,
                "expected a standard levelspec as the second argument",
            ));
        };

        let (apply_head, include_heads) = match args {
            [_, _, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (None, include_heads)
                } else {
                    (Some(third.clone()), false)
                }
            }
            [_, _, third, fourth] => {
                let Some(include_heads) = parse_heads_option(fourth) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Level",
                        args,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };

                (Some(third.clone()), include_heads)
            }
            _ => (None, false),
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

    fn resume_position_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal_result = self.resolve_position_traversal_owned(option_exprs);
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

    fn collect_positions_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        path: Vec<usize>,
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
                    children.push(((*head).clone(), extend_position(&path, 0)));
                }
                for (index, arg) in args.into_iter().enumerate() {
                    children.push((arg, extend_position(&path, index + 1)));
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
                    0,
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
        path: Vec<usize>,
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
        children: Vec<(Expr, Vec<usize>)>,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        path: Vec<usize>,
        seen_before: usize,
        mut positions: Vec<Expr>,
        max_child_depth: usize,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        let Some(((child, child_path), rest)) = children.split_first() else {
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
            let next_max_child_depth = max_child_depth.max(child_outcome.depth);

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
        Expr::Call { head, args } => {
            let mut parts = Vec::new();
            let mut max_child_depth = 0;

            if traversal.include_heads {
                let head_result = collect_level_parts(head, traversal, level + 1);
                max_child_depth = max_child_depth.max(head_result.depth);
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

fn extend_position(path: &[usize], next: usize) -> Vec<usize> {
    let mut extended = path.to_vec();
    extended.push(next);
    extended
}

fn position_expr(path: &[usize]) -> Expr {
    Expr::call(
        system_expr("List"),
        path.iter()
            .map(|index| Expr::integer(BigInt::from(*index)))
            .collect(),
    )
}
