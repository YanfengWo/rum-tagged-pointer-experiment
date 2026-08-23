use super::spans;
use super::*;
use std::collections::HashMap;

const ASSOCIATION_BATCH_KEY_INDEX_MIN_ENTRIES: usize = 32;
const ASSOCIATION_BATCH_KEY_INDEX_MIN_KEYS: usize = 8;
const NO_ASSOCIATION_KEY_POSITION: usize = usize::MAX;

pub(in crate::runtime::kernel_data::structure_data) fn apply_part_selectors(
    expr: &Expr,
    selectors: &[PartSelector],
) -> Option<Expr> {
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

pub(super) fn collect_missing_key_selections(
    expr: &Expr,
    selectors: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) {
    let Some((first, rest)) = selectors.split_first() else {
        return;
    };

    if collect_missing_key_selections_for_key_selector(expr, first, rest, missing_keys) {
        return;
    }

    if let Some(shape) = select_parts_for_selector(expr, first) {
        collect_missing_key_selections_in_shape(shape, rest, missing_keys);
    }
}

fn collect_missing_key_selections_for_key_selector(
    expr: &Expr,
    selector: &PartSelector,
    rest: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) -> bool {
    match selector {
        PartSelector::Key(key) => {
            collect_missing_key_selection_for_key(expr, key, rest, missing_keys);
            true
        }
        PartSelector::Multi(specs)
            if association_entries_from_expr(expr).is_some()
                && specs
                    .iter()
                    .all(|spec| matches!(spec, PartSelector::Key(_))) =>
        {
            for spec in specs {
                let PartSelector::Key(key) = spec else {
                    continue;
                };
                collect_missing_key_selection_for_key(expr, key, rest, missing_keys);
            }
            true
        }
        _ => false,
    }
}

fn collect_missing_key_selection_for_key(
    expr: &Expr,
    key: &Expr,
    rest: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) {
    let Some(entries) = association_entries_from_expr(expr) else {
        return;
    };

    if let Some((_, value)) = entries.iter().find(|(entry_key, _)| entry_key == key) {
        collect_missing_key_selections(value, rest, missing_keys);
    } else {
        missing_keys.push((key.clone(), expr.clone()));
    }
}

fn collect_missing_key_selections_in_shape(
    shape: SelectedPartShape,
    selectors: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) {
    match shape {
        SelectedPartShape::Leaf(expr) => {
            collect_missing_key_selections(&expr, selectors, missing_keys);
        }
        SelectedPartShape::Branch(children) => {
            for child in children {
                collect_missing_key_selections_in_shape(child, selectors, missing_keys);
            }
        }
        SelectedPartShape::AssociationBranch(children) => {
            for (_, child) in children {
                collect_missing_key_selections_in_shape(child, selectors, missing_keys);
            }
        }
    }
}

fn selector_contains_key(selector: &PartSelector) -> bool {
    match selector {
        PartSelector::Single(_) => false,
        PartSelector::Key(_) => true,
        PartSelector::Multi(specs) => specs.iter().any(selector_contains_key),
    }
}

pub(super) fn selector_mixes_key_and_non_key(selector: &PartSelector) -> bool {
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

pub(super) fn select_parts_for_selector(
    expr: &Expr,
    selector: &PartSelector,
) -> Option<SelectedPartShape> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return select_association_parts_for_selector(expr, &entries, selector);
    }

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

fn select_association_parts_for_selector(
    expr: &Expr,
    entries: &[(Expr, Expr)],
    selector: &PartSelector,
) -> Option<SelectedPartShape> {
    match selector {
        PartSelector::Key(key) => Some(SelectedPartShape::Leaf(
            entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| missing_key_absent_expr(key_expr(key.clone()))),
        )),
        PartSelector::Single(AtomicPartSpec::Index(0)) => {
            Some(SelectedPartShape::Leaf(structural_head_expr(expr)))
        }
        PartSelector::Single(AtomicPartSpec::Index(index)) => {
            let (_, value) = association_entry_by_index(entries, *index)?;
            Some(SelectedPartShape::Leaf(value))
        }
        PartSelector::Single(spec @ (AtomicPartSpec::All | AtomicPartSpec::Span(_))) => {
            Some(SelectedPartShape::AssociationBranch(
                select_association_entries_for_atomic_spec(entries, spec)?
                    .into_iter()
                    .map(|(key, value)| (key, SelectedPartShape::Leaf(value)))
                    .collect(),
            ))
        }
        PartSelector::Multi(specs) => {
            if specs
                .iter()
                .all(|spec| matches!(spec, PartSelector::Key(_)))
            {
                return Some(SelectedPartShape::AssociationBranch(
                    select_association_entries_for_keys(entries, specs),
                ));
            }

            Some(SelectedPartShape::AssociationBranch(
                specs
                    .iter()
                    .map(|spec| {
                        let PartSelector::Single(AtomicPartSpec::Index(index)) = spec else {
                            return None;
                        };
                        let (key, value) = association_entry_by_index(entries, *index)?;
                        Some((key, SelectedPartShape::Leaf(value)))
                    })
                    .collect::<Option<Vec<_>>>()?,
            ))
        }
    }
}

