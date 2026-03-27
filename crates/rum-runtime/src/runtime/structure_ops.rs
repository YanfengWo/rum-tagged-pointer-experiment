use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomicPartSpec {
    Index(isize),
    All,
    Span(PartSpanSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PartSpanSpec {
    start: SpanEndpoint,
    end: SpanEndpoint,
    step: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpanEndpoint {
    Index(isize),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartSelector {
    Single(AtomicPartSpec),
    Multi(Vec<PartSelector>),
}

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
enum ReplacePartHeadsMode {
    Automatic,
    Include,
    Exclude,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedPartShape {
    Leaf(Expr),
    Branch(Vec<SelectedPartShape>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReplacePartTarget {
    pub(super) path: Vec<isize>,
    pub(super) rhs: Expr,
    pub(super) delayed: bool,
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
    pub(super) fn eval_part(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Part",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        let Some(selectors) = parse_part_selectors(&args[1..]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(apply_part_selectors(&args[0], &selectors)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_extract(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Extract", args, 2, 3));
        }

        let Some(position_spec) = parse_extract_position_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let extracted = if position_spec.multiple {
            let mut items = Vec::new();
            for selectors in &position_spec.position_specs {
                let Some(value) = apply_part_selectors(&args[0], selectors) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                items.push(wrap_extracted_value(value, args.get(2)));
            }
            Expr::call(system_expr("List"), items)
        } else {
            let Some(value) = apply_part_selectors(&args[0], &position_spec.position_specs[0])
            else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            wrap_extracted_value(value, args.get(2))
        };

        Ok(extracted)
    }

    pub(super) fn eval_replace_part(
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
            session.apply_delayed_replace_targets_owned(
                updated,
                rest_targets,
                remaining_rules,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtractPositionSpec {
    position_specs: Vec<Vec<PartSelector>>,
    multiple: bool,
}

fn parse_part_selectors(exprs: &[Expr]) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector).collect()
}

fn parse_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(spec) = parse_atomic_part_spec(expr) {
        return Some(PartSelector::Single(spec));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter().map(parse_part_selector).collect::<Option<Vec<_>>>()?,
    ))
}

fn parse_atomic_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(AtomicPartSpec::Index(index));
    }

    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(AtomicPartSpec::All);
    }

    parse_span_part_spec(expr)
}

fn parse_span_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::SPAN) {
        return None;
    }

    let spec = match args.as_slice() {
        [value] => {
            let endpoint = parse_span_endpoint(value)?;
            PartSpanSpec {
                start: endpoint,
                end: endpoint,
                step: 1,
            }
        }
        [start, end] => PartSpanSpec {
            start: parse_span_endpoint(start)?,
            end: parse_span_endpoint(end)?,
            step: 1,
        },
        [start, end, step] => {
            let step = expr_as_signed_isize(step)?;
            if step == 0 {
                return None;
            }
            PartSpanSpec {
                start: parse_span_endpoint(start)?,
                end: parse_span_endpoint(end)?,
                step,
            }
        }
        _ => return None,
    };

    Some(AtomicPartSpec::Span(spec))
}

fn parse_span_endpoint(expr: &Expr) -> Option<SpanEndpoint> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(SpanEndpoint::All);
    }

    expr_as_signed_isize(expr).map(SpanEndpoint::Index)
}

fn apply_part_selectors(expr: &Expr, selectors: &[PartSelector]) -> Option<Expr> {
    let Some((first, rest)) = selectors.split_first() else {
        return Some(expr.clone());
    };

    match select_parts_for_selector(expr, first)? {
        SelectedPartShape::Leaf(selected) => apply_part_selectors(&selected, rest),
        SelectedPartShape::Branch(selected) => Some(selected_shape_to_expr(
            SelectedPartShape::Branch(apply_part_selectors_to_shape(selected, rest)?),
            Some(structural_head_expr(expr)),
        )),
    }
}

fn apply_part_selectors_to_shape(
    selected: Vec<SelectedPartShape>,
    selectors: &[PartSelector],
) -> Option<Vec<SelectedPartShape>> {
    selected
        .into_iter()
        .map(|shape| apply_part_selectors_to_selected_shape(shape, selectors))
        .collect()
}

