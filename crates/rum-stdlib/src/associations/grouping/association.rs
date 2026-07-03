use super::*;

pub(super) fn resume_association_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    let Some(entry) = entries.get(index).cloned() else {
        return finish_association_group_by_grouped(
            session, head, call_args, specs, spec_index, reducer, grouped,
        );
    };

    let spec = specs[spec_index].clone();
    let key_result = session.evaluate(Expr::call(
        spec.key_function.clone(),
        vec![entry.value.clone()],
    ));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_specs = specs.clone();
    let suspended_reducer = reducer.clone();
    let suspended_entries = entries.clone();
    let suspended_grouped = grouped.clone();
    let suspended_entry = entry.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_association_group_by_with_key(
            session,
            suspended_head,
            suspended_call_args,
            suspended_specs,
            spec_index,
            suspended_reducer,
            suspended_entries,
            index,
            suspended_grouped,
            suspended_entry,
            key,
        )
    })?;

    resume_association_group_by_with_key(
        session, head, call_args, specs, spec_index, reducer, entries, index, grouped, entry, key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_association_group_by_with_key(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    entry: AssociationEntryForm,
    key: Expr,
) -> Result<Expr, EvalError> {
    let spec = specs[spec_index].clone();
    let Some(value_function) = spec.value_function.clone() else {
        return resume_association_group_by_with_entry(
            session, head, call_args, specs, spec_index, reducer, entries, index, grouped, entry,
            key,
        );
    };

    let value_result = session.evaluate(Expr::call(value_function, vec![entry.value.clone()]));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_specs = specs.clone();
    let suspended_reducer = reducer.clone();
    let suspended_entries = entries.clone();
    let suspended_grouped = grouped.clone();
    let suspended_entry = entry.clone();
    let suspended_key = key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        let transformed_entry = AssociationEntryForm {
            value,
            ..suspended_entry
        };
        resume_association_group_by_with_entry(
            session,
            suspended_head,
            suspended_call_args,
            suspended_specs,
            spec_index,
            suspended_reducer,
            suspended_entries,
            index,
            suspended_grouped,
            transformed_entry,
            suspended_key,
        )
    })?;

    let transformed_entry = AssociationEntryForm { value, ..entry };
    resume_association_group_by_with_entry(
        session,
        head,
        call_args,
        specs,
        spec_index,
        reducer,
        entries,
        index,
        grouped,
        transformed_entry,
        key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_association_group_by_with_entry(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    mut grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    entry: AssociationEntryForm,
    key: Expr,
) -> Result<Expr, EvalError> {
    if let Some((_, values)) = grouped.iter_mut().find(|(existing, _)| existing == &key) {
        values.push(entry);
    } else {
        grouped.push((key, vec![entry]));
    }

    resume_association_group_by_from_index(
        session,
        head,
        call_args,
        specs,
        spec_index,
        reducer,
        entries,
        index + 1,
        grouped,
    )
}

pub(super) fn finish_association_group_by_leaf(
    session: &mut Session,
    _head: Expr,
    _call_args: Vec<Expr>,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    if reducer.is_some() && entries.is_empty() {
        return Ok(render_association(Vec::new()));
    }

    let group = render_association_entry_forms_preserving_heads(entries);
    match reducer {
        None => Ok(group),
        Some(reducer) => session.evaluate(Expr::call(reducer, vec![group])),
    }
}

fn finish_association_group_by_grouped(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    if spec_index + 1 < specs.len() {
        return resume_association_group_by_nested_from_index(
            session,
            head,
            call_args,
            specs,
            spec_index + 1,
            reducer,
            grouped,
            0,
            Vec::new(),
        );
    }

    match reducer {
        None => Ok(render_association(association_group_by_groups(grouped))),
        Some(reducer) => resume_association_group_by_reduce_from_index(
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

#[allow(clippy::too_many_arguments)]
fn resume_association_group_by_nested_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    nested: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    let Some((key, entries)) = grouped.get(index).cloned() else {
        return Ok(render_association(nested));
    };

    let value_result = resume_association_group_by_from_index(
        session,
        head.clone(),
        call_args.clone(),
        specs.clone(),
        spec_index,
        reducer.clone(),
        entries,
        0,
        Vec::new(),
    );
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_specs = specs.clone();
    let suspended_reducer = reducer.clone();
    let suspended_grouped = grouped.clone();
    let suspended_nested = nested.clone();
    let suspended_key = key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        resume_association_group_by_nested_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_specs,
            spec_index,
            suspended_reducer,
            suspended_grouped,
            index,
            suspended_nested,
            suspended_key,
            value,
        )
    })?;

    resume_association_group_by_nested_with_value(
        session, head, call_args, specs, spec_index, reducer, grouped, index, nested, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_association_group_by_nested_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    mut nested: Vec<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    nested.push((key, value));
    resume_association_group_by_nested_from_index(
        session,
        head,
        call_args,
        specs,
        spec_index,
        reducer,
        grouped,
        index + 1,
        nested,
    )
}

fn resume_association_group_by_reduce_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    let Some((key, entries)) = grouped.get(index).cloned() else {
        return Ok(render_association(reduced));
    };

    let group = render_association_entry_forms_preserving_heads(entries);
    let value_result = session.evaluate(Expr::call(reducer.clone(), vec![group]));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_reducer = reducer.clone();
    let suspended_grouped = grouped.clone();
    let suspended_reduced = reduced.clone();
    let suspended_key = key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        resume_association_group_by_reduce_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_reducer,
            suspended_grouped,
            index,
            suspended_reduced,
            suspended_key,
            value,
        )
    })?;

    resume_association_group_by_reduce_with_value(
        session, head, call_args, reducer, grouped, index, reduced, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_association_group_by_reduce_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    mut reduced: Vec<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    reduced.push((key, value));
    resume_association_group_by_reduce_from_index(
        session,
        head,
        call_args,
        reducer,
        grouped,
        index + 1,
        reduced,
    )
}

fn association_group_by_groups(
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Vec<(Expr, Expr)> {
    grouped
        .into_iter()
        .map(|(key, entries)| {
            (
                key,
                render_association_entry_forms_preserving_heads(entries),
            )
        })
        .collect()
}
