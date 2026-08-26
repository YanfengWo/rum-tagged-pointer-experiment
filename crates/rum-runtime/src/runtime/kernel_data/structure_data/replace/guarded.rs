use super::guarded_plan::{
    activate_guarded_replace_part_pattern_target, append_guarded_replace_part_exact_targets,
    append_guarded_replace_part_pattern_candidates, build_guarded_replace_plan_from_targets,
};
use super::plan::{
    replace_part_pattern_includes_heads, replace_part_position_expr,
    wrap_replace_part_pattern_position,
};
use super::*;

impl Session {
    pub(super) fn apply_guarded_replace_part_rules_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        debug_assert!(rules.iter().any(|rule| !rule.guards.is_empty()));
        self.plan_guarded_replace_part_targets_owned(expr, rules, Vec::new(), 0, 0)
    }

    fn plan_guarded_replace_part_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        mut rule_index: usize,
        mut position_spec_index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(rule) = rules.get(rule_index) else {
                targets
                    .retain(|target| target.state == GuardedReplacePartTargetState::PendingGuard);
                let remaining = targets.len();
                return self.select_guarded_replace_part_targets_owned(
                    expr, rules, targets, remaining, None,
                );
            };

            let Some(position_spec) = rule.position_specs.get(position_spec_index).cloned() else {
                rule_index += 1;
                position_spec_index = 0;
                continue;
            };

            match position_spec {
                ReplacePartPositionSpec::Exact(selectors) => {
                    append_guarded_replace_part_exact_targets(
                        &expr,
                        &selectors,
                        rule.heads,
                        rule_index,
                        &mut targets,
                    );
                    position_spec_index += 1;
                }
                ReplacePartPositionSpec::Pattern(pattern_expr) => {
                    let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
                    return self.drive_internal_value(pattern_result, move |session, pattern| {
                        session.plan_guarded_replace_part_pattern_targets_owned(
                            expr,
                            rules,
                            targets,
                            rule_index,
                            position_spec_index,
                            pattern,
                        )
                    });
                }
            }
        }
    }

    fn plan_guarded_replace_part_pattern_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        rule_index: usize,
        position_spec_index: usize,
        resolved_pattern: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern = wrap_replace_part_pattern_position(resolved_pattern);
        let heads = rules[rule_index].heads;
        let include_heads = replace_part_pattern_includes_heads(&pattern, heads);
        let range = append_guarded_replace_part_pattern_candidates(
            &expr,
            include_heads,
            rule_index,
            &mut targets,
        );

        self.match_guarded_replace_part_pattern_targets_owned(
            expr,
            rules,
            targets,
            rule_index,
            position_spec_index,
            pattern,
            range.end,
            range.start,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn match_guarded_replace_part_pattern_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        rule_index: usize,
        position_spec_index: usize,
        pattern: Expr,
        range_end: usize,
        mut index: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                if outcome.matched {
                    let position = replace_part_position_expr(&targets[index].path);
                    let option_scope = self.option_value_scope_for_definition(&pattern, &position);
                    activate_guarded_replace_part_pattern_target(
                        &mut targets,
                        index,
                        outcome.bindings,
                        option_scope,
                    );
                } else {
                    targets[index].state = GuardedReplacePartTargetState::Inactive;
                }
                index += 1;
            }

            if index == range_end {
                return self.plan_guarded_replace_part_targets_owned(
                    expr,
                    rules,
                    targets,
                    rule_index,
                    position_spec_index + 1,
                );
            }

            let position = replace_part_position_expr(&targets[index].path);
            let match_result =
                self.match_expr_owned(pattern.clone(), position, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
                        session.match_guarded_replace_part_pattern_targets_owned(
                            expr,
                            rules,
                            targets,
                            rule_index,
                            position_spec_index,
                            pattern,
                            range_end,
                            index,
                            Some(outcome),
                        )
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn select_guarded_replace_part_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        mut remaining: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Expr, EvalError> {
        // Later rules have priority at a contested position. Walking the
        // rule-major target vector backward lets an accepting later rule claim
        // the position before any earlier guard at that position is run. A
        // candidate advances only after its one guard outcome is recorded, so
        // suspension cannot replay it and duplicate position specs cannot run
        // it twice.
        loop {
            if let Some(outcome) = pending.take() {
                let index = remaining - 1;
                targets[index].state = if outcome.matched {
                    GuardedReplacePartTargetState::Accepted
                } else {
                    GuardedReplacePartTargetState::Inactive
                };
                remaining -= 1;
                continue;
            }

            if remaining == 0 {
                let plan = build_guarded_replace_plan_from_targets(targets, &rules);
                return self.apply_replace_plan_owned(expr, plan);
            }

            let index = remaining - 1;
            let claimed_by_later_rule = targets[index + 1..].iter().any(|later| {
                later.state == GuardedReplacePartTargetState::Accepted
                    && later.path == targets[index].path
            });
            if claimed_by_later_rule {
                targets[index].state = GuardedReplacePartTargetState::Inactive;
                remaining -= 1;
                continue;
            }

            let target = &targets[index];
            let rule = &rules[target.rule_index];
            if rule.guards.is_empty() {
                targets[index].state = GuardedReplacePartTargetState::Accepted;
                remaining -= 1;
                continue;
            }

            let guard_result = self.evaluate_guard_sequence_owned(
                rule.guards.clone(),
                target.bindings.clone(),
                target.bindings.clone(),
                target.option_scope.clone(),
            );
            match guard_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, value| {
                        let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
                        session.select_guarded_replace_part_targets_owned(
                            expr,
                            rules,
                            targets,
                            remaining,
                            Some(outcome),
                        )
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}