fn apply_part_selectors_to_selected_shape(
    shape: SelectedPartShape,
    selectors: &[PartSelector],
) -> Option<SelectedPartShape> {
    match shape {
        SelectedPartShape::Leaf(expr) => Some(SelectedPartShape::Leaf(apply_part_selectors(
            &expr, selectors,
        )?)),
        SelectedPartShape::Branch(children) => Some(SelectedPartShape::Branch(
            children
                .into_iter()
                .map(|child| apply_part_selectors_to_selected_shape(child, selectors))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

fn selected_shape_to_expr(shape: SelectedPartShape, root_head: Option<Expr>) -> Expr {
    match shape {
        SelectedPartShape::Leaf(expr) => expr,
        SelectedPartShape::Branch(children) => Expr::call(
            root_head.unwrap_or_else(|| system_expr("List")),
            children
                .into_iter()
                .map(|child| selected_shape_to_expr(child, None))
                .collect(),
        ),
    }
}

fn select_parts_for_selector(expr: &Expr, selector: &PartSelector) -> Option<SelectedPartShape> {
    match selector {
        PartSelector::Single(spec) => match spec {
            AtomicPartSpec::Index(index) => {
                Some(SelectedPartShape::Leaf(select_exact_part(expr, *index)?))
            }
            AtomicPartSpec::All | AtomicPartSpec::Span(_) => Some(SelectedPartShape::Branch(
                select_parts_for_atomic_spec(expr, spec)?
                    .into_iter()
                    .map(SelectedPartShape::Leaf)
                    .collect(),
            )),
        },
        PartSelector::Multi(specs) => {
            let mut selected = Vec::new();
            for spec in specs {
                match (spec, select_parts_for_selector(expr, spec)?) {
                    (PartSelector::Single(_), SelectedPartShape::Leaf(expr)) => {
                        selected.push(SelectedPartShape::Leaf(expr));
                    }
                    (PartSelector::Single(_), SelectedPartShape::Branch(children)) => {
                        selected.extend(children);
                    }
                    (PartSelector::Multi(_), shape) => selected.push(shape),
                }
            }
            Some(SelectedPartShape::Branch(selected))
        }
    }
}

fn select_parts_for_atomic_spec(expr: &Expr, spec: &AtomicPartSpec) -> Option<Vec<Expr>> {
    select_positioned_parts_for_atomic_spec(expr, spec).map(|parts| {
        parts
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>()
    })
}

fn select_positioned_parts_for_atomic_spec(
    expr: &Expr,
    spec: &AtomicPartSpec,
) -> Option<Vec<(isize, Expr)>> {
    match spec {
        AtomicPartSpec::Index(index) => {
            let value = select_exact_part(expr, *index)?;
            let component = exact_part_component(expr, *index)?;
            Some(vec![(component, value)])
        }
        AtomicPartSpec::All => match expr {
            Expr::Atom(_) => None,
            Expr::Call { args, .. } => Some(
                args.iter()
                    .enumerate()
                    .map(|(index, value)| (index as isize + 1, value.clone()))
                    .collect(),
            ),
        },
        AtomicPartSpec::Span(span) => {
            select_positioned_parts_for_span(expr, *span, part_span_includes_head(*span))
        }
    }
}

fn resolve_span_endpoint(
    endpoint: SpanEndpoint,
    len: usize,
    step: isize,
    include_head: bool,
    is_start: bool,
) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => {
            let len = isize::try_from(len).ok()?;
            Some(if step > 0 {
                if is_start {
                    if include_head { 0 } else { 1 }
                } else {
                    len
                }
            } else if is_start {
                len
            } else {
                if include_head { 0 } else { 1 }
            })
        }
        SpanEndpoint::Index(0) if include_head => Some(0),
        SpanEndpoint::Index(index) => normalize_clamped_structural_endpoint(index, len),
    }
}

fn select_positioned_parts_for_span(
    expr: &Expr,
    spec: PartSpanSpec,
    include_head: bool,
) -> Option<Vec<(isize, Expr)>> {
    match expr {
        Expr::Atom(_) => {
            let components = resolve_span_components(spec, 0, include_head)?;
            Some(
                components
                    .into_iter()
                    .filter_map(|component| {
                        if component == 0 {
                            Some((0, structural_head_expr(expr)))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        }
        Expr::Call { args, .. } => {
            let components = resolve_span_components(spec, args.len(), include_head)?;
            Some(
                components
                    .into_iter()
                    .filter_map(|component| {
                        if component == 0 {
                            Some((0, structural_head_expr(expr)))
                        } else {
                            let index = normalize_structural_arg_position(component, args.len())?;
                            Some((component, args[index].clone()))
                        }
                    })
                    .collect(),
            )
        }
    }
}

fn part_span_includes_head(spec: PartSpanSpec) -> bool {
    matches!(spec.start, SpanEndpoint::Index(0)) || matches!(spec.end, SpanEndpoint::Index(0))
}

fn resolve_span_components(spec: PartSpanSpec, len: usize, include_head: bool) -> Option<Vec<isize>> {
    let start = resolve_span_endpoint(spec.start, len, spec.step, include_head, true)?;
    let end = resolve_span_endpoint(spec.end, len, spec.step, include_head, false)?;
    let mut current = start;
    let mut components = Vec::new();

    if spec.step > 0 {
        while current <= end {
            components.push(current);
            current += spec.step;
        }
    } else {
        while current >= end {
            components.push(current);
            current += spec.step;
        }
    }

    Some(components)
}

fn normalize_clamped_structural_endpoint(position: isize, len: usize) -> Option<isize> {
    if position == 0 || len == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position
    } else {
        len + position + 1
    };

    Some(normalized.clamp(1, len))
}

fn parse_extract_position_spec(expr: &Expr) -> Option<ExtractPositionSpec> {
    if is_empty_plain_list_expr(expr) {
        return Some(ExtractPositionSpec {
            position_specs: Vec::new(),
            multiple: true,
        });
    }

    if let Some(position_spec) = parse_single_position_spec(expr) {
        return Some(ExtractPositionSpec {
            position_specs: vec![position_spec],
            multiple: false,
        });
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let position_specs = args
        .iter()
        .map(parse_single_position_spec)
        .collect::<Option<Vec<_>>>()?;
    Some(ExtractPositionSpec {
        position_specs,
        multiple: true,
    })
}

fn parse_single_position_spec(expr: &Expr) -> Option<Vec<PartSelector>> {
    if let Some(spec) = parse_atomic_part_spec(expr) {
        return Some(vec![PartSelector::Single(spec)]);
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

    args.iter().map(parse_part_selector).collect()
}

fn is_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)
    )
}

fn is_empty_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::structure::LIST) && args.is_empty()
    )
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

    if let Some(position_spec) = parse_single_position_spec(expr) {
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
                parse_single_position_spec(item)
                    .map(ReplacePartPositionSpec::Exact)
                    .unwrap_or_else(|| ReplacePartPositionSpec::Pattern(item.clone()))
            })
            .collect(),
    )
}

