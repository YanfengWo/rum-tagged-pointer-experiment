use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeySortOrderingDecision {
    InsertBefore,
    KeepScanning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeySortByEntry {
    entry: AssociationEntryForm,
    sort_key: Expr,
}

pub(crate) fn eval_key_sort_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [function] => Ok(keyed_operator_form_slot_first(
            "KeySortBy",
            function.clone(),
        )),
        [expr, function] => {
            let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                return Ok(session.invalid_call_with_message(
                    "KeySortBy",
                    args,
                    "invrl",
                    invalid_association_or_rule_list_message(expr),
                ));
            };

            let decorated = Vec::with_capacity(entries.len());
            resume_key_sort_by_from_index(session, function.clone(), entries, 0, decorated)
        }
        [expr, function, ordering] => {
            let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                return Ok(session.invalid_call_with_message(
                    "KeySortBy",
                    args,
                    "invrl",
                    invalid_association_or_rule_list_message(expr),
                ));
            };

            resume_key_sort_by_ordering_from_index(
                session,
                head.clone(),
                args.to_vec(),
                function.clone(),
                ordering.clone(),
                entries,
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_message(
            "KeySortBy",
            args,
            "argt",
            one_to_three_argument_count_message("KeySortBy", args.len()),
        )),
    }
}

