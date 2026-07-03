use super::*;

mod key_predicates;

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
            _ => args.to_vec().into(),
        };

        let Some(entries) = parse_association_entry_forms_deep(&source_items) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.resume_association_key_evaluation(entries, Vec::new(), 0)
    }

    fn resume_association_key_evaluation(
        &mut self,
        entries: Vec<AssociationEntryForm>,
        mut evaluated_entries: Vec<AssociationEntryForm>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= entries.len() {
            return Ok(render_association_entry_forms_preserving_heads(
                evaluated_entries,
            ));
        }

        let entry = entries[index].clone();
        let key_result = self.evaluate(entry.key.clone());
        let key = self.suspend_with_frame_lazy(key_result, || {
            let entries = entries.clone();
            let evaluated_entries = evaluated_entries.clone();
            let entry_for_resume = entry.clone();
            move |session, key| {
                let mut entry = entry_for_resume.clone();
                entry.key = key;
                let mut evaluated_entries = evaluated_entries;
                evaluated_entries.push(entry);
                session.resume_association_key_evaluation(entries, evaluated_entries, index + 1)
            }
        })?;

        let mut entry = entry;
        entry.key = key;
        evaluated_entries.push(entry);
        self.resume_association_key_evaluation(entries, evaluated_entries, index + 1)
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

        let expr_result = self.evaluate(expr.clone());
        let expr = self.suspend_with_frame(expr_result, {
            let head = head.clone();
            let key = key.clone();
            let rest = rest.to_vec();
            move |session, expr| session.eval_lookup_with_evaluated_data(head, expr, key, rest)
        })?;

        self.eval_lookup_with_evaluated_data(head.clone(), expr, key.clone(), rest.to_vec())
    }

    fn eval_lookup_with_evaluated_data(
        &mut self,
        head: Expr,
        expr: Expr,
        key: Expr,
        rest: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let key_result = self.evaluate(key);
        let key = self.suspend_with_frame(key_result, {
            let head = head.clone();
            let expr = expr.clone();
            let rest = rest.clone();
            move |session, key| session.finish_eval_lookup(head, expr, key, rest)
        })?;

        self.finish_eval_lookup(head, expr, key, rest)
    }

    fn finish_eval_lookup(
        &mut self,
        head: Expr,
        expr: Expr,
        key: Expr,
        rest: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let default = rest.first().cloned();
        let wrapper = rest.get(1).cloned();
        let Some(plan) = lookup_plan(&expr, &key) else {
            if !lookup_data_is_valid(&expr) {
                self.push_message(
                    "Lookup",
                    "invrl",
                    invalid_association_or_rule_list_message(&expr),
                );
            }
            let mut args = vec![expr, key];
            args.extend(rest);
            return Ok(Expr::call(head, args));
        };

        let default = if plan.has_missing {
            if let Some(default) = default {
                let default_result = self.evaluate(default);
                Some(self.suspend_with_frame(default_result, {
                    let head = head.clone();
                    let expr = expr.clone();
                    let key = key.clone();
                    let wrapper = wrapper.clone();
                    move |session, default| {
                        session.finish_lookup_after_delayed_values(
                            head,
                            expr,
                            key,
                            Some(default),
                            wrapper,
                        )
                    }
                })?)
            } else {
                None
            }
        } else {
            None
        };

        self.finish_lookup_after_delayed_values(head, expr, key, default, wrapper)
    }

    fn finish_lookup_after_delayed_values(
        &mut self,
        head: Expr,
        expr: Expr,
        key: Expr,
        default: Option<Expr>,
        wrapper: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(plan) = lookup_plan(&expr, &key) else {
            return Ok(Expr::call(head, vec![expr, key]));
        };
        let wrapper = if plan.has_hit {
            if let Some(wrapper) = wrapper {
                let wrapper_result = self.evaluate(wrapper);
                Some(self.suspend_with_frame(wrapper_result, {
                    let expr = expr.clone();
                    let key = key.clone();
                    let default = default.clone();
                    move |_, wrapper| {
                        Ok(lookup_value(&expr, &key, default.clone(), Some(&wrapper))
                            .expect("lookup plan should stay valid for unchanged inputs"))
                    }
                })?)
            } else {
                None
            }
        } else {
            None
        };

        let value = lookup_value(&expr, &key, default, wrapper.as_ref())
            .expect("lookup plan should stay valid for unchanged inputs");
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
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [key_spec] => Ok(keyed_operator_form_slot_first("KeyTake", key_spec.clone())),
            [expr, key_spec] => {
                match keyed_take_drop_result_for_expr("KeyTake", expr, key_spec, true) {
                    Ok(value) => Ok(value),
                    Err(error) => Ok(self.invalid_call_with_message(
                        "KeyTake",
                        args,
                        error.tag,
                        invalid_association_or_rule_list_message(&error.invalid),
                    )),
                }
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
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [key_spec] => Ok(keyed_operator_form_slot_first("KeyDrop", key_spec.clone())),
            [expr, key_spec] => {
                match keyed_take_drop_result_for_expr("KeyDrop", expr, key_spec, false) {
                    Ok(value) => Ok(value),
                    Err(error) => Ok(self.invalid_call_with_message(
                        "KeyDrop",
                        args,
                        error.tag,
                        invalid_association_or_rule_list_message(&error.invalid),
                    )),
                }
            }
            _ => Ok(self.invalid_call_with_message(
                "KeyDrop",
                args,
                "argt",
                one_or_two_argument_count_message("KeyDrop", args.len()),
            )),
        }
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
}

