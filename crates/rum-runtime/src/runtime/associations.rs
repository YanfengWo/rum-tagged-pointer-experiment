use super::*;

impl Session {
    pub(super) fn eval_association(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let source_items = match args {
            [Expr::Call {
                head: list_head,
                args: items,
            }] if is_system_symbol_name(list_head, heads::structure::LIST) => items.clone(),
            _ => args.to_vec(),
        };

        let Some(entries) = parse_association_entries(&source_items) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(entries))
    }

    pub(super) fn eval_association_q(
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

    pub(super) fn eval_lookup(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_key(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [key] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Key", args, 1));
        };

        Ok(Expr::call(head.clone(), vec![key.clone()]))
    }

    pub(super) fn eval_keys(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_values(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_key_exists_q(
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

    pub(super) fn eval_key_take(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_key_drop(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_key_map(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [function, expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyMap", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            entries
                .into_iter()
                .map(|(key, value)| (Expr::call(function.clone(), vec![key]), value))
                .collect(),
        ))
    }

    pub(super) fn eval_key_value_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [function, expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyValueMap", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("List"),
            entries
                .into_iter()
                .map(|(key, value)| Expr::call(function.clone(), vec![key, value]))
                .collect(),
        ))
    }

    pub(super) fn eval_key_select(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, criterion] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeySelect", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
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

    pub(super) fn eval_key_member_q(
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

    pub(super) fn eval_key_free_q(
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

    pub(super) fn eval_key_sort(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeySort", args, 1));
        };

        let Some(mut entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        entries.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.0, &rhs.0));
        Ok(render_association(entries))
    }

    pub(super) fn eval_key_sort_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, function] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeySortBy", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
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

    pub(super) fn eval_association_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [function, source] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociationMap", args, 2));
        };

        let keys = if let Some(items) = list_items(source) {
            items.to_vec()
        } else if let Some(entries) = association_entries_from_expr(source) {
            entries.into_iter().map(|(key, _)| key).collect()
        } else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
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

    pub(super) fn eval_association_thread(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [keys_expr, values_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociationThread", args, 2));
        };

        let Some(keys) = list_items(keys_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(values) = list_items(values_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if keys.len() != values.len() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(render_association(
            keys.iter().cloned().zip(values.iter().cloned()).collect(),
        ))
    }

    pub(super) fn eval_key_union(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "KeyUnion",
                args,
                "argx",
                "expected at least 2 association arguments",
            ));
        }

        let Some(associations) = parse_association_args(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut seen = std::collections::HashSet::new();
        let mut entries = Vec::new();
        for association in associations {
            for (key, value) in association {
                if seen.insert(key.clone()) {
                    entries.push((key, value));
                }
            }
        }

        Ok(render_association(entries))
    }

    pub(super) fn eval_key_intersection(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "KeyIntersection",
                args,
                "argx",
                "expected at least 2 association arguments",
            ));
        }

        let Some(mut associations) = parse_association_args(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let first = associations.remove(0);
        let other_key_sets = associations
            .into_iter()
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|(key, _)| key)
                    .collect::<std::collections::HashSet<_>>()
            })
            .collect::<Vec<_>>();

        Ok(render_association(
            first
                .into_iter()
                .filter(|(key, _)| other_key_sets.iter().all(|set| set.contains(key)))
                .collect(),
        ))
    }

    pub(super) fn eval_key_complement(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "KeyComplement",
                args,
                "argx",
                "expected at least 2 association arguments",
            ));
        }

        let Some(mut associations) = parse_association_args(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let first = associations.remove(0);
        let excluded = associations
            .into_iter()
            .flat_map(|entries| entries.into_iter().map(|(key, _)| key))
            .collect::<std::collections::HashSet<_>>();

        Ok(render_association(
            first
                .into_iter()
                .filter(|(key, _)| !excluded.contains(key))
                .collect(),
        ))
    }

    pub(super) fn eval_normal(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_associate_to(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociateTo", args, 2));
        };

        let Some(mut entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(updates) = parse_association_update_spec(spec) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        for (key, value) in updates {
            entries.retain(|(existing, _)| existing != &key);
            entries.push((key, value));
        }

        Ok(render_association(entries))
    }

    pub(super) fn eval_key_drop_from(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key_spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyDropFrom", args, 2));
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

    pub(super) fn eval_key_value_pattern(
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

    pub(super) fn eval_merge(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [associations_expr, combiner] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Merge", args, 2));
        };

        let Some(associations) = parse_association_list_expr(associations_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
        for association in associations {
            for (key, value) in association {
                if let Some((_, values)) = grouped.iter_mut().find(|(existing, _)| existing == &key)
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

    pub(super) fn eval_catenate(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [associations_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Catenate", args, 1));
        };

        let Some(associations) = parse_association_list_expr(associations_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            associations.into_iter().flatten().collect(),
        ))
    }

    pub(super) fn eval_counts(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Counts", args, 1));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut counts: Vec<(Expr, usize)> = Vec::new();
        for item in items {
            increment_count(&mut counts, item.clone());
        }

        Ok(render_association(
            counts
                .into_iter()
                .map(|(key, count)| (key, Expr::integer(BigInt::from(count))))
                .collect(),
        ))
    }

    pub(super) fn eval_counts_by(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr, function] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("CountsBy", args, 2));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
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

    pub(super) fn eval_group_by(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr, function] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("GroupBy", args, 2));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.resume_group_by_from_index(
            head.clone(),
            args.to_vec(),
            function.clone(),
            items.to_vec(),
            0,
            Vec::new(),
        )
    }

    pub(super) fn eval_position_index(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PositionIndex", args, 1));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let position = Expr::integer(BigInt::from(index + 1));
            if let Some((_, positions)) = grouped.iter_mut().find(|(key, _)| key == item) {
                positions.push(position);
            } else {
                grouped.push((item.clone(), vec![position]));
            }
        }

        Ok(render_association(
            grouped
                .into_iter()
                .map(|(key, positions)| (key, Expr::call(system_expr("List"), positions)))
                .collect(),
        ))
    }

    pub(super) fn eval_missing(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    fn resume_key_select_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        entries: Vec<(Expr, Expr)>,
        criterion: Expr,
        index: usize,
        selected: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, value)) = entries.get(index).cloned() else {
            return Ok(render_association(selected));
        };

        let call = Expr::call(criterion.clone(), vec![key.clone()]);
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_entries = entries.clone();
        let suspended_criterion = criterion.clone();
        let suspended_selected = selected.clone();
        let suspended_key = key.clone();
        let suspended_entry_value = value.clone();
        let value_result = self.evaluate(call);
        let criterion_result = self.suspend_with_frame(value_result, move |session, outcome| {
            session.resume_key_select_with_value(
                suspended_head,
                suspended_call_args,
                suspended_entries,
                suspended_criterion,
                index,
                suspended_selected,
                suspended_key,
                suspended_entry_value,
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
            key,
            value,
            criterion_result,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_select_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        entries: Vec<(Expr, Expr)>,
        criterion: Expr,
        index: usize,
        mut selected: Vec<(Expr, Expr)>,
        key: Expr,
        entry_value: Expr,
        criterion_value: Expr,
    ) -> Result<Expr, EvalError> {
        if truthiness_symbol(&criterion_value) == Some(true) {
            selected.push((key, entry_value));
        }

        self.resume_key_select_from_index(head, call_args, entries, criterion, index + 1, selected)
    }

    fn resume_key_sort_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        entries: Vec<(Expr, Expr)>,
        index: usize,
        mut decorated: Vec<(Expr, Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, value)) = entries.get(index).cloned() else {
            decorated.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.2, &rhs.2));
            return Ok(render_association(
                decorated
                    .into_iter()
                    .map(|(key, value, _)| (key, value))
                    .collect(),
            ));
        };

        let sort_key_result = self.evaluate(Expr::call(function.clone(), vec![key.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_entries = entries.clone();
        let suspended_decorated = decorated.clone();
        let suspended_key = key.clone();
        let suspended_value = value.clone();
        let sort_key = self.suspend_with_frame(sort_key_result, move |session, sort_key| {
            session.resume_key_sort_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_entries,
                index,
                suspended_decorated,
                suspended_key,
                suspended_value,
                sort_key,
            )
        })?;

        self.resume_key_sort_by_with_value(
            head, call_args, function, entries, index, decorated, key, value, sort_key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_sort_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        entries: Vec<(Expr, Expr)>,
        index: usize,
        mut decorated: Vec<(Expr, Expr, Expr)>,
        key: Expr,
        value: Expr,
        sort_key: Expr,
    ) -> Result<Expr, EvalError> {
        decorated.push((key, value, sort_key));
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
        items: Vec<Expr>,
        index: usize,
        grouped: Vec<(Expr, Vec<Expr>)>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association(
                grouped
                    .into_iter()
                    .map(|(key, values)| (key, Expr::call(system_expr("List"), values)))
                    .collect(),
            ));
        };

        let key_result = self.evaluate(Expr::call(function.clone(), vec![item.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_items = items.clone();
        let suspended_grouped = grouped.clone();
        let suspended_item = item.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_group_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_items,
                index,
                suspended_grouped,
                suspended_item,
                key,
            )
        })?;

        self.resume_group_by_with_value(head, call_args, function, items, index, grouped, item, key)
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_group_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
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

        self.resume_group_by_from_index(head, call_args, function, items, index + 1, grouped)
    }
}

