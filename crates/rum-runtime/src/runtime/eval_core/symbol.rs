use super::*;

impl Session {
    pub(super) fn evaluate_symbol(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        if symbol.context() == SYSTEM_CONTEXT {
            match symbol.name() {
                "$Context" => return Ok(Expr::string(self.current_context.clone())),
                "$ContextPath" => {
                    let contexts = self
                        .context_path
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), contexts));
                }
                "$Packages" => {
                    let packages = self
                        .loaded_packages
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), packages));
                }
                "$InputFileName" => {
                    return Ok(Expr::string(
                        self.load_stack
                            .last()
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                    ));
                }
                "$Path" => {
                    let paths = self
                        .package_search_paths
                        .iter()
                        .map(|path| Expr::string(path.display().to_string()))
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), paths));
                }
                "$MessageList" => {
                    let messages = self.current_message_list();
                    return Ok(Expr::call(system_expr("List"), messages));
                }
                _ => {}
            }
        }

        if self.declared_stubs.contains(&symbol)
            && !self
                .loaded_packages
                .iter()
                .any(|loaded| loaded == symbol.context())
        {
            let load_result = self.eval_needs(&[Expr::string(symbol.context().to_string())])?;
            if !self.capability_policy.is_unrestricted()
                && !self.capability_policy.allows(Capability::FileRead)
            {
                debug_assert!(is_system_symbol_name(&load_result, "$Failed"));
                return Ok(load_result);
            }
        }

        if let Some(value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.own_value.clone())
        {
            if matches!(&value, Expr::Atom(Atom::Symbol(value_symbol)) if value_symbol == &symbol) {
                return Ok(value);
            }
            return self.evaluate_own_value(value);
        }

        if let Some(value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.delayed_own_value.clone())
        {
            if matches!(&value, Expr::Atom(Atom::Symbol(value_symbol)) if value_symbol == &symbol) {
                return Ok(value);
            }
            return self.evaluate_definition_rhs(None, value);
        }

        if let Some(value) = self.dynamic_own_value_binding(&symbol) {
            match value {
                Some(value) => self.evaluate_own_value(value),
                None => Ok(Expr::Atom(Atom::Symbol(symbol))),
            }
        } else if self.removed_symbols.contains(&symbol) {
            Ok(removed_symbol_expr(&symbol))
        } else {
            Ok(Expr::Atom(Atom::Symbol(symbol)))
        }
    }

    fn evaluate_own_value(&mut self, value: Expr) -> Result<Expr, EvalError> {
        match &value {
            Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_)) => Ok(value),
            _ => self.evaluate(value),
        }
    }

    pub(in crate::runtime) fn push_dynamic_own_value_binding(
        &mut self,
        symbol: Symbol,
        value: Option<Expr>,
    ) -> usize {
        let depth = self.dynamic_own_values.len();
        self.dynamic_own_values
            .push(DynamicOwnValueFrame { symbol, value });
        depth
    }

    pub(in crate::runtime) fn truncate_dynamic_own_value_bindings(&mut self, depth: usize) {
        if depth >= self.dynamic_own_values.len() {
            return;
        }

        self.dynamic_own_values.truncate(depth);
    }

    pub(in crate::runtime) fn dynamic_own_value_binding(
        &self,
        symbol: &Symbol,
    ) -> Option<Option<Expr>> {
        self.dynamic_own_values
            .iter()
            .rev()
            .find(|frame| frame.symbol == *symbol)
            .map(|frame| frame.value.clone())
    }

    pub(in crate::runtime) fn dynamic_scope_symbol_state_is_clean(&self, symbol: &Symbol) -> bool {
        self.definitions.get(symbol).is_none()
            && !self.attributes.contains_key(symbol)
            && !self.options.contains_key(symbol)
            && !self.syntax_information.contains_key(symbol)
            && self
                .message_symbol_counts
                .get(symbol)
                .copied()
                .unwrap_or_default()
                == 0
    }

    pub(in crate::runtime) fn clear_dynamic_scope_symbol_side_effects(&mut self, symbol: &Symbol) {
        let current_had_up_values = self
            .definitions
            .get(symbol)
            .is_some_and(|definition| !definition.up_values.is_empty());
        if self.definitions.remove(symbol).is_some() {
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
        if current_had_up_values {
            self.refresh_has_up_value_definitions();
        }

        if self.attributes.remove(symbol).is_some() {
            self.bump_attribute_revision();
        }
        self.options.remove(symbol);
        self.syntax_information.remove(symbol);

        if self
            .message_symbol_counts
            .get(symbol)
            .copied()
            .unwrap_or_default()
            > 0
        {
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
            self.rebuild_message_symbol_counts();
        }
    }

    pub(super) fn has_dynamic_own_value_binding(&self, symbol: &Symbol) -> bool {
        self.dynamic_own_values
            .iter()
            .rev()
            .any(|frame| frame.symbol == *symbol)
    }

    fn current_message_list(&self) -> Vec<Expr> {
        if let Some(checkpoint) = self
            .current_message_list_checkpoint
            .filter(|_| self.current_message_list_dirty)
        {
            return self.messages[checkpoint..]
                .iter()
                .cloned()
                .map(held_runtime_message_name_expr)
                .collect();
        }

        self.last_message_list
            .iter()
            .cloned()
            .map(held_runtime_message_name_expr)
            .collect()
    }
}
