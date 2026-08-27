use super::*;
use crate::structure_ops::{ComparatorResultPolicy, MergeSortItem, sort_with_comparator};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeySortOrderingDecision {
    InsertBefore,
    KeepScanning,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct KeySortByEntry {
    entry: AssociationEntryForm,
    sort_key: Expr,
}

impl MergeSortItem for AssociationEntryForm {
    fn ordering_value(&self) -> &Expr {
        &self.key
    }
}

impl MergeSortItem for KeySortByEntry {
    fn ordering_value(&self) -> &Expr {
        &self.sort_key
    }
}

#[derive(FrameRootTrace)]
struct KeySortByOrderingState {
    function: Expr,
    ordering: Expr,
    remaining: std::vec::IntoIter<AssociationEntryForm>,
    decorated: Vec<KeySortByEntry>,
}

pub(crate) fn eval_key_sort_by(
    session: &mut Session,
    _head: &Expr,
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

            let decorated = Vec::with_capacity(entries.len());
            resume_key_sort_by_ordering(
                session,
                KeySortByOrderingState {
                    function: function.clone(),
                    ordering: ordering.clone(),
                    remaining: entries.into_iter(),
                    decorated,
                },
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

pub(crate) fn sort_key_entries_with_ordering(
    session: &mut Session,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    sort_with_comparator(
        session,
        entries,
        ordering,
        ComparatorResultPolicy::Custom(key_sort_ordering_precedes),
        render_association_entry_forms_preserving_heads,
    )
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

fn resume_key_sort_by_ordering(
    session: &mut Session,
    mut state: KeySortByOrderingState,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = state.remaining.next() else {
            return sort_with_comparator(
                session,
                state.decorated,
                state.ordering,
                ComparatorResultPolicy::Custom(key_sort_ordering_precedes),
                |sorted| {
                    render_association_entry_forms_preserving_heads(
                        sorted.into_iter().map(|entry| entry.entry).collect(),
                    )
                },
            );
        };

        let sort_key =
            match session.evaluate(Expr::call(state.function.clone(), vec![entry.key.clone()])) {
                Ok(sort_key) => sort_key,
                Err(EvalError::InputRequired { request }) => {
                    return session.suspend_with_frame(
                        Err(EvalError::InputRequired { request }),
                        move |session, sort_key| {
                            state.decorated.push(KeySortByEntry { entry, sort_key });
                            resume_key_sort_by_ordering(session, state)
                        },
                    );
                }
                Err(error) => return Err(error),
            };

        state.decorated.push(KeySortByEntry { entry, sort_key });
    }
}

fn key_sort_ordering_precedes(value: &Expr) -> bool {
    key_sort_ordering_decision(value) == KeySortOrderingDecision::InsertBefore
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<KeySortByEntry>();
    assert_traces::<KeySortByOrderingState>();
};
