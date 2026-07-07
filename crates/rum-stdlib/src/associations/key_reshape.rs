use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum NestedPathNode {
    Branch(Vec<(Expr, NestedPathNode)>),
    Leaf { value: Expr, delayed: bool },
}

pub(crate) fn eval_key_group_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
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
                head.clone(),
                args.to_vec(),
                function.clone(),
                None,
                entries,
                0,
                Vec::new(),
            )
        }
        [expr, function, reducer] => {
            let Some(entries) = association_entry_forms_from_expr(expr) else {
                return Ok(inert_call(head, args));
            };

            resume_key_group_by_from_index(
                session,
                head.clone(),
                args.to_vec(),
                function.clone(),
                Some(reducer.clone()),
                entries,
                0,
                Vec::new(),
            )
        }
        _ => Ok(inert_call(head, args)),
    }
}

pub(crate) fn eval_association_key_deflatten(
    _session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
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
    args: &[Expr],
) -> Result<Expr, EvalError> {
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

#[allow(clippy::too_many_arguments)]
fn resume_key_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    let Some(entry) = entries.get(index).cloned() else {
        return finish_key_group_by_grouped(session, head, call_args, reducer, grouped);
    };

    let group_key_result = session.evaluate(Expr::call(function.clone(), vec![entry.key.clone()]));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_function = function.clone();
    let suspended_reducer = reducer.clone();
    let suspended_entries = entries.clone();
    let suspended_grouped = grouped.clone();
    let suspended_entry = entry.clone();
    let group_key = session.suspend_with_frame(group_key_result, move |session, group_key| {
        resume_key_group_by_with_key(
            session,
            suspended_head,
            suspended_call_args,
            suspended_function,
            suspended_reducer,
            suspended_entries,
            index,
            suspended_grouped,
            suspended_entry,
            group_key,
        )
    })?;

    resume_key_group_by_with_key(
        session, head, call_args, function, reducer, entries, index, grouped, entry, group_key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_group_by_with_key(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    mut grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    entry: AssociationEntryForm,
    group_key: Expr,
) -> Result<Expr, EvalError> {
    if let Some((_, values)) = grouped
        .iter_mut()
        .find(|(existing, _)| existing == &group_key)
    {
        values.push(entry);
    } else {
        grouped.push((group_key, vec![entry]));
    }

    resume_key_group_by_from_index(
        session,
        head,
        call_args,
        function,
        reducer,
        entries,
        index + 1,
        grouped,
    )
}

fn finish_key_group_by_grouped(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    match reducer {
        None => Ok(render_association(key_group_by_groups(grouped))),
        Some(reducer) => resume_key_group_by_reduce_from_index(
            session,
            head,
            call_args,
            reducer,
            grouped,
            0,
            Vec::new(),
        ),
    }
}

fn resume_key_group_by_reduce_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    let Some((group_key, entries)) = grouped.get(index).cloned() else {
        return Ok(render_association(reduced));
    };

    let group = render_association_entry_forms_preserving_heads(entries);
    let value_result = session.evaluate(Expr::call(reducer.clone(), vec![group]));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_reducer = reducer.clone();
    let suspended_grouped = grouped.clone();
    let suspended_reduced = reduced.clone();
    let suspended_group_key = group_key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        resume_key_group_by_reduce_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_reducer,
            suspended_grouped,
            index,
            suspended_reduced,
            suspended_group_key,
            value,
        )
    })?;

    resume_key_group_by_reduce_with_value(
        session, head, call_args, reducer, grouped, index, reduced, group_key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_group_by_reduce_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    mut reduced: Vec<(Expr, Expr)>,
    group_key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    reduced.push((group_key, value));
    resume_key_group_by_reduce_from_index(
        session,
        head,
        call_args,
        reducer,
        grouped,
        index + 1,
        reduced,
    )
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
    let mut nested = Vec::new();

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
    nodes: &mut Vec<(Expr, NestedPathNode)>,
    path: &[Expr],
    value: Expr,
    delayed: bool,
) -> Option<()> {
    let (key, remaining) = path.split_first()?;

    if remaining.is_empty() {
        if let Some((_, node)) = nodes.iter_mut().find(|(existing, _)| existing == key) {
            if matches!(node, NestedPathNode::Branch(_)) {
                return None;
            }
            *node = NestedPathNode::Leaf { value, delayed };
        } else {
            nodes.push((key.clone(), NestedPathNode::Leaf { value, delayed }));
        }
        return Some(());
    }

    if let Some((_, node)) = nodes.iter_mut().find(|(existing, _)| existing == key) {
        let NestedPathNode::Branch(children) = node else {
            return None;
        };
        return insert_path(children, remaining, value, delayed);
    }

    let mut children = Vec::new();
    insert_path(&mut children, remaining, value, delayed)?;
    nodes.push((key.clone(), NestedPathNode::Branch(children)));
    Some(())
}

fn render_nested_path_entries(nodes: Vec<(Expr, NestedPathNode)>) -> Expr {
    render_association_entry_forms_preserving_heads(
        nodes.into_iter().map(render_nested_path_entry).collect(),
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