pub(in crate::runtime) fn is_association_expr(expr: &Expr) -> bool {
    association_entries_from_expr(expr).is_some()
}

pub(in crate::runtime) fn apply_association_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    match args {
        [] => association_entries_from_expr(head).map(|_| head.clone()),
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

pub fn missing_key_absent_expr(key: Expr) -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("KeyAbsent"), key])
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
        || matches!(target, Expr::Atom(Atom::Integer(value)) if value.to_bigint() >= BigInt::from(0))
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LookupPlan {
    has_hit: bool,
    has_missing: bool,
}

impl LookupPlan {
    fn combine(self, other: Self) -> Self {
        Self {
            has_hit: self.has_hit || other.has_hit,
            has_missing: self.has_missing || other.has_missing,
        }
    }
}

fn lookup_plan(data: &Expr, key_spec: &Expr) -> Option<LookupPlan> {
    let direct_entries = single_rule_or_keyed_entries_from_expr(data);
    if direct_entries.is_none() {
        if let Some(items) = call_items_with_head(data, &system_expr("List")) {
            if items
                .iter()
                .all(|item| keyed_entries_from_expr(item).is_some())
            {
                return items
                    .iter()
                    .map(|item| lookup_plan(item, key_spec))
                    .collect::<Option<Vec<_>>>()
                    .map(|plans| {
                        plans
                            .into_iter()
                            .fold(LookupPlan::default(), LookupPlan::combine)
                    });
            }
        }
    }

    if let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) {
        return keys
            .iter()
            .map(|key| lookup_plan(data, key))
            .collect::<Option<Vec<_>>>()
            .map(|plans| {
                plans
                    .into_iter()
                    .fold(LookupPlan::default(), LookupPlan::combine)
            });
    }

    direct_entries
        .as_ref()
        .map(|entries| lookup_entries_plan(entries, key_spec))
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

fn lookup_entries_plan(entries: &[(Expr, Expr)], key_spec: &Expr) -> LookupPlan {
    let lookup_key = lookup_key_expr(key_spec);
    if entries.iter().any(|(entry_key, _)| entry_key == lookup_key) {
        LookupPlan {
            has_hit: true,
            has_missing: false,
        }
    } else {
        LookupPlan {
            has_hit: false,
            has_missing: true,
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyTakeDropInvalid {
    tag: &'static str,
    invalid: Expr,
}

fn keyed_take_drop_result_for_expr(
    symbol: &'static str,
    expr: &Expr,
    key_spec: &Expr,
    keep_selected: bool,
) -> Result<Expr, KeyTakeDropInvalid> {
    let selected_keys = parse_key_spec(key_spec).ok_or_else(|| KeyTakeDropInvalid {
        tag: "invrl",
        invalid: expr.clone(),
    })?;

    if list_items(expr).is_some_and(|items| items.is_empty()) {
        return Ok(Expr::call(system_expr(heads::structure::LIST), vec![]));
    }

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

        return Ok(render_association_entry_forms_preserving_heads(filtered));
    }

    let Some(items) = list_items(expr) else {
        return Err(KeyTakeDropInvalid {
            tag: "invrl",
            invalid: expr.clone(),
        });
    };

    let mut results = Vec::with_capacity(items.len());
    for item in items {
        match keyed_take_drop_result_for_expr(symbol, item, key_spec, keep_selected) {
            Ok(value) => results.push(value),
            Err(error) => {
                return Err(KeyTakeDropInvalid {
                    tag: "invru",
                    invalid: if keep_selected {
                        expr.clone()
                    } else {
                        error.invalid
                    },
                });
            }
        }
    }

    Ok(Expr::call(system_expr("List"), results))
}
