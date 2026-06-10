use super::*;

impl Session {
    pub(super) fn eval_up_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("UpSet", args, 2));
        }

        let lhs = args[0].clone();
        let rhs_result = self.evaluate(args[1].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let lhs = lhs.clone();
            move |session, rhs| session.finish_up_set_assignment("UpSet", lhs, rhs, false)
        })?;

        self.finish_up_set_assignment("UpSet", lhs, rhs, false)
    }

    pub(super) fn eval_up_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("UpSetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();
        self.finish_up_set_assignment("UpSetDelayed", lhs, rhs, true)
    }

    pub(super) fn eval_tag_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSet", args, 3));
        }

        let tag = match self.normalize_tag_target(&args[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("TagSet", args, error.message()));
            }
        };

        let lhs = args[1].clone();
        let rhs_result = self.evaluate(args[2].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let tag = tag.clone();
            let lhs = lhs.clone();
            let args = args.to_vec();
            move |session, rhs| {
                session.store_tagged_definition("TagSet", &args, tag, lhs, rhs, false)
            }
        })?;
        self.store_tagged_definition("TagSet", args, tag, lhs, rhs, false)
    }

    pub(super) fn eval_tag_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSetDelayed", args, 3));
        }

        let tag = match self.normalize_tag_target(&args[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("TagSetDelayed", args, error.message()));
            }
        };

        let lhs = args[1].clone();
        let rhs = args[2].clone();
        self.store_tagged_definition("TagSetDelayed", args, tag, lhs, rhs, true)
    }

    pub(super) fn eval_tag_unset(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("TagUnset", args, 2));
        }

        let tag = match self.normalize_tag_target(&args[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("TagUnset", args, error.message()));
            }
        };

        if !self.ensure_symbol_not_protected_for_write("TagUnset", &tag) {
            return Ok(system_expr("Null"));
        }

        match self.unset_tagged_definition(&tag, &args[1]) {
            Ok(_) => {
                self.prune_definition_if_empty(&tag);
                Ok(system_expr("Null"))
            }
            Err(error) => Ok(self.invalid_tagged_definition_call("TagUnset", args, error)),
        }
    }

    pub(super) fn invalid_tagged_definition_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: TagDefinitionError,
    ) -> Expr {
        match error {
            TagDefinitionError::TagNotFound => self.invalid_call_with_message(
                symbol,
                args,
                "tagnf",
                "tag symbol does not occur in the assignment lhs",
            ),
            TagDefinitionError::TagPositionInvalid => self.invalid_call_with_message(
                symbol,
                args,
                "tagpos",
                "tag symbol must appear within the outer two levels of the assignment lhs",
            ),
        }
    }

    pub(super) fn store_tagged_definition(
        &mut self,
        symbol: &str,
        args: &[Expr],
        tag: Symbol,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        match classify_tagged_definition(&tag, &lhs) {
            Ok(target) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, target.symbol()) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                self.store_definition_target(target, lhs, rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            Err(error) => Ok(self.invalid_tagged_definition_call(symbol, args, error)),
        }
    }

    fn finish_up_set_assignment(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        let targets = collect_upset_target_symbols(&lhs);
        if targets.is_empty() {
            self.push_message(
                symbol,
                "nosym",
                format!("No valid tag symbol occurs in the assignment lhs {}.", lhs),
            );
            return Ok(if delayed { system_expr("Null") } else { rhs });
        }

        for target in targets {
            if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                continue;
            }

            self.store_definition_target(
                TaggedDefinitionTarget::UpValue(target),
                lhs.clone(),
                rhs.clone(),
                delayed,
            );
        }

        Ok(if delayed { system_expr("Null") } else { rhs })
    }

    pub(super) fn store_definition_target(
        &mut self,
        target: TaggedDefinitionTarget,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) {
        match target {
            TaggedDefinitionTarget::OwnValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();

                if delayed {
                    definition.own_value = None;
                    definition.delayed_own_value = Some(rhs.clone());
                    self.push_trace(TraceEvent::DefinitionStored {
                        symbol: symbol.to_string(),
                        kind: DefinitionKind::DelayedOwnValue,
                        value: rhs.to_string(),
                    });
                } else {
                    definition.own_value = Some(rhs.clone());
                    definition.delayed_own_value = None;
                    self.push_trace(TraceEvent::DefinitionStored {
                        symbol: symbol.to_string(),
                        kind: DefinitionKind::OwnValue,
                        value: rhs.to_string(),
                    });
                }
            }
            TaggedDefinitionTarget::DownValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                let down_value = DownValue {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    delayed,
                };
                if symbol.context() == SYSTEM_CONTEXT && symbol.name() == heads::patterns::DEFAULT {
                    replace_or_append_definition(
                        &mut definition.down_values,
                        &lhs,
                        down_value,
                        |entry| &entry.lhs,
                    );
                } else {
                    replace_or_insert_down_value(&mut definition.down_values, &lhs, down_value);
                }
                self.push_trace(TraceEvent::DefinitionStored {
                    symbol: symbol.to_string(),
                    kind: DefinitionKind::DownValue,
                    value: format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs),
                });
            }
            TaggedDefinitionTarget::SubValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                replace_or_insert_sub_value(
                    &mut definition.sub_values,
                    &lhs,
                    SubValue {
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                        delayed,
                    },
                );
                self.push_trace(TraceEvent::DefinitionStored {
                    symbol: symbol.to_string(),
                    kind: DefinitionKind::SubValue,
                    value: format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs),
                });
            }
            TaggedDefinitionTarget::UpValue(symbol) => {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                replace_or_insert_up_value(
                    &mut definition.up_values,
                    &lhs,
                    UpValue {
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                        delayed,
                    },
                );
                self.push_trace(TraceEvent::DefinitionStored {
                    symbol: symbol.to_string(),
                    kind: DefinitionKind::UpValue,
                    value: format!("{} {} {}", lhs, if delayed { ":=" } else { "=" }, rhs),
                });
            }
        }
    }
}