pub(super) fn association_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entries(parse_association_entries(
        args,
    )?))
}

fn parse_association_entries(items: &[Expr]) -> Option<Vec<(Expr, Expr)>> {
    items.iter().map(parse_association_entry).collect()
}

pub(super) fn render_association(entries: Vec<(Expr, Expr)>) -> Expr {
    let entries = canonicalize_association_entries(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
            .collect(),
    )
}

pub(super) fn association_value_by_key(expr: &Expr, key: &Expr) -> Option<Expr> {
    association_entries_from_expr(expr)?
        .into_iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, value)| value)
}

pub(super) fn association_position_and_value_by_key(
    expr: &Expr,
    key: &Expr,
) -> Option<(isize, Expr)> {
    association_entries_from_expr(expr)?
        .into_iter()
        .enumerate()
        .find(|(_, (entry_key, _))| entry_key == key)
        .map(|(index, (_, value))| (index as isize + 1, value))
}

fn parse_key_spec(expr: &Expr) -> Option<Vec<Expr>> {
    let Expr::Call { head, args } = expr else {
        return Some(vec![expr.clone()]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![expr.clone()]);
    }

    Some(args.clone())
}

fn parse_association_update_spec(spec: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(spec) {
        return Some(vec![entry]);
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries(args)
}

pub(super) fn normalize_key_value_pattern_spec(spec: &Expr) -> Option<Expr> {
    if let Some((key, value)) = parse_association_entry(spec) {
        return Some(Expr::call(
            system_expr("List"),
            vec![Expr::call(system_expr("Rule"), vec![key, value])],
        ));
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let entries = parse_association_entries(args)?;
    Some(Expr::call(
        system_expr("List"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
            .collect(),
    ))
}

fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn parse_association_args(args: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    args.iter().map(association_entries_from_expr).collect()
}

fn parse_association_list_expr(expr: &Expr) -> Option<Vec<Vec<(Expr, Expr)>>> {
    let items = list_items(expr)?;
    items.iter().map(association_entries_from_expr).collect()
}

fn increment_count(counts: &mut Vec<(Expr, usize)>, key: Expr) {
    if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| existing == &key) {
        *count += 1;
    } else {
        counts.push((key, 1));
    }
}

fn canonicalize_association_entries(entries: Vec<(Expr, Expr)>) -> Vec<(Expr, Expr)> {
    let mut canonical = Vec::new();

    for (key, value) in entries {
        if let Some((_, existing_value)) = canonical
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            *existing_value = value;
        } else {
            canonical.push((key, value));
        }
    }

    canonical
}

pub(super) fn parse_association_entry(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
