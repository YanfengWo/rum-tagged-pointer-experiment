use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_key_select(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [criterion] => Ok(keyed_operator_form_slot_first(
            "KeySelect",
            criterion.clone(),
        )),
        [expr, criterion] => {
            let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                return Ok(session.invalid_call_with_message(
                    "KeySelect",
                    args,
                    "invru",
                    invalid_association_or_rule_list_message(expr),
                ));
            };

            resume_key_select_from_index(session, entries, criterion.clone(), 0, Vec::new())
        }
        _ => Ok(session.invalid_call_with_message(
            "KeySelect",
            args,
            "argt",
            one_or_two_argument_count_message("KeySelect", args.len()),
        )),
    }
}

fn resume_key_select_from_index(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    criterion: Expr,
    mut index: usize,
    mut selected: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(selected));
        };

        let call = Expr::call(criterion.clone(), vec![entry.key.clone()]);
        let criterion_result = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let entries = FrozenAssociationSequence::freeze(entries);
                        let selected = PersistentAssociationSequence::freeze(selected);
                        move |session, outcome| {
                            resume_suspended_key_select_with_value(
                                session, entries, criterion, index, selected, entry, outcome,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&criterion_result) == Some(true) {
            selected.push(entry);
        }
        index += 1;
    }
}

fn resume_suspended_key_select_from_index(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    criterion: Expr,
    mut index: usize,
    mut selected: PersistentAssociationSequence<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(
                selected.to_vec(),
            ));
        };

        let call = Expr::call(criterion.clone(), vec![entry.key.clone()]);
        let criterion_result = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, outcome| {
                            resume_suspended_key_select_with_value(
                                session, entries, criterion, index, selected, entry, outcome,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&criterion_result) == Some(true) {
            selected = selected.appended(entry);
        }
        index += 1;
    }
}

fn resume_suspended_key_select_with_value(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    criterion: Expr,
    index: usize,
    selected: PersistentAssociationSequence<AssociationEntryForm>,
    entry: AssociationEntryForm,
    criterion_value: Expr,
) -> Result<Expr, EvalError> {
    let selected = if truthiness_symbol(&criterion_value) == Some(true) {
        selected.appended(entry)
    } else {
        selected
    };

    resume_suspended_key_select_from_index(session, entries, criterion, index + 1, selected)
}
