use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum CatenateStep {
    Append(Vec<Expr>),
    AssociationValue { value: Expr, delayed: bool },
    Invalid(Expr),
}

pub(crate) fn eval_catenate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [associations_expr] = args else {
        return Ok(session.invalid_call_with_message(
            "Catenate",
            args,
            "argx",
            catenate_arg_count_message(args.len()),
        ));
    };

    if let Some(entries) = association_entry_forms_from_expr(associations_expr) {
        return resume_catenate_direct_association_entries_from_index(
            session,
            head.clone(),
            args.to_vec(),
            entries,
            0,
            Vec::new(),
        );
    }

    let Some(items) = list_items(associations_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    resume_catenate_steps_from_index(
        session,
        head.clone(),
        args.to_vec(),
        catenate_steps_for_list_items(items),
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_counts(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (expr, spec) = match args {
        [expr] => (expr, None),
        [expr, spec] => (expr, Some(spec)),
        _ => {
            return Ok(session.invalid_call_with_message(
                "Counts",
                args,
                "argt",
                one_or_two_argument_count_message("Counts", args.len()),
            ));
        }
    };

    if let Some(entries) = association_entry_forms_from_expr(expr) {
        let spec_keys = match counts_spec_keys(spec) {
            Ok(spec_keys) => spec_keys,
            Err(invalid_spec) => {
                return Ok(session.invalid_call_with_message(
                    "Counts",
                    args,
                    "invl",
                    invalid_list_message(&invalid_spec),
                ));
            }
        };

        return resume_counts_association_entries_from_index(
            session,
            entries,
            0,
            Vec::new(),
            spec_keys,
        );
    }

    let Some(items) = list_items(expr) else {
        return Ok(session.invalid_call_with_message(
            "Counts",
            args,
            "invrp",
            invalid_association_or_list_message(expr),
        ));
    };

    let spec_keys = match counts_spec_keys(spec) {
        Ok(spec_keys) => spec_keys,
        Err(invalid_spec) => {
            return Ok(session.invalid_call_with_message(
                "Counts",
                args,
                "invl",
                invalid_list_message(&invalid_spec),
            ));
        }
    };

    Ok(render_count_association_for_spec(items.to_vec(), spec_keys))
}

pub(crate) fn eval_position_index(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(session.invalid_call_with_message(
            "PositionIndex",
            args,
            "argx",
            exact_argument_count_message("PositionIndex", args.len(), 1),
        ));
    };

    if let Some(entries) = association_entries_from_expr(expr) {
        return Ok(render_position_index_entries(
            entries
                .into_iter()
                .map(|(key, value)| (value, key))
                .collect(),
        ));
    }

    let Some(items) = list_items(expr) else {
        return Ok(session.invalid_call_with_message(
            "PositionIndex",
            args,
            "invrp",
            invalid_association_or_list_message(expr),
        ));
    };

    Ok(render_position_index_entries(
        items
            .iter()
            .enumerate()
            .map(|(index, item)| (item.clone(), Expr::integer(BigInt::from(index + 1))))
            .collect(),
    ))
}

fn resume_counts_association_entries_from_index(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    values: Vec<Expr>,
    spec_keys: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let Some(entry) = entries.get(index).cloned() else {
        return Ok(render_count_association_for_spec(values, spec_keys));
    };

    if entry.delayed {
        let suspended_entries = entries.clone();
        let suspended_values = values.clone();
        let suspended_spec_keys = spec_keys.clone();
        let value_result = session.evaluate(entry.value);
        let value = session.suspend_with_frame(value_result, move |session, value| {
            resume_counts_after_association_value(
                session,
                suspended_entries,
                index,
                suspended_values,
                suspended_spec_keys,
                value,
            )
        })?;
        resume_counts_after_association_value(session, entries, index, values, spec_keys, value)
    } else {
        resume_counts_after_association_value(
            session,
            entries,
            index,
            values,
            spec_keys,
            entry.value,
        )
    }
}

fn resume_counts_after_association_value(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    mut values: Vec<Expr>,
    spec_keys: Option<Vec<Expr>>,
    value: Expr,
) -> Result<Expr, EvalError> {
    values.push(value);
    resume_counts_association_entries_from_index(session, entries, index + 1, values, spec_keys)
}

fn resume_catenate_direct_association_entries_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    values: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let Some(entry) = entries.get(index).cloned() else {
        return finish_catenate_direct_association_values(session, call_args, values);
    };

    if entry.delayed {
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_entries = entries.clone();
        let suspended_values = values.clone();
        let value_result = session.evaluate(entry.value);
        let value = session.suspend_with_frame(value_result, move |session, value| {
            resume_catenate_direct_association_value(
                session,
                suspended_head,
                suspended_call_args,
                suspended_entries,
                index,
                suspended_values,
                value,
            )
        })?;
        resume_catenate_direct_association_value(
            session, head, call_args, entries, index, values, value,
        )
    } else {
        resume_catenate_direct_association_value(
            session,
            head,
            call_args,
            entries,
            index,
            values,
            entry.value,
        )
    }
}

fn resume_catenate_direct_association_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    mut values: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    values.push(value);
    resume_catenate_direct_association_entries_from_index(
        session,
        head,
        call_args,
        entries,
        index + 1,
        values,
    )
}

