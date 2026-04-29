use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacePartRule {
    position_specs: Vec<ReplacePartPositionSpec>,
    rhs: Expr,
    delayed: bool,
    heads: ReplacePartHeadsMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePartPositionSpec {
    Exact(Vec<PartSelector>),
    Pattern(Expr),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReplacePlanNode {
    action: Option<ReplacePlanAction>,
    children: BTreeMap<isize, ReplacePlanNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePlanAction {
    Immediate(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplacePartHeadsMode {
    Automatic,
    Include,
    Exclude,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacePartTarget {
    path: Vec<isize>,
    rhs: Expr,
    delayed: bool,
}

impl ContinuationValue for Vec<ReplacePartTarget> {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplacePartTargets"),
            self.into_iter()
                .map(|target| {
                    Expr::call(
                        system_expr("List"),
                        vec![
                            replace_part_position_expr(&target.path),
                            target.rhs,
                            system_expr(if target.delayed { "True" } else { "False" }),
                        ],
                    )
                })
                .collect(),
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationReplacePartTargets") =>
            {
                args.into_iter()
                    .map(|item| match item {
                        Expr::Call { head, args }
                            if is_system_symbol_name(&head, "List") && args.len() == 3 =>
                        {
                            let mut parts = args.into_iter();
                            let path_expr = parts
                                .next()
                                .expect("replace-part target payload stores a path");
                            let rhs = parts
                                .next()
                                .expect("replace-part target payload stores a rhs");
                            let delayed_expr = parts
                                .next()
                                .expect("replace-part target payload stores a delayed flag");

                            ReplacePartTarget {
                                path: replace_part_path_from_expr(path_expr),
                                rhs,
                                delayed: is_system_symbol_name(&delayed_expr, "True"),
                            }
                        }
                        other => {
                            panic!("unexpected continuation item for ReplacePartTarget: {other:?}")
                        }
                    })
                    .collect()
            }
            other => panic!("unexpected continuation payload for ReplacePartTarget: {other:?}"),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_replace_at(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [rules_expr, position_spec] => Ok(replace_at_operator_form(
                rules_expr.clone(),
                position_spec.clone(),
            )),
            [target_expr, rules_expr, position_spec] => {
                let call_args = args.to_vec();
                let target_result = self.evaluate(target_expr.clone());
                let target = self.suspend_with_frame(target_result, {
                    let rules_expr = rules_expr.clone();
                    let position_spec = position_spec.clone();
                    let call_args = call_args.clone();
                    move |session, target| {
                        session.resume_replace_at_after_target(
                            target,
                            rules_expr,
                            position_spec,
                            call_args,
                        )
                    }
                })?;

                self.resume_replace_at_after_target(
                    target,
                    rules_expr.clone(),
                    position_spec.clone(),
                    call_args,
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceAt", args, 2, 3)),
        }
    }

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

    fn resume_replace_at_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        position_spec: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let positions_result = self.evaluate(position_spec);
        self.drive_evaluated_value(positions_result, move |session, resolved_positions| {
            let Some(paths) = exact_position_paths_from_expr(&target, &resolved_positions) else {
                return Ok(session.invalid_call_with_arg_form(
                    "ReplaceAt",
                    &call_args,
                    "expected an exact part specification or list of exact part specifications as the third argument",
                ));
            };

            session.resume_replace_at_after_positions(target, rules_expr, paths, call_args)
        })
    }

    fn resume_replace_at_after_positions(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        paths: Vec<Vec<isize>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(rules_expr.clone(), move |session, rule_exprs| {
            session.resume_replace_at_after_rule_exprs(target, paths, rule_exprs, call_args)
        })
    }

    fn resume_replace_at_after_rule_exprs(
        &mut self,
        target: Expr,
        paths: Vec<Vec<isize>>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAt",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        self.replace_at_paths_owned(target, paths, rules, 0)
    }

    fn replace_at_paths_owned(
        &mut self,
        expr: Expr,
        paths: Vec<Vec<isize>>,
        rules: Vec<RewriteRule>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        let Some(path) = paths.get(index).cloned() else {
            return Ok(expr);
        };

        let Some(part) = exact_expr_at_path(&expr, &path) else {
            return self.replace_at_paths_owned(expr, paths, rules, index + 1);
        };

        let replaced_result = self.try_apply_rewrite_rules_once_owned(
            part,
            rules.clone(),
            RuleApplicationSource::ReplaceAt,
        );
        self.drive_internal_value(replaced_result, move |session, replaced| {
            let Some(replaced) = replaced else {
                return session.replace_at_paths_owned(expr, paths, rules, index + 1);
            };

            let evaluated_result = session.evaluate(replaced);
            session.drive_evaluated_value::<Expr, Expr, _>(
                evaluated_result,
                move |session, value| {
                    let updated = replace_exact_path(expr, &path, value);
                    session.replace_at_paths_owned(updated, paths, rules, index + 1)
                },
            )
        })
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
            } else if rule.delayed {
                session.apply_delayed_replace_targets_owned(expr, targets, rest)
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
        let match_result = self.match_expr_owned(pattern.clone(), position, BTreeMap::new());
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

    fn apply_delayed_replace_targets_owned(
        &mut self,
        expr: Expr,
        targets: Vec<ReplacePartTarget>,
        remaining_rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        let Some((first_target, rest_targets)) = targets.split_first() else {
            return self.apply_replace_part_rules_owned(expr, remaining_rules);
        };

        let rhs_result = self.evaluate(first_target.rhs.clone());
        let first_path = first_target.path.clone();
        let rest_targets = rest_targets.to_vec();
        self.drive_evaluated_value::<Expr, Expr, _>(rhs_result, move |session, rhs| {
            let updated = replace_exact_path(expr, &first_path, rhs);
            session.apply_delayed_replace_targets_owned(updated, rest_targets, remaining_rules)
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
            Expr::Call { head, mut args } => {
                let mut replaced_head = head.as_ref().clone();
                if let Some(head_plan) = plan.children.remove(&0) {
                    replaced_head = self.apply_replace_plan_owned(replaced_head, head_plan)?;
                }

                for (index, arg) in args.iter_mut().enumerate() {
                    if let Some(child_plan) = plan.children.remove(&(index as isize + 1)) {
                        *arg = self.apply_replace_plan_owned(arg.clone(), child_plan)?;
                    }
                }

                Ok(Expr::call(replaced_head, args))
            }
        }
    }
}

fn parse_replace_part_rules(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<ReplacePartRule>> {
    if let Some(rule) = parse_replace_part_rule_with_heads(expr, heads) {
        return Some(vec![rule]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter()
        .map(|item| parse_replace_part_rule_with_heads(item, heads))
        .collect()
}

fn parse_replace_part_rule_with_heads(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Option<ReplacePartRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let delayed = if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
        false
    } else if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
        true
    } else {
        return None;
    };

    Some(ReplacePartRule {
        position_specs: parse_replace_part_position_specs(&args[0])?,
        rhs: args[1].clone(),
        delayed,
        heads,
    })
}

fn parse_replace_part_position_specs(expr: &Expr) -> Option<Vec<ReplacePartPositionSpec>> {
    if is_empty_plain_list_expr(expr) {
        return Some(Vec::new());
    }

    if let Some(position_spec) = parse_replace_part_exact_position_spec(expr) {
        return Some(vec![ReplacePartPositionSpec::Exact(position_spec)]);
    }

    let Expr::Call { head, args } = expr else {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    if !args.iter().any(is_plain_list_expr) {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    Some(
        args.iter()
            .map(|item| {
                parse_replace_part_exact_position_spec(item)
                    .map(ReplacePartPositionSpec::Exact)
                    .unwrap_or_else(|| ReplacePartPositionSpec::Pattern(item.clone()))
            })
            .collect(),
    )
}

fn parse_replace_part_exact_position_spec(expr: &Expr) -> Option<Vec<PartSelector>> {
    if let Some(selector) = parse_replace_part_exact_selector(expr) {
        return Some(vec![selector]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if args.iter().all(is_plain_list_expr) {
        return None;
    }

    args.iter().map(parse_replace_part_exact_selector).collect()
}

fn parse_replace_part_exact_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(PartSelector::Single(AtomicPartSpec::Index(index)));
    }

    parse_replace_part_key_selector(expr).map(PartSelector::Key)
}

fn parse_replace_part_key_selector(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
}

fn build_replace_plan_from_targets(targets: Vec<ReplacePartTarget>) -> ReplacePlanNode {
    let mut root = ReplacePlanNode::default();
    for target in targets {
        insert_replace_plan_path(
            &mut root,
            &target.path,
            ReplacePlanAction::Immediate(target.rhs),
        );
    }

    root
}

fn insert_replace_plan_path(node: &mut ReplacePlanNode, path: &[isize], action: ReplacePlanAction) {
    let Some((first, rest)) = path.split_first() else {
        node.action = Some(action);
        return;
    };

    insert_replace_plan_path(node.children.entry(*first).or_default(), rest, action);
}

fn path_is_prefix(prefix: &[isize], path: &[isize]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(lhs, rhs)| lhs == rhs)
}

fn resolve_conflicting_replace_part_targets(
    targets: Vec<ReplacePartTarget>,
) -> Vec<ReplacePartTarget> {
    let mut kept = Vec::new();

    'next_target: for target in targets {
        if let Some(existing_index) = kept
            .iter()
            .position(|existing: &ReplacePartTarget| existing.path == target.path)
        {
            kept.remove(existing_index);
            kept.push(target);
            continue;
        }

        for existing in &kept {
            if path_is_prefix(&existing.path, &target.path) {
                continue 'next_target;
            }
        }

        kept.retain(|existing| !path_is_prefix(&target.path, &existing.path));
        kept.push(target);
    }

    kept
}

fn wrap_replace_part_pattern_position(pattern: Expr) -> Expr {
    if replace_part_pattern_is_explicit_list(&pattern) {
        pattern
    } else {
        Expr::call(system_expr("List"), vec![pattern])
    }
}

fn replace_part_pattern_is_explicit_list(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST) => true,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::PATTERN) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[1])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::PATTERN_TEST) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::CONDITION) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::OPTIONAL) && !args.is_empty() =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args } if is_system_symbol_name(head, heads::patterns::ALTERNATIVES) => {
            !args.is_empty() && args.iter().all(replace_part_pattern_is_explicit_list)
        }
        _ => false,
    }
}

fn replace_part_pattern_includes_heads(pattern: &Expr, heads: ReplacePartHeadsMode) -> bool {
    match heads {
        ReplacePartHeadsMode::Include => true,
        ReplacePartHeadsMode::Exclude => false,
        ReplacePartHeadsMode::Automatic => replace_part_pattern_mentions_explicit_zero(pattern),
    }
}

fn replace_part_pattern_mentions_explicit_zero(expr: &Expr) -> bool {
    if expr_as_signed_isize(expr) == Some(0) {
        return true;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            replace_part_pattern_mentions_explicit_zero(head)
                || args.iter().any(replace_part_pattern_mentions_explicit_zero)
        }
    }
}

