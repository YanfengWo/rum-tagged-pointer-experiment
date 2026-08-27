use super::*;

pub(super) fn resume_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: Vec<Expr>,
    index: usize,
    grouped: Vec<(Expr, Vec<Expr>)>,
) -> Result<Expr, EvalError> {
    resume_group_by_from_index_with_groups(
        session,
        head,
        call_args,
        specs,
        spec_index,
        reducer,
        items,
        index,
        IndexedGroups::from_entries(grouped),
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_group_by_from_index_with_groups(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: Vec<Expr>,
    mut index: usize,
    mut grouped: IndexedGroups<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return finish_group_by_grouped(
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
        let key = match session.evaluate(Expr::call(spec.key_function.clone(), vec![item.clone()]))
        {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let specs = FrozenAssociationSequence::freeze(specs);
                        let items = FrozenAssociationSequence::freeze(items);
                        let grouped = PersistentAssociationGroups::freeze(grouped);
                        move |session, key| {
                            resume_suspended_group_by_with_key(
                                session, head, call_args, specs, spec_index, reducer, items, index,
                                grouped, item, key,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        let value = match spec.value_function {
            None => item,
            Some(value_function) => {
                match session.evaluate(Expr::call(value_function, vec![item])) {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return session.suspend_with_frame_lazy(
                            Err(EvalError::InputRequired { request }),
                            || {
                                let specs = FrozenAssociationSequence::freeze(specs);
                                let items = FrozenAssociationSequence::freeze(items);
                                let grouped = PersistentAssociationGroups::freeze(grouped);
                                move |session, value| {
                                    resume_suspended_group_by_with_item_value(
                                        session, head, call_args, specs, spec_index, reducer,
                                        items, index, grouped, key, value,
                                    )
                                }
                            },
                        );
                    }
                    Err(error) => return Err(error),
                }
            }
        };

        grouped.push(key, value);
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: FrozenAssociationSequence<Expr>,
    mut index: usize,
    mut grouped: PersistentAssociationGroups<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return finish_group_by_grouped(
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
            .expect("group-by spec should exist")
            .clone();
        let key = match session.evaluate(Expr::call(spec.key_function.clone(), vec![item.clone()]))
        {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, key| {
                            resume_suspended_group_by_with_key(
                                session, head, call_args, specs, spec_index, reducer, items, index,
                                grouped, item, key,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        let value = match spec.value_function {
            None => item,
            Some(value_function) => {
                match session.evaluate(Expr::call(value_function, vec![item])) {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return session.suspend_with_frame_lazy(
                            Err(EvalError::InputRequired { request }),
                            || {
                                move |session, value| {
                                    resume_suspended_group_by_with_item_value(
                                        session, head, call_args, specs, spec_index, reducer,
                                        items, index, grouped, key, value,
                                    )
                                }
                            },
                        );
                    }
                    Err(error) => return Err(error),
                }
            }
        };

        grouped = grouped.appended(key, value);
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_group_by_with_key(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: FrozenAssociationSequence<Expr>,
    index: usize,
    grouped: PersistentAssociationGroups<Expr>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    let spec = specs
        .get(spec_index)
        .expect("group-by spec should exist")
        .clone();
    let Some(value_function) = spec.value_function else {
        return resume_suspended_group_by_with_item_value(
            session, head, call_args, specs, spec_index, reducer, items, index, grouped, key, item,
        );
    };

    let value = match session.evaluate(Expr::call(value_function, vec![item])) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame_lazy(
                Err(EvalError::InputRequired { request }),
                || {
                    move |session, value| {
                        resume_suspended_group_by_with_item_value(
                            session, head, call_args, specs, spec_index, reducer, items, index,
                            grouped, key, value,
                        )
                    }
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_suspended_group_by_with_item_value(
        session, head, call_args, specs, spec_index, reducer, items, index, grouped, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_group_by_with_item_value(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: FrozenAssociationSequence<Expr>,
    index: usize,
    grouped: PersistentAssociationGroups<Expr>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let grouped = grouped.appended(key, value);
    resume_suspended_group_by_from_index(
        session,
        head,
        call_args,
        specs,
        spec_index,
        reducer,
        items,
        index + 1,
        grouped,
    )
}

pub(super) fn finish_group_by_leaf(
    session: &mut Session,
    _head: Expr,
    _call_args: SharedArgs,
    reducer: Option<Expr>,
    values: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if reducer.is_some() && values.is_empty() {
        return Ok(render_association(Vec::new()));
    }

    let group = Expr::call(system_expr("List"), values);
    match reducer {
        None => Ok(group),
        Some(reducer) => session.evaluate(Expr::call(reducer, vec![group])),
    }
}

fn finish_group_by_grouped(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<Expr>)>,
) -> Result<Expr, EvalError> {
    if spec_index + 1 < specs.len() {
        return resume_group_by_nested_from_index(
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
        None => Ok(render_association(
            grouped
                .into_iter()
                .map(|(key, values)| (key, Expr::call(system_expr("List"), values)))
                .collect(),
        )),
        Some(reducer) => resume_group_by_reduce_from_index(
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
fn resume_group_by_nested_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut nested: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(nested));
        };

        let value_result = resume_group_by_from_index(
            session,
            head.clone(),
            call_args.clone(),
            specs.clone(),
            spec_index,
            reducer.clone(),
            values,
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
                            resume_suspended_group_by_nested_with_value(
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
fn resume_suspended_group_by_nested_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut nested: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(nested.to_vec()));
        };

        let value_result = resume_group_by_from_index(
            session,
            head.clone(),
            call_args.clone(),
            specs.as_slice().to_vec(),
            spec_index,
            reducer.clone(),
            values,
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
                            resume_suspended_group_by_nested_with_value(
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
fn resume_suspended_group_by_nested_with_value(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    specs: FrozenAssociationSequence<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    index: usize,
    nested: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let nested = nested.appended((key, value));
    resume_suspended_group_by_nested_from_index(
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

fn resume_group_by_reduce_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced));
        };

        let value_result = session.evaluate(Expr::call(
            reducer.clone(),
            vec![Expr::call(system_expr("List"), values)],
        ));
        let value = match value_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let grouped = FrozenAssociationSequence::freeze(grouped);
                        let reduced = PersistentAssociationSequence::freeze(reduced);
                        move |session, value| {
                            resume_suspended_group_by_reduce_with_value(
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

fn resume_suspended_group_by_reduce_from_index(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut reduced: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced.to_vec()));
        };

        let value_result = session.evaluate(Expr::call(
            reducer.clone(),
            vec![Expr::call(system_expr("List"), values)],
        ));
        let value = match value_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_group_by_reduce_with_value(
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
fn resume_suspended_group_by_reduce_with_value(
    session: &mut Session,
    head: Expr,
    call_args: SharedArgs,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    index: usize,
    reduced: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let reduced = reduced.appended((key, value));
    resume_suspended_group_by_reduce_from_index(
        session,
        head,
        call_args,
        reducer,
        grouped,
        index + 1,
        reduced,
    )
}