fn select_association_entries_for_keys(
    entries: &[(Expr, Expr)],
    specs: &[PartSelector],
) -> Vec<(Expr, SelectedPartShape)> {
    if entries.len() < ASSOCIATION_BATCH_KEY_INDEX_MIN_ENTRIES
        || specs.len() < ASSOCIATION_BATCH_KEY_INDEX_MIN_KEYS
    {
        return specs
            .iter()
            .filter_map(|spec| match spec {
                PartSelector::Key(key) => Some((
                    key.clone(),
                    SelectedPartShape::Leaf(
                        entries
                            .iter()
                            .find(|(entry_key, _)| entry_key == key)
                            .map(|(_, value)| value.clone())
                            .unwrap_or_else(|| missing_key_absent_expr(key_expr(key.clone()))),
                    ),
                )),
                _ => None,
            })
            .collect();
    }

    // Index the query side, then visit the Association at most once.
    // The linked duplicate positions avoid one allocation per distinct key
    // while preserving selector order and duplicate-key results.
    let mut first_position_by_key: HashMap<Expr, usize> = HashMap::with_capacity(specs.len());
    let mut next_duplicate = vec![NO_ASSOCIATION_KEY_POSITION; specs.len()];
    let mut unmatched_distinct_keys = 0usize;
    for (position, spec) in specs.iter().enumerate() {
        let PartSelector::Key(key) = spec else {
            continue;
        };
        if let Some(previous) = first_position_by_key.insert(key.clone(), position) {
            next_duplicate[position] = previous;
        } else {
            unmatched_distinct_keys += 1;
        }
    }

    let mut selected_values = vec![None; specs.len()];
    for (entry_key, value) in entries {
        let Some(&first_position) = first_position_by_key.get(entry_key) else {
            continue;
        };

        let mut position = first_position;
        loop {
            selected_values[position] = Some(value.clone());
            let next = next_duplicate[position];
            if next == NO_ASSOCIATION_KEY_POSITION {
                break;
            }
            position = next;
        }

        unmatched_distinct_keys -= 1;
        if unmatched_distinct_keys == 0 {
            break;
        }
    }

    specs
        .iter()
        .zip(selected_values)
        .filter_map(|(spec, value)| match spec {
            PartSelector::Key(key) => Some((
                key.clone(),
                SelectedPartShape::Leaf(
                    value.unwrap_or_else(|| missing_key_absent_expr(key_expr(key.clone()))),
                ),
            )),
            _ => None,
        })
        .collect()
}

fn association_entry_by_index(entries: &[(Expr, Expr)], index: isize) -> Option<(Expr, Expr)> {
    if index == 0 {
        return None;
    }

    let normalized = normalize_structural_arg_position(index, entries.len())?;
    entries.get(normalized).cloned()
}

fn select_association_entries_for_atomic_spec(
    entries: &[(Expr, Expr)],
    spec: &AtomicPartSpec,
) -> Option<Vec<(Expr, Expr)>> {
    match spec {
        AtomicPartSpec::Index(index) => {
            association_entry_by_index(entries, *index).map(|entry| vec![entry])
        }
        AtomicPartSpec::All => Some(entries.to_vec()),
        AtomicPartSpec::Span(span) => {
            let components = spans::resolve_span_components(*span, entries.len())?;
            components
                .into_iter()
                .map(|component| {
                    let index = normalize_structural_arg_position(component, entries.len())?;
                    entries.get(index).cloned()
                })
                .collect()
        }
    }
}

pub(super) fn key_expr(key: Expr) -> Expr {
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
        AtomicPartSpec::Span(span) => spans::select_positioned_parts_for_span(expr, *span),
    }
}
