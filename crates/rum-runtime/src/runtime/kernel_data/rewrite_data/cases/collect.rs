mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedCasesChildren {
    children: FrozenRewriteQueue<(Expr, bool)>,
    child_index: usize,
    expr: Expr,
    spec: FrozenRewriteValue<CasesSpec>,
    traversal: CasesTraversalConfig,
    level: usize,
    seen_before: usize,
    matches: PersistentRewriteExprs,
    max_child_depth: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCasesNodeFinish {
    matches: PersistentRewriteExprs,
    traversal: CasesTraversalConfig,
    depth: usize,
    seen_before: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCasesRuleMatch {
    spec: FrozenRewriteValue<CasesSpec>,
}

struct SuspendedCasesCollectionOutcome {
    matches: PersistentRewriteExprs,
    depth: usize,
    exhausted: bool,
}

impl SuspendedCasesCollectionOutcome {
    fn into_owned(self) -> CasesCollectionOutcome {
        CasesCollectionOutcome::new(self.matches.into_vec(), self.depth, self.exhausted)
    }
}

impl Session {
    pub(in crate::runtime) fn eval_cases(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Cases", args_slice));
        }
        if !(2..=5).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Cases", args_slice, 2, 5));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = match target_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                crate::resume_or_suspend!(insert self, EvalCasesSuspensionFrame { args });
                crate::resume_or_suspend!(insert self, ContinueCasesFrame);
                return Err(EvalError::InputRequired { request });
            }
            Err(error) => return Err(error),
        };

        self.resume_cases_after_target(
            target,
            args_slice[1].clone(),
            args_slice[2..].to_vec(),
            args_slice.to_vec(),
        )
    }

    fn resume_cases_after_target(
        &mut self,
        target: Expr,
        spec_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let spec_result = self.resolve_cases_spec_expr_for_matching_owned(spec_expr.clone());
        match spec_result {
            Ok(resolved_spec) => self.resume_cases_after_resolved_spec(
                target,
                resolved_spec,
                option_exprs,
                call_args,
            ),
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeCasesAfterTargetSuspensionFrame { target, call_args });
                crate::resume_or_suspend!(insert self, ContinueCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_cases_after_target_suspended(
        &mut self,
        target: Expr,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let spec_expr = call_args
            .get(1)
            .expect("Cases suspended call stores its specification")
            .clone();
        let spec_result = self.resolve_cases_spec_expr_for_matching_owned(spec_expr);
        match spec_result {
            Ok(resolved_spec) => {
                self.resume_cases_after_resolved_spec_suspended(target, resolved_spec, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeCasesAfterTargetSuspensionFrame { target, call_args });
                crate::resume_or_suspend!(insert self, ContinueCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_cases_after_resolved_spec_suspended(
        &mut self,
        target: Expr,
        resolved_spec: Expr,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let spec =
            FrozenRewriteValue::freeze(self.compile_cases_spec_from_resolved_expr(resolved_spec));
        let mut defaults = CasesTraversalConfig::default_cases();
        defaults.include_heads = self.system_option_bool("Cases", "Heads", false)?;
        let traversal_result =
            self.resolve_cases_traversal_shared_from(call_args.clone(), 2, defaults);
        match traversal_result {
            Ok(traversal) => self.finish_cases_after_resolved_traversal_suspended(
                target, spec, traversal, call_args,
            ),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeCasesAfterResolvedSpecSuspensionFrame { target, spec, call_args });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
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
        let traversal_result = self.resolve_cases_traversal_owned(option_exprs, defaults);
        match traversal_result {
            Ok(traversal) => {
                self.finish_cases_after_resolved_traversal(target, spec, traversal, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                let spec = FrozenRewriteValue::freeze(spec);
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeCasesAfterResolvedSpecSuspensionFrame { target, spec, call_args });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
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

    fn finish_cases_after_resolved_traversal_suspended(
        &mut self,
        target: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        traversal: CasesTraversalResolution,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("Cases", call_args.as_ref(), error));
            }
        };
        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let matches_result = self.collect_cases_with_suspended_spec(target, spec, traversal, 0, 0);
        crate::resume_or_suspend!(
            self,
            matches_result,
            ok(outcome) => {
                let outcome = outcome.into_owned();
                Ok(Expr::call(system_expr("List"), outcome.matches))
            },
            suspend => FinishCasesWithTraversalSuspensionFrame,
        )
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
        crate::resume_or_suspend!(
            self,
            matches_result,
            ok(outcome) => { (|| {
                    Ok(Expr::call(system_expr("List"), outcome.matches))
                })() },
            suspend => FinishCasesWithTraversalSuspensionFrame,
        )
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
                        0,
                        expr,
                        spec,
                        traversal,
                        level + 1,
                        seen_before,
                        Vec::new(),
                        1,
                        None,
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
                    0,
                    expr,
                    spec,
                    traversal,
                    level + 1,
                    seen_before,
                    Vec::new(),
                    1,
                    None,
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
        matches: Vec<Expr>,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        if traversal.limit_reached(seen_before + matches.len()) {
            return Ok(CasesCollectionOutcome::new(matches, depth, true));
        }

        let case_result = if traversal.levels.matches(level, depth) {
            self.try_collect_case_owned(expr, spec)
        } else {
            Ok(None)
        };
        match case_result {
            Ok(case) => Ok(finish_cases_node(
                matches,
                traversal,
                depth,
                seen_before,
                case,
            )),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedCasesNodeFinish {
                    matches: PersistentRewriteExprs::freeze(matches),
                    traversal,
                    depth,
                    seen_before,
                };
                crate::resume_or_suspend!(insert self, FinishCasesAtCurrentNodeSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    /// Walks the children one per loop iteration, keeping the accumulated
    /// matches in place instead of recursing per element. `pending` carries an
    /// already-computed child outcome across a suspension resume, so the merge
    /// step runs exactly once per element on both the live and the resumed
    /// paths.
    fn collect_cases_children(
        &mut self,
        children: Vec<(Expr, bool)>,
        mut child_index: usize,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
        mut matches: Vec<Expr>,
        mut max_child_depth: usize,
        mut pending: Option<CasesCollectionOutcome>,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                matches.extend(child_outcome.matches);
                let contributes_depth = children
                    .get(child_index)
                    .is_some_and(|(_, contributes_depth)| *contributes_depth);
                if contributes_depth {
                    max_child_depth = max_child_depth.max(child_outcome.depth);
                }

                if child_outcome.exhausted {
                    return Ok(CasesCollectionOutcome::new(
                        matches,
                        max_child_depth + 1,
                        true,
                    ));
                }

                child_index += 1;
            }

            let Some((child, _contributes_depth)) = children.get(child_index).cloned() else {
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
                child,
                spec.clone(),
                traversal,
                level,
                seen_before + matches.len(),
            );

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedCasesChildren {
                        children: FrozenRewriteQueue::freeze(children),
                        child_index,
                        expr,
                        spec: FrozenRewriteValue::freeze(spec),
                        traversal,
                        level,
                        seen_before,
                        matches: PersistentRewriteExprs::freeze(matches),
                        max_child_depth,
                    };
                    crate::resume_or_suspend!(insert self, CollectCasesChildrenSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_cases_children(
        &mut self,
        state: SuspendedCasesChildren,
        outcome: CasesCollectionOutcome,
    ) -> Result<SuspendedCasesCollectionOutcome, EvalError> {
        self.collect_cases_children_suspended(state, Some(outcome))
    }

    fn collect_cases_children_suspended(
        &mut self,
        mut state: SuspendedCasesChildren,
        mut pending: Option<CasesCollectionOutcome>,
    ) -> Result<SuspendedCasesCollectionOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                state.matches = state.matches.appended_values(child_outcome.matches);
                let contributes_depth = state
                    .children
                    .as_slice()
                    .get(state.child_index)
                    .is_some_and(|(_, contributes_depth)| *contributes_depth);
                if contributes_depth {
                    state.max_child_depth = state.max_child_depth.max(child_outcome.depth);
                }

                if child_outcome.exhausted {
                    return Ok(SuspendedCasesCollectionOutcome {
                        matches: state.matches,
                        depth: state.max_child_depth + 1,
                        exhausted: true,
                    });
                }
                state.child_index += 1;
            }

            let Some((child, _)) = state.children.as_slice().get(state.child_index).cloned() else {
                let depth = state.max_child_depth.saturating_add(1);
                return self.finish_cases_at_current_node_suspended(
                    state.expr,
                    state.spec,
                    state.traversal,
                    state.level - 1,
                    depth,
                    state.seen_before,
                    state.matches,
                );
            };

            let child_result = self.collect_cases_with_suspended_spec(
                child,
                state.spec.clone(),
                state.traversal,
                state.level,
                state.seen_before + state.matches.len(),
            );
            match child_result {
                Ok(outcome) => pending = Some(outcome.into_owned()),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, CollectCasesChildrenSuspendedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn collect_cases_with_suspended_spec(
        &mut self,
        expr: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
    ) -> Result<SuspendedCasesCollectionOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_cases_at_current_node_suspended(
                expr,
                spec,
                traversal,
                level,
                1,
                seen_before,
                PersistentRewriteExprs::freeze(Vec::new()),
            ),
            Expr::Call { head, .. } if is_system_symbol_name(&head, "Association") => {
                if let Some(entries) = association_entries_from_expr(&expr) {
                    let mut children = Vec::new();
                    if traversal.include_heads {
                        children.push((head.as_ref().clone(), false));
                    }
                    children.extend(entries.into_iter().map(|(_, value)| (value, true)));
                    self.collect_cases_children_suspended(
                        SuspendedCasesChildren {
                            children: FrozenRewriteQueue::freeze(children),
                            child_index: 0,
                            expr,
                            spec,
                            traversal,
                            level: level + 1,
                            seen_before,
                            matches: PersistentRewriteExprs::freeze(Vec::new()),
                            max_child_depth: 1,
                        },
                        None,
                    )
                } else {
                    self.collect_cases_standard_call_with_suspended_spec(
                        expr,
                        spec,
                        traversal,
                        level,
                        seen_before,
                    )
                }
            }
            Expr::Call { .. } => self.collect_cases_standard_call_with_suspended_spec(
                expr,
                spec,
                traversal,
                level,
                seen_before,
            ),
        }
    }

    fn collect_cases_standard_call_with_suspended_spec(
        &mut self,
        expr: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
    ) -> Result<SuspendedCasesCollectionOutcome, EvalError> {
        match expr.clone() {
            Expr::Atom(_) => self.finish_cases_at_current_node_suspended(
                expr,
                spec,
                traversal,
                level,
                1,
                seen_before,
                PersistentRewriteExprs::freeze(Vec::new()),
            ),
            Expr::Call { head, args } => {
                let mut children = Vec::new();
                if traversal.include_heads {
                    children.push((head.into_owned(), false));
                }
                children.extend(args.into_iter().map(|arg| (arg, true)));
                self.collect_cases_children_suspended(
                    SuspendedCasesChildren {
                        children: FrozenRewriteQueue::freeze(children),
                        child_index: 0,
                        expr,
                        spec,
                        traversal,
                        level: level + 1,
                        seen_before,
                        matches: PersistentRewriteExprs::freeze(Vec::new()),
                        max_child_depth: 1,
                    },
                    None,
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_cases_at_current_node_suspended(
        &mut self,
        expr: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        traversal: CasesTraversalConfig,
        level: usize,
        depth: usize,
        seen_before: usize,
        matches: PersistentRewriteExprs,
    ) -> Result<SuspendedCasesCollectionOutcome, EvalError> {
        if traversal.limit_reached(seen_before + matches.len()) {
            return Ok(SuspendedCasesCollectionOutcome {
                matches,
                depth,
                exhausted: true,
            });
        }

        let case_result = if traversal.levels.matches(level, depth) {
            self.try_collect_case_suspended(expr, spec)
        } else {
            Ok(None)
        };
        match case_result {
            Ok(case) => Ok(finish_suspended_cases_node(
                SuspendedCasesNodeFinish {
                    matches,
                    traversal,
                    depth,
                    seen_before,
                },
                case,
            )),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedCasesNodeFinish {
                    matches,
                    traversal,
                    depth,
                    seen_before,
                };
                crate::resume_or_suspend!(insert self, FinishCasesAtCurrentNodeSuspendedSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
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
                self.finish_cases_pattern_match(match_result, expr)
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    SymbolExprMap::default(),
                );
                match match_result {
                    Ok(outcome) => self.finish_cases_rule_match(&rule, outcome),
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedCasesRuleMatch {
                            spec: FrozenRewriteValue::freeze(CasesSpec::Rule(rule)),
                        };
                        crate::resume_or_suspend!(insert self, TryCollectCaseOwnedSuspensionFrame { state });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn try_collect_case_suspended(
        &mut self,
        expr: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        match spec.as_ref() {
            CasesSpec::Pattern(pattern) => {
                let match_result =
                    self.match_expr_owned(pattern.clone(), expr.clone(), SymbolExprMap::default());
                self.finish_cases_pattern_match(match_result, expr)
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    SymbolExprMap::default(),
                );
                match match_result {
                    Ok(outcome) => self.finish_cases_rule_match(rule, outcome),
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedCasesRuleMatch { spec };
                        crate::resume_or_suspend!(insert self, TryCollectCaseSuspendedSuspensionFrame { state });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn finish_cases_pattern_match(
        &mut self,
        match_result: Result<CompiledMatchOutcome, EvalError>,
        expr: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        match match_result {
            Ok(outcome) => Ok(outcome.matched.then_some(expr)),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, FinishCasesPatternMatchSuspensionFrame { expr });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_suspended_cases_rule_match(
        &mut self,
        state: SuspendedCasesRuleMatch,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        let CasesSpec::Rule(rule) = state.spec.as_ref() else {
            unreachable!("suspended Cases rule match retains a rule spec");
        };
        self.finish_cases_rule_match(rule, outcome)
    }

    fn finish_cases_rule_match(
        &mut self,
        rule: &RewriteRule,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        if !outcome.matched {
            return Ok(None);
        }

        let transformed = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &outcome.bindings,
            rule.rhs_may_splice_without_bindings,
        );
        let transformed_result = self.evaluate(transformed);
        crate::resume_or_suspend!(
            self,
            transformed_result,
            ok(value) => Ok(value),
            suspend => FinishCasesRuleMatchSuspensionFrame,
        )
        .map(Some)
    }
}

fn finish_cases_node(
    mut matches: Vec<Expr>,
    traversal: CasesTraversalConfig,
    depth: usize,
    seen_before: usize,
    case: Option<Expr>,
) -> CasesCollectionOutcome {
    if let Some(case) = case {
        matches.push(case);
    }
    let exhausted = traversal.limit_reached(seen_before + matches.len());
    CasesCollectionOutcome::new(matches, depth, exhausted)
}

fn finish_suspended_cases_node(
    mut state: SuspendedCasesNodeFinish,
    case: Option<Expr>,
) -> SuspendedCasesCollectionOutcome {
    if let Some(case) = case {
        state.matches = state.matches.appended(case);
    }
    let exhausted = state
        .traversal
        .limit_reached(state.seen_before + state.matches.len());
    SuspendedCasesCollectionOutcome {
        matches: state.matches,
        depth: state.depth,
        exhausted,
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedCasesChildren>();
    assert_traces::<SuspendedCasesNodeFinish>();
    assert_traces::<SuspendedCasesRuleMatch>();
};