fn finish_catenate_direct_association_values(
    session: &mut Session,
    call_args: Vec<Expr>,
    values: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let flatten_values = values
        .iter()
        .any(|value| is_missing_expr(value) || catenate_item_contribution(value).is_some());

    if !flatten_values {
        return Ok(Expr::call(system_expr("List"), values));
    }

    let mut flattened = Vec::new();
    for value in values {
        if is_missing_expr(&value) {
            continue;
        }

        let Some(items) = catenate_item_contribution(&value) else {
            return Ok(session.invalid_call_with_message(
                "Catenate",
                &call_args,
                "invrp",
                invalid_association_or_list_message(&value),
            ));
        };
        flattened.extend(items);
    }

    Ok(Expr::call(system_expr("List"), flattened))
}

fn resume_catenate_steps_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    steps: Vec<CatenateStep>,
    index: usize,
    mut flattened: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let Some(step) = steps.get(index).cloned() else {
        return Ok(Expr::call(system_expr("List"), flattened));
    };

    match step {
        CatenateStep::Append(items) => {
            flattened.extend(items);
            resume_catenate_steps_from_index(session, head, call_args, steps, index + 1, flattened)
        }
        CatenateStep::AssociationValue { value, delayed } => {
            if delayed {
                let suspended_head = head.clone();
                let suspended_call_args = call_args.clone();
                let suspended_steps = steps.clone();
                let suspended_flattened = flattened.clone();
                let value_result = session.evaluate(value);
                let value = session.suspend_with_frame(value_result, move |session, value| {
                    resume_catenate_after_association_value(
                        session,
                        suspended_head,
                        suspended_call_args,
                        suspended_steps,
                        index,
                        suspended_flattened,
                        value,
                    )
                })?;
                resume_catenate_after_association_value(
                    session, head, call_args, steps, index, flattened, value,
                )
            } else {
                resume_catenate_after_association_value(
                    session, head, call_args, steps, index, flattened, value,
                )
            }
        }
        CatenateStep::Invalid(expr) => Ok(session.invalid_call_with_message(
            "Catenate",
            &call_args,
            "invrp",
            invalid_association_or_list_message(&expr),
        )),
    }
}

fn resume_catenate_after_association_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    steps: Vec<CatenateStep>,
    index: usize,
    mut flattened: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    flattened.push(value);
    resume_catenate_steps_from_index(session, head, call_args, steps, index + 1, flattened)
}

fn catenate_item_contribution(expr: &Expr) -> Option<Vec<Expr>> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(entries.into_iter().map(|(_, value)| value).collect());
    }

    list_items(expr).map(|items| items.to_vec())
}

fn catenate_arg_count_message(count: usize) -> String {
    format!("Catenate called with {count} arguments; 1 argument is expected.")
}

fn invalid_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a list.")
}

fn counts_spec_keys(spec: Option<&Expr>) -> Result<Option<Vec<Expr>>, Expr> {
    let Some(spec) = spec else {
        return Ok(None);
    };

    list_items(spec)
        .map(|items| Some(items.to_vec()))
        .ok_or_else(|| spec.clone())
}

fn count_items(items: Vec<Expr>) -> Vec<(Expr, usize)> {
    let mut counts = Vec::new();
    for item in items {
        increment_count(&mut counts, item);
    }
    counts
}

fn render_count_association(counts: Vec<(Expr, usize)>) -> Expr {
    render_association(
        counts
            .into_iter()
            .map(|(key, count)| (key, Expr::integer(BigInt::from(count))))
            .collect(),
    )
}

fn render_count_association_for_spec(items: Vec<Expr>, spec_keys: Option<Vec<Expr>>) -> Expr {
    let counts = count_items(items);
    let Some(keys) = spec_keys else {
        return render_count_association(counts);
    };

    render_association(
        keys.into_iter()
            .map(|key| {
                let count = counts
                    .iter()
                    .find(|(existing, _)| existing == &key)
                    .map(|(_, count)| *count)
                    .unwrap_or(0);
                (key, Expr::integer(BigInt::from(count)))
            })
            .collect(),
    )
}

fn render_position_index_entries(items: Vec<(Expr, Expr)>) -> Expr {
    let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
    for (item, position) in items {
        if let Some((_, positions)) = grouped.iter_mut().find(|(key, _)| key == &item) {
            positions.push(position);
        } else {
            grouped.push((item, vec![position]));
        }
    }

    render_association(
        grouped
            .into_iter()
            .map(|(key, positions)| (key, Expr::call(system_expr("List"), positions)))
            .collect(),
    )
}

fn catenate_steps_for_association_entries(entries: Vec<AssociationEntryForm>) -> Vec<CatenateStep> {
    entries
        .into_iter()
        .map(|entry| CatenateStep::AssociationValue {
            value: entry.value,
            delayed: entry.delayed,
        })
        .collect()
}

fn catenate_steps_for_list_items(items: &[Expr]) -> Vec<CatenateStep> {
    let mut steps = Vec::new();
    for item in items {
        if let Some(entries) = association_entry_forms_from_expr(item) {
            steps.extend(catenate_steps_for_association_entries(entries));
        } else if let Some(items) = list_items(item) {
            steps.push(CatenateStep::Append(items.to_vec()));
        } else {
            steps.push(CatenateStep::Invalid(item.clone()));
            break;
        }
    }

    steps
}

fn exact_argument_count_message(symbol: &str, count: usize, expected: usize) -> String {
    let count_noun = if count == 1 { "argument" } else { "arguments" };
    let expected_noun = if expected == 1 {
        "argument is"
    } else {
        "arguments are"
    };
    format!("{symbol} called with {count} {count_noun}; {expected} {expected_noun} expected.")
}

fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}