fn wrap_extracted_value(value: Expr, wrapper: Option<&Expr>) -> Expr {
    wrapper
        .cloned()
        .map(|head| Expr::call(head, vec![value.clone()]))
        .unwrap_or(value)
}

fn select_exact_part(expr: &Expr, index: isize) -> Option<Expr> {
    if index == 0 {
        return Some(structural_head_expr(expr));
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            let normalized = normalize_structural_arg_position(index, args.len())?;
            Some(args[normalized].clone())
        }
    }
}

fn exact_part_component(expr: &Expr, index: isize) -> Option<isize> {
    if index == 0 {
        return Some(0);
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            Some(normalize_structural_arg_position(index, args.len())? as isize + 1)
        }
    }
}

fn normalize_structural_arg_position(position: isize, len: usize) -> Option<usize> {
    if position == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position - 1
    } else {
        len + position
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

fn structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
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
        if let Some(existing_index) = kept.iter().position(|existing: &ReplacePartTarget| {
            existing.path == target.path
        }) {
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

fn collect_exact_paths_for_position_spec(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
) -> Vec<Vec<isize>> {
    let Some((first, rest)) = selectors.split_first() else {
        return vec![Vec::new()];
    };

    let Some(selected) = select_positioned_parts_for_selector(expr, first, heads) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for (component, value) in selected {
        for mut tail in collect_exact_paths_for_position_spec(&value, rest, heads) {
            let mut path = vec![component];
            path.append(&mut tail);
            paths.push(path);
        }
    }
    paths
}

fn select_positioned_parts_for_selector(
    expr: &Expr,
    selector: &PartSelector,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(isize, Expr)>> {
    match selector {
        PartSelector::Single(spec) => {
            select_positioned_parts_for_atomic_spec_with_heads(expr, spec, heads)
        }
        PartSelector::Multi(specs) => {
            let mut selected = Vec::new();
            for spec in specs {
                selected.extend(select_positioned_parts_for_selector(expr, spec, heads)?);
            }
            Some(selected)
        }
    }
}

fn select_positioned_parts_for_atomic_spec_with_heads(
    expr: &Expr,
    spec: &AtomicPartSpec,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(isize, Expr)>> {
    match spec {
        AtomicPartSpec::Index(0) => match heads {
            ReplacePartHeadsMode::Exclude => None,
            ReplacePartHeadsMode::Automatic | ReplacePartHeadsMode::Include => {
                Some(vec![(0, structural_head_expr(expr))])
            }
        },
        AtomicPartSpec::Index(index) => {
            let value = select_exact_part(expr, *index)?;
            let component = exact_part_component(expr, *index)?;
            Some(vec![(component, value)])
        }
        AtomicPartSpec::All => match expr {
            Expr::Atom(_) => match heads {
                ReplacePartHeadsMode::Include => Some(vec![(0, structural_head_expr(expr))]),
                ReplacePartHeadsMode::Automatic | ReplacePartHeadsMode::Exclude => Some(Vec::new()),
            },
            Expr::Call { args, .. } => {
                let mut parts = Vec::new();
                if matches!(heads, ReplacePartHeadsMode::Include) {
                    parts.push((0, structural_head_expr(expr)));
                }
                parts.extend(
                    args.iter()
                        .enumerate()
                        .map(|(index, value)| (index as isize + 1, value.clone())),
                );
                Some(parts)
            }
        },
        AtomicPartSpec::Span(span) => match expr {
            _ => select_positioned_parts_for_span(
                expr,
                *span,
                replace_part_span_includes_heads(*span, heads),
            ),
        },
    }
}

fn replace_part_span_includes_heads(spec: PartSpanSpec, heads: ReplacePartHeadsMode) -> bool {
    match heads {
        ReplacePartHeadsMode::Include => true,
        ReplacePartHeadsMode::Exclude => false,
        ReplacePartHeadsMode::Automatic => matches!(spec.start, SpanEndpoint::Index(0))
            || matches!(spec.end, SpanEndpoint::Index(0)),
    }
}

fn parse_replace_part_heads_option(expr: &Expr) -> Option<ReplacePartHeadsMode> {
    match parse_heads_option(expr)? {
        true => Some(ReplacePartHeadsMode::Include),
        false => Some(ReplacePartHeadsMode::Exclude),
    }
}

fn replace_exact_path(expr: Expr, path: &[isize], rhs: Expr) -> Expr {
    let Some((first, rest)) = path.split_first() else {
        return rhs;
    };

    if *first == 0 {
        let replaced_head = replace_exact_path(structural_head_expr(&expr), rest, rhs);
        return match expr {
            Expr::Atom(_) => Expr::call(replaced_head, Vec::new()),
            Expr::Call { args, .. } => Expr::call(replaced_head, args),
        };
    }

    match expr {
        Expr::Atom(_) => expr,
        Expr::Call { head, mut args } => {
            let Some(index) = normalize_structural_arg_position(*first, args.len()) else {
                return Expr::Call { head, args };
            };

            let replaced_child = replace_exact_path(args[index].clone(), rest, rhs);
            args[index] = replaced_child;
            Expr::Call { head, args }
        }
    }
}
