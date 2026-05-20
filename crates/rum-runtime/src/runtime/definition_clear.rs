use super::*;

impl Session {
    pub(super) fn eval_unset(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target] => match self.normalize_unset_target(target) {
                Ok(target) => {
                    if self.ensure_symbol_not_protected_for_write("Unset", target.symbol()) {
                        self.unset_normalized_definition(target);
                    }
                    Ok(system_expr("Null"))
                }
                Err(error) => Ok(self.invalid_call_with_arg_form("Unset", args, error.message())),
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("Unset", args, 1)),
        }
    }

    pub(super) fn eval_clear(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_clear_like("Clear", args, false, false, false, false, false)
    }

    pub(super) fn eval_clear_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_clear_like("ClearAll", args, true, true, true, true, true)
    }

    pub(super) fn eval_remove(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let symbols = match self.normalize_clear_targets(args) {
            Ok(symbols) => symbols,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Remove", args, error.message()));
            }
        };

        for symbol in symbols {
            if self.symbol_is_protected(&symbol) {
                self.push_message(
                    "Remove",
                    "rmptc",
                    format!("Symbol {symbol} is Protected and cannot be removed."),
                );
                continue;
            }

            self.remove_symbol_state(&symbol);
        }

        Ok(system_expr("Null"))
    }

    pub(super) fn eval_clear_like(
        &mut self,
        name: &str,
        args: &[Expr],
        clear_attributes: bool,
        clear_messages: bool,
        clear_options: bool,
        clear_syntax_information: bool,
        clear_default_values: bool,
    ) -> Result<Expr, EvalError> {
        let symbols = match self.normalize_clear_targets(args) {
            Ok(symbols) => symbols,
            Err(error) => return Ok(self.invalid_call_with_arg_form(name, args, error.message())),
        };

        for symbol in symbols {
            if self.ensure_symbol_not_protected_for_write(name, &symbol) {
                self.clear_symbol_state(
                    &symbol,
                    clear_attributes,
                    clear_messages,
                    clear_options,
                    clear_syntax_information,
                    clear_default_values,
                );
            }
        }

        Ok(system_expr("Null"))
    }

    pub(super) fn unset_normalized_definition(&mut self, target: UnsetDefinitionTarget) {
        match target {
            UnsetDefinitionTarget::Message(key) => {
                self.message_definitions.remove(&key);
            }
            UnsetDefinitionTarget::Options(symbol) => {
                self.options.remove(&symbol);
            }
            UnsetDefinitionTarget::SyntaxInformation(symbol) => {
                self.syntax_information.remove(&symbol);
            }
            UnsetDefinitionTarget::OwnValue(symbol) => {
                if let Some(definition) = self.definitions.get_mut(&symbol) {
                    definition.own_value = None;
                    definition.delayed_own_value = None;
                }

                self.prune_definition_if_empty(&symbol);
            }
            UnsetDefinitionTarget::DownValue { symbol, lhs } => {
                self.remove_matching_down_values(&symbol, &lhs);
                self.prune_definition_if_empty(&symbol);
            }
            UnsetDefinitionTarget::SubValue { symbol, lhs } => {
                self.remove_matching_sub_values(&symbol, &lhs);
                self.prune_definition_if_empty(&symbol);
            }
            UnsetDefinitionTarget::TaggedDefinition { tag, lhs } => {
                if self.unset_tagged_definition(&tag, &lhs).is_ok() {
                    self.prune_definition_if_empty(&tag);
                }
            }
        }
    }

    pub(super) fn unset_tagged_definition(
        &mut self,
        tag: &Symbol,
        lhs: &Expr,
    ) -> Result<bool, TagDefinitionError> {
        match classify_tagged_definition(tag, lhs) {
            Ok(TaggedDefinitionTarget::OwnValue(symbol)) => {
                if let Some(definition) = self.definitions.get_mut(&symbol) {
                    let changed = definition.own_value.take().is_some()
                        || definition.delayed_own_value.take().is_some();
                    self.prune_definition_if_empty(&symbol);
                    Ok(changed)
                } else {
                    Ok(false)
                }
            }
            Ok(TaggedDefinitionTarget::DownValue(symbol)) => {
                let changed = self.remove_matching_down_values(&symbol, lhs);
                self.prune_definition_if_empty(&symbol);
                Ok(changed)
            }
            Ok(TaggedDefinitionTarget::SubValue(symbol)) => {
                let changed = self.remove_matching_sub_values(&symbol, lhs);
                self.prune_definition_if_empty(&symbol);
                Ok(changed)
            }
            Ok(TaggedDefinitionTarget::UpValue(symbol)) => {
                let changed = self.remove_matching_up_values(&symbol, lhs);
                self.prune_definition_if_empty(&symbol);
                Ok(changed)
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn remove_matching_down_values(&mut self, symbol: &Symbol, lhs: &Expr) -> bool {
        let Some(definition) = self.definitions.get_mut(symbol) else {
            return false;
        };

        let original_len = definition.down_values.len();
        definition
            .down_values
            .retain(|entry| !lhs_patterns_equivalent(&entry.lhs, lhs));
        definition.down_values.len() != original_len
    }

    pub(super) fn remove_matching_up_values(&mut self, symbol: &Symbol, lhs: &Expr) -> bool {
        let Some(definition) = self.definitions.get_mut(symbol) else {
            return false;
        };

        let original_len = definition.up_values.len();
        definition
            .up_values
            .retain(|entry| !lhs_patterns_equivalent(&entry.lhs, lhs));
        definition.up_values.len() != original_len
    }

    pub(super) fn remove_matching_sub_values(&mut self, symbol: &Symbol, lhs: &Expr) -> bool {
        let Some(definition) = self.definitions.get_mut(symbol) else {
            return false;
        };

        let original_len = definition.sub_values.len();
        definition
            .sub_values
            .retain(|entry| !lhs_patterns_equivalent(&entry.lhs, lhs));
        definition.sub_values.len() != original_len
    }

    pub(super) fn prune_definition_if_empty(&mut self, symbol: &Symbol) {
        if self
            .definitions
            .get(symbol)
            .is_some_and(SymbolDefinition::is_empty)
        {
            self.definitions.remove(symbol);
        }
    }

    pub(super) fn clear_symbol_state(
        &mut self,
        symbol: &Symbol,
        clear_attributes: bool,
        clear_messages: bool,
        clear_options: bool,
        clear_syntax_information: bool,
        clear_default_values: bool,
    ) {
        self.definitions.remove(symbol);

        if clear_attributes {
            self.attributes.remove(symbol);
        }

        if clear_messages {
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
        }

        if clear_options {
            self.options.remove(symbol);
        }

        if clear_syntax_information {
            self.syntax_information.remove(symbol);
        }

        if clear_default_values {
            self.remove_default_values_for_symbol(symbol);
        }
    }

    pub(super) fn remove_symbol_state(&mut self, symbol: &Symbol) {
        self.mark_symbol_removed(symbol);
        self.clear_symbol_state(symbol, true, true, true, true, true);
        self.declared_stubs.remove(symbol);
    }

    fn mark_symbol_removed(&mut self, symbol: &Symbol) {
        self.removed_symbols.insert(symbol.clone());

        if symbol.context() == SYSTEM_CONTEXT {
            return;
        }

        let key = symbol_generation_key(symbol.context(), symbol.name());
        let current_generation = self.symbol_generations.get(&key).copied().unwrap_or(0);
        let removed_generation = symbol.identity().unwrap_or(0);
        self.symbol_generations
            .insert(key, current_generation.max(removed_generation) + 1);
    }

    pub(super) fn remove_default_values_for_symbol(&mut self, symbol: &Symbol) {
        let default_symbol = Symbol::system("Default");
        let Some(definition) = self.definitions.get_mut(&default_symbol) else {
            return;
        };

        definition
            .down_values
            .retain(|entry| !default_value_lhs_targets_symbol(&entry.lhs, symbol));

        if definition.is_empty() {
            self.definitions.remove(&default_symbol);
        }
    }
}
