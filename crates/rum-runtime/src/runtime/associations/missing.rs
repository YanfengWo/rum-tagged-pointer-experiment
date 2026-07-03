use super::*;

impl Session {
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

pub fn is_missing_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Missing"))
}

fn rule_like_expr_has_missing_value(expr: &Expr) -> bool {
    parse_association_entry_form(expr).is_some_and(|entry| is_missing_expr(&entry.value))
}
