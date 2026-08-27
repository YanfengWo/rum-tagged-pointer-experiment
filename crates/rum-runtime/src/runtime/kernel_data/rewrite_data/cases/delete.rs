use super::*;

impl Session {
    pub(in crate::runtime) fn eval_delete_cases(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("DeleteCases", args_slice));
        }
        if !(2..=5).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("DeleteCases", args_slice, 2, 5));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = self.suspend_with_frame_lazy(target_result, || {
            let args = args.clone();
            move |session, target| {
                session.resume_delete_cases_after_target(
                    target,
                    args[1].clone(),
                    args[2..].to_vec(),
                    args.as_ref().to_vec(),
                )
            }
        })?;

        self.resume_delete_cases_after_target(
            target,
            args_slice[1].clone(),
            args_slice[2..].to_vec(),
            args_slice.to_vec(),
        )
    }

    fn resume_delete_cases_after_target(
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
                session.resume_delete_cases_after_pattern(target, pattern, option_exprs, call_args)
            }
        })?;

        self.resume_delete_cases_after_pattern(target, pattern, option_exprs, call_args)
    }

    fn resume_delete_cases_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut defaults = CasesTraversalConfig::default_cases();
        defaults.include_heads = self.system_option_bool("DeleteCases", "Heads", false)?;
        let traversal_result = self.resolve_cases_traversal_owned(option_exprs, defaults);
        self.drive_internal_value(traversal_result, {
            let target = target.clone();
            let pattern = pattern.clone();
            let call_args = call_args.clone();
            move |session, traversal| {
                session.finish_delete_cases_after_resolved_traversal(
                    target, pattern, traversal, call_args,
                )
            }
        })
    }

    fn finish_delete_cases_after_resolved_traversal(
        &mut self,
        target: Expr,
        pattern: Expr,
        traversal: CasesTraversalResolution,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("DeleteCases", &call_args, error));
            }
        };

        let deleted_result = self.delete_cases_owned(target, pattern, traversal, 0, 0);
        self.drive_internal_value(deleted_result, move |_, outcome| {
            Ok(match outcome.result {
                DeleteCasesResult::Keep(expr) => expr,
                DeleteCasesResult::Delete => Expr::call(system_expr("Sequence"), Vec::new()),
                DeleteCasesResult::Splice(items) => Expr::call(system_expr("Sequence"), items),
            })
        })
    }

    fn finish_delete_cases_at_current_node(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        depth: usize,
        deleted_before: usize,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        if traversal.limit_reached(deleted_before) {
            return Ok(DeleteCasesOutcome {
                result: DeleteCasesResult::Keep(expr),
                depth,
                deleted_count: 0,
                exhausted: true,
            });
        }

        let match_result = if traversal.levels.matches(level, depth) {
            self.match_expr_owned(pattern, expr.clone(), SymbolExprMap::default())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: SymbolExprMap::default(),
            })
        };

        self.drive_internal_value(match_result, move |_, outcome| {
            if outcome.matched {
                Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Delete,
                    depth,
                    deleted_count: 1,
                    exhausted: traversal.limit_reached(deleted_before + 1),
                })
            } else {
                Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(expr),
                    depth,
                    deleted_count: 0,
                    exhausted: false,
                })
            }
        })
    }

    fn delete_cases_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        match expr {
            Expr::Atom(_) => self.finish_delete_cases_at_current_node(
                expr,
                pattern,
                traversal,
                level,
                1,
                deleted_before,
            ),
            Expr::Call { head, args } if is_system_symbol_name(&head, "Association") => self
                .delete_cases_association_children(
                    head.into_owned(),
                    args.into_vec(),
                    pattern,
                    traversal,
                    level + 1,
                    deleted_before,
                    Vec::new(),
                    1,
                    false,
                ),
            Expr::Call { head, args } => self.delete_cases_call_children(
                head.into_owned(),
                args.into_vec(),
                pattern,
                traversal,
                level + 1,
                deleted_before,
                Vec::new(),
                1,
                false,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_cases_call_children(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
        rebuilt_args: Vec<Expr>,
        max_child_depth: usize,
        head_deleted: bool,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        let mut children = Vec::new();
        if traversal.include_heads {
            children.push((true, head.clone(), false));
        }
        children.extend(args.into_iter().map(|arg| (false, arg, true)));

        self.delete_cases_call_children_from_queue(
            children,
            0,
            pattern,
            traversal,
            level,
            deleted_before,
            deleted_before,
            rebuilt_args,
            max_child_depth,
            Some(head),
            head_deleted,
            None,
        )
    }

    /// Walks the child queue one element per loop iteration, keeping the
    /// rebuilt arguments in place instead of recursing per element. `pending`
    /// carries an already-computed child outcome across a suspension resume,
    /// so the merge step runs exactly once per element on both the live and
    /// the resumed paths.
    #[allow(clippy::too_many_arguments)]
    fn delete_cases_call_children_from_queue(
        &mut self,
        children: Vec<(bool, Expr, bool)>,
        mut child_index: usize,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        subtree_start: usize,
        mut deleted_before: usize,
        mut rebuilt_args: Vec<Expr>,
        mut max_child_depth: usize,
        mut rebuilt_head: Option<Expr>,
        mut head_deleted: bool,
        mut pending: Option<DeleteCasesOutcome>,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                let (is_head, contributes_depth) = match children.get(child_index) {
                    Some((is_head, _, contributes_depth)) => (*is_head, *contributes_depth),
                    None => unreachable!("pending outcome belongs to a live child index"),
                };

                deleted_before += child_outcome.deleted_count;
                if contributes_depth {
                    max_child_depth = max_child_depth.max(child_outcome.depth);
                }

                if is_head {
                    match child_outcome.result {
                        DeleteCasesResult::Keep(expr) => rebuilt_head = Some(expr),
                        DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                            head_deleted = true;
                        }
                    }
                } else {
                    match child_outcome.result {
                        DeleteCasesResult::Keep(expr) => rebuilt_args.push(expr),
                        DeleteCasesResult::Delete => {}
                        DeleteCasesResult::Splice(items) => rebuilt_args.extend(items),
                    }
                }

                if child_outcome.exhausted {
                    let untouched_rest = children
                        .iter()
                        .skip(child_index + 1)
                        .filter_map(|(is_head, expr, _)| (!*is_head).then_some(expr.clone()))
                        .collect::<Vec<_>>();
                    rebuilt_args.extend(untouched_rest);

                    if head_deleted {
                        return Ok(DeleteCasesOutcome {
                            result: DeleteCasesResult::Splice(rebuilt_args),
                            depth: max_child_depth,
                            deleted_count: deleted_before - subtree_start,
                            exhausted: true,
                        });
                    }

                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(Expr::call(
                            rebuilt_head.unwrap_or_else(|| system_expr("Sequence")),
                            rebuilt_args,
                        )),
                        depth: max_child_depth.saturating_add(1),
                        deleted_count: deleted_before - subtree_start,
                        exhausted: true,
                    });
                }

                child_index += 1;
            }

            let Some((_, child, _)) = children.get(child_index).cloned() else {
                if head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Splice(rebuilt_args),
                        depth: max_child_depth,
                        deleted_count: deleted_before - subtree_start,
                        exhausted: traversal.limit_reached(deleted_before),
                    });
                }

                let depth = max_child_depth.saturating_add(1);
                let expr = Expr::call(
                    rebuilt_head.unwrap_or_else(|| system_expr("Sequence")),
                    rebuilt_args,
                );
                let finish_result = self.finish_delete_cases_at_current_node(
                    expr,
                    pattern,
                    traversal,
                    level - 1,
                    depth,
                    deleted_before,
                );
                return self.drive_internal_value(finish_result, move |_, mut outcome| {
                    outcome.deleted_count += deleted_before - subtree_start;
                    Ok(outcome)
                });
            };

            let child_result =
                self.delete_cases_owned(child, pattern.clone(), traversal, level, deleted_before);

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = DeleteCasesOutcome::from_continuation_expr(value?);
                        session
                            .delete_cases_call_children_from_queue(
                                children,
                                child_index,
                                pattern,
                                traversal,
                                level,
                                subtree_start,
                                deleted_before,
                                rebuilt_args,
                                max_child_depth,
                                rebuilt_head,
                                head_deleted,
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

    #[allow(clippy::too_many_arguments)]
    fn delete_cases_association_children(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
        rebuilt_entries: Vec<Expr>,
        max_child_depth: usize,
        head_deleted: bool,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        let mut children = Vec::new();
        if traversal.include_heads {
            children.push((true, None, head.clone(), false));
        }

        for rule in args {
            match &rule {
                Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } if is_system_symbol_name(rule_head, "Rule") && rule_args.len() == 2 => {
                    children.push((
                        false,
                        Some(rule_args[0].clone()),
                        rule_args[1].clone(),
                        true,
                    ));
                }
                _ => children.push((false, None, rule, true)),
            }
        }

        self.delete_cases_association_children_from_queue(
            children,
            0,
            pattern,
            traversal,
            level,
            deleted_before,
            deleted_before,
            rebuilt_entries,
            max_child_depth,
            Some(head),
            head_deleted,
            None,
        )
    }

    /// Iterative association-entry queue walk; `pending` carries an
    /// already-computed entry outcome across a suspension resume, so the merge
    /// step runs exactly once per entry on both the live and the resumed
    /// paths.
    #[allow(clippy::too_many_arguments)]
    fn delete_cases_association_children_from_queue(
        &mut self,
        children: Vec<(bool, Option<Expr>, Expr, bool)>,
        mut child_index: usize,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        subtree_start: usize,
        mut deleted_before: usize,
        mut rebuilt_entries: Vec<Expr>,
        mut max_child_depth: usize,
        mut rebuilt_head: Option<Expr>,
        mut head_deleted: bool,
        mut pending: Option<DeleteCasesOutcome>,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                let (is_head, entry_key, contributes_depth) = match children.get(child_index) {
                    Some((is_head, entry_key, _, contributes_depth)) => {
                        (*is_head, entry_key.clone(), *contributes_depth)
                    }
                    None => unreachable!("pending outcome belongs to a live child index"),
                };

                deleted_before += child_outcome.deleted_count;
                if contributes_depth {
                    max_child_depth = max_child_depth.max(child_outcome.depth);
                }

                if is_head {
                    match child_outcome.result {
                        DeleteCasesResult::Keep(expr) => rebuilt_head = Some(expr),
                        DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                            head_deleted = true;
                        }
                    }
                } else if let Some(key) = entry_key {
                    if let DeleteCasesResult::Keep(value) = child_outcome.result {
                        rebuilt_entries.push(Expr::call(system_expr("Rule"), vec![key, value]));
                    }
                } else {
                    match child_outcome.result {
                        DeleteCasesResult::Keep(expr) => rebuilt_entries.push(expr),
                        DeleteCasesResult::Delete => {}
                        DeleteCasesResult::Splice(items) => rebuilt_entries.extend(items),
                    }
                }

                if head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(render_association(Vec::new())),
                        depth: 2,
                        deleted_count: deleted_before - subtree_start,
                        exhausted: traversal.limit_reached(deleted_before),
                    });
                }

                if child_outcome.exhausted {
                    let untouched_rest = children
                        .iter()
                        .skip(child_index + 1)
                        .filter_map(|(is_head, key, expr, _)| {
                            if *is_head {
                                None
                            } else if let Some(key) = key {
                                Some(Expr::call(
                                    system_expr("Rule"),
                                    vec![key.clone(), expr.clone()],
                                ))
                            } else {
                                Some(expr.clone())
                            }
                        })
                        .collect::<Vec<_>>();
                    rebuilt_entries.extend(untouched_rest);

                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(Expr::call(
                            rebuilt_head.unwrap_or_else(|| system_expr("Association")),
                            rebuilt_entries,
                        )),
                        depth: max_child_depth.saturating_add(1),
                        deleted_count: deleted_before - subtree_start,
                        exhausted: true,
                    });
                }

                child_index += 1;
            }

            let Some((_, _, child, _)) = children.get(child_index).cloned() else {
                if head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(render_association(Vec::new())),
                        depth: 2,
                        deleted_count: deleted_before - subtree_start,
                        exhausted: traversal.limit_reached(deleted_before),
                    });
                }

                let depth = max_child_depth.saturating_add(1);
                let expr = Expr::call(
                    rebuilt_head.unwrap_or_else(|| system_expr("Association")),
                    rebuilt_entries,
                );
                let finish_result = self.finish_delete_cases_at_current_node(
                    expr,
                    pattern,
                    traversal,
                    level - 1,
                    depth,
                    deleted_before,
                );
                return self.drive_internal_value(finish_result, move |_, mut outcome| {
                    outcome.deleted_count += deleted_before - subtree_start;
                    Ok(outcome)
                });
            };

            let child_result =
                self.delete_cases_owned(child, pattern.clone(), traversal, level, deleted_before);

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = DeleteCasesOutcome::from_continuation_expr(value?);
                        session
                            .delete_cases_association_children_from_queue(
                                children,
                                child_index,
                                pattern,
                                traversal,
                                level,
                                subtree_start,
                                deleted_before,
                                rebuilt_entries,
                                max_child_depth,
                                rebuilt_head,
                                head_deleted,
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
