use super::*;

impl Session {
    pub(super) fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("With", args, 2));
        }

        let Some(bindings) = self.evaluate_with_bindings(&args[0])? else {
            return Ok(self.invalid_call_with_arg_form(
                "With",
                args,
                "expected a list of symbol assignments or rules as the first argument",
            ));
        };

        let body = substitute_lexical_bindings(&args[1], &bindings, &BTreeSet::new());
        self.evaluate(body)
    }

    pub(super) fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Module", args, 2));
        }

        let Some(items) = parse_module_binding_list(&args[0]) else {
            return Ok(self.invalid_call_with_arg_form(
                "Module",
                args,
                "expected a list of symbols or symbol initializers as the first argument",
            ));
        };

        let mut renames = BTreeMap::new();

        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }

        for (symbol, init) in items {
            let Some(fresh_expr) = renames.get(&symbol) else {
                continue;
            };
            let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
                continue;
            };

            let definition = self.definitions.entry(fresh_symbol.clone()).or_default();
            definition.own_value = None;
            definition.delayed_own_value = None;

            if let Some(init) = init {
                let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
                let value = self.evaluate(renamed_init)?;
                let definition = self.definitions.entry(fresh_symbol).or_default();
                definition.own_value = Some(value);
                definition.delayed_own_value = None;
            }
        }

        let body = substitute_lexical_bindings(&args[1], &renames, &BTreeSet::new());
        self.evaluate(body)
    }

    pub(super) fn eval_block(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Block", args, 2));
        }

        let Some(items) = parse_module_binding_list(&args[0]) else {
            return Ok(self.invalid_call_with_arg_form(
                "Block",
                args,
                "expected a list of symbols or symbol initializers as the first argument",
            ));
        };

        let saved_state = self.capture_block_symbol_states(
            &items
                .iter()
                .map(|(symbol, _)| symbol.clone())
                .collect::<Vec<_>>(),
        );

        let result = (|| -> Result<Expr, EvalError> {
            for (symbol, _) in &items {
                self.definitions.remove(symbol);
                self.attributes.remove(symbol);
                self.message_definitions
                    .retain(|key, _| key.symbol != *symbol);
            }

            for (symbol, init) in &items {
                if let Some(init) = init {
                    let value = self.evaluate(init.clone())?;
                    let definition = self.definitions.entry(symbol.clone()).or_default();
                    definition.own_value = Some(value);
                    definition.delayed_own_value = None;
                }
            }

            self.evaluate(args[1].clone())
        })();

        self.restore_block_symbol_states(saved_state);

        result
    }

    pub(super) fn capture_block_symbol_states(
        &self,
        symbols: &[Symbol],
    ) -> BTreeMap<Symbol, BlockSymbolState> {
        let mut states = BTreeMap::new();

        for symbol in symbols {
            states
                .entry(symbol.clone())
                .or_insert_with(|| BlockSymbolState {
                    definition: self.definitions.get(symbol).cloned(),
                    attributes: self.attributes.get(symbol).copied(),
                    messages: self
                        .message_definitions
                        .iter()
                        .filter(|(key, _)| key.symbol == *symbol)
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect(),
                });
        }

        states
    }

    pub(super) fn restore_block_symbol_states(
        &mut self,
        states: BTreeMap<Symbol, BlockSymbolState>,
    ) {
        for (symbol, state) in states {
            match state.definition {
                Some(definition) => {
                    self.definitions.insert(symbol.clone(), definition);
                }
                None => {
                    self.definitions.remove(&symbol);
                }
            }

            match state.attributes {
                Some(attributes) => {
                    self.attributes.insert(symbol.clone(), attributes);
                }
                None => {
                    self.attributes.remove(&symbol);
                }
            }

            self.message_definitions
                .retain(|key, _| key.symbol != symbol);
            for (key, definition) in state.messages {
                self.message_definitions.insert(key, definition);
            }
        }
    }

    pub(super) fn try_apply_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(function) = parse_function(head) else {
            return Ok(None);
        };

        let applied = match function {
            FunctionSpec::SlotBased { body, .. } => substitute_slots(body, args),
            FunctionSpec::Explicit { params, body, .. } => {
                if params.len() != args.len() {
                    self.push_message(
                        "Function",
                        "argx",
                        format!(
                            "expected exactly {} argument(s), got {}",
                            params.len(),
                            args.len()
                        ),
                    );
                    return Ok(Some(Expr::call(head.clone(), args.to_vec())));
                }

                let bindings = params.into_iter().zip(args.iter().cloned()).collect();
                substitute_lexical_bindings(body, &bindings, &BTreeSet::new())
            }
        };

        self.evaluate(applied).map(Some)
    }

    pub(super) fn evaluate_with_bindings(
        &mut self,
        expr: &Expr,
    ) -> Result<Option<BTreeMap<Symbol, Expr>>, EvalError> {
        let Expr::Call { head, args } = expr else {
            return Ok(None);
        };

        if !is_system_symbol_name(head, "List") {
            return Ok(None);
        }

        let mut bindings = BTreeMap::new();

        for item in args {
            let Some((symbol, rhs)) = parse_with_binding_item(item) else {
                return Ok(None);
            };

            let substituted_rhs = substitute_lexical_bindings(&rhs, &bindings, &BTreeSet::new());
            let value = self.evaluate(substituted_rhs)?;
            bindings.insert(symbol, value);
        }

        Ok(Some(bindings))
    }

    pub(super) fn allocate_fresh_symbol(&mut self, symbol: &Symbol) -> Symbol {
        loop {
            self.next_generated_symbol_id += 1;
            let candidate = Symbol::new(
                symbol.context().to_string(),
                format!("{}${}", symbol.name(), self.next_generated_symbol_id),
            );

            if !self.generated_symbol_exists(&candidate) {
                return candidate;
            }
        }
    }

    pub(super) fn generated_symbol_exists(&self, symbol: &Symbol) -> bool {
        self.symbol_exists(symbol)
            || self
                .message_definitions
                .keys()
                .any(|key| key.symbol == *symbol)
    }
}
