use super::*;

impl Session {
    pub(in crate::runtime) fn eval_association_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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
                    return self.resume_association_map_association_from_index(
                        head.clone(),
                        args.to_vec(),
                        function.clone(),
                        items,
                        0,
                        Vec::new(),
                    );
                } else {
                    return Ok(self.invalid_call_with_message(
                        "AssociationMap",
                        args,
                        "invrp",
                        invalid_association_or_list_message(source),
                    ));
                };

                self.resume_association_map_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
                    keys,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "AssociationMap",
                args,
                "argt",
                one_or_two_argument_count_message("AssociationMap", args.len()),
            )),
        }
    }

    fn resume_association_map_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        keys: Vec<Expr>,
        index: usize,
        entries: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some(key) = keys.get(index).cloned() else {
            return Ok(render_association(entries));
        };

        let value_result = self.evaluate(Expr::call(function.clone(), vec![key.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_keys = keys.clone();
        let suspended_entries = entries.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_association_map_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_keys,
                index,
                suspended_entries,
                suspended_key,
                value,
            )
        })?;

        self.resume_association_map_with_value(
            head, call_args, function, keys, index, entries, key, value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_map_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        keys: Vec<Expr>,
        index: usize,
        mut entries: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        entries.push((key, value));
        self.resume_association_map_from_index(head, call_args, function, keys, index + 1, entries)
    }

    fn resume_association_map_association_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        mapped_items: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association_preserving_rule_heads(&mapped_items)
                .unwrap_or_else(|| Expr::call(system_expr("Association"), mapped_items)));
        };

        let value_result = self.evaluate(Expr::call(function.clone(), vec![item.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_items = items.clone();
        let suspended_mapped_items = mapped_items.clone();
        let suspended_item = item.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_association_map_association_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_items,
                index,
                suspended_mapped_items,
                suspended_item,
                value,
            )
        })?;

        self.resume_association_map_association_with_value(
            head,
            call_args,
            function,
            items,
            index,
            mapped_items,
            item,
            value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_map_association_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        mut mapped_items: Vec<Expr>,
        item: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if render_association_preserving_rule_heads(std::slice::from_ref(&value)).is_none() {
            self.push_message(
                "AssociationMap",
                "invrlf",
                format!(
                    "Applying {function} to {item} yields {value}, which is not a valid rule, list of rules or association."
                ),
            );
        }

        mapped_items.push(value);
        self.resume_association_map_association_from_index(
            head,
            call_args,
            function,
            items,
            index + 1,
            mapped_items,
        )
    }
}
