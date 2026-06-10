use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeldAssociationPartMutationResult {
    UpdatedTarget,
    UpdatedPart,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactPartSelectorFailure {
    PartW { part: Expr, target: Expr },
    PartD { part: Expr },
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

fn failed_exact_part_selector_path(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<ExactPartSelectorFailure> {
    let mut selected = expr.clone();
    for selector in selector_exprs {
        if let Some(index) = expr_as_signed_isize(selector) {
            if let Some(next) = exact_expr_at_path(&selected, &[index]) {
                selected = next;
                continue;
            }

            if index == 0 {
                return None;
            }

            return Some(match selected {
                Expr::Atom(_) => ExactPartSelectorFailure::PartD {
                    part: Expr::call(system_expr("Part"), vec![selected, selector.clone()]),
                },
                Expr::Call { .. } => ExactPartSelectorFailure::PartW {
                    part: selector.clone(),
                    target: selected,
                },
            });
        }

        let key = part_selector_key_expr(selector)?;
        let entries = association_entry_forms_from_expr(&selected)?;
        if let Some(entry) = entries.into_iter().find(|entry| entry.key == key) {
            selected = entry.value;
        } else {
            return None;
        }
    }

    None
}

fn partw_message(part: &Expr, target: &Expr) -> String {
    format!("Part {part} of {target} does not exist.")
}

fn partd_message(part: &Expr) -> String {
    format!("Part specification {part} is longer than depth of object.")
}

fn mutation_rvalue_message(target: &Expr) -> String {
    format!("{target} is not a variable with a value, so its value cannot be changed.")
}

fn is_non_symbol_atom(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(atom) if !matches!(atom, Atom::Symbol(_)))
}

impl Session {
    pub(in crate::runtime) fn eval_associate_to(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [target_expr, spec] = args else {
            return Ok(self.invalid_call_with_message(
                "AssociateTo",
                args,
                two_argument_mutation_arity_tag(args.len()),
                exact_argument_count_message("AssociateTo", args.len(), 2),
            ));
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

        if is_non_symbol_atom(target_expr) {
            return Ok(self.invalid_call_with_message(
                "AssociateTo",
                args,
                "rvalue",
                mutation_rvalue_message(target_expr),
            ));
        }

        if let Some(target_symbol) = self.protected_definition_target_symbol(target_expr) {
            self.push_protected_write_message("AssociateTo", &target_symbol);
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let target_result = self.evaluate(target_expr.clone());
        let expr = self.suspend_with_frame(target_result, {
            let call_args = args.to_vec();
            let target_expr = target_expr.clone();
            let spec = spec.clone();
            move |session, expr| {
                session.finish_associate_to_value(call_args, target_expr, expr, spec)
            }
        })?;

        self.finish_associate_to_value(args.to_vec(), target_expr.clone(), expr, spec.clone())
    }

    fn finish_associate_to_value(
        &mut self,
        call_args: Vec<Expr>,
        target_expr: Expr,
        expr: Expr,
        spec: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(updated) = Self::apply_associate_to_value(expr, spec) else {
            return Ok(self.invalid_call_with_message(
                "AssociateTo",
                &call_args,
                "invak",
                invalid_association_message(&target_expr),
            ));
        };

        self.store_association_lvalue_update(&target_expr, updated.clone());
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
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [target_expr, key_spec] = args else {
            return Ok(self.invalid_call_with_message(
                "KeyDropFrom",
                args,
                two_argument_mutation_arity_tag(args.len()),
                exact_argument_count_message("KeyDropFrom", args.len(), 2),
            ));
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

        if rule_list_entries_from_expr(target_expr).is_some()
            || parse_association_entry_form(target_expr).is_some()
        {
            return Ok(self.invalid_call_with_message(
                "KeyDropFrom",
                args,
                "invak",
                invalid_association_message(target_expr),
            ));
        }

        if is_non_symbol_atom(target_expr) {
            return Ok(self.invalid_call_with_message(
                "KeyDropFrom",
                args,
                "rvalue",
                mutation_rvalue_message(target_expr),
            ));
        }

        let target_result = self.evaluate(target_expr.clone());
        let expr = self.suspend_with_frame(target_result, {
            let call_args = args.to_vec();
            let target_expr = target_expr.clone();
            let key_spec = key_spec.clone();
            move |session, expr| {
                session.finish_key_drop_from_value(call_args, target_expr, expr, key_spec)
            }
        })?;

        self.finish_key_drop_from_value(args.to_vec(), target_expr.clone(), expr, key_spec.clone())
    }

    fn finish_key_drop_from_value(
        &mut self,
        call_args: Vec<Expr>,
        target_expr: Expr,
        expr: Expr,
        key_spec: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(updated) = Self::apply_key_drop_from_value(expr, key_spec) else {
            return Ok(self.invalid_call_with_message(
                "KeyDropFrom",
                &call_args,
                "invak",
                invalid_association_message(&target_expr),
            ));
        };

        if let Some(message) = self.protected_definition_target_write_message(&target_expr) {
            self.push_message("Set", "write", message);
        } else {
            self.store_association_lvalue_update(&target_expr, updated.clone());
        }
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

    fn protected_definition_target_symbol(&self, target_expr: &Expr) -> Option<Symbol> {
        let Some(DefinitionAssignmentTarget::Definition(target)) =
            self.normalize_definition_assignment_target(target_expr)
        else {
            return None;
        };

        let symbol = target.symbol().clone();
        (self.symbol_is_protected(&symbol) || is_known_system_symbol(&symbol)).then_some(symbol)
    }

    fn protected_definition_target_write_message(&self, target_expr: &Expr) -> Option<String> {
        self.protected_definition_target_symbol(target_expr)?;
        protected_tag_write_message(target_expr)
    }

    fn store_association_lvalue_update(&mut self, target_expr: &Expr, updated: Expr) {
        if let Some(DefinitionAssignmentTarget::Definition(target)) =
            self.normalize_definition_assignment_target(target_expr)
        {
            self.store_definition_target(target, target_expr.clone(), updated, false);
        }
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
        self.bump_evaluation_epoch();

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
            let part_failure = failed_exact_part_selector_path(&current, &selector_exprs);
            if operator == "AssociateTo" {
                if let Some((key, target)) =
                    missing_key_in_exact_part_selector_path(&current, &selector_exprs)
                {
                    self.push_message(
                        "AssociateTo",
                        "keynx",
                        format!("Key {key} does not exist in {target}."),
                    );
                } else if let Some(ExactPartSelectorFailure::PartW { part, target }) = part_failure
                {
                    self.push_message("AssociateTo", "partw", partw_message(&part, &target));
                }
                return Ok(Some(Expr::call(system_expr(operator), held_call)));
            }

            if operator == "KeyDropFrom" {
                match part_failure {
                    Some(ExactPartSelectorFailure::PartW { part, target }) => {
                        self.push_message("Part", "partw", partw_message(&part, &target));
                    }
                    Some(ExactPartSelectorFailure::PartD { part }) => {
                        self.push_message("Part", "partd", partd_message(&part));
                    }
                    None => {}
                }
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
        self.bump_evaluation_epoch();

        let value = match result {
            HeldAssociationPartMutationResult::UpdatedTarget => updated_target,
            HeldAssociationPartMutationResult::UpdatedPart => selected_result,
        };
        Ok(Some(value))
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

fn two_argument_mutation_arity_tag(count: usize) -> &'static str {
    if count == 1 { "argr" } else { "argrx" }
}
