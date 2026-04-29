use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AtomicPartSpec {
    Index(isize),
    All,
    Span(PartSpanSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PartSpanSpec {
    pub(super) start: SpanEndpoint,
    pub(super) end: SpanEndpoint,
    pub(super) step: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpanEndpoint {
    Index(isize),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PartSelector {
    Single(AtomicPartSpec),
    Key(Expr),
    Multi(Vec<PartSelector>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedPartShape {
    Leaf(Expr),
    Branch(Vec<SelectedPartShape>),
    AssociationBranch(Vec<(Expr, SelectedPartShape)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtractPositionSpec {
    position_specs: Vec<Vec<PartSelector>>,
    multiple: bool,
}

impl Session {
    pub(in crate::runtime) fn eval_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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
        if association_entries_from_expr(&args[0]).is_some()
            && selectors.iter().any(selector_mixes_key_and_non_key)
        {
            self.push_message(
                "Part",
                "pmix",
                "Integer and Key specifications cannot be mixed in the same part specification.",
            );
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(apply_part_selectors(&args[0], &selectors)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(in crate::runtime) fn eval_extract(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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
}

pub(super) fn parse_part_selectors(exprs: &[Expr]) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector).collect()
}

fn parse_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn parse_scalar_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(spec) = parse_atomic_part_spec(expr) {
        return Some(PartSelector::Single(spec));
    }

    parse_key_part_selector(expr).map(PartSelector::Key)
}

fn parse_key_part_selector(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
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

pub(super) fn apply_part_selectors(expr: &Expr, selectors: &[PartSelector]) -> Option<Expr> {
    let Some((first, rest)) = selectors.split_first() else {
        return Some(expr.clone());
    };

    match select_parts_for_selector(expr, first)? {
        SelectedPartShape::Leaf(selected) => apply_part_selectors(&selected, rest),
        SelectedPartShape::Branch(selected) => Some(selected_shape_to_expr(
            SelectedPartShape::Branch(apply_part_selectors_to_shape(selected, rest)?),
            Some(selected_shape_head_for_expr(expr, first)),
        )),
        SelectedPartShape::AssociationBranch(selected) => Some(selected_shape_to_expr(
            SelectedPartShape::AssociationBranch(apply_part_selectors_to_keyed_shape(
                selected, rest,
            )?),
            None,
        )),
    }
}

fn selected_shape_head_for_expr(expr: &Expr, selector: &PartSelector) -> Expr {
    if association_entries_from_expr(expr).is_some() && selector_contains_key(selector) {
        system_expr("Association")
    } else {
        structural_head_expr(expr)
    }
}

fn selector_contains_key(selector: &PartSelector) -> bool {
    match selector {
        PartSelector::Single(_) => false,
        PartSelector::Key(_) => true,
        PartSelector::Multi(specs) => specs.iter().any(selector_contains_key),
    }
}

fn selector_mixes_key_and_non_key(selector: &PartSelector) -> bool {
    let PartSelector::Multi(specs) = selector else {
        return false;
    };

    let has_key = specs
        .iter()
        .any(|spec| matches!(spec, PartSelector::Key(_)));
    let has_non_key = specs
        .iter()
        .any(|spec| !matches!(spec, PartSelector::Key(_)));
    (has_key && has_non_key) || specs.iter().any(selector_mixes_key_and_non_key)
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
        SelectedPartShape::AssociationBranch(children) => {
            Some(SelectedPartShape::AssociationBranch(
                apply_part_selectors_to_keyed_shape(children, selectors)?,
            ))
        }
    }
}

fn apply_part_selectors_to_keyed_shape(
    selected: Vec<(Expr, SelectedPartShape)>,
    selectors: &[PartSelector],
) -> Option<Vec<(Expr, SelectedPartShape)>> {
    selected
        .into_iter()
        .map(|(key, shape)| {
            Some((
                key,
                apply_part_selectors_to_selected_shape(shape, selectors)?,
            ))
        })
        .collect()
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
        SelectedPartShape::AssociationBranch(children) => render_association(
            children
                .into_iter()
                .map(|(key, child)| (key, selected_shape_to_expr(child, None)))
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
        PartSelector::Key(key) => {
            let entries = association_entries_from_expr(expr)?;
            Some(SelectedPartShape::Leaf(
                entries
                    .iter()
                    .find(|(entry_key, _)| entry_key == key)
                    .map(|(_, value)| value.clone())
                    .unwrap_or_else(|| missing_key_absent_expr(key_expr(key.clone()))),
            ))
        }
        PartSelector::Multi(specs) => {
            if let Some(entries) = association_entries_from_expr(expr) {
                if specs
                    .iter()
                    .all(|spec| matches!(spec, PartSelector::Key(_)))
                {
                    return Some(SelectedPartShape::AssociationBranch(
                        specs
                            .iter()
                            .filter_map(|spec| match spec {
                                PartSelector::Key(key) => Some((
                                    key.clone(),
                                    SelectedPartShape::Leaf(
                                        entries
                                            .iter()
                                            .find(|(entry_key, _)| entry_key == key)
                                            .map(|(_, value)| value.clone())
                                            .unwrap_or_else(|| {
                                                missing_key_absent_expr(key_expr(key.clone()))
                                            }),
                                    ),
                                )),
                                _ => None,
                            })
                            .collect(),
                    ));
                }
            }

            let mut selected = Vec::new();
            for spec in specs {
                match (spec, select_parts_for_selector(expr, spec)?) {
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::Leaf(expr),
                    ) => {
                        selected.push(SelectedPartShape::Leaf(expr));
                    }
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::Branch(children),
                    ) => {
                        selected.extend(children);
                    }
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::AssociationBranch(children),
                    ) => {
                        selected.push(SelectedPartShape::AssociationBranch(children));
                    }
                    (PartSelector::Multi(_), shape) => selected.push(shape),
                }
            }
            Some(SelectedPartShape::Branch(selected))
        }
    }
}

fn key_expr(key: Expr) -> Expr {
    Expr::call(system_expr("Key"), vec![key])
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
            Expr::Atom(_) => Some(Vec::new()),
            Expr::Call { args, .. } => Some(
                args.iter()
                    .enumerate()
                    .map(|(index, value)| (index as isize + 1, value.clone()))
                    .collect(),
            ),
        },
        AtomicPartSpec::Span(span) => select_positioned_parts_for_span(expr, *span),
    }
}

pub(super) fn select_positioned_parts_for_span(
    expr: &Expr,
    spec: PartSpanSpec,
) -> Option<Vec<(isize, Expr)>> {
    match expr {
        Expr::Atom(_) => {
            resolve_span_components(spec, 0)?;
            Some(Vec::new())
        }
        Expr::Call { args, .. } => {
            let components = resolve_span_components(spec, args.len())?;
            Some(
                components
                    .into_iter()
                    .map(|component| {
                        let index = normalize_structural_arg_position(component, args.len())?;
                        Some((component, args[index].clone()))
                    })
                    .collect::<Option<Vec<_>>>()?,
            )
        }
    }
}

fn resolve_span_components(spec: PartSpanSpec, len: usize) -> Option<Vec<isize>> {
    let len = isize::try_from(len).ok()?;
    if spec.step > 0 {
        let start = resolve_positive_span_start(spec.start, len)?;
        let end = resolve_positive_span_end(spec.end, len)?;
        resolve_ordered_span_components(start, end, spec.step)
    } else {
        let start = resolve_negative_span_start(spec.start, len)?;
        let end = resolve_negative_span_end(spec.end, len)?;
        resolve_ordered_span_components(start, end, spec.step)
    }
}

fn resolve_ordered_span_components(start: isize, end: isize, step: isize) -> Option<Vec<isize>> {
    let mut current = start;
    let mut components = Vec::new();

    if step > 0 {
        if start > end {
            return (start == end + 1).then_some(Vec::new());
        }
        while current <= end {
            components.push(current);
            current += step;
        }
    } else {
        if start < end {
            return (start + 1 == end).then_some(Vec::new());
        }
        while current >= end {
            components.push(current);
            current += step;
        }
    }

    Some(components)
}

fn resolve_positive_span_start(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => Some(1),
        SpanEndpoint::Index(0) => Some(len + 1),
        SpanEndpoint::Index(index) if index > 0 => (index <= len + 1).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_positive_span_end(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => Some(len),
        SpanEndpoint::Index(0) => Some(0),
        SpanEndpoint::Index(index) if index > 0 => (index <= len).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (0..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_negative_span_start(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    if len == 0 {
        return None;
    }

    match endpoint {
        SpanEndpoint::All => Some(1),
        SpanEndpoint::Index(0) => None,
        SpanEndpoint::Index(index) if index > 0 => (index <= len).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_negative_span_end(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    if len == 0 {
        return None;
    }

    match endpoint {
        SpanEndpoint::All => Some(len),
        SpanEndpoint::Index(0) => None,
        SpanEndpoint::Index(index) if index > 0 => (index <= len + 1).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
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

pub(super) fn parse_single_position_spec(expr: &Expr) -> Option<Vec<PartSelector>> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
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

    args.iter().map(parse_part_selector).collect()
}

fn wrap_extracted_value(value: Expr, wrapper: Option<&Expr>) -> Expr {
    wrapper
        .cloned()
        .map(|head| Expr::call(head, vec![value.clone()]))
        .unwrap_or(value)
}
