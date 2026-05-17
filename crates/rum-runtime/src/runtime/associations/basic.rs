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

        let Some(association) = render_association_preserving_rule_heads(&source_items) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(association)
    }

    pub(in crate::runtime) fn eval_association_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_message(
                "AssociationQ",
                args,
                "argx",
                exact_argument_count_message("AssociationQ", args.len(), 1),
            ));
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
        if args.is_empty() || args.len() > 4 {
            return Ok(self.invalid_call_with_message(
                "Lookup",
                args,
                "argb",
                lookup_arg_count_message(args.len()),
            ));
        }

        if let [key] = args {
            return Ok(keyed_operator_form_slot_first("Lookup", key.clone()));
        }

        let [expr, key, rest @ ..] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let default = rest.first().cloned();
        let wrapper = rest.get(1);
        let Some(value) = lookup_value(expr, key, default, wrapper) else {
            if !lookup_data_is_valid(expr) {
                self.push_message(
                    "Lookup",
                    "invrl",
                    invalid_association_or_rule_list_message(expr),
                );
            }
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
            return Ok(self.invalid_call_with_message(
                "Key",
                args,
                "argx",
                exact_argument_count_message("Key", args.len(), 1),
            ));
        };

        Ok(Expr::call(head.clone(), vec![key.clone()]))
    }

    pub(in crate::runtime) fn eval_keys(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Keys",
                args,
                "argt",
                one_or_two_argument_count_message("Keys", args.len()),
            ));
        }

        let wrapper = args.get(1);
        match keys_result_for_expr(&args[0], wrapper) {
            Ok(value) => Ok(value),
            Err(invalid) => Ok(self.invalid_call_with_message(
                "Keys",
                args,
                "invrl",
                invalid_association_or_rule_list_message(&invalid),
            )),
        }
    }

    pub(in crate::runtime) fn eval_values(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Values",
                args,
                "argt",
                one_or_two_argument_count_message("Values", args.len()),
            ));
        };

        let wrapper = args.get(1);
        match values_result_for_expr(&args[0], wrapper) {
            Ok(value) => Ok(value),
            Err(invalid) => Ok(self.invalid_call_with_message(
                "Values",
                args,
                "invrl",
                invalid_association_or_rule_list_message(&invalid),
            )),
        }
    }

    pub(in crate::runtime) fn eval_key_exists_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, key) = match args {
            [key] => return Ok(keyed_operator_form_slot_first("KeyExistsQ", key.clone())),
            [expr, key] => (expr, key),
            _ => {
                return Ok(self.invalid_call_with_message(
                    "KeyExistsQ",
                    args,
                    "argt",
                    one_or_two_argument_count_message("KeyExistsQ", args.len()),
                ));
            }
        };

        let Some(entries) = single_rule_or_keyed_entries_from_expr(expr) else {
            self.push_message(
                "KeyExistsQ",
                "invrl",
                invalid_association_or_rule_list_message(expr),
            );
            return Ok(predicate_bool_expr(false));
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
            _ => Ok(self.invalid_call_with_message(
                "KeyTake",
                args,
                "argt",
                one_or_two_argument_count_message("KeyTake", args.len()),
            )),
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
            _ => Ok(self.invalid_call_with_message(
                "KeyDrop",
                args,
                "argt",
                one_or_two_argument_count_message("KeyDrop", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_key_member_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, key) = match args {
            [key] => return Ok(keyed_operator_form_slot_first("KeyMemberQ", key.clone())),
            [expr, key] => (expr, key),
            _ => {
                return Ok(self.invalid_call_with_message(
                    "KeyMemberQ",
                    args,
                    "argrx",
                    exact_argument_count_message("KeyMemberQ", args.len(), 2),
                ));
            }
        };

        let Some(entries) = keyed_entries_from_expr(expr).map(|keyed| keyed.entries) else {
            return Ok(self.invalid_call_with_message(
                "KeyMemberQ",
                args,
                "invas",
                invalid_association_or_rule_message(expr),
            ));
        };

        Ok(predicate_bool_expr(
            entries.iter().any(|(entry_key, _)| entry_key == key),
        ))
    }

    pub(in crate::runtime) fn eval_key_free_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, key) = match args {
            [key] => return Ok(keyed_operator_form_slot_first("KeyFreeQ", key.clone())),
            [expr, key] => (expr, key),
            _ => {
                return Ok(self.invalid_call_with_message(
                    "KeyFreeQ",
                    args,
                    "argrx",
                    exact_argument_count_message("KeyFreeQ", args.len(), 2),
                ));
            }
        };

        let Some(entries) = keyed_entries_from_expr(expr).map(|keyed| keyed.entries) else {
            return Ok(self.invalid_call_with_message(
                "KeyFreeQ",
                args,
                "invas",
                invalid_association_or_rule_message(expr),
            ));
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
        if args.is_empty() || args.len() > 3 {
            return Ok(self.invalid_call_with_message(
                "Normal",
                args,
                "argb",
                normal_arg_count_message(args.len()),
            ));
        }

        match args {
            [expr] => Ok(normalize_association_expr(expr)),
            [expr, target] => {
                if normal_target_matches_association(target) {
                    return Ok(normalize_association_expr(expr));
                }

                Ok(expr.clone())
            }
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(in crate::runtime) fn eval_key_value_pattern(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [spec] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
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
        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_delete_missing(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, mode) = match args {
            [expr] => (
                expr,
                DeleteMissingMode::Recursive(DeleteMissingDepth::Finite(1)),
            ),
            [expr, depth_expr] => {
                let Some(depth) = parse_delete_missing_positive_depth(depth_expr) else {
                    return Ok(self.invalid_call_with_message(
                        "DeleteMissing",
                        args,
                        "arg2",
                        delete_missing_arg2_message(depth_expr),
                    ));
                };
                (expr, DeleteMissingMode::Recursive(depth))
            }
            [expr, level_expr, scan_expr] => {
                let Some(level) = parse_delete_missing_positive_depth(level_expr) else {
                    return Ok(self.invalid_call_with_message(
                        "DeleteMissing",
                        args,
                        "arg2",
                        delete_missing_arg2_message(level_expr),
                    ));
                };
                let Some(scan) = parse_delete_missing_scan_depth(scan_expr) else {
                    return Ok(self.invalid_call_with_message(
                        "DeleteMissing",
                        args,
                        "arg3",
                        delete_missing_arg3_message(scan_expr),
                    ));
                };
                (expr, DeleteMissingMode::Target { level, scan })
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "argb",
                    delete_missing_arg_count_message(args.len()),
                ));
            }
        };

        let Some(value) = self.delete_missing_from_keyed_data(expr, mode)? else {
            return Ok(self.invalid_call_with_message(
                "DeleteMissing",
                args,
                "invrp",
                invalid_association_or_list_message(expr),
            ));
        };

        Ok(value)
    }
}