pub(crate) fn resume_key_sort_with_ordering_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    let Some(candidate) = entries.get(index).cloned() else {
        return Ok(render_association_entry_forms_preserving_heads(sorted));
    };

    resume_key_sort_with_ordering_insert(
        session, head, call_args, ordering, entries, index, sorted, candidate, 0,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_sort_with_ordering_insert(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<AssociationEntryForm>,
    candidate: AssociationEntryForm,
    insert_index: usize,
) -> Result<Expr, EvalError> {
    let Some(existing) = sorted.get(insert_index).cloned() else {
        let mut next_sorted = sorted;
        next_sorted.push(candidate);
        return resume_key_sort_with_ordering_from_index(
            session,
            head,
            call_args,
            ordering,
            entries,
            index + 1,
            next_sorted,
        );
    };

    let compare_result = session.evaluate(Expr::call(
        ordering.clone(),
        vec![candidate.key.clone(), existing.key],
    ));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_ordering = ordering.clone();
    let suspended_entries = entries.clone();
    let suspended_sorted = sorted.clone();
    let suspended_candidate = candidate.clone();
    let compare_value =
        session.suspend_with_frame(compare_result, move |session, compare_value| {
            resume_key_sort_with_ordering_after_compare(
                session,
                suspended_head,
                suspended_call_args,
                suspended_ordering,
                suspended_entries,
                index,
                suspended_sorted,
                suspended_candidate,
                insert_index,
                compare_value,
            )
        })?;

    resume_key_sort_with_ordering_after_compare(
        session,
        head,
        call_args,
        ordering,
        entries,
        index,
        sorted,
        candidate,
        insert_index,
        compare_value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_sort_with_ordering_after_compare(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<AssociationEntryForm>,
    candidate: AssociationEntryForm,
    insert_index: usize,
    compare_value: Expr,
) -> Result<Expr, EvalError> {
    match key_sort_ordering_decision(&compare_value) {
        KeySortOrderingDecision::InsertBefore => {
            let mut next_sorted = sorted;
            next_sorted.insert(insert_index, candidate);
            resume_key_sort_with_ordering_from_index(
                session,
                head,
                call_args,
                ordering,
                entries,
                index + 1,
                next_sorted,
            )
        }
        KeySortOrderingDecision::KeepScanning => resume_key_sort_with_ordering_insert(
            session,
            head,
            call_args,
            ordering,
            entries,
            index,
            sorted,
            candidate,
            insert_index + 1,
        ),
    }
}

fn resume_key_sort_by_from_index(
    session: &mut Session,
    function: Expr,
    entries: Vec<AssociationEntryForm>,
    mut index: usize,
    mut decorated: Vec<(AssociationEntryForm, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            decorated.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.1, &rhs.1));
            return Ok(render_association_entry_forms_preserving_heads(
                decorated.into_iter().map(|(entry, _)| entry).collect(),
            ));
        };

        let sort_key = match session.evaluate(Expr::call(function.clone(), vec![entry.key.clone()]))
        {
            Ok(sort_key) => sort_key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, sort_key| {
                        resume_key_sort_by_with_value(
                            session, function, entries, index, decorated, entry, sort_key,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated.push((entry, sort_key));
        index += 1;
    }
}

fn resume_key_sort_by_with_value(
    session: &mut Session,
    function: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    mut decorated: Vec<(AssociationEntryForm, Expr)>,
    entry: AssociationEntryForm,
    sort_key: Expr,
) -> Result<Expr, EvalError> {
    decorated.push((entry, sort_key));
    resume_key_sort_by_from_index(session, function, entries, index + 1, decorated)
}

#[allow(clippy::too_many_arguments)]
fn resume_key_sort_by_ordering_from_index(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<KeySortByEntry>,
) -> Result<Expr, EvalError> {
    let Some(entry) = entries.get(index).cloned() else {
        return Ok(render_association_entry_forms_preserving_heads(
            sorted.into_iter().map(|entry| entry.entry).collect(),
        ));
    };

    let sort_key_result = session.evaluate(Expr::call(function.clone(), vec![entry.key.clone()]));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_function = function.clone();
    let suspended_ordering = ordering.clone();
    let suspended_entries = entries.clone();
    let suspended_sorted = sorted.clone();
    let suspended_entry = entry.clone();
    let sort_key = session.suspend_with_frame(sort_key_result, move |session, sort_key| {
        resume_key_sort_by_ordering_with_key(
            session,
            suspended_head,
            suspended_call_args,
            suspended_function,
            suspended_ordering,
            suspended_entries,
            index,
            suspended_sorted,
            suspended_entry,
            sort_key,
        )
    })?;

    resume_key_sort_by_ordering_with_key(
        session, head, call_args, function, ordering, entries, index, sorted, entry, sort_key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_sort_by_ordering_with_key(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<KeySortByEntry>,
    entry: AssociationEntryForm,
    sort_key: Expr,
) -> Result<Expr, EvalError> {
    resume_key_sort_by_ordering_insert(
        session,
        head,
        call_args,
        function,
        ordering,
        entries,
        index,
        sorted,
        KeySortByEntry { entry, sort_key },
        0,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_sort_by_ordering_insert(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<KeySortByEntry>,
    candidate: KeySortByEntry,
    insert_index: usize,
) -> Result<Expr, EvalError> {
    let Some(existing) = sorted.get(insert_index).cloned() else {
        let mut next_sorted = sorted;
        next_sorted.push(candidate);
        return resume_key_sort_by_ordering_from_index(
            session,
            head,
            call_args,
            function,
            ordering,
            entries,
            index + 1,
            next_sorted,
        );
    };

    let compare_result = session.evaluate(Expr::call(
        ordering.clone(),
        vec![candidate.sort_key.clone(), existing.sort_key],
    ));
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_function = function.clone();
    let suspended_ordering = ordering.clone();
    let suspended_entries = entries.clone();
    let suspended_sorted = sorted.clone();
    let suspended_candidate = candidate.clone();
    let compare_value =
        session.suspend_with_frame(compare_result, move |session, compare_value| {
            resume_key_sort_by_ordering_after_compare(
                session,
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_ordering,
                suspended_entries,
                index,
                suspended_sorted,
                suspended_candidate,
                insert_index,
                compare_value,
            )
        })?;

    resume_key_sort_by_ordering_after_compare(
        session,
        head,
        call_args,
        function,
        ordering,
        entries,
        index,
        sorted,
        candidate,
        insert_index,
        compare_value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_sort_by_ordering_after_compare(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    function: Expr,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
    index: usize,
    sorted: Vec<KeySortByEntry>,
    candidate: KeySortByEntry,
    insert_index: usize,
    compare_value: Expr,
) -> Result<Expr, EvalError> {
    match key_sort_ordering_decision(&compare_value) {
        KeySortOrderingDecision::InsertBefore => {
            let mut next_sorted = sorted;
            next_sorted.insert(insert_index, candidate);
            resume_key_sort_by_ordering_from_index(
                session,
                head,
                call_args,
                function,
                ordering,
                entries,
                index + 1,
                next_sorted,
            )
        }
        KeySortOrderingDecision::KeepScanning => resume_key_sort_by_ordering_insert(
            session,
            head,
            call_args,
            function,
            ordering,
            entries,
            index,
            sorted,
            candidate,
            insert_index + 1,
        ),
    }
}

fn key_sort_ordering_decision(value: &Expr) -> KeySortOrderingDecision {
    match value {
        Expr::Atom(Atom::Integer(integer)) if integer.as_i64() == Some(1) => {
            KeySortOrderingDecision::InsertBefore
        }
        Expr::Atom(Atom::Integer(integer))
            if integer.as_i64() == Some(0) || integer.as_i64() == Some(-1) =>
        {
            KeySortOrderingDecision::KeepScanning
        }
        _ => match truthiness_symbol(value) {
            Some(true) => KeySortOrderingDecision::InsertBefore,
            Some(false) => KeySortOrderingDecision::KeepScanning,
            None => KeySortOrderingDecision::InsertBefore,
        },
    }
}
