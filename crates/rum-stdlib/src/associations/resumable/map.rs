use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_association_map(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [function] => Ok(keyed_operator_form_slot_second(
            "AssociationMap",
            function.clone(),
        )),
        [function, source] => {
            let keys = if let Some(items) = list_items(source) {
                items.to_vec()
            } else if let Some(entries) = association_entry_forms_from_expr(source) {
                let items = entries
                    .into_iter()
                    .map(association_entry_form_expr)
                    .collect();
                return resume_association_map_association_from_index(
                    session,
                    function.clone(),
                    items,
                    0,
                    Vec::new(),
                );
            } else {
                return Ok(session.invalid_call_with_message(
                    "AssociationMap",
                    args,
                    "invrp",
                    invalid_association_or_list_message(source),
                ));
            };

            resume_association_map_from_index(session, function.clone(), keys, 0, Vec::new())
        }
        _ => Ok(session.invalid_call_with_message(
            "AssociationMap",
            args,
            "argt",
            one_or_two_argument_count_message("AssociationMap", args.len()),
        )),
    }
}

fn resume_association_map_from_index(
    session: &mut Session,
    function: Expr,
    keys: Vec<Expr>,
    mut index: usize,
    mut entries: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(key) = keys.get(index).cloned() else {
            return Ok(render_association(entries));
        };

        let value = match session.evaluate(Expr::call(function.clone(), vec![key.clone()])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let keys = FrozenAssociationSequence::freeze(keys);
                        let entries = PersistentAssociationSequence::freeze(entries);
                        move |session, value| {
                            resume_suspended_association_map_with_value(
                                session, function, keys, index, entries, key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        entries.push((key, value));
        index += 1;
    }
}

fn resume_suspended_association_map_from_index(
    session: &mut Session,
    function: Expr,
    keys: FrozenAssociationSequence<Expr>,
    mut index: usize,
    mut entries: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(key) = keys.get(index).cloned() else {
            return Ok(render_association(entries.to_vec()));
        };

        let value = match session.evaluate(Expr::call(function.clone(), vec![key.clone()])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_association_map_with_value(
                                session, function, keys, index, entries, key, value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        entries = entries.appended((key, value));
        index += 1;
    }
}

fn resume_suspended_association_map_with_value(
    session: &mut Session,
    function: Expr,
    keys: FrozenAssociationSequence<Expr>,
    index: usize,
    entries: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    resume_suspended_association_map_from_index(
        session,
        function,
        keys,
        index + 1,
        entries.appended((key, value)),
    )
}

fn resume_association_map_association_from_index(
    session: &mut Session,
    function: Expr,
    items: Vec<Expr>,
    mut index: usize,
    mut mapped_items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association_preserving_rule_heads(&mapped_items)
                .unwrap_or_else(|| Expr::call(system_expr("Association"), mapped_items)));
        };

        let value = match session.evaluate(Expr::call(function.clone(), vec![item.clone()])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        let items = FrozenAssociationSequence::freeze(items);
                        let mapped_items = PersistentAssociationSequence::freeze(mapped_items);
                        move |session, value| {
                            resume_suspended_association_map_association_with_value(
                                session,
                                function,
                                items,
                                index,
                                mapped_items,
                                item,
                                value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        push_association_map_association_value(session, &function, &item, &mut mapped_items, value);
        index += 1;
    }
}

fn resume_suspended_association_map_association_from_index(
    session: &mut Session,
    function: Expr,
    items: FrozenAssociationSequence<Expr>,
    mut index: usize,
    mut mapped_items: PersistentAssociationSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            let mapped_items = mapped_items.to_vec();
            return Ok(render_association_preserving_rule_heads(&mapped_items)
                .unwrap_or_else(|| Expr::call(system_expr("Association"), mapped_items)));
        };

        let value = match session.evaluate(Expr::call(function.clone(), vec![item.clone()])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_suspended_association_map_association_with_value(
                                session,
                                function,
                                items,
                                index,
                                mapped_items,
                                item,
                                value,
                            )
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        report_invalid_association_map_value(session, &function, &item, &value);
        mapped_items = mapped_items.appended(value);
        index += 1;
    }
}

fn resume_suspended_association_map_association_with_value(
    session: &mut Session,
    function: Expr,
    items: FrozenAssociationSequence<Expr>,
    index: usize,
    mapped_items: PersistentAssociationSequence<Expr>,
    item: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    report_invalid_association_map_value(session, &function, &item, &value);

    resume_suspended_association_map_association_from_index(
        session,
        function,
        items,
        index + 1,
        mapped_items.appended(value),
    )
}

fn push_association_map_association_value(
    session: &mut Session,
    function: &Expr,
    item: &Expr,
    mapped_items: &mut Vec<Expr>,
    value: Expr,
) {
    report_invalid_association_map_value(session, function, item, &value);
    mapped_items.push(value);
}

fn report_invalid_association_map_value(
    session: &mut Session,
    function: &Expr,
    item: &Expr,
    value: &Expr,
) {
    if render_association_preserving_rule_heads(std::slice::from_ref(value)).is_none() {
        session.push_message(
                "AssociationMap",
                "invrlf",
                format!(
                    "Applying {function} to {item} yields {value}, which is not a valid rule, list of rules or association."
                ),
            );
    }
}
