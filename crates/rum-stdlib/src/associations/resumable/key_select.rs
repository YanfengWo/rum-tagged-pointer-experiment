use super::*;

pub(crate) fn eval_key_select(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
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

            resume_key_select_from_index(
                session,
                head.clone(),
                args.to_vec(),
                entries,
                criterion.clone(),
                0,
                Vec::new(),
            )
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
    head: Expr,
    call_args: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
    criterion: Expr,
    index: usize,
    selected: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    let Some(entry) = entries.get(index).cloned() else {
        return Ok(render_association_entry_forms_preserving_heads(selected));
    };

    let call = Expr::call(criterion.clone(), vec![entry.key.clone()]);
    let suspended_head = head.clone();
    let suspended_call_args = call_args.clone();
    let suspended_entries = entries.clone();
    let suspended_criterion = criterion.clone();
    let suspended_selected = selected.clone();
    let suspended_entry = entry.clone();
    let value_result = session.evaluate(call);
    let criterion_result = session.suspend_with_frame(value_result, move |session, outcome| {
        resume_key_select_with_value(
            session,
            suspended_head,
            suspended_call_args,
            suspended_entries,
            suspended_criterion,
            index,
            suspended_selected,
            suspended_entry,
            outcome,
        )
    })?;

    resume_key_select_with_value(
        session,
        head,
        call_args,
        entries,
        criterion,
        index,
        selected,
        entry,
        criterion_result,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_key_select_with_value(
    session: &mut Session,
    head: Expr,
    call_args: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
    criterion: Expr,
    index: usize,
    mut selected: Vec<AssociationEntryForm>,
    entry: AssociationEntryForm,
    criterion_value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&criterion_value) == Some(true) {
        selected.push(entry);
    }

    resume_key_select_from_index(
        session,
        head,
        call_args,
        entries,
        criterion,
        index + 1,
        selected,
    )
}
