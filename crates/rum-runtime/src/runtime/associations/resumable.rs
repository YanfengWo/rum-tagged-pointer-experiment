use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeySortOrderingDecision {
    InsertBefore,
    KeepScanning,
}

impl Session {
    pub(in crate::runtime) fn eval_key_select(
        &mut self,
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
                    return Ok(self.invalid_call_with_message(
                        "KeySelect",
                        args,
                        "invru",
                        invalid_association_or_rule_list_message(expr),
                    ));
                };

                self.resume_key_select_from_index(
                    head.clone(),
                    args.to_vec(),
                    entries,
                    criterion.clone(),
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "KeySelect",
                args,
                "argt",
                one_or_two_argument_count_message("KeySelect", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_key_sort_by(
        &mut self,
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
                    return Ok(self.invalid_call_with_message(
                        "KeySortBy",
                        args,
                        "invrl",
                        invalid_association_or_rule_list_message(expr),
                    ));
                };

                self.resume_key_sort_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
                    entries,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "KeySortBy",
                args,
                "argt",
                one_or_two_argument_count_message("KeySortBy", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn resume_key_sort_with_ordering_from_index(
        &mut self,
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

        self.resume_key_sort_with_ordering_insert(
            head, call_args, ordering, entries, index, sorted, candidate, 0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_sort_with_ordering_insert(
        &mut self,
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
            return self.resume_key_sort_with_ordering_from_index(
                head,
                call_args,
                ordering,
                entries,
                index + 1,
                next_sorted,
            );
        };

        let compare_result = self.evaluate(Expr::call(
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
            self.suspend_with_frame(compare_result, move |session, compare_value| {
                session.resume_key_sort_with_ordering_after_compare(
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

        self.resume_key_sort_with_ordering_after_compare(
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
        &mut self,
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
                self.resume_key_sort_with_ordering_from_index(
                    head,
                    call_args,
                    ordering,
                    entries,
                    index + 1,
                    next_sorted,
                )
            }
            KeySortOrderingDecision::KeepScanning => self.resume_key_sort_with_ordering_insert(
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

    pub(in crate::runtime) fn eval_merge(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [combiner] => Ok(keyed_operator_form_slot_first("Merge", combiner.clone())),
            [associations_expr, combiner] => {
                let associations =
                    match parse_keyed_entries_list_expr_with_diagnostics(associations_expr) {
                        Ok(associations) => associations,
                        Err(KeyedEntriesListError::NotList) => {
                            return Ok(self.invalid_call_with_message(
                                "Merge",
                                args,
                                "list1",
                                invalid_keyed_collection_list_message(associations_expr),
                            ));
                        }
                        Err(KeyedEntriesListError::InvalidElement(expr)) => {
                            return Ok(self.invalid_call_with_message(
                                "Merge",
                                args,
                                "list1",
                                invalid_keyed_collection_list_message(&expr),
                            ));
                        }
                    };

                if associations.is_empty() {
                    return Ok(render_association(Vec::new()));
                };

                let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
                for association in associations {
                    for (key, value) in association {
                        if let Some((_, values)) =
                            grouped.iter_mut().find(|(existing, _)| existing == &key)
                        {
                            values.push(value);
                        } else {
                            grouped.push((key, vec![value]));
                        }
                    }
                }

                self.resume_merge_from_index(
                    head.clone(),
                    args.to_vec(),
                    combiner.clone(),
                    grouped,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "Merge",
                args,
                "argt",
                one_or_two_argument_count_message("Merge", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_counts_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_first("CountsBy", function.clone())),
            [expr, function] => {
                let Some(items) = list_items(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "CountsBy",
                        args,
                        "list",
                        list_expected_message(1, expr),
                    ));
                };

                self.resume_counts_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
                    items.to_vec(),
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "CountsBy",
                args,
                "argt",
                one_or_two_argument_count_message("CountsBy", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_group_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_first("GroupBy", function.clone())),
            [expr, function] => {
                let Some(items) = list_items(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "GroupBy",
                        args,
                        "list1",
                        invalid_keyed_collection_list_message(expr),
                    ));
                };

                self.resume_group_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
                    None,
                    items.to_vec(),
                    0,
                    Vec::new(),
                )
            }
            [expr, function, reducer] => {
                let Some(items) = list_items(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "GroupBy",
                        args,
                        "list1",
                        invalid_keyed_collection_list_message(expr),
                    ));
                };

                self.resume_group_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
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

    fn resume_key_select_from_index(
        &mut self,
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
        let value_result = self.evaluate(call);
        let criterion_result = self.suspend_with_frame(value_result, move |session, outcome| {
            session.resume_key_select_with_value(
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

        self.resume_key_select_with_value(
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
        &mut self,
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

        self.resume_key_select_from_index(head, call_args, entries, criterion, index + 1, selected)
    }

    fn resume_key_sort_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        mut decorated: Vec<(AssociationEntryForm, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some(entry) = entries.get(index).cloned() else {
            decorated.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.1, &rhs.1));
            return Ok(render_association_entry_forms_preserving_heads(
                decorated.into_iter().map(|(entry, _)| entry).collect(),
            ));
        };

        let sort_key_result = self.evaluate(Expr::call(function.clone(), vec![entry.key.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_entries = entries.clone();
        let suspended_decorated = decorated.clone();
        let suspended_entry = entry.clone();
        let sort_key = self.suspend_with_frame(sort_key_result, move |session, sort_key| {
            session.resume_key_sort_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_entries,
                index,
                suspended_decorated,
                suspended_entry,
                sort_key,
            )
        })?;

        self.resume_key_sort_by_with_value(
            head, call_args, function, entries, index, decorated, entry, sort_key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_sort_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        mut decorated: Vec<(AssociationEntryForm, Expr)>,
        entry: AssociationEntryForm,
        sort_key: Expr,
    ) -> Result<Expr, EvalError> {
        decorated.push((entry, sort_key));
        self.resume_key_sort_by_from_index(head, call_args, function, entries, index + 1, decorated)
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

    fn resume_merge_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        combiner: Expr,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        merged: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged));
        };

        let combined_result = self.evaluate(Expr::call(
            combiner.clone(),
            vec![Expr::call(system_expr("List"), values)],
        ));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_combiner = combiner.clone();
        let suspended_grouped = grouped.clone();
        let suspended_merged = merged.clone();
        let suspended_key = key.clone();
        let combined = self.suspend_with_frame(combined_result, move |session, combined| {
            session.resume_merge_with_value(
                suspended_head,
                suspended_call_args,
                suspended_combiner,
                suspended_grouped,
                index,
                suspended_merged,
                suspended_key,
                combined,
            )
        })?;

        self.resume_merge_with_value(
            head, call_args, combiner, grouped, index, merged, key, combined,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_merge_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        combiner: Expr,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        mut merged: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        merged.push((key, value));
        self.resume_merge_from_index(head, call_args, combiner, grouped, index + 1, merged)
    }

    fn resume_counts_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        counts: Vec<(Expr, usize)>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association(
                counts
                    .into_iter()
                    .map(|(key, count)| (key, Expr::integer(BigInt::from(count))))
                    .collect(),
            ));
        };

        let key_result = self.evaluate(Expr::call(function.clone(), vec![item]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_items = items.clone();
        let suspended_counts = counts.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_counts_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_items,
                index,
                suspended_counts,
                key,
            )
        })?;

        self.resume_counts_by_with_value(head, call_args, function, items, index, counts, key)
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_counts_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        mut counts: Vec<(Expr, usize)>,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        increment_count(&mut counts, key);
        self.resume_counts_by_from_index(head, call_args, function, items, index + 1, counts)
    }

    fn resume_group_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        reducer: Option<Expr>,
        items: Vec<Expr>,
        index: usize,
        grouped: Vec<(Expr, Vec<Expr>)>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return match reducer {
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
            };
        };

        let key_result = self.evaluate(Expr::call(function.clone(), vec![item.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_reducer = reducer.clone();
        let suspended_items = items.clone();
        let suspended_grouped = grouped.clone();
        let suspended_item = item.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_group_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_reducer,
                suspended_items,
                index,
                suspended_grouped,
                suspended_item,
                key,
            )
        })?;

        self.resume_group_by_with_value(
            head, call_args, function, reducer, items, index, grouped, item, key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        reducer: Option<Expr>,
        items: Vec<Expr>,
        index: usize,
        mut grouped: Vec<(Expr, Vec<Expr>)>,
        item: Expr,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some((_, values)) = grouped.iter_mut().find(|(existing, _)| existing == &key) {
            values.push(item);
        } else {
            grouped.push((key, vec![item]));
        }

        self.resume_group_by_from_index(
            head,
            call_args,
            function,
            reducer,
            items,
            index + 1,
            grouped,
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
}

fn key_sort_ordering_decision(value: &Expr) -> KeySortOrderingDecision {
    match value {
        Expr::Atom(Atom::Integer(integer)) if integer == &BigInt::from(1_i8) => {
            KeySortOrderingDecision::InsertBefore
        }
        Expr::Atom(Atom::Integer(integer))
            if integer == &BigInt::from(0_i8) || integer == &BigInt::from(-1_i8) =>
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

fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}

fn between_one_and_three_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; between 1 and 3 arguments are expected.")
}
