use super::*;

pub(super) fn resume_group_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
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
    call_args: Vec<Expr>,
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
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, key| {
                        resume_group_by_with_key(
                            session, head, call_args, specs, spec_index, reducer, items, index,
                            grouped, item, key,
                        )
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
                        return session.suspend_with_frame(
                            Err(EvalError::InputRequired { request }),
                            move |session, value| {
                                resume_group_by_with_item_value(
                                    session, head, call_args, specs, spec_index, reducer, items,
                                    index, grouped, key, value,
                                )
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
fn resume_group_by_with_key(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: Vec<Expr>,
    index: usize,
    grouped: IndexedGroups<Expr>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    let spec = specs[spec_index].clone();
    let Some(value_function) = spec.value_function.clone() else {
        return resume_group_by_with_item_value(
            session, head, call_args, specs, spec_index, reducer, items, index, grouped, key, item,
        );
    };

    let value = match session.evaluate(Expr::call(value_function, vec![item])) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame(
                Err(EvalError::InputRequired { request }),
                move |session, value| {
                    resume_group_by_with_item_value(
                        session, head, call_args, specs, spec_index, reducer, items, index,
                        grouped, key, value,
                    )
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_group_by_with_item_value(
        session, head, call_args, specs, spec_index, reducer, items, index, grouped, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_group_by_with_item_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    items: Vec<Expr>,
    index: usize,
    mut grouped: IndexedGroups<Expr>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    grouped.push(key, value);

    resume_group_by_from_index_with_groups(
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
    _call_args: Vec<Expr>,
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
    call_args: Vec<Expr>,
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
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    nested: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
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
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_specs = specs.clone();
    let suspended_reducer = reducer.clone();
    let suspended_grouped = grouped.clone();
    let suspended_nested = nested.clone();
    let suspended_key = key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        resume_group_by_nested_with_value(
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

    resume_group_by_nested_with_value(
        session, head, call_args, specs, spec_index, reducer, grouped, index, nested, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_group_by_nested_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    specs: Vec<GroupBySpec>,
    spec_index: usize,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    mut nested: Vec<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    nested.push((key, value));
    resume_group_by_nested_from_index(
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
    call_args: Vec<Expr>,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    let Some((key, values)) = grouped.get(index).cloned() else {
        return Ok(render_association(reduced));
    };

    let value_result = session.evaluate(Expr::call(
        reducer.clone(),
        vec![Expr::call(system_expr("List"), values)],
    ));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_reducer = reducer.clone();
    let suspended_grouped = grouped.clone();
    let suspended_reduced = reduced.clone();
    let suspended_key = key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        resume_group_by_reduce_with_value(
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

    resume_group_by_reduce_with_value(
        session, head, call_args, reducer, grouped, index, reduced, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_group_by_reduce_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    mut reduced: Vec<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    reduced.push((key, value));
    resume_group_by_reduce_from_index(
        session,
        head,
        call_args,
        reducer,
        grouped,
        index + 1,
        reduced,
    )
}
