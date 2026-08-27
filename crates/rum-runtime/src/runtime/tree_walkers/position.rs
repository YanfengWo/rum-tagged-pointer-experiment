use super::*;

impl Session {
    pub(in crate::runtime) fn eval_position(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Position", args_slice));
        }
        if !(2..=5).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Position", args_slice, 2, 5));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = self.suspend_with_frame_lazy(target_result, || {
            let args = args.clone();
            move |session, target| {
                session.resume_position_after_target(
                    target,
                    args[1].clone(),
                    args[2..].to_vec(),
                    args.as_ref().to_vec(),
                )
            }
        })?;

        self.resume_position_after_target(
            target,
            args_slice[1].clone(),
            args_slice[2..].to_vec(),
            args_slice.to_vec(),
        )
    }

    fn resume_position_after_target(
        &mut self,
        target: Expr,
        pattern_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame_lazy(pattern_result, || {
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
                return Ok(self.invalid_cases_traversal_call("Position", &call_args, error));
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
                        0,
                        expr,
                        pattern,
                        traversal,
                        level + 1,
                        path,
                        seen_before,
                        Vec::new(),
                        1,
                        None,
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
                    0,
                    expr,
                    pattern,
                    traversal,
                    level + 1,
                    path,
                    seen_before,
                    Vec::new(),
                    1,
                    None,
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
            self.match_expr_owned(pattern, expr, SymbolExprMap::default())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: SymbolExprMap::default(),
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

    /// Walks the children one per loop iteration, keeping the accumulated
    /// positions in place instead of recursing per element. `pending` carries
    /// an already-computed child outcome across a suspension resume, so the
    /// merge step runs exactly once per element on both the live and the
    /// resumed paths.
    fn collect_position_children(
        &mut self,
        children: Vec<(Expr, Vec<Expr>, bool)>,
        mut child_index: usize,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        path: Vec<Expr>,
        seen_before: usize,
        mut positions: Vec<Expr>,
        mut max_child_depth: usize,
        mut pending: Option<PositionCollectionOutcome>,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                positions.extend(child_outcome.positions);
                let contributes_depth = children
                    .get(child_index)
                    .is_some_and(|(_, _, contributes_depth)| *contributes_depth);
                if contributes_depth {
                    max_child_depth = max_child_depth.max(child_outcome.depth);
                }

                if child_outcome.exhausted {
                    return Ok(PositionCollectionOutcome::new(
                        positions,
                        max_child_depth + 1,
                        true,
                    ));
                }

                child_index += 1;
            }

            let Some((child, child_path, _contributes_depth)) = children.get(child_index).cloned()
            else {
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
                child,
                pattern.clone(),
                traversal,
                level,
                child_path,
                seen_before + positions.len(),
            );

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = PositionCollectionOutcome::from_continuation_expr(value?);
                        session
                            .collect_position_children(
                                children,
                                child_index,
                                expr,
                                pattern,
                                traversal,
                                level,
                                path,
                                seen_before,
                                positions,
                                max_child_depth,
                                Some(outcome),
                            )
                            .map(|outcome| outcome.into_continuation_expr())
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
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
