use super::*;

impl Session {
    pub(in crate::runtime) fn eval_association(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let source_items = match args {
            [
                Expr::Call {
                    head: list_head,
                    args: items,
                },
            ] if is_system_symbol_name(list_head, heads::structure::LIST) => items.clone(),
            _ => args.to_vec(),
        };

        let Some(entries) = parse_association_entries(&source_items) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(entries))
    }

    pub(in crate::runtime) fn eval_association_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociationQ", args, 1));
        };

        Ok(predicate_bool_expr(
            association_entries_from_expr(expr).is_some(),
        ))
    }

    pub(in crate::runtime) fn eval_lookup(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Lookup", args, 2, 4));
        }

        let default = args.get(2).cloned();
        let wrapper = args.get(3);
        let Some(value) = lookup_value(&args[0], &args[1], default, wrapper) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(value)
    }

    pub(in crate::runtime) fn eval_key(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Key", args, 1));
        };

        Ok(Expr::call(head.clone(), vec![key.clone()]))
    }

    pub(in crate::runtime) fn eval_keys(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Keys", args, 1, 2));
        };

        let wrapper = args.get(1);
        let Some(value) = keys_result_for_expr(&args[0], wrapper) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(value)
    }

    pub(in crate::runtime) fn eval_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Values", args, 1, 2));
        };

        let wrapper = args.get(1);
        let Some(value) = values_result_for_expr(&args[0], wrapper) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(value)
    }

    pub(in crate::runtime) fn eval_key_exists_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyExistsQ", args, 2));
        };

        let Some(entries) = keyed_entries_from_expr(expr).map(|keyed| keyed.entries) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(
            entries.iter().any(|(entry_key, _)| entry_key == key),
        ))
    }

    pub(in crate::runtime) fn eval_key_take(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [key_spec] => Ok(keyed_operator_form_slot_first("KeyTake", key_spec.clone())),
            [expr, key_spec] => {
                let Some(value) = keyed_take_drop_result_for_expr("KeyTake", expr, key_spec, true)
                else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };

                Ok(value)
            }
            _ => Ok(self.invalid_call_with_arg_range("KeyTake", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_key_drop(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [key_spec] => Ok(keyed_operator_form_slot_first("KeyDrop", key_spec.clone())),
            [expr, key_spec] => {
                let Some(value) = keyed_take_drop_result_for_expr("KeyDrop", expr, key_spec, false)
                else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };

                Ok(value)
            }
            _ => Ok(self.invalid_call_with_arg_range("KeyDrop", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_key_member_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyMemberQ", args, 2));
        };

        let Some(entries) = keyed_entries_from_expr(expr).map(|keyed| keyed.entries) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(
            entries.iter().any(|(entry_key, _)| entry_key == key),
        ))
    }

    pub(in crate::runtime) fn eval_key_free_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyFreeQ", args, 2));
        };

        let Some(entries) = keyed_entries_from_expr(expr).map(|keyed| keyed.entries) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(
            !entries.iter().any(|(entry_key, _)| entry_key == key),
        ))
    }

    pub(in crate::runtime) fn eval_normal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Normal", args, 1));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("List"),
            entries
                .into_iter()
                .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_value_pattern(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyValuePattern", args, 1));
        };

        let Some(normalized_spec) = normalize_key_value_pattern_spec(spec) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(head.clone(), vec![normalized_spec]))
    }

    pub(in crate::runtime) fn eval_missing(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() > 2 {
            return Ok(self.invalid_call_with_arg_range("Missing", args, 0, 2));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}

fn lookup_value(
    data: &Expr,
    key_spec: &Expr,
    default: Option<Expr>,
    wrapper: Option<&Expr>,
) -> Option<Expr> {
    if let Some(value) = apply_keyed_part_spec(data, key_spec) {
        return Some(wrap_keyed_result(value, wrapper));
    }

    if let Some(items) = call_items_with_head(data, &system_expr("List")) {
        if items
            .iter()
            .all(|item| association_entries_from_expr(item).is_some())
        {
            return Some(Expr::call(
                system_expr("List"),
                items
                    .iter()
                    .map(|item| lookup_value(item, key_spec, default.clone(), wrapper))
                    .collect::<Option<Vec<_>>>()?,
            ));
        }
    }

    if let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) {
        return Some(Expr::call(
            system_expr("List"),
            keys.iter()
                .map(|key| lookup_value(data, key, default.clone(), wrapper))
                .collect::<Option<Vec<_>>>()?,
        ));
    }

    let entries = association_entries_from_expr(data)?;
    if let Some((_, value)) = entries
        .into_iter()
        .find(|(entry_key, _)| entry_key == key_spec)
    {
        return Some(wrap_keyed_result(value, wrapper));
    }

    Some(wrap_keyed_result(
        default.unwrap_or_else(|| {
            Expr::call(
                system_expr("Missing"),
                vec![Expr::string("KeyAbsent"), key_spec.clone()],
            )
        }),
        wrapper,
    ))
}

