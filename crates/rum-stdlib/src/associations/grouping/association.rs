use super::*;

pub(super) fn resume_association_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    resume_association_group_by_from_index_with_groups(
        session,
        head,
        call_args,
        specs,
        spec_index,
        reducer,
        entries,
        index,
        IndexedGroups::from_entries(grouped),
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_association_group_by_from_index_with_groups(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    mut index: usize,
    mut grouped: IndexedGroups<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_association_group_by_grouped(
                session,
                head,
                call_args,
                specs,
                spec_index,
                reducer,
                grouped.into_entries(),
            );
        };

        let spec = specs[spec_index].clone();
        let key = match session.evaluate(Expr::call(
            spec.key_function.clone(),
            vec![entry.value.clone()],
        )) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let specs = FrozenAssociationSequence::freeze(specs);
                        let entries = FrozenAssociationSequence::freeze(entries);
                        let grouped = PersistentAssociationGroups::freeze(grouped);
                        move |session, key| {
                            resume_suspended_association_group_by_with_key(
                                session, head, call_args, specs, spec_index, reducer, entries,
                                index, grouped, entry, key,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        let grouped_entry = match spec.value_function {
            None => entry,
            Some(value_function) => {
                let value = match session
                    .evaluate(Expr::call(value_function, vec![entry.value.clone()]))
                {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return session.suspend_with_frame_lazy(
                            Err(EvalError::InputRequired { request }),
                            || {
                                let specs = FrozenAssociationSequence::freeze(specs);
                                let entries = FrozenAssociationSequence::freeze(entries);
                                let grouped = PersistentAssociationGroups::freeze(grouped);
                                move |session, value| {
                                    let transformed_entry = AssociationEntryForm { value, ..entry };
                                    resume_suspended_association_group_by_with_entry(
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
                            },
                        );
                    }
                    Err(error) => return Err(error),
                };
                AssociationEntryForm { value, ..entry }
            }
        };

        grouped.push(key, grouped_entry);
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_association_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    mut index: usize,
    mut grouped: PersistentAssociationGroups<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_association_group_by_grouped(
                session,
                head,
                call_args,
                specs.as_slice().to_vec(),
                spec_index,
                reducer,
                grouped.to_entries(),
            );
        };

        let spec = specs
            .get(spec_index)
            .expect("association GroupBy spec should exist")
            .clone();
        let key = match session.evaluate(Expr::call(
            spec.key_function.clone(),
            vec![entry.value.clone()],
        )) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, key| {
                            resume_suspended_association_group_by_with_key(
                                session, head, call_args, specs, spec_index, reducer, entries,
                                index, grouped, entry, key,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        let grouped_entry = match spec.value_function {
            None => entry,
            Some(value_function) => {
                let value = match session
                    .evaluate(Expr::call(value_function, vec![entry.value.clone()]))
                {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return session.suspend_with_frame_lazy(
                            Err(EvalError::InputRequired { request }),
                            || {
                                move |session, value| {
                                    let transformed_entry = AssociationEntryForm { value, ..entry };
                                    resume_suspended_association_group_by_with_entry(
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
                            },
                        );
                    }
                    Err(error) => return Err(error),
                };
                AssociationEntryForm { value, ..entry }
            }
        };

        grouped = grouped.appended(key, grouped_entry);
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_association_group_by_with_key(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    grouped: PersistentAssociationGroups<AssociationEntryForm>,
    entry: AssociationEntryForm,
    key: Expr,
) -> Result<Expr, EvalError> {
    let spec = specs
        .get(spec_index)
        .expect("association GroupBy spec should exist")
        .clone();
    let Some(value_function) = spec.value_function else {
        return resume_suspended_association_group_by_with_entry(
            session, head, call_args, specs, spec_index, reducer, entries, index, grouped, entry,
            key,
        );
    };

    let value = match session.evaluate(Expr::call(value_function, vec![entry.value.clone()])) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame_lazy(
                Err(EvalError::InputRequired { request }),
                || {
                    move |session, value| {
                        let transformed_entry = AssociationEntryForm { value, ..entry };
                        resume_suspended_association_group_by_with_entry(
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
                },
            );
        }
        Err(error) => return Err(error),
    };

    let transformed_entry = AssociationEntryForm { value, ..entry };
    resume_suspended_association_group_by_with_entry(
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
fn resume_suspended_association_group_by_with_entry(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    grouped: PersistentAssociationGroups<AssociationEntryForm>,
    entry: AssociationEntryForm,
    key: Expr,
) -> Result<Expr, EvalError> {
    let grouped = grouped.appended(key, entry);
    resume_suspended_association_group_by_from_index(
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
    _call_args: SharedArgs,
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
    call_args: SharedArgs,
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
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut nested: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
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
        let value = match value_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let specs = FrozenAssociationSequence::freeze(specs);
                        let grouped = FrozenAssociationSequence::freeze(grouped);
                        let nested = PersistentAssociationSequence::freeze(nested);
                        move |session, value| {
                            resume_suspended_association_group_by_nested_with_value(
                                session, head, call_args, specs, spec_index, reducer, grouped,
                                index, nested, key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        nested.push((key, value));
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_association_group_by_nested_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut nested: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(nested.to_vec()));
        };

        let value_result = resume_association_group_by_from_index(
            session,
            head.clone(),
            call_args.clone(),
            specs.as_slice().to_vec(),
            spec_index,
            reducer.clone(),
            entries,
            0,
            Vec::new(),
        );
        let value = match value_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_association_group_by_nested_with_value(
                                session, head, call_args, specs, spec_index, reducer, grouped,
                                index, nested, key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        nested = nested.appended((key, value));
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_association_group_by_nested_with_value(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    nested: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let nested = nested.appended((key, value));
    resume_suspended_association_group_by_nested_from_index(
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
    call_args: SharedArgs,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value_result = session.evaluate(Expr::call(reducer.clone(), vec![group]));
        let value = match value_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let grouped = FrozenAssociationSequence::freeze(grouped);
                        let reduced = PersistentAssociationSequence::freeze(reduced);
                        move |session, value| {
                            resume_suspended_association_group_by_reduce_with_value(
                                session, head, call_args, reducer, grouped, index, reduced, key,
                                value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        reduced.push((key, value));
        index += 1;
    }
}

fn resume_suspended_association_group_by_reduce_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut reduced: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced.to_vec()));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value_result = session.evaluate(Expr::call(reducer.clone(), vec![group]));
        let value = match value_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_association_group_by_reduce_with_value(
                                session, head, call_args, reducer, grouped, index, reduced, key,
                                value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        reduced = reduced.appended((key, value));
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_association_group_by_reduce_with_value(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    reduced: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let reduced = reduced.appended((key, value));
    resume_suspended_association_group_by_reduce_from_index(
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
