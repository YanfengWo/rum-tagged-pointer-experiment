use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct GroupBySpec {
    key_function: Expr,
    value_function: Option<Expr>,
}

fn group_by_spec(function: &Expr) -> GroupBySpec {
    match function {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            GroupBySpec {
                key_function: args[0].clone(),
                value_function: Some(args[1].clone()),
            }
        }
        _ => GroupBySpec {
            key_function: function.clone(),
            value_function: None,
        },
    }
}

fn group_by_specs(function: &Expr) -> Vec<GroupBySpec> {
    if let Some(items) = list_items(function) {
        return items.iter().map(group_by_spec).collect();
    }

    vec![group_by_spec(function)]
}

impl Session {
    pub(in crate::runtime) fn eval_group_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_first("GroupBy", function.clone())),
            [expr, function] => {
                let specs = group_by_specs(function);
                if let Some(entries) = association_entry_forms_from_expr(expr) {
                    if specs.is_empty() {
                        return self.finish_association_group_by_leaf(
                            head.clone(),
                            args.to_vec(),
                            None,
                            entries,
                        );
                    }

                    return self.resume_association_group_by_from_index(
                        head.clone(),
                        args.to_vec(),
                        specs,
                        0,
                        None,
                        entries,
                        0,
                        Vec::new(),
                    );
                }

                let Some(items) = list_items(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "GroupBy",
                        args,
                        "list1",
                        invalid_keyed_collection_list_message(expr),
                    ));
                };

                if specs.is_empty() {
                    return self.finish_group_by_leaf(
                        head.clone(),
                        args.to_vec(),
                        None,
                        items.to_vec(),
                    );
                }

                self.resume_group_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    specs,
                    0,
                    None,
                    items.to_vec(),
                    0,
                    Vec::new(),
                )
            }
            [expr, function, reducer] => {
                let specs = group_by_specs(function);
                if let Some(entries) = association_entry_forms_from_expr(expr) {
                    if specs.is_empty() {
                        return self.finish_association_group_by_leaf(
                            head.clone(),
                            args.to_vec(),
                            Some(reducer.clone()),
                            entries,
                        );
                    }

                    return self.resume_association_group_by_from_index(
                        head.clone(),
                        args.to_vec(),
                        specs,
                        0,
                        Some(reducer.clone()),
                        entries,
                        0,
                        Vec::new(),
                    );
                }

                let Some(items) = list_items(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "GroupBy",
                        args,
                        "list1",
                        invalid_keyed_collection_list_message(expr),
                    ));
                };

                if specs.is_empty() {
                    return self.finish_group_by_leaf(
                        head.clone(),
                        args.to_vec(),
                        Some(reducer.clone()),
                        items.to_vec(),
                    );
                }

                self.resume_group_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    specs,
                    0,
                    Some(reducer.clone()),
                    items.to_vec(),
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "GroupBy",
                args,
                "argb",
                between_one_and_three_argument_count_message("GroupBy", args.len()),
            )),
        }
    }

    fn resume_group_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        items: Vec<Expr>,
        index: usize,
        grouped: Vec<(Expr, Vec<Expr>)>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return self
                .finish_group_by_grouped(head, call_args, specs, spec_index, reducer, grouped);
        };

        let spec = specs[spec_index].clone();
        let key_result = self.evaluate(Expr::call(spec.key_function.clone(), vec![item.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_specs = specs.clone();
        let suspended_reducer = reducer.clone();
        let suspended_items = items.clone();
        let suspended_grouped = grouped.clone();
        let suspended_item = item.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_group_by_with_key(
                suspended_head,
                suspended_call_args,
                suspended_specs,
                spec_index,
                suspended_reducer,
                suspended_items,
                index,
                suspended_grouped,
                suspended_item,
                key,
            )
        })?;

        self.resume_group_by_with_key(
            head, call_args, specs, spec_index, reducer, items, index, grouped, item, key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_with_key(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        items: Vec<Expr>,
        index: usize,
        grouped: Vec<(Expr, Vec<Expr>)>,
        item: Expr,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        let spec = specs[spec_index].clone();
        let Some(value_function) = spec.value_function.clone() else {
            return self.resume_group_by_with_item_value(
                head, call_args, specs, spec_index, reducer, items, index, grouped, key, item,
            );
        };

        let value_result = self.evaluate(Expr::call(value_function, vec![item]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_specs = specs.clone();
        let suspended_reducer = reducer.clone();
        let suspended_items = items.clone();
        let suspended_grouped = grouped.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_group_by_with_item_value(
                suspended_head,
                suspended_call_args,
                suspended_specs,
                spec_index,
                suspended_reducer,
                suspended_items,
                index,
                suspended_grouped,
                suspended_key,
                value,
            )
        })?;

        self.resume_group_by_with_item_value(
            head, call_args, specs, spec_index, reducer, items, index, grouped, key, value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_with_item_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        items: Vec<Expr>,
        index: usize,
        mut grouped: Vec<(Expr, Vec<Expr>)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some((_, values)) = grouped.iter_mut().find(|(existing, _)| existing == &key) {
            values.push(value);
        } else {
            grouped.push((key, vec![value]));
        }

        self.resume_group_by_from_index(
            head,
            call_args,
            specs,
            spec_index,
            reducer,
            items,
            index + 1,
            grouped,
        )
    }

    fn finish_group_by_leaf(
        &mut self,
        _head: Expr,
        _call_args: Vec<Expr>,
        reducer: Option<Expr>,
        values: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if reducer.is_some() && values.is_empty() {
            return Ok(render_association(Vec::new()));
        }

        let group = Expr::call(system_expr("List"), values);
        match reducer {
            None => Ok(group),
            Some(reducer) => self.evaluate(Expr::call(reducer, vec![group])),
        }
    }

    fn finish_group_by_grouped(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        grouped: Vec<(Expr, Vec<Expr>)>,
    ) -> Result<Expr, EvalError> {
        if spec_index + 1 < specs.len() {
            return self.resume_group_by_nested_from_index(
                head,
                call_args,
                specs,
                spec_index + 1,
                reducer,
                grouped,
                0,
                Vec::new(),
            );
        }

        match reducer {
            None => Ok(render_association(
                grouped
                    .into_iter()
                    .map(|(key, values)| (key, Expr::call(system_expr("List"), values)))
                    .collect(),
            )),
            Some(reducer) => self.resume_group_by_reduce_from_index(
                head,
                call_args,
                reducer,
                grouped,
                0,
                Vec::new(),
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_nested_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        nested: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(nested));
        };

        let value_result = self.resume_group_by_from_index(
            head.clone(),
            call_args.clone(),
            specs.clone(),
            spec_index,
            reducer.clone(),
            values,
            0,
            Vec::new(),
        );
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_specs = specs.clone();
        let suspended_reducer = reducer.clone();
        let suspended_grouped = grouped.clone();
        let suspended_nested = nested.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_group_by_nested_with_value(
                suspended_head,
                suspended_call_args,
                suspended_specs,
                spec_index,
                suspended_reducer,
                suspended_grouped,
                index,
                suspended_nested,
                suspended_key,
                value,
            )
        })?;

        self.resume_group_by_nested_with_value(
            head, call_args, specs, spec_index, reducer, grouped, index, nested, key, value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_nested_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        mut nested: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        nested.push((key, value));
        self.resume_group_by_nested_from_index(
            head,
            call_args,
            specs,
            spec_index,
            reducer,
            grouped,
            index + 1,
            nested,
        )
    }

    fn resume_group_by_reduce_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        reducer: Expr,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        reduced: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced));
        };

        let value_result = self.evaluate(Expr::call(
            reducer.clone(),
            vec![Expr::call(system_expr("List"), values)],
        ));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_reducer = reducer.clone();
        let suspended_grouped = grouped.clone();
        let suspended_reduced = reduced.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_group_by_reduce_with_value(
                suspended_head,
                suspended_call_args,
                suspended_reducer,
                suspended_grouped,
                index,
                suspended_reduced,
                suspended_key,
                value,
            )
        })?;

        self.resume_group_by_reduce_with_value(
            head, call_args, reducer, grouped, index, reduced, key, value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_reduce_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        reducer: Expr,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        mut reduced: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        reduced.push((key, value));
        self.resume_group_by_reduce_from_index(
            head,
            call_args,
            reducer,
            grouped,
            index + 1,
            reduced,
        )
    }

    fn resume_association_group_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    ) -> Result<Expr, EvalError> {
        let Some(entry) = entries.get(index).cloned() else {
            return self.finish_association_group_by_grouped(
                head, call_args, specs, spec_index, reducer, grouped,
            );
        };

        let spec = specs[spec_index].clone();
        let key_result = self.evaluate(Expr::call(
            spec.key_function.clone(),
            vec![entry.value.clone()],
        ));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_specs = specs.clone();
        let suspended_reducer = reducer.clone();
        let suspended_entries = entries.clone();
        let suspended_grouped = grouped.clone();
        let suspended_entry = entry.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_association_group_by_with_key(
                suspended_head,
                suspended_call_args,
                suspended_specs,
                spec_index,
                suspended_reducer,
                suspended_entries,
                index,
                suspended_grouped,
                suspended_entry,
                key,
            )
        })?;

        self.resume_association_group_by_with_key(
            head, call_args, specs, spec_index, reducer, entries, index, grouped, entry, key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_group_by_with_key(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
        entry: AssociationEntryForm,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        let spec = specs[spec_index].clone();
        let Some(value_function) = spec.value_function.clone() else {
            return self.resume_association_group_by_with_entry(
                head, call_args, specs, spec_index, reducer, entries, index, grouped, entry, key,
            );
        };

        let value_result = self.evaluate(Expr::call(value_function, vec![entry.value.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_specs = specs.clone();
        let suspended_reducer = reducer.clone();
        let suspended_entries = entries.clone();
        let suspended_grouped = grouped.clone();
        let suspended_entry = entry.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            let transformed_entry = AssociationEntryForm {
                value,
                ..suspended_entry
            };
            session.resume_association_group_by_with_entry(
                suspended_head,
                suspended_call_args,
                suspended_specs,
                spec_index,
                suspended_reducer,
                suspended_entries,
                index,
                suspended_grouped,
                transformed_entry,
                suspended_key,
            )
        })?;

        let transformed_entry = AssociationEntryForm { value, ..entry };
        self.resume_association_group_by_with_entry(
            head,
            call_args,
            specs,
            spec_index,
            reducer,
            entries,
            index,
            grouped,
            transformed_entry,
            key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_group_by_with_entry(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        mut grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
        entry: AssociationEntryForm,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some((_, values)) = grouped.iter_mut().find(|(existing, _)| existing == &key) {
            values.push(entry);
        } else {
            grouped.push((key, vec![entry]));
        }

        self.resume_association_group_by_from_index(
            head,
            call_args,
            specs,
            spec_index,
            reducer,
            entries,
            index + 1,
            grouped,
        )
    }

    fn finish_association_group_by_leaf(
        &mut self,
        _head: Expr,
        _call_args: Vec<Expr>,
        reducer: Option<Expr>,
        entries: Vec<AssociationEntryForm>,
    ) -> Result<Expr, EvalError> {
        if reducer.is_some() && entries.is_empty() {
            return Ok(render_association(Vec::new()));
        }

        let group = render_association_entry_forms_preserving_heads(entries);
        match reducer {
            None => Ok(group),
            Some(reducer) => self.evaluate(Expr::call(reducer, vec![group])),
        }
    }

    fn finish_association_group_by_grouped(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    ) -> Result<Expr, EvalError> {
        if spec_index + 1 < specs.len() {
            return self.resume_association_group_by_nested_from_index(
                head,
                call_args,
                specs,
                spec_index + 1,
                reducer,
                grouped,
                0,
                Vec::new(),
            );
        }

        match reducer {
            None => Ok(render_association(association_group_by_groups(grouped))),
            Some(reducer) => self.resume_association_group_by_reduce_from_index(
                head,
                call_args,
                reducer,
                grouped,
                0,
                Vec::new(),
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_group_by_nested_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
        index: usize,
        nested: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(nested));
        };

        let value_result = self.resume_association_group_by_from_index(
            head.clone(),
            call_args.clone(),
            specs.clone(),
            spec_index,
            reducer.clone(),
            entries,
            0,
            Vec::new(),
        );
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_specs = specs.clone();
        let suspended_reducer = reducer.clone();
        let suspended_grouped = grouped.clone();
        let suspended_nested = nested.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_association_group_by_nested_with_value(
                suspended_head,
                suspended_call_args,
                suspended_specs,
                spec_index,
                suspended_reducer,
                suspended_grouped,
                index,
                suspended_nested,
                suspended_key,
                value,
            )
        })?;

        self.resume_association_group_by_nested_with_value(
            head, call_args, specs, spec_index, reducer, grouped, index, nested, key, value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_group_by_nested_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        specs: Vec<GroupBySpec>,
        spec_index: usize,
        reducer: Option<Expr>,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
        index: usize,
        mut nested: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        nested.push((key, value));
        self.resume_association_group_by_nested_from_index(
            head,
            call_args,
            specs,
            spec_index,
            reducer,
            grouped,
            index + 1,
            nested,
        )
    }

    fn resume_association_group_by_reduce_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        reducer: Expr,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
        index: usize,
        reduced: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value_result = self.evaluate(Expr::call(reducer.clone(), vec![group]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_reducer = reducer.clone();
        let suspended_grouped = grouped.clone();
        let suspended_reduced = reduced.clone();
        let suspended_key = key.clone();
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_association_group_by_reduce_with_value(
                suspended_head,
                suspended_call_args,
                suspended_reducer,
                suspended_grouped,
                index,
                suspended_reduced,
                suspended_key,
                value,
            )
        })?;

        self.resume_association_group_by_reduce_with_value(
            head, call_args, reducer, grouped, index, reduced, key, value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_association_group_by_reduce_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        reducer: Expr,
        grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
        index: usize,
        mut reduced: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        reduced.push((key, value));
        self.resume_association_group_by_reduce_from_index(
            head,
            call_args,
            reducer,
            grouped,
            index + 1,
            reduced,
        )
    }
}

fn association_group_by_groups(
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Vec<(Expr, Expr)> {
    grouped
        .into_iter()
        .map(|(key, entries)| {
            (
                key,
                render_association_entry_forms_preserving_heads(entries),
            )
        })
        .collect()
}

fn between_one_and_three_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; between 1 and 3 arguments are expected.")
}
