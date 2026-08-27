use rum_core::SharedArgs;
use std::collections::HashMap;

use super::grouping::IndexedGroups;
use super::*;

const NESTED_PATH_INDEX_THRESHOLD: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
enum NestedPathNode {
    Branch(NestedPathNodes),
    Leaf { value: Expr, delayed: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NestedPathNodes {
    entries: Vec<(Expr, NestedPathNode)>,
    positions: Option<HashMap<Expr, usize>>,
}

impl NestedPathNodes {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            positions: None,
        }
    }

    fn get_mut(&mut self, key: &Expr) -> Option<&mut NestedPathNode> {
        let position = self.positions.as_ref().map_or_else(
            || {
                self.entries
                    .iter()
                    .position(|(existing, _)| existing == key)
            },
            |positions| positions.get(key).copied(),
        )?;
        Some(&mut self.entries[position].1)
    }

    fn push(&mut self, key: Expr, node: NestedPathNode) {
        let position = self.entries.len();
        if let Some(positions) = self.positions.as_mut() {
            positions.insert(key.clone(), position);
        }
        self.entries.push((key, node));
        if self.positions.is_none() && self.entries.len() > NESTED_PATH_INDEX_THRESHOLD {
            self.positions = Some(
                self.entries
                    .iter()
                    .enumerate()
                    .map(|(index, (key, _))| (key.clone(), index))
                    .collect(),
            );
        }
    }
}

pub(crate) fn eval_key_group_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [function] => Ok(keyed_operator_form_slot_first(
            "KeyGroupBy",
            function.clone(),
        )),
        [expr, function] => {
            let Some(entries) = association_entry_forms_from_expr(expr) else {
                return Ok(inert_call(head, args));
            };

            resume_key_group_by_from_index(
                session,
                function.clone(),
                None,
                entries,
                0,
                IndexedGroups::from_entries(Vec::new()),
            )
        }
        [expr, function, reducer] => {
            let Some(entries) = association_entry_forms_from_expr(expr) else {
                return Ok(inert_call(head, args));
            };

            resume_key_group_by_from_index(
                session,
                function.clone(),
                Some(reducer.clone()),
                entries,
                0,
                IndexedGroups::from_entries(Vec::new()),
            )
        }
        _ => Ok(inert_call(head, args)),
    }
}

pub(crate) fn eval_association_key_deflatten(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(inert_call(head, args));
    };

    let Some(entries) = association_entry_forms_from_expr(expr) else {
        return Ok(inert_call(head, args));
    };

    let Some(deflattened) = association_key_deflatten_entries(entries) else {
        return Ok(inert_call(head, args));
    };

    Ok(deflattened)
}

pub(crate) fn eval_association_key_flatten(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(inert_call(head, args));
    };

    let Some(entries) = association_entry_forms_from_expr(expr) else {
        return Ok(inert_call(head, args));
    };

    Ok(render_association_entry_forms_preserving_heads(
        flatten_association_entries(Vec::new(), entries),
    ))
}

fn resume_key_group_by_from_index(
    session: &mut Session,
    function: Expr,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    mut index: usize,
    mut grouped: IndexedGroups<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_key_group_by_grouped(session, reducer, grouped.into_entries());
        };

        let group_key =
            match session.evaluate(Expr::call(function.clone(), vec![entry.key.clone()])) {
                Ok(group_key) => group_key,
                Err(EvalError::InputRequired { request }) => {
                    return session.suspend_with_frame_lazy(
                        Err(EvalError::InputRequired { request }),
                        || {
                            let entries = FrozenAssociationSequence::freeze(entries);
                            let grouped = PersistentAssociationGroups::freeze(grouped);
                            move |session, group_key| {
                                resume_suspended_key_group_by_with_key(
                                    session, function, reducer, entries, index, grouped, entry,
                                    group_key,
                                )
                            }
                        },
                    );
                }
                Err(error) => return Err(error),
            };

        grouped.push(group_key, entry);
        index += 1;
    }
}

fn resume_suspended_key_group_by_from_index(
    session: &mut Session,
    function: Expr,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    mut index: usize,
    mut grouped: PersistentAssociationGroups<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_key_group_by_grouped(session, reducer, grouped.to_entries());
        };

        let group_key =
            match session.evaluate(Expr::call(function.clone(), vec![entry.key.clone()])) {
                Ok(group_key) => group_key,
                Err(EvalError::InputRequired { request }) => {
                    return session.suspend_with_frame_lazy(
                        Err(EvalError::InputRequired { request }),
                        || {
                            move |session, group_key| {
                                resume_suspended_key_group_by_with_key(
                                    session, function, reducer, entries, index, grouped, entry,
                                    group_key,
                                )
                            }
                        },
                    );
                }
                Err(error) => return Err(error),
            };

        grouped = grouped.appended(group_key, entry);
        index += 1;
    }
}

