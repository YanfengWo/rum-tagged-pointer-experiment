use super::super::grouping::IndexedGroups;
use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_merge(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
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

            let grouped = IndexedGroups::collect(associations.into_iter().flatten()).into_entries();

            resume_merge_from_index(session, combiner.clone(), grouped, 0, Vec::new())
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
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
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
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
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
                function.clone(),
                items.to_vec(),
                0,
                IndexedGroups::from_entries(Vec::new()),
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
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut merged: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged));
        };

        let value = match evaluate_merge_nested_group(session, combiner.clone(), values) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let grouped = FrozenAssociationSequence::freeze(grouped);
                        let merged = PersistentAssociationSequence::freeze(merged);
                        move |session, value| {
                            resume_suspended_merge_nested_with_value(
                                session, combiner, grouped, index, merged, key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        merged.push((key, value));
        index += 1;
    }
}

fn resume_suspended_merge_nested_from_index(
    session: &mut Session,
    combiner: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut merged: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged.to_vec()));
        };

        let value = match evaluate_merge_nested_group(session, combiner.clone(), values) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_merge_nested_with_value(
                                session, combiner, grouped, index, merged, key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        merged = merged.appended((key, value));
        index += 1;
    }
}

fn resume_suspended_merge_nested_with_value(
    session: &mut Session,
    combiner: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    index: usize,
    merged: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let merged = merged.appended((key, value));
    resume_suspended_merge_nested_from_index(session, combiner, grouped, index + 1, merged)
}

fn evaluate_merge_nested_group(
    session: &mut Session,
    combiner: Expr,
    values: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if let Some(nested_associations) = nested_association_values(&values) {
        return resume_merge_nested_from_index(
            session,
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
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut merged: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged));
        };

        let combined = match session.evaluate(Expr::call(
            combiner.clone(),
            vec![Expr::call(system_expr("List"), values)],
        )) {
            Ok(combined) => combined,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let grouped = FrozenAssociationSequence::freeze(grouped);
                        let merged = PersistentAssociationSequence::freeze(merged);
                        move |session, combined| {
                            resume_suspended_merge_with_value(
                                session, combiner, grouped, index, merged, key, combined,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        merged.push((key, combined));
        index += 1;
    }
}

fn parse_association_list(items: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    items.iter().map(association_entries_from_expr).collect()
}

fn group_merge_nested_values(associations: Vec<Vec<(Expr, Expr)>>) -> Vec<(Expr, Vec<Expr>)> {
    IndexedGroups::collect(associations.into_iter().flatten()).into_entries()
}

fn inert_call(head: &Expr, args: &[Expr]) -> Expr {
    Expr::call(head.clone(), args.to_vec())
}

fn resume_suspended_merge_from_index(
    session: &mut Session,
    combiner: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut merged: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged.to_vec()));
        };

        let combined = match session.evaluate(Expr::call(
            combiner.clone(),
            vec![Expr::call(system_expr("List"), values)],
        )) {
            Ok(combined) => combined,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, combined| {
                            resume_suspended_merge_with_value(
                                session, combiner, grouped, index, merged, key, combined,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        merged = merged.appended((key, combined));
        index += 1;
    }
}

fn resume_suspended_merge_with_value(
    session: &mut Session,
    combiner: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    index: usize,
    merged: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let merged = merged.appended((key, value));
    resume_suspended_merge_from_index(session, combiner, grouped, index + 1, merged)
}

fn resume_counts_by_from_index(
    session: &mut Session,
    function: Expr,
    items: Vec<Expr>,
    mut index: usize,
    mut counts: IndexedGroups<()>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association(
                counts
                    .into_entries()
                    .into_iter()
                    .map(|(key, occurrences)| (key, Expr::integer(BigInt::from(occurrences.len()))))
                    .collect(),
            ));
        };

        let key = match session.evaluate(Expr::call(function.clone(), vec![item])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let items = FrozenAssociationSequence::freeze(items);
                        let counts = PersistentAssociationGroups::freeze(counts);
                        move |session, key| {
                            resume_suspended_counts_by_with_value(
                                session, function, items, index, counts, key,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        counts.push(key, ());
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_counts_by_from_index(
    session: &mut Session,
    function: Expr,
    items: FrozenAssociationSequence<Expr>,
    mut index: usize,
    mut counts: PersistentAssociationGroups<()>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association(
                counts
                    .to_entries()
                    .into_iter()
                    .map(|(key, occurrences)| (key, Expr::integer(BigInt::from(occurrences.len()))))
                    .collect(),
            ));
        };

        let key = match session.evaluate(Expr::call(function.clone(), vec![item])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, key| {
                            resume_suspended_counts_by_with_value(
                                session, function, items, index, counts, key,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        counts = counts.appended(key, ());
        index += 1;
    }
}

fn resume_suspended_counts_by_with_value(
    session: &mut Session,
    function: Expr,
    items: FrozenAssociationSequence<Expr>,
    index: usize,
    counts: PersistentAssociationGroups<()>,
    key: Expr,
) -> Result<Expr, EvalError> {
    let counts = counts.appended(key, ());
    resume_suspended_counts_by_from_index(session, function, items, index + 1, counts)
}
