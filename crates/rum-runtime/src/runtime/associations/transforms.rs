use super::*;

impl Session {
    pub(in crate::runtime) fn eval_key_map(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_second("KeyMap", function.clone())),
            [function, expr] => {
                let Some(entries) = association_entries_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeyMap",
                        args,
                        "invak",
                        invalid_association_message(expr),
                    ));
                };

                Ok(render_association(
                    entries
                        .into_iter()
                        .map(|(key, value)| (Expr::call(function.clone(), vec![key]), value))
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
                let Some(mut keyed) = keyed_entries_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeySort",
                        args,
                        "invrl",
                        invalid_association_or_rule_list_message(expr),
                    ));
                };

                keyed
                    .entries
                    .sort_by(|lhs, rhs| compare_expr_canonical(&lhs.0, &rhs.0));
                Ok(render_association(keyed.entries))
            }
            [expr, ordering] => {
                let Some(keyed) = keyed_entries_from_expr(expr) else {
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
                    keyed.entries,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("KeySort", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_association_thread(
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

    pub(in crate::runtime) fn eval_key_union(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [collections_expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(collections) = parse_keyed_entries_list_expr(collections_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut union_keys = Vec::new();
        for entries in &collections {
            for (key, _) in entries {
                if !union_keys.iter().any(|existing| existing == key) {
                    union_keys.push(key.clone());
                }
            }
        }

        Ok(Expr::call(
            system_expr("List"),
            collections
                .into_iter()
                .map(|entries| {
                    render_association(
                        union_keys
                            .iter()
                            .cloned()
                            .map(|key| {
                                let value = entries
                                    .iter()
                                    .find(|(entry_key, _)| entry_key == &key)
                                    .map(|(_, value)| value.clone())
                                    .unwrap_or_else(|| {
                                        Expr::call(
                                            system_expr("Missing"),
                                            vec![Expr::string("KeyAbsent"), key.clone()],
                                        )
                                    });
                                (key, value)
                            })
                            .collect(),
                    )
                })
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_intersection(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [collections_expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(collections) = parse_keyed_entries_list_expr(collections_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(first) = collections.first() else {
            return Ok(Expr::call(system_expr("List"), vec![]));
        };

        let intersection_keys = first
            .iter()
            .filter(|(key, _)| {
                collections[1..]
                    .iter()
                    .all(|entries| entries.iter().any(|(entry_key, _)| entry_key == key))
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();

        Ok(Expr::call(
            system_expr("List"),
            collections
                .into_iter()
                .map(|entries| {
                    render_association(
                        intersection_keys
                            .iter()
                            .filter_map(|key| {
                                entries
                                    .iter()
                                    .find(|(entry_key, _)| entry_key == key)
                                    .map(|(_, value)| (key.clone(), value.clone()))
                            })
                            .collect(),
                    )
                })
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_complement(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [collections_expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(collections) = parse_keyed_entries_list_expr(collections_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some((first, rest)) = collections.split_first() else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let excluded = rest
            .iter()
            .flat_map(|entries| entries.iter().map(|(key, _)| key.clone()))
            .collect::<std::collections::HashSet<_>>();

        Ok(render_association(
            first
                .iter()
                .filter(|(key, _)| !excluded.contains(key))
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

        if parse_association_update_spec(spec).is_none() {
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
        let mut entries = association_entries_from_expr(&expr)?;
        let updates = parse_association_update_spec(&spec)?;

        for (key, value) in updates {
            entries.retain(|(existing, _)| existing != &key);
            entries.push((key, value));
        }

        Some(render_association(entries))
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
        let entries = association_entries_from_expr(&expr)?;
        let selected_keys = parse_key_spec(&key_spec)?;

        Some(render_association(
            entries
                .into_iter()
                .filter(|(key, _)| !selected_keys.iter().any(|candidate| candidate == key))
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

    pub(in crate::runtime) fn eval_catenate(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [associations_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Catenate", args, 1));
        };

        let Some(associations) = parse_keyed_entries_list_expr(associations_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            associations.into_iter().flatten().collect(),
        ))
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
