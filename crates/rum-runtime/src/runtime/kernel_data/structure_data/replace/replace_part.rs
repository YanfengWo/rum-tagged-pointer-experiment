use super::parsing::{parse_replace_part_heads_option, parse_replace_part_rules};
use super::plan::{
    build_replace_plan_from_targets, collect_replace_part_candidate_paths,
    replace_part_pattern_includes_heads, replace_part_position_expr,
    resolve_conflicting_replace_part_targets, wrap_replace_part_pattern_position,
};
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_replace_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ReplacePart", args, 2, 3));
        }

        let heads = if let Some(option_expr) = args.get(2) {
            let Some(heads) = parse_replace_part_heads_option(option_expr) else {
                return Ok(self.invalid_call_with_arg_form(
                    "ReplacePart",
                    args,
                    "expected a Heads -> True/False option as the third argument",
                ));
            };
            heads
        } else {
            ReplacePartHeadsMode::Automatic
        };

        let Some(rules) = parse_replace_part_rules(&args[1], heads) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.apply_replace_part_rules_owned(args[0].clone(), rules)
    }
    fn apply_replace_part_rules_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        let Some((first, rest)) = rules.split_first() else {
            return Ok(expr);
        };

        let rule = first.clone();
        let rest = rest.to_vec();
        let targets_result = self.resolve_rule_targets_owned(expr.clone(), rule.clone());
        self.drive_internal_value(targets_result, move |session, targets| {
            if targets.is_empty() {
                session.apply_replace_part_rules_owned(expr, rest)
            } else {
                let plan = build_replace_plan_from_targets(targets);
                let updated = session.apply_replace_plan_owned(expr, plan)?;
                session.apply_replace_part_rules_owned(updated, rest)
            }
        })
    }

    fn resolve_rule_targets_owned(
        &mut self,
        expr: Expr,
        rule: ReplacePartRule,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        self.collect_rule_targets_owned(
            expr,
            rule.position_specs,
            rule.rhs,
            rule.delayed,
            rule.heads,
            Vec::new(),
            0,
        )
    }

    fn collect_rule_targets_owned(
        &mut self,
        expr: Expr,
        position_specs: Vec<ReplacePartPositionSpec>,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
        mut collected: Vec<ReplacePartTarget>,
        index: usize,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let Some(position_spec) = position_specs.get(index).cloned() else {
            return Ok(resolve_conflicting_replace_part_targets(collected));
        };

        let targets_result = self.collect_position_spec_targets_owned(
            expr.clone(),
            position_spec,
            rhs.clone(),
            delayed,
            heads,
        );
        self.drive_internal_value(targets_result, move |session, mut targets| {
            collected.append(&mut targets);
            session.collect_rule_targets_owned(
                expr,
                position_specs,
                rhs,
                delayed,
                heads,
                collected,
                index + 1,
            )
        })
    }

    fn collect_position_spec_targets_owned(
        &mut self,
        expr: Expr,
        position_spec: ReplacePartPositionSpec,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        match position_spec {
            ReplacePartPositionSpec::Exact(selectors) => Ok(collect_exact_paths_for_position_spec(
                &expr, &selectors, heads,
            )
            .into_iter()
            .map(|path| ReplacePartTarget {
                path,
                rhs: rhs.clone(),
                delayed,
            })
            .collect()),
            ReplacePartPositionSpec::Pattern(pattern_expr) => {
                self.collect_pattern_position_targets_owned(expr, pattern_expr, rhs, delayed, heads)
            }
        }
    }

    fn collect_pattern_position_targets_owned(
        &mut self,
        expr: Expr,
        pattern_expr: Expr,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        self.drive_internal_value(pattern_result, move |session, resolved_pattern| {
            let pattern = wrap_replace_part_pattern_position(resolved_pattern);
            let include_heads = replace_part_pattern_includes_heads(&pattern, heads);
            let candidates = collect_replace_part_candidate_paths(&expr, include_heads);
            session.collect_pattern_position_targets_from_candidates_owned(
                pattern,
                candidates,
                rhs,
                delayed,
                Vec::new(),
                0,
            )
        })
    }

    fn collect_pattern_position_targets_from_candidates_owned(
        &mut self,
        pattern: Expr,
        candidates: Vec<Vec<isize>>,
        rhs: Expr,
        delayed: bool,
        mut collected: Vec<ReplacePartTarget>,
        index: usize,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let Some(path) = candidates.get(index).cloned() else {
            return Ok(collected);
        };

        let position = replace_part_position_expr(&path);
        let match_result =
            self.match_expr_owned(pattern.clone(), position, SymbolExprMap::default());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                collected.push(ReplacePartTarget {
                    path,
                    rhs: substitute_rule_bindings(&rhs, &outcome.bindings),
                    delayed,
                });
            }

            session.collect_pattern_position_targets_from_candidates_owned(
                pattern,
                candidates,
                rhs,
                delayed,
                collected,
                index + 1,
            )
        })
    }

    fn apply_replace_plan_owned(
        &mut self,
        expr: Expr,
        mut plan: ReplacePlanNode,
    ) -> Result<Expr, EvalError> {
        if let Some(action) = plan.action.take() {
            return match action {
                ReplacePlanAction::Immediate(rhs) => Ok(rhs),
            };
        }

        match expr {
            Expr::Atom(_) => {
                if let Some(head_plan) = plan.children.remove(&0) {
                    let replaced_head =
                        self.apply_replace_plan_owned(structural_head_expr(&expr), head_plan)?;
                    Ok(Expr::call(replaced_head, Vec::new()))
                } else {
                    Ok(expr)
                }
            }
            Expr::Call { head, args } => {
                let mut replaced_head = head.as_ref().clone();
                if let Some(head_plan) = plan.children.remove(&0) {
                    replaced_head = self.apply_replace_plan_owned(replaced_head, head_plan)?;
                }

                let mut args = Expr::unwrap_shared_args(args);
                for (index, arg) in args.iter_mut().enumerate() {
                    if let Some(child_plan) = plan.children.remove(&(index as isize + 1)) {
                        *arg = self.apply_replace_plan_owned(arg.clone(), child_plan)?;
                    }
                }

                Ok(Expr::call_with_shared_args(
                    replaced_head,
                    SharedArgs::new(args),
                ))
            }
        }
    }
}
