mod resume_frames;
use resume_frames::*;

use super::grouping::IndexedGroups;
use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum CatenateStep {
    Append(Vec<Expr>),
    AssociationValue { value: Expr, delayed: bool },
    Invalid(Expr),
}

pub(crate) fn eval_catenate(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    let [associations_expr] = args_slice else {
        return Ok(session.invalid_call_with_message(
            "Catenate",
            args_slice,
            "argx",
            catenate_arg_count_message(args_slice.len()),
        ));
    };

    if let Some(entries) = association_entry_forms_from_expr(associations_expr) {
        return resume_catenate_direct_association_entries_from_index(
            session,
            args.clone(),
            entries,
            0,
            Vec::new(),
        );
    }

    let Some(items) = list_items(associations_expr) else {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    };

    resume_catenate_steps_from_index(
        session,
        args.clone(),
        catenate_steps_for_list_items(items),
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_counts(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
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
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
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
    mut index: usize,
    mut values: Vec<Expr>,
    spec_keys: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return Ok(render_count_association_for_spec(values, spec_keys));
        };

        let value = if entry.delayed {
            match session.evaluate(entry.value) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let entries = FrozenAssociationSequence::freeze(entries);
                                                let values = PersistentAssociationSequence::freeze(values);
                                                let spec_keys = spec_keys.map(FrozenAssociationSequence::freeze);
                    ResumeCountsAssociationEntriesFromIndexSuspensionFrameL181C29 { entries, index, values, spec_keys }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            entry.value
        };

        values.push(value);
        index += 1;
    }
}

fn resume_suspended_counts_association_entries_from_index(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    mut index: usize,
    mut values: PersistentAssociationSequence<Expr>,
    spec_keys: Option<FrozenAssociationSequence<Expr>>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return Ok(render_count_association_for_spec(
                values.to_vec(),
                spec_keys.map(|keys| keys.as_slice().to_vec()),
            ));
        };

        let value = if entry.delayed {
            match session.evaluate(entry.value) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedCountsAssociationEntriesFromIndexSuspensionFrameL222C29 { entries, index, values, spec_keys },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            entry.value
        };

        values = values.appended(value);
        index += 1;
    }
}

fn resume_suspended_counts_after_association_value(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    values: PersistentAssociationSequence<Expr>,
    spec_keys: Option<FrozenAssociationSequence<Expr>>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let values = values.appended(value);
    resume_suspended_counts_association_entries_from_index(
        session,
        entries,
        index + 1,
        values,
        spec_keys,
    )
}

fn resume_catenate_direct_association_entries_from_index(
    session: &mut Session,
    call_args: SharedArgs,
    entries: Vec<AssociationEntryForm>,
    mut index: usize,
    mut values: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_catenate_direct_association_values(session, call_args, values);
        };

        let value = if entry.delayed {
            match session.evaluate(entry.value) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let entries = FrozenAssociationSequence::freeze(entries);
                                                let values = PersistentAssociationSequence::freeze(values);
                    ResumeCatenateDirectAssociationEntriesFromIndexSuspensionFrameL280C29 { call_args, entries, index, values }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            entry.value
        };

        values.push(value);
        index += 1;
    }
}

fn resume_suspended_catenate_direct_association_entries_from_index(
    session: &mut Session,
    call_args: SharedArgs,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    mut index: usize,
    mut values: PersistentAssociationSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_catenate_direct_association_values(session, call_args, values.to_vec());
        };

        let value = if entry.delayed {
            match session.evaluate(entry.value) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedCatenateDirectAssociationEntriesFromIndexSuspensionFrameL318C29 { call_args, entries, index, values },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            entry.value
        };

        values = values.appended(value);
        index += 1;
    }
}

fn resume_suspended_catenate_direct_association_value(
    session: &mut Session,
    call_args: SharedArgs,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    values: PersistentAssociationSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let values = values.appended(value);
    resume_suspended_catenate_direct_association_entries_from_index(
        session,
        call_args,
        entries,
        index + 1,
        values,
    )
}