fn resume_suspended_key_group_by_with_key(
    session: &mut Session,
    function: Expr,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    grouped: PersistentAssociationGroups<AssociationEntryForm>,
    entry: AssociationEntryForm,
    group_key: Expr,
) -> Result<Expr, EvalError> {
    let grouped = grouped.appended(group_key, entry);
    resume_suspended_key_group_by_from_index(
        session,
        function,
        reducer,
        entries,
        index + 1,
        grouped,
    )
}

fn finish_key_group_by_grouped(
    session: &mut Session,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    match reducer {
        None => Ok(render_association(key_group_by_groups(grouped))),
        Some(reducer) => {
            resume_key_group_by_reduce_from_index(session, reducer, grouped, 0, Vec::new())
        }
    }
}

fn resume_key_group_by_reduce_from_index(
    session: &mut Session,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((group_key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value = match session.evaluate(Expr::call(reducer.clone(), vec![group])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let grouped = FrozenAssociationSequence::freeze(grouped);
                        let reduced = PersistentAssociationSequence::freeze(reduced);
                        move |session, value| {
                            resume_suspended_key_group_by_reduce_with_value(
                                session, reducer, grouped, index, reduced, group_key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        reduced.push((group_key, value));
        index += 1;
    }
}

fn resume_suspended_key_group_by_reduce_from_index(
    session: &mut Session,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut reduced: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((group_key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced.to_vec()));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value = match session.evaluate(Expr::call(reducer.clone(), vec![group])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_key_group_by_reduce_with_value(
                                session, reducer, grouped, index, reduced, group_key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        reduced = reduced.appended((group_key, value));
        index += 1;
    }
}

fn resume_suspended_key_group_by_reduce_with_value(
    session: &mut Session,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    reduced: PersistentAssociationSequence<(Expr, Expr)>,
    group_key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let reduced = reduced.appended((group_key, value));
    resume_suspended_key_group_by_reduce_from_index(session, reducer, grouped, index + 1, reduced)
}

fn key_group_by_groups(grouped: Vec<(Expr, Vec<AssociationEntryForm>)>) -> Vec<(Expr, Expr)> {
    grouped
        .into_iter()
        .map(|(group_key, entries)| {
            (
                group_key,
                render_association_entry_forms_preserving_heads(entries),
            )
        })
        .collect()
}

fn association_key_deflatten_entries(entries: Vec<AssociationEntryForm>) -> Option<Expr> {
    let mut nested = NestedPathNodes::new();

    for entry in entries {
        let path = path_from_key(entry.key)?;
        insert_path(&mut nested, &path, entry.value, entry.delayed)?;
    }

    Some(render_nested_path_entries(nested))
}

fn path_from_key(key: Expr) -> Option<Vec<Expr>> {
    match list_items(&key) {
        Some(items) if items.is_empty() => None,
        Some(items) => Some(items.to_vec()),
        None => Some(vec![key]),
    }
}

fn insert_path(
    nodes: &mut NestedPathNodes,
    path: &[Expr],
    value: Expr,
    delayed: bool,
) -> Option<()> {
    let (key, remaining) = path.split_first()?;

    if remaining.is_empty() {
        if let Some(node) = nodes.get_mut(key) {
            if matches!(node, NestedPathNode::Branch(_)) {
                return None;
            }
            *node = NestedPathNode::Leaf { value, delayed };
        } else {
            nodes.push(key.clone(), NestedPathNode::Leaf { value, delayed });
        }
        return Some(());
    }

    if let Some(node) = nodes.get_mut(key) {
        let NestedPathNode::Branch(children) = node else {
            return None;
        };
        return insert_path(children, remaining, value, delayed);
    }

    let mut children = NestedPathNodes::new();
    insert_path(&mut children, remaining, value, delayed)?;
    nodes.push(key.clone(), NestedPathNode::Branch(children));
    Some(())
}

fn render_nested_path_entries(nodes: NestedPathNodes) -> Expr {
    render_association_entry_forms_preserving_heads(
        nodes
            .entries
            .into_iter()
            .map(render_nested_path_entry)
            .collect(),
    )
}

fn render_nested_path_entry((key, node): (Expr, NestedPathNode)) -> AssociationEntryForm {
    match node {
        NestedPathNode::Branch(children) => AssociationEntryForm {
            key,
            value: render_nested_path_entries(children),
            delayed: false,
        },
        NestedPathNode::Leaf { value, delayed } => AssociationEntryForm {
            key,
            value,
            delayed,
        },
    }
}

fn flatten_association_entries(
    prefix: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    let mut flattened = Vec::new();

    for entry in entries {
        let mut path = prefix.clone();
        path.push(entry.key);

        match association_entry_forms_from_expr(&entry.value) {
            Some(children) if !children.is_empty() => {
                flattened.extend(flatten_association_entries(path, children));
            }
            _ => flattened.push(AssociationEntryForm {
                key: Expr::call(system_expr(heads::structure::LIST), path),
                value: entry.value,
                delayed: entry.delayed,
            }),
        }
    }

    flattened
}

fn inert_call(head: &Expr, args: &[Expr]) -> Expr {
    Expr::call(head.clone(), args.to_vec())
}
