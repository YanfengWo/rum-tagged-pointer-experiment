use super::*;

mod key_collections;

impl Session {
    pub(in crate::runtime) fn eval_key_map(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_second("KeyMap", function.clone())),
            [function, expr] => {
                let Some(entries) = association_entry_forms_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeyMap",
                        args,
                        "invak",
                        invalid_association_message(expr),
                    ));
                };

                Ok(render_association_entry_forms_preserving_heads(
                    entries
                        .into_iter()
                        .map(|entry| AssociationEntryForm {
                            key: Expr::call(function.clone(), vec![entry.key]),
                            value: entry.value,
                            delayed: entry.delayed,
                        })
                        .collect(),
                ))
            }
            _ => Ok(self.invalid_call_with_message(
                "KeyMap",
                args,
                "argt",
                one_or_two_argument_count_message("KeyMap", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_key_value_map(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_second(
                "KeyValueMap",
                function.clone(),
            )),
            [function, expr] => {
                let Some(entries) = association_entries_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeyValueMap",
                        args,
                        "invak",
                        invalid_association_message(expr),
                    ));
                };

                Ok(Expr::call(
                    system_expr("List"),
                    entries
                        .into_iter()
                        .map(|(key, value)| Expr::call(function.clone(), vec![key, value]))
                        .collect(),
                ))
            }
            _ => Ok(self.invalid_call_with_message(
                "KeyValueMap",
                args,
                "argt",
                one_or_two_argument_count_message("KeyValueMap", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_key_sort(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                let Some(mut entries) = keyed_entry_forms_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeySort",
                        args,
                        "invrl",
                        invalid_association_or_rule_list_message(expr),
                    ));
                };

                entries.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.key, &rhs.key));
                Ok(render_association_entry_forms_preserving_heads(entries))
            }
            [expr, ordering] => {
                let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeySort",
                        args,
                        "invrl",
                        invalid_association_or_rule_list_message(expr),
                    ));
                };

                self.resume_key_sort_with_ordering_from_index(
                    head.clone(),
                    args.to_vec(),
                    ordering.clone(),
                    entries,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "KeySort",
                args,
                "argt",
                one_or_two_argument_count_message("KeySort", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_association_thread(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (keys_expr, values_expr, delayed) = match args {
            [spec] => {
                let Some(entry) = parse_association_entry_form(spec) else {
                    return Ok(self.invalid_call_with_message(
                        "AssociationThread",
                        args,
                        "invrk",
                        invalid_rule_message(spec),
                    ));
                };
                (entry.key, entry.value, entry.delayed)
            }
            [keys_expr, values_expr] => (keys_expr.clone(), values_expr.clone(), false),
            _ => {
                return Ok(self.invalid_call_with_message(
                    "AssociationThread",
                    args,
                    "argt",
                    format!(
                        "AssociationThread called with {} arguments; 1 or 2 arguments are expected.",
                        args.len()
                    ),
                ));
            }
        };

        let entries =
            match association_thread_entry_forms(keys_expr.clone(), values_expr.clone(), delayed) {
                Some(entries) => entries,
                None => {
                    return Ok(self.invalid_call_with_message(
                        "AssociationThread",
                        args,
                        "idim",
                        same_length_message(&keys_expr, &values_expr),
                    ));
                }
            };

        Ok(render_association_entry_forms_preserving_heads(entries))
    }
}

fn association_thread_entry_forms(
    keys_expr: Expr,
    values_expr: Expr,
    delayed: bool,
) -> Option<Vec<AssociationEntryForm>> {
    let key_items = list_items(&keys_expr).map(|items| items.to_vec());
    let value_items = list_items(&values_expr).map(|items| items.to_vec());

    match (key_items, value_items) {
        (Some(keys), Some(values)) => {
            if keys.len() != values.len() {
                return None;
            }
            Some(
                keys.into_iter()
                    .zip(values)
                    .map(|(key, value)| association_thread_entry(key, value, delayed))
                    .collect(),
            )
        }
        (Some(keys), None) => Some(
            keys.into_iter()
                .map(|key| association_thread_entry(key, values_expr.clone(), delayed))
                .collect(),
        ),
        (None, Some(values)) if values.is_empty() => Some(vec![association_thread_entry(
            keys_expr,
            system_expr(heads::structure::LIST),
            delayed,
        )]),
        (None, Some(values)) => Some(
            values
                .into_iter()
                .map(|value| association_thread_entry(keys_expr.clone(), value, delayed))
                .collect(),
        ),
        (None, None) => Some(vec![association_thread_entry(
            keys_expr,
            values_expr,
            delayed,
        )]),
    }
}

fn association_thread_entry(key: Expr, value: Expr, delayed: bool) -> AssociationEntryForm {
    AssociationEntryForm {
        key,
        value,
        delayed,
    }
}

fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}