fn replace_or_insert_down_value(entries: &mut Vec<DownValue>, lhs: &Expr, new_entry: DownValue) {
    replace_or_insert_definition_by_specificity(entries, lhs, new_entry, |entry| &entry.lhs);
}

fn replace_or_insert_sub_value(entries: &mut Vec<SubValue>, lhs: &Expr, new_entry: SubValue) {
    replace_or_insert_definition_by_specificity(entries, lhs, new_entry, |entry| &entry.lhs);
}

fn replace_or_insert_up_value(entries: &mut Vec<UpValue>, lhs: &Expr, new_entry: UpValue) {
    replace_or_insert_definition_by_specificity(entries, lhs, new_entry, |entry| &entry.lhs);
}

fn replace_or_insert_definition_by_specificity<T, F>(
    entries: &mut Vec<T>,
    lhs: &Expr,
    new_entry: T,
    lhs_of: F,
) where
    F: Fn(&T) -> &Expr,
{
    if let Some(index) = entries
        .iter()
        .position(|entry| lhs_patterns_equivalent(lhs_of(entry), lhs))
    {
        entries[index] = new_entry;
        return;
    }

    let new_specificity = definition_specificity(lhs);
    let index = entries
        .iter()
        .position(|entry| new_specificity > definition_specificity(lhs_of(entry)))
        .unwrap_or(entries.len());
    entries.insert(index, new_entry);
}

fn definition_specificity(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 4,
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            definition_specificity(&args[1])
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Blank") => {
            if args.is_empty() {
                1
            } else {
                2
            }
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Optional")
                || is_system_symbol_name(head, "Repeated")
                || is_system_symbol_name(head, "RepeatedNull"))
                && !args.is_empty() =>
        {
            definition_specificity(&args[0])
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Condition")
                || is_system_symbol_name(head, "PatternTest"))
                && !args.is_empty() =>
        {
            1 + definition_specificity(&args[0])
        }
        Expr::Call { head, args } => {
            3 + definition_specificity(head)
                + args.iter().map(definition_specificity).sum::<usize>()
        }
    }
}

fn replace_or_append_definition<T, F>(entries: &mut Vec<T>, lhs: &Expr, new_entry: T, lhs_of: F)
where
    F: Fn(&T) -> &Expr,
{
    if let Some(index) = entries
        .iter()
        .position(|entry| lhs_patterns_equivalent(lhs_of(entry), lhs))
    {
        entries[index] = new_entry;
    } else {
        entries.push(new_entry);
    }
}