fn collect_replace_part_candidate_paths(expr: &Expr, include_heads: bool) -> Vec<Vec<isize>> {
    let mut paths = Vec::new();
    collect_replace_part_candidate_paths_inner(expr, include_heads, Vec::new(), &mut paths);
    paths
}

fn collect_replace_part_candidate_paths_inner(
    expr: &Expr,
    include_heads: bool,
    path: Vec<isize>,
    paths: &mut Vec<Vec<isize>>,
) {
    if let Expr::Call { head, args } = expr {
        if include_heads {
            let mut head_path = path.clone();
            head_path.push(0);
            collect_replace_part_candidate_paths_inner(head, include_heads, head_path, paths);
        }

        for (index, arg) in args.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(index as isize + 1);
            collect_replace_part_candidate_paths_inner(arg, include_heads, child_path, paths);
        }
    }

    paths.push(path);
}

fn replace_part_position_expr(path: &[isize]) -> Expr {
    Expr::call(
        system_expr("List"),
        path.iter()
            .map(|component| Expr::integer(BigInt::from(*component)))
            .collect(),
    )
}

fn replace_part_path_from_expr(expr: Expr) -> Vec<isize> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, heads::structure::LIST) => args
            .into_iter()
            .map(|component| {
                expr_as_signed_isize(&component)
                    .expect("replace-part continuation paths should store integers")
            })
            .collect(),
        other => panic!("replace-part continuation paths should use List expressions: {other:?}"),
    }
}

fn parse_replace_part_heads_option(expr: &Expr) -> Option<ReplacePartHeadsMode> {
    match parse_heads_option(expr)? {
        true => Some(ReplacePartHeadsMode::Include),
        false => Some(ReplacePartHeadsMode::Exclude),
    }
}

fn replace_at_operator_form(rules_expr: Expr, position_spec: Expr) -> Expr {
    let parameter = Expr::symbol("rum$replaceAtArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(
                system_expr("ReplaceAt"),
                vec![parameter, rules_expr, position_spec],
            ),
        ],
    )
}
