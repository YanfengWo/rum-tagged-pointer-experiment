use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct WithBindingsConfig {
    items: Vec<(Symbol, Expr)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopedBindingsConfig {
    items: Vec<(Symbol, Option<Expr>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopingBindingNormalizationError {
    InvalidWithBindings,
    InvalidScopedBindings,
}

impl ScopingBindingNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidWithBindings => {
                "expected a list of symbol assignments or rules as the first argument"
            }
            Self::InvalidScopedBindings => {
                "expected a list of symbols or symbol initializers as the first argument"
            }
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("With", args, 2));
        }

        let config = match self.compile_with_call_from_raw_args(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("With", args, error.message()));
            }
        };

        self.resume_with_bindings(config.items, args[1].clone(), BTreeMap::new(), 0)
    }

    pub(in crate::runtime) fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Module", args, 2));
        }

        let config = match self.compile_scoped_call_from_raw_args(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Module", args, error.message()));
            }
        };
        let items = config.items;

        let mut renames = BTreeMap::new();

        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }

        for (symbol, _init) in &items {
            let Some(fresh_expr) = renames.get(&symbol) else {
                continue;
            };
            let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
                continue;
            };

            let definition = self.definitions.entry(fresh_symbol.clone()).or_default();
            definition.own_value = None;
            definition.delayed_own_value = None;
        }

        self.resume_module_initializers(items, args[1].clone(), renames, 0)
    }

    pub(in crate::runtime) fn eval_block(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Block", args, 2));
        }

        let config = match self.compile_scoped_call_from_raw_args(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Block", args, error.message()));
            }
        };
        let items = config.items;

        let saved_state = self.capture_block_symbol_states(
            &items
                .iter()
                .map(|(symbol, _)| symbol.clone())
                .collect::<Vec<_>>(),
        );
        for (symbol, _) in &items {
            self.definitions.remove(symbol);
            self.attributes.remove(symbol);
            self.options.remove(symbol);
            self.syntax_information.remove(symbol);
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
        }
        if !items.is_empty() {
            self.bump_attribute_revision();
        }

        self.resume_block_initializers(items, args[1].clone(), saved_state, 0)
    }

    fn capture_block_symbol_states(
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
                    options: self.options.get(symbol).cloned(),
                    syntax_information: self.syntax_information.get(symbol).cloned(),
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

    fn restore_block_symbol_states(&mut self, states: BTreeMap<Symbol, BlockSymbolState>) {
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

            match state.options {
                Some(options) => {
                    self.options.insert(symbol.clone(), options);
                }
                None => {
                    self.options.remove(&symbol);
                }
            }

            match state.syntax_information {
                Some(syntax_information) => {
                    self.syntax_information
                        .insert(symbol.clone(), syntax_information);
                }
                None => {
                    self.syntax_information.remove(&symbol);
                }
            }

            self.message_definitions
                .retain(|key, _| key.symbol != symbol);
            for (key, definition) in state.messages {
                self.message_definitions.insert(key, definition);
            }
        }
        self.bump_attribute_revision();
    }

    fn resume_with_bindings(
        &mut self,
        items: Vec<(Symbol, Expr)>,
        body: Expr,
        mut bindings: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= items.len() {
            let body = substitute_lexical_bindings(&body, &bindings, &BTreeSet::new());
            return self.evaluate(body);
        }

        let (symbol, rhs) = items[index].clone();
        let substituted_rhs = substitute_lexical_bindings(&rhs, &bindings, &BTreeSet::new());
        let rhs_result = self.evaluate(substituted_rhs);
        let outcome = self.suspend_with_frame(rhs_result, {
            let items = items.clone();
            let body = body.clone();
            let bindings = bindings.clone();
            let symbol = symbol.clone();
            move |session, value| {
                let mut bindings = bindings;
                bindings.insert(symbol, value);
                session.resume_with_bindings(items, body, bindings, index + 1)
            }
        })?;
        bindings.insert(symbol, outcome);
        self.resume_with_bindings(items, body, bindings, index + 1)
    }

    fn resume_module_initializers(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        renames: BTreeMap<Symbol, Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= items.len() {
            let body = substitute_lexical_bindings(&body, &renames, &BTreeSet::new());
            return self.evaluate(body);
        }

        let (symbol, init) = items[index].clone();
        if let Some(init) = init {
            let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
            let init_result = self.evaluate(renamed_init);
            let outcome = self.suspend_with_frame(init_result, {
                let items = items.clone();
                let body = body.clone();
                let renames = renames.clone();
                let symbol = symbol.clone();
                move |session, value| {
                    session.store_module_initializer_value(&renames, &symbol, value);
                    session.resume_module_initializers(items, body, renames, index + 1)
                }
            })?;
            self.store_module_initializer_value(&renames, &symbol, outcome);
        }

        self.resume_module_initializers(items, body, renames, index + 1)
    }

    fn store_module_initializer_value(
        &mut self,
        renames: &BTreeMap<Symbol, Expr>,
        symbol: &Symbol,
        value: Expr,
    ) {
        let Some(fresh_expr) = renames.get(symbol) else {
            return;
        };
        let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
            return;
        };

        let definition = self.definitions.entry(fresh_symbol).or_default();
        definition.own_value = Some(value);
        definition.delayed_own_value = None;
    }

    fn resume_block_initializers(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= items.len() {
            return self.resume_block_body(body, saved_state);
        }

        let (symbol, init) = items[index].clone();
        if let Some(init) = init {
            let init_result = self.evaluate(init);
            let outcome = self.suspend_with_result_frame(init_result, {
                let items = items.clone();
                let body = body.clone();
                let saved_state = saved_state.clone();
                let symbol = symbol.clone();
                move |session, outcome| {
                    session.resume_block_after_initializer(
                        items,
                        body,
                        saved_state,
                        index,
                        symbol,
                        outcome,
                    )
                }
            });

            return match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => self.resume_block_after_initializer(
                    items,
                    body,
                    saved_state,
                    index,
                    symbol,
                    outcome,
                ),
            };
        }

        self.resume_block_initializers(items, body, saved_state, index + 1)
    }

    fn resume_block_after_initializer(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        index: usize,
        symbol: Symbol,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(value);
                definition.delayed_own_value = None;
                self.resume_block_initializers(items, body, saved_state, index + 1)
            }
            Err(error) => {
                self.restore_block_symbol_states(saved_state);
                Err(error)
            }
        }
    }

    fn resume_block_body(
        &mut self,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
    ) -> Result<Expr, EvalError> {
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let saved_state = saved_state.clone();
            move |session, outcome| session.finish_block_scope(saved_state, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_block_scope(saved_state, outcome),
        }
    }

    fn finish_block_scope(
        &mut self,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.restore_block_symbol_states(saved_state);
        outcome
    }

    fn compile_with_call_from_raw_args(
        &self,
        args: &[Expr],
    ) -> Result<WithBindingsConfig, ScopingBindingNormalizationError> {
        Ok(WithBindingsConfig {
            items: parse_with_binding_list(&args[0])
                .ok_or(ScopingBindingNormalizationError::InvalidWithBindings)?,
        })
    }

    fn compile_scoped_call_from_raw_args(
        &self,
        args: &[Expr],
    ) -> Result<ScopedBindingsConfig, ScopingBindingNormalizationError> {
        Ok(ScopedBindingsConfig {
            items: parse_module_binding_list(&args[0])
                .ok_or(ScopingBindingNormalizationError::InvalidScopedBindings)?,
        })
    }
}
