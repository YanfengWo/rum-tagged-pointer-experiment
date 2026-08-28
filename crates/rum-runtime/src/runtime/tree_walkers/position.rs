mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct PersistentPositionAccumulator {
    prefix: SharedArgs,
    tail: Option<Arc<PersistentPositionNode>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct PersistentPositionNode {
    previous: Option<Arc<PersistentPositionNode>>,
    values: SharedArgs,
}

impl PersistentPositionAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: None,
            len,
        }
    }

    fn appended_values(&self, values: Vec<Expr>) -> Self {
        if values.is_empty() {
            return self.clone();
        }
        let added = values.len();
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(PersistentPositionNode {
                previous: self.tail.clone(),
                values: SharedArgs::new(values),
            })),
            len: self.len + added,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        self.appended_values(vec![value])
    }

    fn len(&self) -> usize {
        self.len
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::new();
        let mut node = self.tail;
        while let Some(current) = node {
            let current = match Arc::try_unwrap(current) {
                Ok(current) => current,
                Err(_) => panic!("completed position accumulator must be uniquely owned"),
            };
            let values = current.values.into_vec();
            suffix.push(values);
            node = current.previous;
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(self.len.saturating_sub(values.len()));
        for chunk in suffix {
            values.extend(chunk);
        }
        values
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedPositionChildren {
    children: Arc<[(Expr, Vec<Expr>, bool)]>,
    child_index: usize,
    expr: Expr,
    pattern: Expr,
    traversal: CasesTraversalConfig,
    level: usize,
    path: Arc<[Expr]>,
    seen_before: usize,
    positions: PersistentPositionAccumulator,
    max_child_depth: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedPositionNode {
    traversal: CasesTraversalConfig,
    depth: usize,
    path: Arc<[Expr]>,
    seen_before: usize,
    positions: PersistentPositionAccumulator,
}

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
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalPositionSuspensionFrame { args }
        },
        )?;

        self.resume_position_after_target(target, args)
    }

    fn resume_position_after_target(
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
        ResumePositionAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_position_after_pattern(target, pattern, args)
    }

    fn resume_position_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let mut defaults = CasesTraversalConfig::default_position();
        defaults.include_heads = self.system_option_bool("Position", "Heads", true)?;
        let traversal_result = self.resolve_position_traversal_owned(args[2..].to_vec(), defaults);
        match traversal_result {
            Ok(traversal) => {
                self.finish_position_after_resolved_traversal(target, pattern, traversal, args)
            }
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                crate::resume_or_suspend!(insert self, ResumePositionAfterPatternSuspensionFrame { target, pattern, args });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_position_after_resolved_traversal(
        &mut self,
        target: Expr,
        pattern: Expr,
        traversal: CasesTraversalResolution,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("Position", args.as_ref(), error));
            }
        };

        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let positions_result =
            self.collect_positions_owned(target, pattern, traversal, 0, Vec::new(), 0);
        crate::resume_or_suspend!(
            self,
            positions_result,
            ok(outcome) => { (|| {
                    Ok(Expr::call(system_expr("List"), outcome.positions))
                })() },
            suspend => FinishPositionAfterResolvedTraversalSuspensionFrame,
        )
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

        match match_result {
            Ok(outcome) => {
                if outcome.matched {
                    positions.push(position_expr(&path));
                }
                let exhausted = traversal.limit_reached(seen_before + positions.len());
                Ok(PositionCollectionOutcome::new(positions, depth, exhausted))
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedPositionNode {
                    traversal,
                    depth,
                    path: Arc::from(path),
                    seen_before,
                    positions: PersistentPositionAccumulator::freeze(positions),
                };
                crate::resume_or_suspend!(insert self, FinishPositionsAtCurrentNodeSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_suspended_position_node(
        &mut self,
        state: SuspendedPositionNode,
        outcome: Result<Expr, EvalError>,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        let outcome = CompiledMatchOutcome::from_continuation_expr(outcome?);
        self.finish_suspended_position_node_outcome(state, outcome)
    }

    fn finish_suspended_position_node_outcome(
        &mut self,
        mut state: SuspendedPositionNode,
        outcome: CompiledMatchOutcome,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        if outcome.matched {
            state.positions = state.positions.appended(position_expr(state.path.as_ref()));
        }
        let exhausted = state
            .traversal
            .limit_reached(state.seen_before + state.positions.len());
        Ok(PositionCollectionOutcome::new(
            state.positions.into_vec(),
            state.depth,
            exhausted,
        ))
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
                    let state = SuspendedPositionChildren {
                        children: Arc::from(children),
                        child_index,
                        expr,
                        pattern,
                        traversal,
                        level,
                        path: Arc::from(path),
                        seen_before,
                        positions: PersistentPositionAccumulator::freeze(positions),
                        max_child_depth,
                    };
                    crate::resume_or_suspend!(insert self, CollectPositionChildrenSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn collect_position_children_suspended(
        &mut self,
        mut state: SuspendedPositionChildren,
        mut pending: Option<PositionCollectionOutcome>,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                state.positions = state.positions.appended_values(child_outcome.positions);
                let contributes_depth = state
                    .children
                    .get(state.child_index)
                    .is_some_and(|(_, _, contributes_depth)| *contributes_depth);
                if contributes_depth {
                    state.max_child_depth = state.max_child_depth.max(child_outcome.depth);
                }

                if child_outcome.exhausted {
                    return Ok(PositionCollectionOutcome::new(
                        state.positions.into_vec(),
                        state.max_child_depth + 1,
                        true,
                    ));
                }

                state.child_index += 1;
            }

            let Some((child, child_path, _contributes_depth)) =
                state.children.get(state.child_index).cloned()
            else {
                let depth = state.max_child_depth.saturating_add(1);
                let node = SuspendedPositionNode {
                    traversal: state.traversal,
                    depth,
                    path: state.path,
                    seen_before: state.seen_before,
                    positions: state.positions,
                };
                return self.finish_positions_at_current_node_suspended(
                    state.expr,
                    state.pattern,
                    state.level - 1,
                    node,
                );
            };

            let child_result = self.collect_positions_owned(
                child,
                state.pattern.clone(),
                state.traversal,
                state.level,
                child_path,
                state.seen_before + state.positions.len(),
            );

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, CollectPositionChildrenSuspendedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn finish_positions_at_current_node_suspended(
        &mut self,
        expr: Expr,
        pattern: Expr,
        level: usize,
        state: SuspendedPositionNode,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        if state
            .traversal
            .limit_reached(state.seen_before + state.positions.len())
        {
            return Ok(PositionCollectionOutcome::new(
                state.positions.into_vec(),
                state.depth,
                true,
            ));
        }

        let match_result = if state.traversal.levels.matches(level, state.depth) {
            self.match_expr_owned(pattern, expr, SymbolExprMap::default())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: SymbolExprMap::default(),
            })
        };
        match match_result {
            Ok(outcome) => self.finish_suspended_position_node_outcome(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, FinishPositionsAtCurrentNodeSuspendedSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
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

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PersistentPositionAccumulator>();
    assert_traces::<SuspendedPositionChildren>();
    assert_traces::<SuspendedPositionNode>();
};
