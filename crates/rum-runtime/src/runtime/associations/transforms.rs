use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeldAssociationPartMutationResult {
    UpdatedTarget,
    UpdatedPart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CatenateStep {
    Append(Vec<Expr>),
    AssociationValue { value: Expr, delayed: bool },
    Invalid(Expr),
}

fn part_target_selectors_are_exact_leaf_path(selector_exprs: &[Expr]) -> bool {
    selector_exprs.iter().all(|expr| {
        expr_as_signed_isize(expr).is_some()
            || matches!(
                expr,
                Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1
            )
    })
}

fn part_selector_key_expr(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn missing_key_in_exact_part_selector_path(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<(Expr, Expr)> {
    let mut selected = expr.clone();
    for selector in selector_exprs {
        if let Some(index) = expr_as_signed_isize(selector) {
            selected = exact_expr_at_path(&selected, &[index])?;
            continue;
        }

        let key = part_selector_key_expr(selector)?;
        let entries = association_entry_forms_from_expr(&selected)?;
        if let Some(entry) = entries.into_iter().find(|entry| entry.key == key) {
            selected = entry.value;
        } else {
            return Some((key, selected));
        }
    }

    None
}

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
            _ => Ok(self.invalid_call_with_arg_range("KeyMap", args, 1, 2)),
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
            _ => Ok(self.invalid_call_with_arg_range("KeyValueMap", args, 1, 2)),
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
            _ => Ok(self.invalid_call_with_arg_range("KeySort", args, 1, 2)),
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
            _ => return Ok(self.invalid_call_with_message(
                "AssociationThread",
                args,
                "argt",
                format!(
                    "AssociationThread called with {} arguments; 1 or 2 arguments are expected.",
                    args.len()
                ),
            )),
        };

        let keys = threaded_items(&keys_expr);
        let values = threaded_items(&values_expr);
        if keys.len() != values.len() {
            return Ok(self.invalid_call_with_message(
                "AssociationThread",
                args,
                "idim",
                same_length_message(&keys_expr, &values_expr),
            ));
        }

        Ok(render_association_entry_forms_preserving_heads(
            keys.into_iter()
                .zip(values)
                .map(|(key, value)| AssociationEntryForm {
                    key,
                    value,
                    delayed,
                })
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_union(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (collections_expr, fill_function) = match args {
            [collections_expr] => (collections_expr, None),
            [collections_expr, fill_function] => (collections_expr, Some(fill_function.clone())),
            _ => {
                return Ok(self.invalid_call_with_message(
                    "KeyUnion",
                    args,
                    "argt",
                    one_or_two_argument_count_message("KeyUnion", args.len()),
                ));
            }
        };

        let collections = match parse_keyed_entry_forms_list_expr_with_diagnostics(collections_expr)
        {
            Ok(collections) => collections,
            Err(KeyedEntriesListError::NotList) => {
                return Ok(self.invalid_call_with_message(
                    "KeyUnion",
                    args,
                    "invar",
                    invalid_keyed_collection_outer_list_message(collections_expr),
                ));
            }
            Err(KeyedEntriesListError::InvalidElement(expr)) => {
                return Ok(self.invalid_call_with_message(
                    "KeyUnion",
                    args,
                    "invas",
                    invalid_association_or_rule_message(&expr),
                ));
            }
        };

        let union_entries = key_union_entries(&collections);
        let missing_keys = key_union_missing_keys(&collections, &union_entries);

        match fill_function {
            None => Ok(render_key_union_result(&collections, &union_entries, &[])),
            Some(fill_function) => self.resume_key_union_missing_values_from_index(
                head.clone(),
                args.to_vec(),
                fill_function,
                collections,
                union_entries,
                missing_keys,
                0,
                Vec::new(),
            ),
        }
    }

    pub(in crate::runtime) fn eval_key_intersection(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [collections_expr] = args else {
            return Ok(self.invalid_call_with_message(
                "KeyIntersection",
                args,
                "argx",
                exact_argument_count_message("KeyIntersection", args.len(), 1),
            ));
        };

        let collections = match parse_keyed_entry_forms_list_expr_with_diagnostics(collections_expr)
        {
            Ok(collections) => collections,
            Err(KeyedEntriesListError::NotList) => {
                return Ok(self.invalid_call_with_message(
                    "KeyIntersection",
                    args,
                    "invar",
                    invalid_keyed_collection_outer_list_message(collections_expr),
                ));
            }
            Err(KeyedEntriesListError::InvalidElement(expr)) => {
                return Ok(self.invalid_call_with_message(
                    "KeyIntersection",
                    args,
                    "invas",
                    invalid_association_or_rule_message(&expr),
                ));
            }
        };

        let Some(first) = collections.first() else {
            return Ok(Expr::call(system_expr("List"), vec![]));
        };

        let intersection_keys = first
            .iter()
            .filter(|entry| {
                collections[1..]
                    .iter()
                    .all(|entries| entries.iter().any(|candidate| candidate.key == entry.key))
            })
            .map(|entry| entry.key.clone())
            .collect::<Vec<_>>();

        Ok(Expr::call(
            system_expr("List"),
            collections
                .into_iter()
                .map(|entries| {
                    render_association_entry_forms_preserving_heads(
                        intersection_keys
                            .iter()
                            .filter_map(|key| {
                                entries.iter().find(|entry| entry.key == *key).map(|entry| {
                                    AssociationEntryForm {
                                        key: key.clone(),
                                        value: entry.value.clone(),
                                        delayed: entry.delayed,
                                    }
                                })
                            })
                            .collect(),
                    )
                })
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_complement(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [collections_expr] = args else {
            if args.is_empty() {
                return Ok(self.invalid_call_with_message(
                    "KeyComplement",
                    args,
                    "argx",
                    exact_argument_count_message("KeyComplement", args.len(), 1),
                ));
            }

            let held_input = Expr::call(system_expr("KeyComplement"), args.to_vec());
            return Ok(match collect_option_names_from_tail(self, &args[1..]) {
                Some(option_names) => self.invalid_call_with_message(
                    "KeyComplement",
                    args,
                    "optx",
                    unknown_option_message(&option_names[0], &held_input),
                ),
                None => self.invalid_call_with_message(
                    "KeyComplement",
                    args,
                    "nonopt",
                    non_option_syntax_message(&args[args.len() - 1], 1, &held_input),
                ),
            });
        };

        let collections = match parse_keyed_entry_forms_list_expr_with_diagnostics(collections_expr)
        {
            Ok(collections) => collections,
            Err(KeyedEntriesListError::NotList) => {
                return Ok(self.invalid_call_with_message(
                    "KeyComplement",
                    args,
                    "invar",
                    invalid_keyed_collection_outer_list_message(collections_expr),
                ));
            }
            Err(KeyedEntriesListError::InvalidElement(expr)) => {
                return Ok(self.invalid_call_with_message(
                    "KeyComplement",
                    args,
                    "invrl",
                    invalid_association_or_rule_list_message(&expr),
                ));
            }
        };

        let Some((first, rest)) = collections.split_first() else {
            return Ok(self.invalid_call_with_message(
                "KeyComplement",
                args,
                "empt",
                nonempty_list_message(collections_expr),
            ));
        };

        let excluded = rest
            .iter()
            .flat_map(|entries| entries.iter().map(|entry| entry.key.clone()))
            .collect::<std::collections::HashSet<_>>();

        Ok(render_association_entry_forms_preserving_heads(
            first
                .iter()
                .filter(|entry| !excluded.contains(&entry.key))
                .cloned()
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_associate_to(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [target_expr, spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociateTo", args, 2));
        };

        if parse_association_update_spec_forms(spec).is_none() {
            return Ok(self.invalid_call_with_message(
                "AssociateTo",
                args,
                "invlb",
                invalid_association_update_spec_message(spec),
            ));
        }

        if let Some(target) = self.normalize_symbol_target_expr(target_expr, true) {
            return self.eval_held_association_mutation(
                "AssociateTo",
                target_expr.clone(),
                target,
                spec.clone(),
                Self::apply_associate_to_value,
            );
        };

        if let Some((target, selector_exprs)) = self.normalize_part_mutation_target(target_expr) {
            if let Some(updated) = self.try_eval_held_association_part_mutation(
                "AssociateTo",
                target_expr.clone(),
                target,
                selector_exprs,
                spec.clone(),
                Self::apply_associate_to_value,
                HeldAssociationPartMutationResult::UpdatedTarget,
            )? {
                return Ok(updated);
            }
        }

        if association_entries_from_expr(target_expr).is_some()
            || rule_list_entries_from_expr(target_expr).is_some()
        {
            if let Expr::Call {
                head: target_head, ..
            } = target_expr
            {
                if let Some(target_symbol) = expr_as_symbol(target_head) {
                    self.push_protected_write_message("AssociateTo", target_symbol);
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
            }
        }

        let target_result = self.evaluate(target_expr.clone());
        let expr = self.suspend_with_frame(target_result, {
            let head = head.clone();
            let call_args = args.to_vec();
            let spec = spec.clone();
            move |session, expr| session.finish_associate_to_value(head, call_args, expr, spec)
        })?;

        self.finish_associate_to_value(head.clone(), args.to_vec(), expr, spec.clone())
    }

    fn finish_associate_to_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        expr: Expr,
        spec: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(updated) = Self::apply_associate_to_value(expr, spec) else {
            return Ok(Expr::call(head, call_args));
        };

        Ok(updated)
    }

    fn apply_associate_to_value(expr: Expr, spec: Expr) -> Option<Expr> {
        let mut entries = association_entry_forms_from_expr(&expr)?;
        let updates = parse_association_update_spec_forms(&spec)?;

        for update in updates {
            entries.retain(|existing| existing.key != update.key);
            entries.push(update);
        }

        Some(render_association_entry_forms_preserving_heads(entries))
    }

    pub(in crate::runtime) fn eval_key_drop_from(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [target_expr, key_spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyDropFrom", args, 2));
        };

        if let Some(target) = self.normalize_symbol_target_expr(target_expr, true) {
            return self.eval_held_association_mutation(
                "KeyDropFrom",
                target_expr.clone(),
                target,
                key_spec.clone(),
                Self::apply_key_drop_from_value,
            );
        };

        if let Some((target, selector_exprs)) = self.normalize_part_mutation_target(target_expr) {
            if part_target_selectors_are_exact_leaf_path(&selector_exprs) {
                if let Some(updated) = self.try_eval_held_association_part_mutation(
                    "KeyDropFrom",
                    target_expr.clone(),
                    target,
                    selector_exprs,
                    key_spec.clone(),
                    Self::apply_key_drop_from_value,
                    HeldAssociationPartMutationResult::UpdatedPart,
                )? {
                    return Ok(updated);
                }
            }

            return Ok(self.invalid_call_with_message(
                "KeyDropFrom",
                args,
                "invak",
                invalid_association_message(target_expr),
            ));
        }

        if association_entries_from_expr(target_expr).is_some() {
            let updated = Self::apply_key_drop_from_value(target_expr.clone(), key_spec.clone())
                .expect("association literal should be droppable");
            if let Some(message) = protected_tag_write_message(target_expr) {
                self.push_message("Set", "write", message);
            }
            return Ok(updated);
        }

        if rule_list_entries_from_expr(target_expr).is_some() {
            return Ok(self.invalid_call_with_message(
                "KeyDropFrom",
                args,
                "invak",
                invalid_association_message(target_expr),
            ));
        }

        let target_result = self.evaluate(target_expr.clone());
        let expr = self.suspend_with_frame(target_result, {
            let head = head.clone();
            let call_args = args.to_vec();
            let key_spec = key_spec.clone();
            move |session, expr| session.finish_key_drop_from_value(head, call_args, expr, key_spec)
        })?;

        self.finish_key_drop_from_value(head.clone(), args.to_vec(), expr, key_spec.clone())
    }

    fn finish_key_drop_from_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        expr: Expr,
        key_spec: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(updated) = Self::apply_key_drop_from_value(expr, key_spec) else {
            return Ok(Expr::call(head, call_args));
        };

        Ok(updated)
    }

    fn apply_key_drop_from_value(expr: Expr, key_spec: Expr) -> Option<Expr> {
        let entries = association_entry_forms_from_expr(&expr)?;
        let selected_keys = parse_key_spec(&key_spec)?;

        Some(render_association_entry_forms_preserving_heads(
            entries
                .into_iter()
                .filter(|entry| {
                    !selected_keys
                        .iter()
                        .any(|candidate| candidate == &entry.key)
                })
                .collect(),
        ))
    }

    fn eval_held_association_mutation(
        &mut self,
        operator: &'static str,
        target_expr: Expr,
        target: Symbol,
        spec: Expr,
        mutate: fn(Expr, Expr) -> Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let held_call = vec![target_expr.clone(), spec.clone()];

        if !self.ensure_symbol_not_protected_for_write(operator, &target) {
            return Ok(Expr::call(system_expr(operator), held_call));
        }

        let Some(current) = self
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.clone())
        else {
            self.push_message(
                operator,
                "blnoval",
                missing_immediate_value_message(&target, 1),
            );
            return Ok(Expr::call(system_expr(operator), held_call));
        };

        self.finish_held_association_mutation(
            operator,
            target_expr,
            target,
            held_call,
            spec,
            current,
            mutate,
        )
    }

    fn finish_held_association_mutation(
        &mut self,
        operator: &'static str,
        target_expr: Expr,
        target: Symbol,
        held_call: Vec<Expr>,
        spec: Expr,
        current: Expr,
        mutate: fn(Expr, Expr) -> Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(updated) = mutate(current, spec) else {
            return Ok(self.invalid_call_with_message(
                operator,
                &held_call,
                "invak",
                invalid_association_message(&target_expr),
            ));
        };

        let definition = self.definitions.entry(target).or_default();
        definition.own_value = Some(updated.clone());
        definition.delayed_own_value = None;

        Ok(updated)
    }

    fn normalize_part_mutation_target(&self, target_expr: &Expr) -> Option<(Symbol, Vec<Expr>)> {
        let Expr::Call { head, args } = target_expr else {
            return None;
        };

        if !is_system_symbol_name(head, "Part") || args.len() < 2 {
            return None;
        }

        let target = self.normalize_symbol_target_expr(&args[0], true)?;
        Some((target, args[1..].to_vec()))
    }

    fn try_eval_held_association_part_mutation(
        &mut self,
        operator: &'static str,
        target_expr: Expr,
        target: Symbol,
        selector_exprs: Vec<Expr>,
        spec: Expr,
        mutate: fn(Expr, Expr) -> Option<Expr>,
        result: HeldAssociationPartMutationResult,
    ) -> Result<Option<Expr>, EvalError> {
        let held_call = vec![target_expr.clone(), spec.clone()];

        if !self.ensure_symbol_not_protected_for_write(operator, &target) {
            return Ok(Some(Expr::call(system_expr(operator), held_call)));
        }

        let Some(current) = self
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.clone())
        else {
            return Ok(None);
        };

        let Some(paths) = exact_position_paths_from_part_selector_exprs(&current, &selector_exprs)
        else {
            return Ok(None);
        };

        if paths.is_empty() {
            if operator == "AssociateTo" {
                if let Some((key, target)) =
                    missing_key_in_exact_part_selector_path(&current, &selector_exprs)
                {
                    self.push_message(
                        "AssociateTo",
                        "keynx",
                        format!("Key {key} does not exist in {target}."),
                    );
                }
                return Ok(Some(Expr::call(system_expr(operator), held_call)));
            }

            if operator == "KeyDropFrom" {
                return Ok(None);
            }

            return Ok(Some(Expr::call(system_expr(operator), held_call)));
        }

        let mut replacements = Vec::with_capacity(paths.len());
        for path in paths {
            let Some(part) = exact_expr_at_path(&current, &path) else {
                return Ok(Some(Expr::call(system_expr(operator), held_call)));
            };
            let Some(updated_part) = mutate(part.clone(), spec.clone()) else {
                let invalid_expr = match result {
                    HeldAssociationPartMutationResult::UpdatedTarget => part,
                    HeldAssociationPartMutationResult::UpdatedPart => target_expr,
                };
                return Ok(Some(self.invalid_call_with_message(
                    operator,
                    &held_call,
                    "invak",
                    invalid_association_message(&invalid_expr),
                )));
            };
            replacements.push((path, updated_part));
        }

        let selected_result = replacements
            .first()
            .map(|(_, updated_part)| updated_part.clone())
            .unwrap_or_else(|| Expr::call(system_expr(operator), held_call.clone()));

        let mut updated_target = current;
        for (path, updated_part) in replacements {
            updated_target = replace_exact_path(updated_target, &path, updated_part);
        }

        let definition = self.definitions.entry(target).or_default();
        definition.own_value = Some(updated_target.clone());
        definition.delayed_own_value = None;

        let value = match result {
            HeldAssociationPartMutationResult::UpdatedTarget => updated_target,
            HeldAssociationPartMutationResult::UpdatedPart => selected_result,
        };
        Ok(Some(value))
    }

    pub(in crate::runtime) fn eval_catenate(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [associations_expr] = args else {
            return Ok(self.invalid_call_with_message(
                "Catenate",
                args,
                "argx",
                catenate_arg_count_message(args.len()),
            ));
        };

        if let Some(entries) = association_entry_forms_from_expr(associations_expr) {
            return self.resume_catenate_direct_association_entries_from_index(
                head.clone(),
                args.to_vec(),
                entries,
                0,
                Vec::new(),
            );
        }

        let Some(items) = list_items(associations_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.resume_catenate_steps_from_index(
            head.clone(),
            args.to_vec(),
            catenate_steps_for_list_items(items),
            0,
            Vec::new(),
        )
    }

    pub(in crate::runtime) fn eval_counts(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_position_index(
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
}

fn threaded_items(expr: &Expr) -> Vec<Expr> {
    list_items(expr)
        .map(|items| items.to_vec())
        .unwrap_or_else(|| vec![expr.clone()])
}

fn catenate_item_contribution(expr: &Expr) -> Option<Vec<Expr>> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(entries.into_iter().map(|(_, value)| value).collect());
    }

    list_items(expr).map(|items| items.to_vec())
}

fn catenate_arg_count_message(count: usize) -> String {
    format!("Catenate called with {count} arguments; 1 argument is expected.")
}

fn catenate_steps_for_association_entries(entries: Vec<AssociationEntryForm>) -> Vec<CatenateStep> {
    entries
        .into_iter()
        .map(|entry| CatenateStep::AssociationValue {
            value: entry.value,
            delayed: entry.delayed,
        })
        .collect()
}

fn catenate_steps_for_list_items(items: &[Expr]) -> Vec<CatenateStep> {
    let mut steps = Vec::new();
    for item in items {
        if let Some(entries) = association_entry_forms_from_expr(item) {
            steps.extend(catenate_steps_for_association_entries(entries));
        } else if let Some(items) = list_items(item) {
            steps.push(CatenateStep::Append(items.to_vec()));
        } else {
            steps.push(CatenateStep::Invalid(item.clone()));
            break;
        }
    }

    steps
}

impl Session {
    fn resume_catenate_direct_association_entries_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        values: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(entry) = entries.get(index).cloned() else {
            return self.finish_catenate_direct_association_values(call_args, values);
        };

        if entry.delayed {
            let suspended_head = head.clone();
            let suspended_call_args = call_args.clone();
            let suspended_entries = entries.clone();
            let suspended_values = values.clone();
            let value_result = self.evaluate(entry.value);
            let value = self.suspend_with_frame(value_result, move |session, value| {
                session.resume_catenate_direct_association_value(
                    suspended_head,
                    suspended_call_args,
                    suspended_entries,
                    index,
                    suspended_values,
                    value,
                )
            })?;
            self.resume_catenate_direct_association_value(
                head, call_args, entries, index, values, value,
            )
        } else {
            self.resume_catenate_direct_association_value(
                head,
                call_args,
                entries,
                index,
                values,
                entry.value,
            )
        }
    }

    fn resume_catenate_direct_association_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        mut values: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        values.push(value);
        self.resume_catenate_direct_association_entries_from_index(
            head,
            call_args,
            entries,
            index + 1,
            values,
        )
    }

    fn finish_catenate_direct_association_values(
        &mut self,
        call_args: Vec<Expr>,
        values: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let flatten_values = values
            .iter()
            .any(|value| is_missing_expr(value) || catenate_item_contribution(value).is_some());

        if !flatten_values {
            return Ok(Expr::call(system_expr("List"), values));
        }

        let mut flattened = Vec::new();
        for value in values {
            if is_missing_expr(&value) {
                continue;
            }

            let Some(items) = catenate_item_contribution(&value) else {
                return Ok(self.invalid_call_with_message(
                    "Catenate",
                    &call_args,
                    "invrp",
                    invalid_association_or_list_message(&value),
                ));
            };
            flattened.extend(items);
        }

        Ok(Expr::call(system_expr("List"), flattened))
    }

    fn resume_catenate_steps_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        steps: Vec<CatenateStep>,
        index: usize,
        mut flattened: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(step) = steps.get(index).cloned() else {
            return Ok(Expr::call(system_expr("List"), flattened));
        };

        match step {
            CatenateStep::Append(items) => {
                flattened.extend(items);
                self.resume_catenate_steps_from_index(head, call_args, steps, index + 1, flattened)
            }
            CatenateStep::AssociationValue { value, delayed } => {
                if delayed {
                    let suspended_head = head.clone();
                    let suspended_call_args = call_args.clone();
                    let suspended_steps = steps.clone();
                    let suspended_flattened = flattened.clone();
                    let value_result = self.evaluate(value);
                    let value = self.suspend_with_frame(value_result, move |session, value| {
                        session.resume_catenate_after_association_value(
                            suspended_head,
                            suspended_call_args,
                            suspended_steps,
                            index,
                            suspended_flattened,
                            value,
                        )
                    })?;
                    self.resume_catenate_after_association_value(
                        head, call_args, steps, index, flattened, value,
                    )
                } else {
                    self.resume_catenate_after_association_value(
                        head, call_args, steps, index, flattened, value,
                    )
                }
            }
            CatenateStep::Invalid(expr) => Ok(self.invalid_call_with_message(
                "Catenate",
                &call_args,
                "invrp",
                invalid_association_or_list_message(&expr),
            )),
        }
    }

    fn resume_catenate_after_association_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        steps: Vec<CatenateStep>,
        index: usize,
        mut flattened: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        flattened.push(value);
        self.resume_catenate_steps_from_index(head, call_args, steps, index + 1, flattened)
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_union_missing_values_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        fill_function: Expr,
        collections: Vec<Vec<AssociationEntryForm>>,
        union_entries: Vec<AssociationEntryForm>,
        missing_keys: Vec<Expr>,
        index: usize,
        fill_values: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some(key) = missing_keys.get(index).cloned() else {
            return Ok(render_key_union_result(
                &collections,
                &union_entries,
                &fill_values,
            ));
        };

        let fill_result = self.evaluate(Expr::call(fill_function.clone(), vec![key.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_fill_function = fill_function.clone();
        let suspended_collections = collections.clone();
        let suspended_union_entries = union_entries.clone();
        let suspended_missing_keys = missing_keys.clone();
        let suspended_fill_values = fill_values.clone();
        let suspended_key = key.clone();
        let fill_value = self.suspend_with_frame(fill_result, move |session, fill_value| {
            session.resume_key_union_missing_value(
                suspended_head,
                suspended_call_args,
                suspended_fill_function,
                suspended_collections,
                suspended_union_entries,
                suspended_missing_keys,
                index,
                suspended_fill_values,
                suspended_key,
                fill_value,
            )
        })?;

        self.resume_key_union_missing_value(
            head,
            call_args,
            fill_function,
            collections,
            union_entries,
            missing_keys,
            index,
            fill_values,
            key,
            fill_value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_union_missing_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        fill_function: Expr,
        collections: Vec<Vec<AssociationEntryForm>>,
        union_entries: Vec<AssociationEntryForm>,
        missing_keys: Vec<Expr>,
        index: usize,
        mut fill_values: Vec<(Expr, Expr)>,
        key: Expr,
        fill_value: Expr,
    ) -> Result<Expr, EvalError> {
        fill_values.push((key, fill_value));
        self.resume_key_union_missing_values_from_index(
            head,
            call_args,
            fill_function,
            collections,
            union_entries,
            missing_keys,
            index + 1,
            fill_values,
        )
    }
}

fn key_union_entries(collections: &[Vec<AssociationEntryForm>]) -> Vec<AssociationEntryForm> {
    let mut union_entries = Vec::new();
    for entries in collections {
        for entry in entries {
            if !union_entries
                .iter()
                .any(|existing: &AssociationEntryForm| existing.key == entry.key)
            {
                union_entries.push(entry.clone());
            }
        }
    }
    union_entries
}

fn key_union_missing_keys(
    collections: &[Vec<AssociationEntryForm>],
    union_entries: &[AssociationEntryForm],
) -> Vec<Expr> {
    union_entries
        .iter()
        .filter(|entry| {
            collections
                .iter()
                .any(|entries| !entries.iter().any(|candidate| candidate.key == entry.key))
        })
        .map(|entry| entry.key.clone())
        .collect()
}

fn render_key_union_result(
    collections: &[Vec<AssociationEntryForm>],
    union_entries: &[AssociationEntryForm],
    fill_values: &[(Expr, Expr)],
) -> Expr {
    Expr::call(
        system_expr("List"),
        collections
            .iter()
            .map(|entries| {
                render_association_entry_forms_preserving_heads(
                    union_entries
                        .iter()
                        .map(|union_entry| {
                            entries
                                .iter()
                                .find(|entry| entry.key == union_entry.key)
                                .cloned()
                                .or_else(|| {
                                    fill_values
                                        .iter()
                                        .find(|(fill_key, _)| fill_key == &union_entry.key)
                                        .map(|(_, value)| AssociationEntryForm {
                                            key: union_entry.key.clone(),
                                            value: value.clone(),
                                            delayed: union_entry.delayed,
                                        })
                                })
                                .unwrap_or_else(|| AssociationEntryForm {
                                    key: union_entry.key.clone(),
                                    value: Expr::call(
                                        system_expr("Missing"),
                                        vec![Expr::string("KeyAbsent"), union_entry.key.clone()],
                                    ),
                                    delayed: union_entry.delayed,
                                })
                        })
                        .collect(),
                )
            })
            .collect(),
    )
}

fn collect_option_names_from_tail(session: &Session, tail: &[Expr]) -> Option<Vec<Symbol>> {
    let mut names = Vec::new();
    for expr in tail {
        collect_option_names_from_expr(session, expr, &mut names)?;
    }
    Some(names)
}

fn collect_option_names_from_expr(
    session: &Session,
    expr: &Expr,
    names: &mut Vec<Symbol>,
) -> Option<()> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            for item in args {
                collect_option_names_from_expr(session, item, names)?;
            }
            Some(())
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            names.push(session.normalize_symbol_target_expr(&args[0], true)?);
            Some(())
        }
        _ => None,
    }
}

fn exact_argument_count_message(symbol: &str, count: usize, expected: usize) -> String {
    let noun = if expected == 1 {
        "argument is"
    } else {
        "arguments are"
    };
    format!("{symbol} called with {count} arguments; {expected} {noun} expected.")
}

fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}

fn non_option_syntax_message(
    offender: &Expr,
    positional_count: usize,
    held_input: &Expr,
) -> String {
    format!(
        "Options expected (instead of {offender}) beyond position {positional_count} in {held_input}. An option must be a rule or a list of rules."
    )
}

fn unknown_option_message(option_name: &Symbol, held_input: &Expr) -> String {
    format!("Unknown option {option_name} in {held_input}.")
}
