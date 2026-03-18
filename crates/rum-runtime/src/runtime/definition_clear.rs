use super::*;

impl Session {
    pub(super) fn eval_unset(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target] => {
                if self.unset_definition(target) {
                    Ok(system_expr("Null"))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "Unset",
                        args,
                        "expected a symbol, definition lhs, message name, or tagged definition target",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("Unset", args, 1)),
        }
    }

    pub(super) fn eval_clear(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_clear_like("Clear", args, false, false)
    }

    pub(super) fn eval_clear_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_clear_like("ClearAll", args, true, true)
    }

    pub(super) fn eval_clear_like(
        &mut self,
        name: &str,
        args: &[Expr],
        clear_attributes: bool,
        clear_messages: bool,
    ) -> Result<Expr, EvalError> {
        let Some(symbols) = self.lookup_clear_targets(args) else {
            return Ok(self.invalid_call_with_arg_form(
                name,
                args,
                "expected one or more symbols, symbol-name strings, or lists of them",
            ));
        };

        for symbol in symbols {
            self.clear_symbol_state(&symbol, clear_attributes, clear_messages);
        }

        Ok(system_expr("Null"))
    }

    pub(super) fn unset_definition(&mut self, expr: &Expr) -> bool {
        if let Some(key) = self.message_name_key_from_expr(expr) {
            self.message_definitions.remove(&key);
            return true;
        }

        if let Some(symbol) = expr_as_symbol(expr).cloned() {
            if let Some(definition) = self.definitions.get_mut(&symbol) {
                definition.own_value = None;
                definition.delayed_own_value = None;
            }

            self.prune_definition_if_empty(&symbol);
            return true;
        }

        if let Expr::Call { head, args } = expr {
            if is_system_symbol_name(head, "TagSet") && args.len() == 2 {
                if let Some(tag) = self.lookup_attribute_target(&args[0], true) {
                    if self.unset_tagged_definition(&tag, &args[1]).is_ok() {
                        self.prune_definition_if_empty(&tag);
                        return true;
                    }

                    return false;
                }

                return false;
            }
        }

        if let Some(head_symbol) = downvalue_head_symbol(expr).cloned() {
            self.remove_matching_down_values(&head_symbol, expr);
            self.prune_definition_if_empty(&head_symbol);
            return true;
        }

        if let Some(base_symbol) = subvalue_base_symbol(expr).cloned() {
            self.remove_matching_sub_values(&base_symbol, expr);
            self.prune_definition_if_empty(&base_symbol);
            return true;
        }

        false
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

    pub(super) fn lookup_clear_targets(&self, args: &[Expr]) -> Option<Vec<Symbol>> {
        if args.is_empty() {
            return None;
        }

        let mut symbols = Vec::new();

        for arg in args {
            self.collect_clear_targets(arg, &mut symbols)?;
        }

        Some(symbols)
    }

    pub(super) fn collect_clear_targets(
        &self,
        expr: &Expr,
        symbols: &mut Vec<Symbol>,
    ) -> Option<()> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                for item in args {
                    self.collect_clear_targets(item, symbols)?;
                }

                Some(())
            }
            _ => {
                symbols.push(self.lookup_attribute_target(expr, true)?);
                Some(())
            }
        }
    }

    pub(super) fn clear_symbol_state(
        &mut self,
        symbol: &Symbol,
        clear_attributes: bool,
        clear_messages: bool,
    ) {
        self.definitions.remove(symbol);

        if clear_attributes {
            self.attributes.remove(symbol);
        }

        if clear_messages {
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
        }
    }
}