fn finish_catenate_direct_association_values(
    session: &mut Session,
    call_args: SharedArgs,
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
                call_args.as_ref(),
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
    call_args: SharedArgs,
    steps: Vec<CatenateStep>,
    mut index: usize,
    mut flattened: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(step) = steps.get(index).cloned() else {
            return Ok(Expr::call(system_expr("List"), flattened));
        };

        match step {
            CatenateStep::Append(items) => {
                flattened.extend(items);
                index += 1;
            }
            CatenateStep::AssociationValue { value, delayed } => {
                let value = if delayed {
                    match session.evaluate(value) {
                        Ok(value) => value,
                        Err(EvalError::InputRequired { request }) => {
                            return rum_runtime::resume_or_suspend!(
                                session,
                                Err(EvalError::InputRequired { request }),
                                ok(value) => Ok(value),
                                suspend => {
                            let steps = FrozenAssociationSequence::freeze(steps);
                                                                let flattened =
                                                                    PersistentAssociationSequence::freeze(flattened);
                            ResumeCatenateStepsFromIndexSuspensionFrameL416C37 { call_args, steps, index, flattened }
                            },
                            );
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    value
                };
                flattened.push(value);
                index += 1;
            }
            CatenateStep::Invalid(expr) => {
                return Ok(session.invalid_call_with_message(
                    "Catenate",
                    &call_args,
                    "invrp",
                    invalid_association_or_list_message(&expr),
                ));
            }
        }
    }
}

fn resume_suspended_catenate_steps_from_index(
    session: &mut Session,
    call_args: SharedArgs,
    steps: FrozenAssociationSequence<CatenateStep>,
    mut index: usize,
    mut flattened: PersistentAssociationSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(step) = steps.get(index) else {
            return Ok(Expr::call(system_expr("List"), flattened.to_vec()));
        };

        match step {
            CatenateStep::Append(items) => {
                for item in items {
                    flattened = flattened.appended(item.clone());
                }
                index += 1;
            }
            CatenateStep::AssociationValue { value, delayed } => {
                let value = if *delayed {
                    match session.evaluate(value.clone()) {
                        Ok(value) => value,
                        Err(EvalError::InputRequired { request }) => {
                            return rum_runtime::resume_or_suspend!(
                                session,
                                Err(EvalError::InputRequired { request }),
                                ok(value) => Ok(value),
                                suspend => ResumeSuspendedCatenateStepsFromIndexSuspensionFrameL471C37 { call_args, steps, index, flattened },
                            );
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    value.clone()
                };
                flattened = flattened.appended(value);
                index += 1;
            }
            CatenateStep::Invalid(expr) => {
                return Ok(session.invalid_call_with_message(
                    "Catenate",
                    call_args.as_ref(),
                    "invrp",
                    invalid_association_or_list_message(expr),
                ));
            }
        }
    }
}

fn resume_suspended_catenate_after_association_value(
    session: &mut Session,
    call_args: SharedArgs,
    steps: FrozenAssociationSequence<CatenateStep>,
    index: usize,
    flattened: PersistentAssociationSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let flattened = flattened.appended(value);
    resume_suspended_catenate_steps_from_index(session, call_args, steps, index + 1, flattened)
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

const COUNT_INDEX_THRESHOLD: usize = 16;

fn count_items(items: Vec<Expr>) -> Vec<(Expr, usize)> {
    let mut remaining = items.into_iter();
    let mut counts = Vec::new();
    while let Some(item) = remaining.next() {
        if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| existing == &item) {
            *count += 1;
        } else {
            counts.push((item, 1));
            if counts.len() > COUNT_INDEX_THRESHOLD {
                return count_items_indexed(counts, remaining);
            }
        }
    }
    counts
}

#[inline(never)]
fn count_items_indexed(
    mut counts: Vec<(Expr, usize)>,
    remaining: impl Iterator<Item = Expr>,
) -> Vec<(Expr, usize)> {
    let mut positions: HashMap<Expr, usize> = counts
        .iter()
        .enumerate()
        .map(|(index, (key, _))| (key.clone(), index))
        .collect();
    for item in remaining {
        if let Some(index) = positions.get(&item).copied() {
            counts[index].1 += 1;
        } else {
            let index = counts.len();
            positions.insert(item.clone(), index);
            counts.push((item, 1));
        }
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

    let positions = (counts.len() > COUNT_INDEX_THRESHOLD).then(|| {
        counts
            .iter()
            .enumerate()
            .map(|(index, (key, _))| (key.clone(), index))
            .collect::<HashMap<_, _>>()
    });

    render_association(
        keys.into_iter()
            .map(|key| {
                let count = if let Some(positions) = &positions {
                    positions
                        .get(&key)
                        .map(|index| counts[*index].1)
                        .unwrap_or(0)
                } else {
                    counts
                        .iter()
                        .find(|(existing, _)| existing == &key)
                        .map(|(_, count)| *count)
                        .unwrap_or(0)
                };
                (key, Expr::integer(BigInt::from(count)))
            })
            .collect(),
    )
}

fn render_position_index_entries(items: Vec<(Expr, Expr)>) -> Expr {
    let grouped = IndexedGroups::collect(items);

    render_association(
        grouped
            .into_entries()
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<CatenateStep>();
};
