use super::*;

pub(crate) fn eval_merge(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [combiner] => Ok(keyed_operator_form_slot_first("Merge", combiner.clone())),
        [associations_expr, combiner] => {
            let associations =
                match parse_keyed_entries_list_expr_with_diagnostics(associations_expr) {
                    Ok(associations) => associations,
                    Err(KeyedEntriesListError::NotList) => {
                        return Ok(session.invalid_call_with_message(
                            "Merge",
                            args,
                            "list1",
                            invalid_keyed_collection_list_message(associations_expr),
                        ));
                    }
                    Err(KeyedEntriesListError::InvalidElement(expr)) => {
                        return Ok(session.invalid_call_with_message(
                            "Merge",
                            args,
                            "list1",
                            invalid_keyed_collection_list_message(&expr),
                        ));
                    }
                };

            if associations.is_empty() {
                return Ok(render_association(Vec::new()));
            };

            let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
            for association in associations {
                for (key, value) in association {
                    if let Some((_, values)) =
                        grouped.iter_mut().find(|(existing, _)| existing == &key)
                    {
                        values.push(value);
                    } else {
                        grouped.push((key, vec![value]));
                    }
                }
            }

            resume_merge_from_index(
                session,
                head.clone(),
                args.to_vec(),
                combiner.clone(),
                grouped,
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_message(
            "Merge",
            args,
            "argt",
            one_or_two_argument_count_message("Merge", args.len()),
        )),
    }
}

pub(crate) fn eval_merge_nested(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [combiner] => Ok(keyed_operator_form_slot_first(
            "MergeNested",
            combiner.clone(),
        )),
        [associations_expr, combiner] => {
            let Some(items) = list_items(associations_expr) else {
                return Ok(inert_call(head, args));
            };

            let Some(associations) = parse_association_list(items) else {
                return Ok(inert_call(head, args));
            };

            resume_merge_nested_from_index(
                session,
                head.clone(),
                args.to_vec(),
                combiner.clone(),
                group_merge_nested_values(associations),
                0,
                Vec::new(),
            )
        }
        _ => Ok(inert_call(head, args)),
    }
}

pub(crate) fn eval_counts_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [function] => Ok(keyed_operator_form_slot_first("CountsBy", function.clone())),
        [expr, function] => {
            let Some(items) = list_items(expr) else {
                return Ok(session.invalid_call_with_message(
                    "CountsBy",
                    args,
                    "list",
                    list_expected_message(1, expr),
                ));
            };

            resume_counts_by_from_index(
                session,
                head.clone(),
                args.to_vec(),
                function.clone(),
                items.to_vec(),
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_message(
            "CountsBy",
            args,
            "argt",
            one_or_two_argument_count_message("CountsBy", args.len()),
        )),
    }
}

fn resume_merge_nested_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    merged: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    let Some((key, values)) = grouped.get(index).cloned() else {
        return Ok(render_association(merged));
    };

    let value_result = evaluate_merge_nested_group(
        session,
        head.clone(),
        call_args.clone(),
        combiner.clone(),
        values,
    );
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_combiner = combiner.clone();
    let suspended_grouped = grouped.clone();
    let suspended_merged = merged.clone();
    let suspended_key = key.clone();
    let value = session.suspend_with_frame(value_result, move |session, value| {
        resume_merge_nested_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_combiner,
            suspended_grouped,
            index,
            suspended_merged,
            suspended_key,
            value,
        )
    })?;

    resume_merge_nested_with_value(
        session, head, call_args, combiner, grouped, index, merged, key, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_merge_nested_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    mut merged: Vec<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    merged.push((key, value));
    resume_merge_nested_from_index(
        session,
        head,
        call_args,
        combiner,
        grouped,
        index + 1,
        merged,
    )
}

fn evaluate_merge_nested_group(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    combiner: Expr,
    values: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if let Some(nested_associations) = nested_association_values(&values) {
        return resume_merge_nested_from_index(
            session,
            head,
            call_args,
            combiner,
            group_merge_nested_values(nested_associations),
            0,
            Vec::new(),
        );
    }

    session.evaluate(Expr::call(
        combiner,
        vec![Expr::call(system_expr("List"), values)],
    ))
}

fn nested_association_values(values: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    values.iter().map(association_entries_from_expr).collect()
}

fn resume_merge_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    merged: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    let Some((key, values)) = grouped.get(index).cloned() else {
        return Ok(render_association(merged));
    };

    let combined_result = session.evaluate(Expr::call(
        combiner.clone(),
        vec![Expr::call(system_expr("List"), values)],
    ));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_combiner = combiner.clone();
    let suspended_grouped = grouped.clone();
    let suspended_merged = merged.clone();
    let suspended_key = key.clone();
    let combined = session.suspend_with_frame(combined_result, move |session, combined| {
        resume_merge_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_combiner,
            suspended_grouped,
            index,
            suspended_merged,
            suspended_key,
            combined,
        )
    })?;

    resume_merge_with_value(
        session, head, call_args, combiner, grouped, index, merged, key, combined,
    )
}

fn parse_association_list(items: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    items.iter().map(association_entries_from_expr).collect()
}

fn group_merge_nested_values(associations: Vec<Vec<(Expr, Expr)>>) -> Vec<(Expr, Vec<Expr>)> {
    let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();

    for association in associations {
        for (key, value) in association {
            if let Some((_, values)) = grouped.iter_mut().find(|(existing, _)| existing == &key) {
                values.push(value);
            } else {
                grouped.push((key, vec![value]));
            }
        }
    }

    grouped
}

fn inert_call(head: &Expr, args: &[Expr]) -> Expr {
    Expr::call(head.clone(), args.to_vec())
}

#[allow(clippy::too_many_arguments)]
fn resume_merge_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    mut merged: Vec<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    merged.push((key, value));
    resume_merge_from_index(
        session,
        head,
        call_args,
        combiner,
        grouped,
        index + 1,
        merged,
    )
}

fn resume_counts_by_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    counts: Vec<(Expr, usize)>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        return Ok(render_association(
            counts
                .into_iter()
                .map(|(key, count)| (key, Expr::integer(BigInt::from(count))))
                .collect(),
        ));
    };

    let key_result = session.evaluate(Expr::call(function.clone(), vec![item]));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_function = function.clone();
    let suspended_items = items.clone();
    let suspended_counts = counts.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_counts_by_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_function,
            suspended_items,
            index,
            suspended_counts,
            key,
        )
    })?;

    resume_counts_by_with_value(
        session, head, call_args, function, items, index, counts, key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_counts_by_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    mut counts: Vec<(Expr, usize)>,
    key: Expr,
) -> Result<Expr, EvalError> {
    increment_count(&mut counts, key);
    resume_counts_by_from_index(session, head, call_args, function, items, index + 1, counts)
}
