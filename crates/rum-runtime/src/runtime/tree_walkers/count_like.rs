mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Copy, FrameRootTrace)]
enum CountLikeFinish {
    Count,
    MemberQ,
    FreeQ,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCountChildren {
    children: Arc<[(Expr, bool)]>,
    child_index: usize,
    expr: Expr,
    pattern: Expr,
    traversal: CasesTraversalConfig,
    level: usize,
    seen_before: usize,
    count: usize,
    max_child_depth: usize,
}

impl CountLikeFinish {
    fn finish(self, count: usize) -> Expr {
        match self {
            Self::Count => Expr::integer(BigInt::from(count)),
            Self::MemberQ => system_expr(if count > 0 { "True" } else { "False" }),
            Self::FreeQ => system_expr(if count == 0 { "True" } else { "False" }),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_count(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Count", args_slice));
        }
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Count", args_slice, 2, 4));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalCountSuspensionFrame { args }
        },
        )?;

        self.resume_count_after_target(target, args)
    }

    pub(in crate::runtime) fn eval_member_q(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("MemberQ", args_slice));
        }
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("MemberQ", args_slice, 2, 4));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalMemberQSuspensionFrame { args }
        },
        )?;

        self.resume_member_q_after_target(target, args)
    }

    pub(in crate::runtime) fn eval_free_q(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("FreeQ", args_slice));
        }
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("FreeQ", args_slice, 2, 4));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalFreeQSuspensionFrame { args }
        },
        )?;

        self.resume_free_q_after_target(target, args)
    }

    fn resume_count_after_target(
        &mut self,
        target: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(args[1].clone());
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let target = target.clone();
                    let args = args.clone();
        ResumeCountAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_count_after_pattern(target, pattern, args)
    }

    fn resume_count_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("Count", "Heads", false)?;
        self.finish_count_like_after_pattern(
            "Count",
            target,
            pattern,
            args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_cases()
            },
            CountLikeFinish::Count,
        )
    }

    fn resume_member_q_after_target(
        &mut self,
        target: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(args[1].clone());
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let target = target.clone();
                    let args = args.clone();
        ResumeMemberQAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_member_q_after_pattern(target, pattern, args)
    }

    fn resume_member_q_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("MemberQ", "Heads", false)?;
        self.finish_count_like_after_pattern(
            "MemberQ",
            target,
            pattern,
            args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_member_q()
            },
            CountLikeFinish::MemberQ,
        )
    }

    fn resume_free_q_after_target(
        &mut self,
        target: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(args[1].clone());
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let target = target.clone();
                    let args = args.clone();
        ResumeFreeQAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_free_q_after_pattern(target, pattern, args)
    }

    fn resume_free_q_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("FreeQ", "Heads", true)?;
        self.finish_count_like_after_pattern(
            "FreeQ",
            target,
            pattern,
            args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_free_q()
            },
            CountLikeFinish::FreeQ,
        )
    }

    fn finish_count_like_after_pattern(
        &mut self,
        builtin_name: &'static str,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
        defaults: CasesTraversalConfig,
        finish: CountLikeFinish,
    ) -> Result<Expr, EvalError> {
        let traversal_result =
            self.resolve_traversal_without_limit_owned(args[2..].to_vec(), defaults);
        match traversal_result {
            Ok(traversal) => self.finish_count_like_after_traversal(
                builtin_name,
                target,
                pattern,
                args,
                finish,
                traversal,
            ),
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                crate::resume_or_suspend!(insert self, FinishCountLikeAfterPatternSuspensionFrame { builtin_name, target, pattern, args, finish });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_count_like_after_traversal(
        &mut self,
        builtin_name: &'static str,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
        finish: CountLikeFinish,
        traversal: TraversalWithoutLimitResolution,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            TraversalWithoutLimitResolution::Valid(traversal) => traversal,
            TraversalWithoutLimitResolution::Invalid(error) => {
                return Ok(self.invalid_traversal_without_limit_call(
                    builtin_name,
                    args.as_ref(),
                    error,
                ));
            }
        };

        let outcome_result = self.count_pattern_matches_owned(target, pattern, traversal, 0, 0);
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| {
                    Ok(finish.finish(outcome.count))
                })() },
            suspend => FinishCountLikeAfterTraversalSuspensionFrame { finish },
        )
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
                        None,
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
                    None,
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

        crate::resume_or_suspend!(
            self,
            match_result,
            ok(outcome) => { (|| {
                    let next_count = count + usize::from(outcome.matched);
                    let exhausted = traversal.limit_reached(seen_before + next_count);
                    Ok(TraversalCountOutcome::new(next_count, depth, exhausted))
                })() },
            suspend => FinishCountAtCurrentNodeSuspensionFrame { count, traversal, seen_before, depth },
        )
    }

    /// Walks the children one per loop iteration, keeping the running count in
    /// place instead of recursing per element. `pending` carries an
    /// already-computed child outcome across a suspension resume, so the merge
    /// step runs exactly once per element on both the live and the resumed
    /// paths.
    fn count_pattern_children(
        &mut self,
        children: Vec<(Expr, bool)>,
        mut child_index: usize,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        seen_before: usize,
        mut count: usize,
        mut max_child_depth: usize,
        mut pending: Option<TraversalCountOutcome>,
    ) -> Result<TraversalCountOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                count += child_outcome.count;
                let contributes_depth = children
                    .get(child_index)
                    .is_some_and(|(_, contributes_depth)| *contributes_depth);
                if contributes_depth {
                    max_child_depth = max_child_depth.max(child_outcome.depth);
                }

                if child_outcome.exhausted {
                    return Ok(TraversalCountOutcome::new(count, max_child_depth + 1, true));
                }

                child_index += 1;
            }

            let Some((child, _contributes_depth)) = children.get(child_index).cloned() else {
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

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedCountChildren {
                        children: Arc::from(children),
                        child_index,
                        expr,
                        pattern,
                        traversal,
                        level,
                        seen_before,
                        count,
                        max_child_depth,
                    };
                    crate::resume_or_suspend!(insert self, CountPatternChildrenSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn count_pattern_children_suspended(
        &mut self,
        mut state: SuspendedCountChildren,
        mut pending: Option<TraversalCountOutcome>,
    ) -> Result<TraversalCountOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                state.count += child_outcome.count;
                let contributes_depth = state
                    .children
                    .get(state.child_index)
                    .is_some_and(|(_, contributes_depth)| *contributes_depth);
                if contributes_depth {
                    state.max_child_depth = state.max_child_depth.max(child_outcome.depth);
                }

                if child_outcome.exhausted {
                    return Ok(TraversalCountOutcome::new(
                        state.count,
                        state.max_child_depth + 1,
                        true,
                    ));
                }

                state.child_index += 1;
            }

            let Some((child, _contributes_depth)) = state.children.get(state.child_index).cloned()
            else {
                let depth = state.max_child_depth.saturating_add(1);
                return self.finish_count_at_current_node(
                    state.expr,
                    state.pattern,
                    state.traversal,
                    state.level - 1,
                    depth,
                    state.seen_before,
                    state.count,
                );
            };

            let child_result = self.count_pattern_matches_owned(
                child,
                state.pattern.clone(),
                state.traversal,
                state.level,
                state.seen_before + state.count,
            );

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, CountPatternChildrenSuspendedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<CountLikeFinish>();
    assert_traces::<SuspendedCountChildren>();
};