pub(in crate::runtime) fn is_association_expr(expr: &Expr) -> bool {
    association_entries_from_expr(expr).is_some()
}

pub(in crate::runtime) fn apply_association_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    match args {
        [] => Some(head.clone()),
        [key] => lookup_association_key(head, key),
        _ => apply_association_to_key_path(head, args),
    }
}

fn lookup_association_key(expr: &Expr, key: &Expr) -> Option<Expr> {
    let entries = association_entries_from_expr(expr)?;
    Some(
        entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
    )
}

fn apply_association_to_key_path(head: &Expr, keys: &[Expr]) -> Option<Expr> {
    let mut current = head.clone();
    for (index, key) in keys.iter().enumerate() {
        let Some(value) = lookup_association_key(&current, key) else {
            return None;
        };

        if is_missing_expr(&value) || index + 1 == keys.len() {
            return Some(value);
        }

        if !is_association_expr(&value) {
            return None;
        }
        current = value;
    }

    Some(current)
}

pub(in crate::runtime) fn missing_key_absent_expr(key: Expr) -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("KeyAbsent"), key])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeleteMissingMode {
    Recursive(DeleteMissingDepth),
    Target {
        level: DeleteMissingDepth,
        scan: DeleteMissingDepth,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeleteMissingDepth {
    Finite(usize),
    Infinity,
}

impl DeleteMissingDepth {
    fn descend(self) -> Option<Self> {
        match self {
            Self::Finite(depth) if depth > 1 => Some(Self::Finite(depth - 1)),
            Self::Finite(_) => None,
            Self::Infinity => Some(Self::Infinity),
        }
    }

    fn descend_scan(self) -> Option<Self> {
        match self {
            Self::Finite(depth) if depth > 0 => Some(Self::Finite(depth - 1)),
            Self::Finite(_) => None,
            Self::Infinity => Some(Self::Infinity),
        }
    }

    fn expr(self) -> Expr {
        match self {
            Self::Finite(depth) => Expr::integer(BigInt::from(depth)),
            Self::Infinity => system_expr("Infinity"),
        }
    }
}

fn parse_delete_missing_positive_depth(expr: &Expr) -> Option<DeleteMissingDepth> {
    if is_system_symbol_name(expr, "Infinity") || is_system_symbol_name(expr, "All") {
        return Some(DeleteMissingDepth::Infinity);
    }

    expr_as_non_negative_usize(expr)
        .filter(|depth| *depth > 0)
        .map(DeleteMissingDepth::Finite)
}

fn parse_delete_missing_scan_depth(expr: &Expr) -> Option<DeleteMissingDepth> {
    if is_system_symbol_name(expr, "Infinity") || is_system_symbol_name(expr, "All") {
        return Some(DeleteMissingDepth::Infinity);
    }

    expr_as_non_negative_usize(expr).map(DeleteMissingDepth::Finite)
}

fn delete_missing_arg2_message(expr: &Expr) -> String {
    format!("The second argument {expr} is expected to be a positive integer or Infinity.")
}

fn delete_missing_arg3_message(expr: &Expr) -> String {
    format!("The third argument {expr} is expected to be a non-negative integer or Infinity.")
}

fn delete_missing_arg_count_message(count: usize) -> String {
    format!("DeleteMissing called with {count} arguments; between 1 and 3 arguments are expected.")
}

fn lookup_arg_count_message(count: usize) -> String {
    format!("Lookup called with {count} arguments; between 2 and 4 arguments are expected.")
}

fn normal_arg_count_message(count: usize) -> String {
    format!("Normal called with {count} arguments; between 1 and 3 arguments are expected.")
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

fn normalize_association_expr(expr: &Expr) -> Expr {
    if let Some(entries) = association_rule_exprs_preserving_heads(expr) {
        return Expr::call(system_expr("List"), entries);
    }

    match expr {
        Expr::Call { head, args } => Expr::call(
            (**head).clone(),
            args.iter().map(normalize_association_expr).collect(),
        ),
        Expr::Atom(_) => expr.clone(),
    }
}

fn normal_target_matches_association(target: &Expr) -> bool {
    is_system_symbol_name(target, "Association")
        || is_system_symbol_name(target, "All")
        || matches!(target, Expr::Atom(Atom::Integer(value)) if value >= &BigInt::from(0))
}

impl Session {
    fn delete_missing_from_keyed_data(
        &mut self,
        expr: &Expr,
        mode: DeleteMissingMode,
    ) -> Result<Option<Expr>, EvalError> {
        match mode {
            DeleteMissingMode::Recursive(depth) => {
                Ok(delete_missing_from_keyed_data_recursive(expr, depth))
            }
            DeleteMissingMode::Target {
                level: DeleteMissingDepth::Infinity,
                ..
            } => Ok(delete_missing_from_keyed_data_recursive(
                expr,
                DeleteMissingDepth::Infinity,
            )),
            DeleteMissingMode::Target {
                level: DeleteMissingDepth::Finite(level),
                scan,
            } => self.delete_missing_at_target_level(expr, level, scan),
        }
    }

    fn delete_missing_at_target_level(
        &mut self,
        expr: &Expr,
        level: usize,
        scan: DeleteMissingDepth,
    ) -> Result<Option<Expr>, EvalError> {
        if let Some(items) = list_items(expr) {
            return self
                .resume_delete_missing_list_at_target_level(
                    items.to_vec(),
                    level,
                    scan,
                    0,
                    Vec::new(),
                )
                .map(Some);
        }

        let Some(entries) = association_entry_forms_from_expr(expr) else {
            return Ok(None);
        };
        self.resume_delete_missing_association_at_target_level(entries, level, scan, 0, Vec::new())
            .map(Some)
    }

    fn resume_delete_missing_list_at_target_level(
        &mut self,
        items: Vec<Expr>,
        level: usize,
        scan: DeleteMissingDepth,
        index: usize,
        mut kept: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return Ok(Expr::call(system_expr("List"), kept));
        };

        if level == 1 {
            if !rule_like_expr_has_missing_value(&item) && !contains_missing_within(&item, scan) {
                kept.push(item);
            }

            return self.resume_delete_missing_list_at_target_level(
                items,
                level,
                scan,
                index + 1,
                kept,
            );
        }

        if let Some(child_items) = list_items(&item) {
            let child_result = self.resume_delete_missing_list_at_target_level(
                child_items.to_vec(),
                level - 1,
                scan,
                0,
                Vec::new(),
            );
            let child_value = self.suspend_with_frame(child_result, {
                let items = items.clone();
                let kept = kept.clone();
                move |session, value| {
                    session.resume_delete_missing_list_after_child(
                        items, level, scan, index, kept, value,
                    )
                }
            })?;
            return self.resume_delete_missing_list_after_child(
                items,
                level,
                scan,
                index,
                kept,
                child_value,
            );
        }

        if let Some(child_entries) = association_entry_forms_from_expr(&item) {
            let child_result = self.resume_delete_missing_association_at_target_level(
                child_entries,
                level - 1,
                scan,
                0,
                Vec::new(),
            );
            let child_value = self.suspend_with_frame(child_result, {
                let items = items.clone();
                let kept = kept.clone();
                move |session, value| {
                    session.resume_delete_missing_list_after_child(
                        items, level, scan, index, kept, value,
                    )
                }
            })?;
            return self.resume_delete_missing_list_after_child(
                items,
                level,
                scan,
                index,
                kept,
                child_value,
            );
        }

        kept.push(item);
        self.resume_delete_missing_list_at_target_level(items, level, scan, index + 1, kept)
    }

    fn resume_delete_missing_list_after_child(
        &mut self,
        items: Vec<Expr>,
        level: usize,
        scan: DeleteMissingDepth,
        index: usize,
        mut kept: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        kept.push(value);
        self.resume_delete_missing_list_at_target_level(items, level, scan, index + 1, kept)
    }

    fn resume_delete_missing_association_at_target_level(
        &mut self,
        entries: Vec<AssociationEntryForm>,
        level: usize,
        scan: DeleteMissingDepth,
        index: usize,
        mut kept: Vec<AssociationEntryForm>,
    ) -> Result<Expr, EvalError> {
        let Some(mut entry) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(kept));
        };

        if is_missing_expr(&entry.value) {
            return self.resume_delete_missing_association_at_target_level(
                entries,
                level,
                scan,
                index + 1,
                kept,
            );
        }

        if level == 1 {
            let predicate_result = if entry.delayed {
                self.evaluate(entry.value.clone())
            } else {
                Ok(entry.value.clone())
            };
            let predicate_value = self.suspend_with_frame(predicate_result, {
                let entries = entries.clone();
                let entry = entry.clone();
                let kept = kept.clone();
                move |session, value| {
                    session.resume_delete_missing_association_after_predicate(
                        entries, level, scan, index, kept, entry, value,
                    )
                }
            })?;
            return self.resume_delete_missing_association_after_predicate(
                entries,
                level,
                scan,
                index,
                kept,
                entry,
                predicate_value,
            );
        }

        if entry.delayed {
            entry.value = delete_missing_delayed_wrapper(
                entry.value,
                DeleteMissingMode::Target {
                    level: DeleteMissingDepth::Finite(level - 1),
                    scan,
                },
            );
            kept.push(entry);
            return self.resume_delete_missing_association_at_target_level(
                entries,
                level,
                scan,
                index + 1,
                kept,
            );
        }

        if let Some(child_items) = list_items(&entry.value) {
            let child_result = self.resume_delete_missing_list_at_target_level(
                child_items.to_vec(),
                level - 1,
                scan,
                0,
                Vec::new(),
            );
            let child_value = self.suspend_with_frame(child_result, {
                let entries = entries.clone();
                let entry = entry.clone();
                let kept = kept.clone();
                move |session, value| {
                    session.resume_delete_missing_association_after_child(
                        entries, level, scan, index, kept, entry, value,
                    )
                }
            })?;
            return self.resume_delete_missing_association_after_child(
                entries,
                level,
                scan,
                index,
                kept,
                entry,
                child_value,
            );
        }

        if let Some(child_entries) = association_entry_forms_from_expr(&entry.value) {
            let child_result = self.resume_delete_missing_association_at_target_level(
                child_entries,
                level - 1,
                scan,
                0,
                Vec::new(),
            );
            let child_value = self.suspend_with_frame(child_result, {
                let entries = entries.clone();
                let entry = entry.clone();
                let kept = kept.clone();
                move |session, value| {
                    session.resume_delete_missing_association_after_child(
                        entries, level, scan, index, kept, entry, value,
                    )
                }
            })?;
            return self.resume_delete_missing_association_after_child(
                entries,
                level,
                scan,
                index,
                kept,
                entry,
                child_value,
            );
        }

        kept.push(entry);
        self.resume_delete_missing_association_at_target_level(
            entries,
            level,
            scan,
            index + 1,
            kept,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_delete_missing_association_after_predicate(
        &mut self,
        entries: Vec<AssociationEntryForm>,
        level: usize,
        scan: DeleteMissingDepth,
        index: usize,
        mut kept: Vec<AssociationEntryForm>,
        entry: AssociationEntryForm,
        predicate_value: Expr,
    ) -> Result<Expr, EvalError> {
        if !is_missing_expr(&predicate_value) && !contains_missing_within(&predicate_value, scan) {
            kept.push(entry);
        }

        self.resume_delete_missing_association_at_target_level(
            entries,
            level,
            scan,
            index + 1,
            kept,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_delete_missing_association_after_child(
        &mut self,
        entries: Vec<AssociationEntryForm>,
        level: usize,
        scan: DeleteMissingDepth,
        index: usize,
        mut kept: Vec<AssociationEntryForm>,
        mut entry: AssociationEntryForm,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        entry.value = value;
        kept.push(entry);
        self.resume_delete_missing_association_at_target_level(
            entries,
            level,
            scan,
            index + 1,
            kept,
        )
    }
}

fn delete_missing_from_keyed_data_recursive(
    expr: &Expr,
    depth: DeleteMissingDepth,
) -> Option<Expr> {
    if let Some(items) = list_items(expr) {
        return Some(delete_missing_from_list_recursive(items, depth));
    }

    let entries = association_entry_forms_from_expr(expr)?;
    Some(render_association_entry_forms_preserving_heads(
        delete_missing_from_association_entries_recursive(entries, depth),
    ))
}

fn delete_missing_from_list_recursive(items: &[Expr], depth: DeleteMissingDepth) -> Expr {
    Expr::call(
        system_expr("List"),
        items
            .iter()
            .filter_map(|item| delete_missing_list_item_recursive(item, depth))
            .collect(),
    )
}

fn delete_missing_list_item_recursive(item: &Expr, depth: DeleteMissingDepth) -> Option<Expr> {
    if is_missing_expr(item) || rule_like_expr_has_missing_value(item) {
        return None;
    }

    if let Some(child_depth) = depth.descend() {
        if list_items(item).is_some() || association_entry_forms_from_expr(item).is_some() {
            return delete_missing_from_keyed_data_recursive(item, child_depth);
        }
    }

    Some(item.clone())
}

fn delete_missing_from_association_entries_recursive(
    entries: Vec<AssociationEntryForm>,
    depth: DeleteMissingDepth,
) -> Vec<AssociationEntryForm> {
    entries
        .into_iter()
        .filter_map(|mut entry| {
            if is_missing_expr(&entry.value) {
                return None;
            }

            if !entry.delayed {
                if let Some(child_depth) = depth.descend() {
                    if let Some(value) =
                        delete_missing_from_keyed_data_recursive(&entry.value, child_depth)
                    {
                        entry.value = value;
                    }
                }
            } else if let Some(child_depth) = depth.descend() {
                entry.value = delete_missing_delayed_wrapper(
                    entry.value,
                    DeleteMissingMode::Recursive(child_depth),
                );
            }

            Some(entry)
        })
        .collect()
}

fn delete_missing_delayed_wrapper(value: Expr, mode: DeleteMissingMode) -> Expr {
    let local = system_expr("rum$deleteMissingValue");
    let cleanup = Expr::call(
        system_expr("DeleteMissing"),
        delete_missing_call_args(local.clone(), mode),
    );
    let should_cleanup = Expr::call(
        system_expr("Or"),
        vec![
            Expr::call(system_expr("ListQ"), vec![local.clone()]),
            Expr::call(system_expr("AssociationQ"), vec![local.clone()]),
        ],
    );

    Expr::call(
        system_expr("With"),
        vec![
            Expr::call(
                system_expr("List"),
                vec![Expr::call(system_expr("Rule"), vec![local.clone(), value])],
            ),
            Expr::call(system_expr("If"), vec![should_cleanup, cleanup, local]),
        ],
    )
}

fn delete_missing_call_args(expr: Expr, mode: DeleteMissingMode) -> Vec<Expr> {
    match mode {
        DeleteMissingMode::Recursive(depth) => vec![expr, depth.expr()],
        DeleteMissingMode::Target { level, scan } => vec![expr, level.expr(), scan.expr()],
    }
}

fn contains_missing_within(expr: &Expr, depth: DeleteMissingDepth) -> bool {
    if is_missing_expr(expr) {
        return true;
    }

    let Some(child_depth) = depth.descend_scan() else {
        return false;
    };

    let Expr::Call { args, .. } = expr else {
        return false;
    };

    args.iter()
        .any(|arg| contains_missing_within(arg, child_depth))
}

pub(in crate::runtime) fn is_missing_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Missing"))
}

fn rule_like_expr_has_missing_value(expr: &Expr) -> bool {
    parse_association_entry_form(expr).is_some_and(|entry| is_missing_expr(&entry.value))
}

fn lookup_value(
    data: &Expr,
    key_spec: &Expr,
    default: Option<Expr>,
    wrapper: Option<&Expr>,
) -> Option<Expr> {
    let direct_entries = single_rule_or_keyed_entries_from_expr(data);
    if direct_entries.is_none() {
        if let Some(items) = call_items_with_head(data, &system_expr("List")) {
            if items
                .iter()
                .all(|item| keyed_entries_from_expr(item).is_some())
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
    }

    if let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) {
        return Some(Expr::call(
            system_expr("List"),
            keys.iter()
                .map(|key| lookup_value(data, key, default.clone(), wrapper))
                .collect::<Option<Vec<_>>>()?,
        ));
    }

    if let Some(entries) = direct_entries.as_ref() {
        return Some(lookup_entries_value(entries, key_spec, default, wrapper));
    }

    None
}

fn wrap_keyed_result(value: Expr, wrapper: Option<&Expr>) -> Expr {
    match wrapper.cloned() {
        Some(head) => Expr::call(head, vec![value]),
        None => value,
    }
}

fn lookup_entries_value(
    entries: &[(Expr, Expr)],
    key_spec: &Expr,
    default: Option<Expr>,
    wrapper: Option<&Expr>,
) -> Expr {
    let lookup_key = lookup_key_expr(key_spec);
    if let Some((_, value)) = entries
        .iter()
        .find(|(entry_key, _)| entry_key == lookup_key)
    {
        return wrap_keyed_result(value.clone(), wrapper);
    }

    default.unwrap_or_else(|| missing_key_absent_expr(lookup_key.clone()))
}

fn lookup_key_expr(key_spec: &Expr) -> &Expr {
    match key_spec {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            &args[0]
        }
        _ => key_spec,
    }
}

fn lookup_data_is_valid(expr: &Expr) -> bool {
    if single_rule_or_keyed_entries_from_expr(expr).is_some() {
        return true;
    }

    let Some(items) = call_items_with_head(expr, &system_expr("List")) else {
        return false;
    };

    items
        .iter()
        .all(|item| keyed_entries_from_expr(item).is_some())
}

fn keys_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    if let Some((key, _)) = parse_association_entry(expr) {
        return Ok(wrap_keyed_result(key, wrapper));
    }

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        return Ok(Expr::call(
            system_expr("List"),
            keyed
                .entries
                .into_iter()
                .map(|(key, _)| wrap_keyed_result(key, wrapper))
                .collect(),
        ));
    }

    let Some(items) = list_items(expr) else {
        return Err(expr.clone());
    };

    Ok(Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|item| keys_result_for_expr(item, wrapper))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn values_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    if let Some((_, value)) = parse_association_entry(expr) {
        return Ok(wrap_keyed_result(value, wrapper));
    }

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        return Ok(Expr::call(
            system_expr("List"),
            keyed
                .entries
                .into_iter()
                .map(|(_, value)| wrap_keyed_result(value, wrapper))
                .collect(),
        ));
    }

    let Some(items) = list_items(expr) else {
        return Err(expr.clone());
    };

    let mut values = Vec::with_capacity(items.len());
    for item in items {
        match values_result_for_expr(item, wrapper) {
            Ok(value) => values.push(value),
            Err(_) => return Err(expr.clone()),
        }
    }

    Ok(Expr::call(system_expr("List"), values))
}

fn keyed_take_drop_result_for_expr(
    symbol: &'static str,
    expr: &Expr,
    key_spec: &Expr,
    keep_selected: bool,
) -> Option<Expr> {
    let selected_keys = parse_key_spec(key_spec)?;

    if let Some(entries) = single_rule_or_keyed_entry_forms_from_expr(expr) {
        let filtered = if keep_selected {
            let mut taken = Vec::new();
            for selected_key in &selected_keys {
                taken.extend(
                    entries
                        .iter()
                        .filter(|entry| &entry.key == selected_key)
                        .cloned(),
                );
            }
            taken
        } else {
            entries
                .into_iter()
                .filter(|entry| {
                    !selected_keys
                        .iter()
                        .any(|candidate| candidate == &entry.key)
                })
                .collect()
        };

        return Some(render_association_entry_forms_preserving_heads(filtered));
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
