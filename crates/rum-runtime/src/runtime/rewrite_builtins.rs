use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DeleteCasesResult {
    Keep(Expr),
    Delete,
    Splice(Vec<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeleteCasesOutcome {
    result: DeleteCasesResult,
    depth: usize,
    deleted_count: usize,
    exhausted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplaceOutcome {
    expr: Expr,
    depth: usize,
}

impl ContinuationValue for DeleteCasesOutcome {
    fn into_continuation_expr(self) -> Expr {
        let result_expr = match self.result {
            DeleteCasesResult::Keep(expr) => Expr::call(
                system_expr("InternalContinuationDeleteCasesKeep"),
                vec![expr],
            ),
            DeleteCasesResult::Delete => Expr::call(
                system_expr("InternalContinuationDeleteCasesDelete"),
                Vec::new(),
            ),
            DeleteCasesResult::Splice(items) => Expr::call(
                system_expr("InternalContinuationDeleteCasesSplice"),
                vec![Expr::call(system_expr("List"), items)],
            ),
        };

        Expr::call(
            system_expr("InternalContinuationDeleteCasesOutcome"),
            vec![
                result_expr,
                Expr::integer(BigInt::from(self.depth)),
                Expr::integer(BigInt::from(self.deleted_count)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationDeleteCasesOutcome")
                    && args.len() == 4 =>
            {
                let exhausted_expr = args.pop().expect("delete-cases payload stores exhausted");
                let deleted_count_expr = args
                    .pop()
                    .expect("delete-cases payload stores deleted count");
                let depth_expr = args.pop().expect("delete-cases payload stores depth");
                let result_expr = args.pop().expect("delete-cases payload stores result");

                let result = match result_expr {
                    Expr::Call { head, mut args }
                        if is_system_symbol_name(&head, "InternalContinuationDeleteCasesKeep")
                            && args.len() == 1 =>
                    {
                        DeleteCasesResult::Keep(
                            args.pop().expect("delete-cases keep payload stores expr"),
                        )
                    }
                    Expr::Call { head, args }
                        if is_system_symbol_name(
                            &head,
                            "InternalContinuationDeleteCasesDelete",
                        ) && args.is_empty() =>
                    {
                        DeleteCasesResult::Delete
                    }
                    Expr::Call { head, mut args }
                        if is_system_symbol_name(
                            &head,
                            "InternalContinuationDeleteCasesSplice",
                        ) && args.len() == 1 =>
                    {
                        let items_expr = args
                            .pop()
                            .expect("delete-cases splice payload stores items");
                        let items = match items_expr {
                            Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                                args
                            }
                            other => vec![other],
                        };
                        DeleteCasesResult::Splice(items)
                    }
                    other => panic!("unexpected delete-cases result payload: {other:?}"),
                };

                Self {
                    result,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("delete-cases depth should be non-negative"),
                    deleted_count: expr_as_non_negative_usize(&deleted_count_expr)
                        .expect("delete-cases deleted count should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => panic!("unexpected continuation payload for DeleteCasesOutcome: {other:?}"),
        }
    }
}

impl ContinuationValue for ReplaceOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplaceOutcome"),
            vec![self.expr, Expr::integer(BigInt::from(self.depth))],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationReplaceOutcome")
                    && args.len() == 2 =>
            {
                let depth_expr = args.pop().expect("replace outcome payload stores depth");
                let expr = args.pop().expect("replace outcome payload stores expr");

                Self {
                    expr,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("replace outcome depth should be non-negative"),
                }
            }
            other => panic!("unexpected continuation payload for ReplaceOutcome: {other:?}"),
        }
    }
}

impl Session {
    pub(super) fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MatchQ", args, 2));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let pattern_expr = args[1].clone();
            move |session, value| session.resume_match_q_after_value(pattern_expr, value)
        })?;

        self.resume_match_q_after_value(args[1].clone(), value)
    }

    pub(super) fn eval_cases(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_delete_cases(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_replace(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_replace_list(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_replace_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_replace_repeated(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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
    pub(super) fn normalize_pattern_expr(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        self.resolve_matcher_pattern_expr_owned(expr.clone())
    }

    pub(super) fn resolve_matcher_pattern_expr_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let normalized = strip_hold_pattern(&expr).clone();
        let resolved = match normalized {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(normalized),
            other => Ok(other),
        }?;

        Ok(self.normalize_match_expr(&resolved))
    }

    #[allow(dead_code)]
    pub(super) fn normalize_cases_spec(&mut self, expr: &Expr) -> Result<CasesSpec, EvalError> {
        let resolved = self.resolve_cases_spec_expr_for_matching_owned(expr.clone())?;
        Ok(self.compile_cases_spec_from_resolved_expr(resolved))
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
                return Ok(self.invalid_call_with_arg_form("Cases", &call_args, error.message()));
            }
        };

        self.finish_cases_with_traversal(target, spec, traversal)
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
                return Ok(self.invalid_call_with_arg_form(
                    "DeleteCases",
                    &call_args,
                    error.message(),
                ));
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
                        children.push(head.as_ref().clone());
                    }
                    children.extend(entries.into_iter().map(|(_, value)| value));

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
                    children.push(*head);
                }
                children.extend(args);

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

    fn collect_cases_children(
        &mut self,
        children: Vec<Expr>,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
        mut matches: Vec<Expr>,
        max_child_depth: usize,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        let Some((child, rest)) = children.split_first() else {
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
            let next_max_child_depth = max_child_depth.max(child_outcome.depth);

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
            children.push((true, head.clone()));
        }
        children.extend(args.into_iter().map(|arg| (false, arg)));

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
        children: Vec<(bool, Expr)>,
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

        let (is_head, child) = children[0].clone();
        let rest = children[1..].to_vec();

        let child_result =
            self.delete_cases_owned(child, pattern.clone(), traversal, level, deleted_before);

        self.drive_internal_value(child_result, move |session, child_outcome| {
            let deleted_before_next = deleted_before + child_outcome.deleted_count;
            let next_max_child_depth = max_child_depth.max(child_outcome.depth);
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
                    .filter_map(|(is_head, expr)| (!*is_head).then_some(expr.clone()))
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
            children.push((true, None, head.clone()));
        }

        for rule in args {
            match &rule {
                Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } if is_system_symbol_name(rule_head, "Rule") && rule_args.len() == 2 => {
                    children.push((false, Some(rule_args[0].clone()), rule_args[1].clone()));
                }
                _ => children.push((false, None, rule)),
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
        children: Vec<(bool, Option<Expr>, Expr)>,
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

        let (is_head, entry_key, child) = children[0].clone();
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
            let next_max_child_depth = max_child_depth.max(child_outcome.depth);
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
                    .filter_map(|(is_head, key, expr)| {
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

    fn try_collect_case_owned(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);

        match spec {
            CasesSpec::Pattern(pattern) => {
                let match_result = self.match_expr_owned(pattern, expr.clone(), BTreeMap::new());
                self.drive_internal_value(match_result, move |_, outcome| {
                    Ok(if outcome.matched { Some(expr) } else { None })
                })
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    BTreeMap::new(),
                );
                self.drive_internal_value(match_result, move |session, outcome| {
                    if !outcome.matched {
                        return Ok(None);
                    }

                    let transformed = substitute_rule_bindings(&rule.rhs, &outcome.bindings);
                    let transformed_result = session.evaluate(transformed);
                    session
                        .suspend_with_frame(transformed_result, |_, value| Ok(value))
                        .map(Some)
                })
            }
        }
    }

    fn resume_match_q_after_value(
        &mut self,
        pattern_expr: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = self.suspend_with_frame(pattern_result, {
            let value = value.clone();
            move |session, resolved| session.finish_match_q_with_pattern(resolved, value)
        })?;

        self.finish_match_q_with_pattern(pattern, value)
    }

    fn finish_match_q_with_pattern(
        &mut self,
        pattern: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let match_result = self.match_expr_owned(pattern, value, BTreeMap::new());
        self.drive_internal_value(match_result, move |_, outcome| {
            Ok(system_expr(if outcome.matched { "True" } else { "False" }))
        })
    }

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
