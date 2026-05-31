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

impl Session {
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
    let count_noun = if count == 1 { "argument" } else { "arguments" };
    let expected_noun = if expected == 1 {
        "argument is"
    } else {
        "arguments are"
    };
    format!("{symbol} called with {count} {count_noun}; {expected} {expected_noun} expected.")
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
