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
    Multi(Vec<AtomicPartSpec>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacePartRule {
    position_specs: Vec<Vec<PartSelector>>,
    rhs: Expr,
    delayed: bool,
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
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplacePart", args, 2));
        }

        let Some(rules) = parse_replace_part_rules(&args[1]) else {
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

        let rest = rest.to_vec();
        let exact_paths = resolve_rule_exact_paths(&expr, &first.position_specs);

        if exact_paths.is_empty() {
            self.apply_replace_part_rules_owned(expr, rest)
        } else if first.delayed {
            self.apply_delayed_replace_paths_owned(expr, exact_paths, first.rhs.clone(), rest)
        } else {
            let plan = build_replace_plan_from_paths(
                exact_paths,
                ReplacePlanAction::Immediate(first.rhs.clone()),
            );
            let updated = self.apply_replace_plan_owned(expr, plan)?;
            self.apply_replace_part_rules_owned(updated, rest)
        }
    }

    fn apply_delayed_replace_paths_owned(
        &mut self,
        expr: Expr,
        exact_paths: Vec<Vec<isize>>,
        rhs_expr: Expr,
        remaining_rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        let Some((first_path, rest_paths)) = exact_paths.split_first() else {
            return self.apply_replace_part_rules_owned(expr, remaining_rules);
        };

        let rhs_result = self.evaluate(rhs_expr.clone());
        let first_path = first_path.clone();
        let rest_paths = rest_paths.to_vec();
        self.drive_evaluated_value::<Expr, Expr, _>(rhs_result, move |session, rhs| {
            let updated = replace_exact_path(expr, &first_path, rhs);
            session.apply_delayed_replace_paths_owned(
                updated,
                rest_paths,
                rhs_expr,
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
        args.iter()
            .map(parse_atomic_part_spec)
            .collect::<Option<Vec<_>>>()?,
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

    let (selected, preserve_head) = select_parts_for_selector(expr, first)?;

    if preserve_head {
        let mut parts = Vec::with_capacity(selected.len());
        for selected_part in selected {
            parts.push(apply_part_selectors(&selected_part, rest)?);
        }
        Some(Expr::call(structural_head_expr(expr), parts))
    } else {
        apply_part_selectors(selected.first()?, rest)
    }
}

fn select_parts_for_selector(expr: &Expr, selector: &PartSelector) -> Option<(Vec<Expr>, bool)> {
    match selector {
        PartSelector::Single(spec) => match spec {
            AtomicPartSpec::Index(index) => Some((vec![select_exact_part(expr, *index)?], false)),
            AtomicPartSpec::All | AtomicPartSpec::Span(_) => {
                Some((select_parts_for_atomic_spec(expr, spec)?, true))
            }
        },
        PartSelector::Multi(specs) => {
            let mut parts = Vec::new();
            for spec in specs {
                parts.extend(select_parts_for_atomic_spec(expr, spec)?);
            }
            Some((parts, true))
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
        AtomicPartSpec::Span(span) => match expr {
            Expr::Atom(_) => None,
            Expr::Call { args, .. } => Some(
                resolve_span_indices(*span, args.len())?
                    .into_iter()
                    .map(|index| (index as isize + 1, args[index].clone()))
                    .collect(),
            ),
        },
    }
}

fn resolve_span_indices(spec: PartSpanSpec, len: usize) -> Option<Vec<usize>> {
    if len == 0 {
        return Some(Vec::new());
    }

    let start = resolve_span_endpoint(spec.start, len, spec.step, true)?;
    let end = resolve_span_endpoint(spec.end, len, spec.step, false)?;
    let mut current = start;
    let mut indices = Vec::new();

    if spec.step > 0 {
        while current <= end {
            indices.push((current - 1) as usize);
            current += spec.step;
        }
    } else {
        while current >= end {
            indices.push((current - 1) as usize);
            current += spec.step;
        }
    }

    Some(indices)
}

fn resolve_span_endpoint(
    endpoint: SpanEndpoint,
    len: usize,
    step: isize,
    is_start: bool,
) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => {
            let len = isize::try_from(len).ok()?;
            Some(if step > 0 {
                if is_start { 1 } else { len }
            } else if is_start {
                len
            } else {
                1
            })
        }
        SpanEndpoint::Index(index) => normalize_clamped_structural_endpoint(index, len),
    }
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

fn parse_replace_part_rules(expr: &Expr) -> Option<Vec<ReplacePartRule>> {
    if let Some(rule) = parse_replace_part_rule(expr) {
        return Some(vec![rule]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter().map(parse_replace_part_rule).collect()
}

fn parse_replace_part_rule(expr: &Expr) -> Option<ReplacePartRule> {
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
        position_specs: parse_replace_part_lhs(&args[0])?,
        rhs: args[1].clone(),
        delayed,
    })
}

fn parse_replace_part_lhs(expr: &Expr) -> Option<Vec<Vec<PartSelector>>> {
    if let Some(position_spec) = parse_single_position_spec(expr) {
        return Some(vec![position_spec]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter().map(parse_single_position_spec).collect()
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

fn build_replace_plan_from_paths(
    exact_paths: Vec<Vec<isize>>,
    action: ReplacePlanAction,
) -> ReplacePlanNode {
    let mut root = ReplacePlanNode::default();
    for path in exact_paths {
        insert_replace_plan_path(&mut root, &path, action.clone());
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

fn collect_rule_exact_paths(expr: &Expr, position_specs: &[Vec<PartSelector>]) -> Vec<Vec<isize>> {
    let mut paths = Vec::new();
    for position_spec in position_specs {
        paths.extend(collect_exact_paths_for_position_spec(expr, position_spec));
    }
    paths
}

fn resolve_rule_exact_paths(expr: &Expr, position_specs: &[Vec<PartSelector>]) -> Vec<Vec<isize>> {
    resolve_conflicting_exact_paths(collect_rule_exact_paths(expr, position_specs))
}

fn resolve_conflicting_exact_paths(paths: Vec<Vec<isize>>) -> Vec<Vec<isize>> {
    let mut kept = Vec::new();

    'next_path: for path in paths {
        if let Some(existing_index) = kept.iter().position(|existing| existing == &path) {
            kept.remove(existing_index);
            kept.push(path);
            continue;
        }

        for existing in &kept {
            if path_is_prefix(existing, &path) {
                continue 'next_path;
            }
        }

        kept.retain(|existing| !path_is_prefix(&path, existing));
        kept.push(path);
    }

    kept
}

fn path_is_prefix(prefix: &[isize], path: &[isize]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(lhs, rhs)| lhs == rhs)
}

fn collect_exact_paths_for_position_spec(
    expr: &Expr,
    selectors: &[PartSelector],
) -> Vec<Vec<isize>> {
    let Some((first, rest)) = selectors.split_first() else {
        return vec![Vec::new()];
    };

    let Some(selected) = select_positioned_parts_for_selector(expr, first) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for (component, value) in selected {
        for mut tail in collect_exact_paths_for_position_spec(&value, rest) {
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
) -> Option<Vec<(isize, Expr)>> {
    match selector {
        PartSelector::Single(spec) => select_positioned_parts_for_atomic_spec(expr, spec),
        PartSelector::Multi(specs) => {
            let mut selected = Vec::new();
            for spec in specs {
                selected.extend(select_positioned_parts_for_atomic_spec(expr, spec)?);
            }
            Some(selected)
        }
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
