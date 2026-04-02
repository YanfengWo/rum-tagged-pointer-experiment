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
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Lookup", args, 2, 3));
        }

        if let Some(value) = apply_keyed_part_spec(&args[0], &args[1]) {
            return Ok(value);
        }

        let Some(entries) = association_entries_from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let key = &args[1];
        if let Some((_, value)) = entries.into_iter().find(|(entry_key, _)| entry_key == key) {
            return Ok(value);
        }

        Ok(args.get(2).cloned().unwrap_or_else(|| {
            Expr::call(
                system_expr("Missing"),
                vec![Expr::string("KeyAbsent"), key.clone()],
            )
        }))
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
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Keys", args, 1));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("List"),
            entries.into_iter().map(|(key, _)| key).collect(),
        ))
    }

    pub(in crate::runtime) fn eval_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Values", args, 1));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("List"),
            entries.into_iter().map(|(_, value)| value).collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_exists_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyExistsQ", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
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
        let [expr, key_spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyTake", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(selected_keys) = parse_key_spec(key_spec) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            entries
                .into_iter()
                .filter(|(key, _)| selected_keys.iter().any(|candidate| candidate == key))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_drop(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key_spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyDrop", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(selected_keys) = parse_key_spec(key_spec) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            entries
                .into_iter()
                .filter(|(key, _)| !selected_keys.iter().any(|candidate| candidate == key))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_member_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyMemberQ", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
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

        let Some(entries) = association_entries_from_expr(expr) else {
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
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Missing",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}