fn wrap_keyed_result(value: Expr, wrapper: Option<&Expr>) -> Expr {
    match wrapper.cloned() {
        Some(head) => Expr::call(head, vec![value]),
        None => value,
    }
}

fn keys_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Option<Expr> {
    if let Some((key, _)) = parse_association_entry(expr) {
        return Some(wrap_keyed_result(key, wrapper));
    }

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        return Some(Expr::call(
            system_expr("List"),
            keyed
                .entries
                .into_iter()
                .map(|(key, _)| wrap_keyed_result(key, wrapper))
                .collect(),
        ));
    }

    let items = list_items(expr)?;
    if parse_association_entries(items).is_some() {
        return None;
    }

    Some(Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|item| {
                keys_result_for_expr(item, wrapper).unwrap_or_else(|| {
                    build_keyed_fallback_call("Keys", item.clone(), wrapper.cloned())
                })
            })
            .collect(),
    ))
}

fn values_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Option<Expr> {
    if let Some((_, value)) = parse_association_entry(expr) {
        return Some(wrap_keyed_result(value, wrapper));
    }

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        return Some(Expr::call(
            system_expr("List"),
            keyed
                .entries
                .into_iter()
                .map(|(_, value)| wrap_keyed_result(value, wrapper))
                .collect(),
        ));
    }

    let items = list_items(expr)?;
    if parse_association_entries(items).is_some() {
        return None;
    }

    Some(Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|item| {
                values_result_for_expr(item, wrapper).unwrap_or_else(|| {
                    build_keyed_fallback_call("Values", item.clone(), wrapper.cloned())
                })
            })
            .collect(),
    ))
}

fn keyed_take_drop_result_for_expr(
    symbol: &'static str,
    expr: &Expr,
    key_spec: &Expr,
    keep_selected: bool,
) -> Option<Expr> {
    let selected_keys = parse_key_spec(key_spec)?;

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        let filtered = if keep_selected {
            let mut taken = Vec::new();
            for selected_key in &selected_keys {
                taken.extend(
                    keyed
                        .entries
                        .iter()
                        .filter(|(key, _)| key == selected_key)
                        .cloned(),
                );
            }
            taken
        } else {
            keyed
                .entries
                .into_iter()
                .filter(|(key, _)| !selected_keys.iter().any(|candidate| candidate == key))
                .collect()
        };

        return Some(render_association(filtered));
    }

    let items = list_items(expr)?;
    if parse_association_entries(items).is_some() {
        return None;
    }

    Some(Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|item| {
                keyed_take_drop_result_for_expr(symbol, item, key_spec, keep_selected)
                    .unwrap_or_else(|| {
                        Expr::call(system_expr(symbol), vec![item.clone(), key_spec.clone()])
                    })
            })
            .collect(),
    ))
}

fn build_keyed_fallback_call(symbol: &str, expr: Expr, wrapper: Option<Expr>) -> Expr {
    let mut args = vec![expr];
    if let Some(wrapper) = wrapper {
        args.push(wrapper);
    }
    Expr::call(system_expr(symbol), args)
}
