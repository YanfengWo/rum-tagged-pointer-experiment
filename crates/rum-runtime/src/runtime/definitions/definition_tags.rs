use super::super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_up_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("UpSet", args, 2));
        }

        let lhs = args[0].clone();
        let rhs_result = self.evaluate(args[1].clone());
        let rhs = crate::resume_or_suspend!(
            self,
            rhs_result,
            ok(value) => Ok(value),
            suspend => {
        let lhs = lhs.clone();
        EvalUpSetSuspensionFrame { lhs }
        },
        )?;

        self.finish_up_set_assignment("UpSet", lhs, rhs, false)
    }

    pub(in crate::runtime) fn eval_up_set_delayed(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("UpSetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();
        self.finish_up_set_assignment("UpSetDelayed", lhs, rhs, true)
    }

    pub(in crate::runtime) fn eval_tag_set(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSet", args_slice, 3));
        }

        let tag = match self.normalize_tag_target(&args_slice[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("TagSet", args_slice, error.message()));
            }
        };

        let lhs = args_slice[1].clone();
        let rhs_result = self.evaluate(args_slice[2].clone());
        let rhs = crate::resume_or_suspend!(
            self,
            rhs_result,
            ok(value) => Ok(value),
            suspend => {
        let tag = tag.clone();
                    let lhs = lhs.clone();
                    let args = args.clone();
        EvalTagSetSuspensionFrame { args, tag, lhs }
        },
        )?;
        self.store_tagged_definition("TagSet", args_slice, tag, lhs, rhs, false)
    }

    pub(in crate::runtime) fn eval_tag_set_delayed(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_tag_unset(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn invalid_tagged_definition_call(
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

    pub(in crate::runtime) fn store_tagged_definition(
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

    pub(in crate::runtime) fn store_definition_target(
        &mut self,
        target: TaggedDefinitionTarget,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) {
        let symbol = match target {
            TaggedDefinitionTarget::OwnValue(symbol) => {
                let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
                let definition = self.definitions.entry(symbol.clone()).or_default();
                definition.store_own_value(stored_rhs, guards, delayed);

                if delayed {
                    self.push_definition_stored_trace(
                        &symbol,
                        DefinitionKind::DelayedOwnValue,
                        || rhs.clone(),
                    );
                } else {
                    self.push_definition_stored_trace(&symbol, DefinitionKind::OwnValue, || {
                        rhs.clone()
                    });
                }
                symbol
            }
            TaggedDefinitionTarget::DownValue(symbol) => {
                let stores_default_value =
                    symbol.context() == SYSTEM_CONTEXT && symbol.name() == heads::patterns::DEFAULT;
                let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
                let rhs_may_splice_without_bindings =
                    rule_rhs_may_splice_without_bindings(&stored_rhs);
                let matcher = self.compile_down_value_matcher(&lhs, &guards);
                let matcher_attribute_revision = self.attribute_revision;
                let lhs_is_ground =
                    guards.is_empty() && !expr_contains_pattern_syntax(strip_hold_pattern(&lhs));
                let definition = self.definitions.entry(symbol.clone()).or_default();
                let down_value = DownValue {
                    lhs: lhs.clone(),
                    rhs: stored_rhs,
                    guards,
                    delayed,
                    lhs_is_ground,
                    rhs_may_splice_without_bindings,
                    matcher,
                    matcher_attribute_revision,
                };
                if stores_default_value {
                    replace_or_append_default_value(&mut definition.down_values, &lhs, down_value);
                    definition.note_down_values_rebuilt(&symbol);
                } else {
                    let (store_index, replaced_was_ground) =
                        replace_or_insert_down_value(definition, &lhs, down_value);
                    definition.note_down_value_stored(
                        &symbol,
                        lhs_is_ground,
                        replaced_was_ground,
                        store_index,
                    );
                }
                self.push_definition_stored_trace(&symbol, DefinitionKind::DownValue, || {
                    Expr::call(
                        system_expr(if delayed { "SetDelayed" } else { "Set" }),
                        vec![lhs.clone(), rhs.clone()],
                    )
                });
                symbol
            }
            TaggedDefinitionTarget::SubValue(symbol) => {
                let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
                let rhs_may_splice_without_bindings =
                    rule_rhs_may_splice_without_bindings(&stored_rhs);
                let matcher = self.compile_down_value_matcher(&lhs, &guards);
                let matcher_attribute_revision = self.attribute_revision;
                let definition = self.definitions.entry(symbol.clone()).or_default();
                replace_or_insert_sub_value(
                    definition,
                    &lhs,
                    SubValue {
                        lhs: lhs.clone(),
                        rhs: stored_rhs,
                        delayed,
                        rhs_may_splice_without_bindings,
                        matcher,
                        matcher_attribute_revision,
                        guards,
                    },
                );
                self.push_definition_stored_trace(&symbol, DefinitionKind::SubValue, || {
                    Expr::call(
                        system_expr(if delayed { "SetDelayed" } else { "Set" }),
                        vec![lhs.clone(), rhs.clone()],
                    )
                });
                symbol
            }
            TaggedDefinitionTarget::UpValue(symbol) => {
                let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
                let rhs_may_splice_without_bindings =
                    rule_rhs_may_splice_without_bindings(&stored_rhs);
                let matcher = self.compile_down_value_matcher(&lhs, &guards);
                let matcher_attribute_revision = self.attribute_revision;
                let definition = self.definitions.entry(symbol.clone()).or_default();
                replace_or_insert_up_value(
                    definition,
                    &lhs,
                    UpValue {
                        lhs: lhs.clone(),
                        rhs: stored_rhs,
                        delayed,
                        rhs_may_splice_without_bindings,
                        matcher,
                        matcher_attribute_revision,
                        guards,
                    },
                );
                self.has_up_value_definitions = true;
                self.push_definition_stored_trace(&symbol, DefinitionKind::UpValue, || {
                    Expr::call(
                        system_expr(if delayed { "SetDelayed" } else { "Set" }),
                        vec![lhs.clone(), rhs.clone()],
                    )
                });
                symbol
            }
        };

        self.installed_system_own_values.remove(&symbol);
        self.bump_evaluation_epoch_for_symbol(&symbol);
    }
}

fn replace_or_insert_down_value(
    definition: &mut SymbolDefinition,
    lhs: &Expr,
    new_entry: DownValue,
) -> (usize, Option<bool>) {
    if let Some(index) =
        definition.equivalent_down_value_index(lhs, &new_entry.guards, new_entry.lhs_is_ground)
    {
        let replaced_was_ground = definition.down_values[index].lhs_is_ground;
        definition.down_values[index] = new_entry;
        return (index, Some(replaced_was_ground));
    }

    let new_specificity = definition_specificity(lhs);
    let index = definition.down_values.last().map_or(0, |last| {
        if new_specificity <= definition_specificity(&last.lhs) {
            definition.down_values.len()
        } else {
            definition
                .down_values
                .partition_point(|entry| definition_specificity(&entry.lhs) >= new_specificity)
        }
    });
    definition.down_values.insert(index, new_entry);
    (index, None)
}

fn replace_or_insert_sub_value(definition: &mut SymbolDefinition, lhs: &Expr, new_entry: SubValue) {
    let lookup = definition.prepare_sub_value_store(lhs);
    let equivalent_index = guarded_definition_identity_index(
        &definition.sub_values,
        lhs,
        &new_entry.guards,
        lookup.equivalent_index(),
        |entry| (&entry.lhs, &entry.guards),
    );
    let (store_index, replaced) = replace_or_insert_definition_by_specificity(
        &mut definition.sub_values,
        lhs,
        new_entry,
        |entry| &entry.lhs,
        equivalent_index,
    );
    definition.note_sub_value_stored(store_index, replaced, lookup);
}

fn replace_or_insert_up_value(definition: &mut SymbolDefinition, lhs: &Expr, new_entry: UpValue) {
    let lookup = definition.prepare_up_value_store(lhs);
    let equivalent_index = guarded_definition_identity_index(
        &definition.up_values,
        lhs,
        &new_entry.guards,
        lookup.equivalent_index(),
        |entry| (&entry.lhs, &entry.guards),
    );
    let (store_index, replaced) = replace_or_insert_definition_by_specificity(
        &mut definition.up_values,
        lhs,
        new_entry,
        |entry| &entry.lhs,
        equivalent_index,
    );
    definition.note_up_value_stored(store_index, replaced, lookup);
}

fn guarded_definition_identity_index<T, F>(
    entries: &[T],
    lhs: &Expr,
    guards: &[Expr],
    lhs_equivalent_index: Option<usize>,
    identity_of: F,
) -> Option<usize>
where
    F: Fn(&T) -> (&Expr, &[Expr]),
{
    let index = lhs_equivalent_index?;
    let (stored_lhs, stored_guards) = identity_of(&entries[index]);
    if down_value_patterns_equivalent(stored_lhs, stored_guards, lhs, guards) {
        return Some(index);
    }

    entries
        .iter()
        .enumerate()
        .find_map(|(candidate_index, entry)| {
            (candidate_index != index)
                .then(|| identity_of(entry))
                .and_then(|(stored_lhs, stored_guards)| {
                    down_value_patterns_equivalent(stored_lhs, stored_guards, lhs, guards)
                        .then_some(candidate_index)
                })
        })
}

fn replace_or_insert_definition_by_specificity<T, F>(
    entries: &mut Vec<T>,
    lhs: &Expr,
    new_entry: T,
    lhs_of: F,
    equivalent_index: Option<usize>,
) -> (usize, bool)
where
    F: Fn(&T) -> &Expr,
{
    if let Some(index) = equivalent_index {
        entries[index] = new_entry;
        return (index, true);
    }

    let new_specificity = definition_specificity(lhs);
    let index = entries.last().map_or(0, |last| {
        if new_specificity <= definition_specificity(lhs_of(last)) {
            entries.len()
        } else {
            entries
                .partition_point(|entry| definition_specificity(lhs_of(entry)) >= new_specificity)
        }
    });
    entries.insert(index, new_entry);
    (index, false)
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

fn replace_or_append_default_value(entries: &mut Vec<DownValue>, lhs: &Expr, new_entry: DownValue) {
    let equivalent_index = if new_entry.guards.is_empty() {
        entries
            .iter()
            .position(|entry| entry.guards.is_empty() && lhs_patterns_equivalent(&entry.lhs, lhs))
    } else {
        entries.iter().position(|entry| {
            down_value_patterns_equivalent(&entry.lhs, &entry.guards, lhs, &new_entry.guards)
        })
    };

    if let Some(index) = equivalent_index {
        entries[index] = new_entry;
    } else {
        entries.push(new_entry);
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalUpSetSuspensionFrame {
    lhs: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalUpSetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => (|| session.finish_up_set_assignment("UpSet", lhs, rhs, false))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalTagSetSuspensionFrame {
    args: rum_core::SharedArgs,
    tag: rum_core::Symbol,
    lhs: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalTagSetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => {
                (|| session.store_tagged_definition("TagSet", args.as_ref(), tag, lhs, rhs, false))(
                )
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
