use super::*;

impl Session {
    pub(in crate::runtime) fn eval_delete_cases(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=5).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("DeleteCases", args, 2, 5));
        }

        let call_args = args.to_vec();
        let target_result = self.evaluate(args[0].clone());
        let target = self.suspend_with_frame(target_result, {
            let pattern_expr = args[1].clone();
            let option_exprs = args[2..].to_vec();
            let call_args = call_args.clone();
            move |session, target| {
                session.resume_delete_cases_after_target(
                    target,
                    pattern_expr,
                    option_exprs,
                    call_args,
                )
            }
        })?;

        self.resume_delete_cases_after_target(
            target,
            args[1].clone(),
            args[2..].to_vec(),
            call_args,
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
        let pattern = self.suspend_with_frame(pattern_result, {
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
            self.match_expr_owned(pattern, expr.clone(), BTreeMap::new())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: BTreeMap::new(),
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
                    *head,
                    args,
                    pattern,
                    traversal,
                    level + 1,
                    deleted_before,
                    Vec::new(),
                    1,
                    false,
                ),
            Expr::Call { head, args } => self.delete_cases_call_children(
                *head,
                args,
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
            pattern,
            traversal,
            level,
            deleted_before,
            deleted_before,
            rebuilt_args,
            max_child_depth,
            Some(head),
            head_deleted,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_cases_call_children_from_queue(
        &mut self,
        children: Vec<(bool, Expr, bool)>,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        subtree_start: usize,
        deleted_before: usize,
        rebuilt_args: Vec<Expr>,
        max_child_depth: usize,
        rebuilt_head: Option<Expr>,
        head_deleted: bool,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        if children.is_empty() {
            if head_deleted {
                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Splice(rebuilt_args),
                    depth: max_child_depth,
                    deleted_count: deleted_before - subtree_start,
                    exhausted: traversal.limit_reached(deleted_before),
                });
            }

            let depth = max_child_depth.saturating_add(1);
            let expr = Expr::Call {
                head: Box::new(rebuilt_head.unwrap_or_else(|| system_expr("Sequence"))),
                args: rebuilt_args,
            };
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
        }

        let (is_head, child, contributes_depth) = children[0].clone();
        let rest = children[1..].to_vec();

        let child_result =
            self.delete_cases_owned(child, pattern.clone(), traversal, level, deleted_before);

        self.drive_internal_value(child_result, move |session, child_outcome| {
            let deleted_before_next = deleted_before + child_outcome.deleted_count;
            let next_max_child_depth = if contributes_depth {
                max_child_depth.max(child_outcome.depth)
            } else {
                max_child_depth
            };
            let mut next_args = rebuilt_args;
            let mut next_head = rebuilt_head;
            let mut next_head_deleted = head_deleted;

            if is_head {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => next_head = Some(expr),
                    DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                        next_head_deleted = true;
                    }
                }
            } else {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => next_args.push(expr),
                    DeleteCasesResult::Delete => {}
                    DeleteCasesResult::Splice(items) => next_args.extend(items),
                }
            }

            if child_outcome.exhausted {
                let untouched_rest = rest
                    .iter()
                    .filter_map(|(is_head, expr, _)| (!*is_head).then_some(expr.clone()))
                    .collect::<Vec<_>>();
                next_args.extend(untouched_rest);

                if next_head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Splice(next_args),
                        depth: next_max_child_depth,
                        deleted_count: deleted_before_next - subtree_start,
                        exhausted: true,
                    });
                }

                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(Expr::Call {
                        head: Box::new(next_head.unwrap_or_else(|| system_expr("Sequence"))),
                        args: next_args,
                    }),
                    depth: next_max_child_depth.saturating_add(1),
                    deleted_count: deleted_before_next - subtree_start,
                    exhausted: true,
                });
            }

            session.delete_cases_call_children_from_queue(
                rest,
                pattern,
                traversal,
                level,
                subtree_start,
                deleted_before_next,
                next_args,
                next_max_child_depth,
                next_head,
                next_head_deleted,
            )
        })
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
            pattern,
            traversal,
            level,
            deleted_before,
            deleted_before,
            rebuilt_entries,
            max_child_depth,
            Some(head),
            head_deleted,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_cases_association_children_from_queue(
        &mut self,
        children: Vec<(bool, Option<Expr>, Expr, bool)>,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        subtree_start: usize,
        deleted_before: usize,
        rebuilt_entries: Vec<Expr>,
        max_child_depth: usize,
        rebuilt_head: Option<Expr>,
        head_deleted: bool,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        if children.is_empty() {
            if head_deleted {
                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(render_association(Vec::new())),
                    depth: 2,
                    deleted_count: deleted_before - subtree_start,
                    exhausted: traversal.limit_reached(deleted_before),
                });
            }

            let depth = max_child_depth.saturating_add(1);
            let expr = Expr::Call {
                head: Box::new(rebuilt_head.unwrap_or_else(|| system_expr("Association"))),
                args: rebuilt_entries,
            };
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
        }

        let (is_head, entry_key, child, contributes_depth) = children[0].clone();
        let rest = children[1..].to_vec();

        let child_expr = child.clone();
        let child_result = self.delete_cases_owned(
            child_expr,
            pattern.clone(),
            traversal,
            level,
            deleted_before,
        );

        self.drive_internal_value(child_result, move |session, child_outcome| {
            let deleted_before_next = deleted_before + child_outcome.deleted_count;
            let next_max_child_depth = if contributes_depth {
                max_child_depth.max(child_outcome.depth)
            } else {
                max_child_depth
            };
            let mut next_entries = rebuilt_entries;
            let mut next_head = rebuilt_head;
            let mut next_head_deleted = head_deleted;

            if is_head {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => next_head = Some(expr),
                    DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                        next_head_deleted = true;
                    }
                }
            } else if let Some(key) = entry_key.clone() {
                if let DeleteCasesResult::Keep(value) = child_outcome.result {
                    next_entries.push(Expr::call(system_expr("Rule"), vec![key, value]));
                }
            } else {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => next_entries.push(expr),
                    DeleteCasesResult::Delete => {}
                    DeleteCasesResult::Splice(items) => next_entries.extend(items),
                }
            }

            if next_head_deleted {
                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(render_association(Vec::new())),
                    depth: 2,
                    deleted_count: deleted_before_next - subtree_start,
                    exhausted: traversal.limit_reached(deleted_before_next),
                });
            }

            if child_outcome.exhausted {
                let untouched_rest = rest
                    .iter()
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
                next_entries.extend(untouched_rest);

                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(Expr::Call {
                        head: Box::new(next_head.unwrap_or_else(|| system_expr("Association"))),
                        args: next_entries,
                    }),
                    depth: next_max_child_depth.saturating_add(1),
                    deleted_count: deleted_before_next - subtree_start,
                    exhausted: true,
                });
            }

            session.delete_cases_association_children_from_queue(
                rest,
                pattern,
                traversal,
                level,
                subtree_start,
                deleted_before_next,
                next_entries,
                next_max_child_depth,
                next_head,
                next_head_deleted,
            )
        })
    }
}
