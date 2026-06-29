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

        self.resume_with_bindings(config.items, args[1].clone(), SymbolExprMap::default(), 0)
    }

    pub(in crate::runtime) fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.record_profile_eval_module_entry();
        let profile_start = lexical_profile_start();
        let eval_cache_profile_start =
            evaluation_cache_profile_source_timing_start("eval_module_by_source");

        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Module", args, 2));
        }

        let runtime_profile_compile = runtime_path_timing_start("module.compile_bindings");
        let config_result = self.compile_scoped_call_from_raw_args(args);
        record_profile_runtime_path_timing(
            runtime_profile_compile,
            if config_result.is_ok() {
                RuntimeProfileOutcome::Ok
            } else {
                RuntimeProfileOutcome::Error
            },
        );
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Module", args, error.message()));
            }
        };
        let items = config.items;

        let mut renames = SymbolExprMap::default();

        let runtime_profile_allocate = runtime_path_timing_start("module.allocate_fresh_symbols");
        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }
        record_profile_runtime_path_timing(runtime_profile_allocate, RuntimeProfileOutcome::Ok);

        let gensyms: Vec<Symbol> = renames
            .values()
            .filter_map(|expr| expr_as_symbol(expr).cloned())
            .collect();
        let activation_depth = self.module_activations.len();
        let side_table_stamp = self.current_side_table_stamp();
        self.module_activations.push(ModuleActivation {
            gensyms,
            escaped: false,
            side_table_stamp,
        });

        let runtime_profile_total = runtime_path_timing_start("module.eval.total");
        let result =
            self.resume_module_initializers(items, args[1].clone(), renames, activation_depth, 0);
        record_profile_eval_module(profile_start);
        record_profile_eval_module_by_source(eval_cache_profile_start);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
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

        if items.len() == 1 {
            let (symbol, init) = items
                .into_iter()
                .next()
                .expect("single Block item should exist");
            return self.eval_single_symbol_block(symbol, init, args[1].clone());
        }

        let saved_state = self.capture_block_symbol_states(
            &items
                .iter()
                .map(|(symbol, _)| symbol.clone())
                .collect::<Vec<_>>(),
        );
        let mut attributes_changed = false;
        let mut messages_changed = false;
        for (symbol, _) in &items {
            if self.definitions.remove(symbol).is_some() {
                self.bump_evaluation_epoch_for_symbol(symbol);
            }
            attributes_changed |= self.attributes.remove(symbol).is_some();
            self.options.remove(symbol);
            self.syntax_information.remove(symbol);
            let messages_before = self.message_definitions.len();
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
            messages_changed |= self.message_definitions.len() != messages_before;
        }
        if !items.is_empty() {
            self.refresh_has_up_value_definitions();
        }
        if attributes_changed {
            self.bump_attribute_revision();
        }
        if messages_changed {
            self.rebuild_message_symbol_counts();
        }

        let dynamic_depth = self.push_block_dynamic_shadow_bindings(&items);
        self.resume_block_initializers(items, args[1].clone(), saved_state, dynamic_depth, 0)
    }

    fn eval_single_symbol_block(
        &mut self,
        symbol: Symbol,
        init: Option<Expr>,
        body: Expr,
    ) -> Result<Expr, EvalError> {
        let saved_state = self.capture_single_block_symbol_state(&symbol);
        self.clear_single_block_symbol_state(&symbol, &saved_state);
        let dynamic_depth = self.push_dynamic_own_value_binding(symbol, None);

        if let Some(init) = init {
            let init_result = self.evaluate(init);
            let outcome = self.suspend_with_result_frame(init_result, {
                let saved_state = saved_state.clone();
                let body = body.clone();
                move |session, outcome| {
                    session.resume_single_symbol_block_after_initializer(
                        symbol,
                        body,
                        saved_state,
                        dynamic_depth,
                        outcome,
                    )
                }
            });

            return match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => self.resume_single_symbol_block_after_initializer(
                    symbol,
                    body,
                    saved_state,
                    dynamic_depth,
                    outcome,
                ),
            };
        }

        self.resume_single_symbol_block_body(symbol, body, saved_state, dynamic_depth)
    }

    fn push_block_dynamic_shadow_bindings(&mut self, items: &[(Symbol, Option<Expr>)]) -> usize {
        let dynamic_depth = self.dynamic_own_values.len();
        for (symbol, _) in items {
            self.push_dynamic_own_value_binding(*symbol, None);
        }
        dynamic_depth
    }

    fn capture_single_block_symbol_state(&self, symbol: &Symbol) -> BlockSymbolState {
        BlockSymbolState {
            definition: self.definitions.get(symbol).cloned(),
            attributes: self.attributes.get(symbol).copied(),
            options: self.options.get(symbol).cloned(),
            syntax_information: self.syntax_information.get(symbol).cloned(),
            messages: self.single_symbol_message_definitions(symbol),
        }
    }

    fn single_symbol_message_definitions(
        &self,
        symbol: &Symbol,
    ) -> Vec<(MessageNameKey, MessageDefinition)> {
        if self
            .message_symbol_counts
            .get(symbol)
            .copied()
            .unwrap_or_default()
            == 0
        {
            return Vec::new();
        }

        self.message_definitions
            .iter()
            .filter(|(key, _)| key.symbol == *symbol)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    fn clear_single_block_symbol_state(&mut self, symbol: &Symbol, saved_state: &BlockSymbolState) {
        let removed_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        if self.definitions.remove(symbol).is_some() {
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
        if removed_up_values {
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
        let mut attributes_changed = false;
        let mut messages_changed = false;
        for (symbol, state) in states {
            let definition_changed = match state.definition {
                Some(definition) => {
                    self.definitions.insert(symbol, definition);
                    true
                }
                None => self.definitions.remove(&symbol).is_some(),
            };
            if definition_changed {
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }

            match state.attributes {
                Some(attributes) => {
                    self.attributes.insert(symbol, attributes);
                    attributes_changed = true;
                }
                None => {
                    attributes_changed |= self.attributes.remove(&symbol).is_some();
                }
            }

            match state.options {
                Some(options) => {
                    self.options.insert(symbol, options);
                }
                None => {
                    self.options.remove(&symbol);
                }
            }

            match state.syntax_information {
                Some(syntax_information) => {
                    self.syntax_information.insert(symbol, syntax_information);
                }
                None => {
                    self.syntax_information.remove(&symbol);
                }
            }

            let messages_before = self.message_definitions.len();
            self.message_definitions
                .retain(|key, _| key.symbol != symbol);
            messages_changed |=
                self.message_definitions.len() != messages_before || !state.messages.is_empty();
            for (key, definition) in state.messages {
                self.message_definitions.insert(key, definition);
            }
        }
        self.refresh_has_up_value_definitions();
        if attributes_changed {
            self.bump_attribute_revision();
        }
        if messages_changed {
            self.rebuild_message_symbol_counts();
        }
    }

    fn resume_with_bindings(
        &mut self,
        items: Vec<(Symbol, Expr)>,
        body: Expr,
        mut bindings: SymbolExprMap,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= items.len() {
            let body = substitute_lexical_bindings(&body, &bindings, &BTreeSet::new());
            return self.evaluate(body);
        }

        let (symbol, rhs) = items[index].clone();
        let substituted_rhs = substitute_lexical_bindings(&rhs, &bindings, &BTreeSet::new());
        let rhs_result = self.evaluate(substituted_rhs);
        let outcome = self.suspend_with_frame_lazy(rhs_result, || {
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
        renames: SymbolExprMap,
        activation_depth: usize,
        index: usize,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_resume = runtime_path_timing_start("module.resume_initializers");
        record_profile_module_resume_entry(index);
        if index >= items.len() {
            let profile_start = lexical_profile_start();
            let eval_cache_profile_start =
                evaluation_cache_profile_source_timing_start("module_body_substitution_by_source");
            let body = substitute_lexical_bindings(&body, &renames, &BTreeSet::new());
            record_profile_module_body_substitution(profile_start);
            record_profile_module_body_substitution_by_source(eval_cache_profile_start);
            let runtime_profile_body = runtime_path_timing_start("module.body.evaluate");
            let result = self.evaluate(body);
            record_profile_runtime_path_timing(
                runtime_profile_body,
                runtime_profile_outcome_for_expr_result(&result),
            );
            let outcome = self.suspend_with_result_frame(result, move |session, outcome| {
                session.finish_module_scope(activation_depth, outcome)
            });

            let result = match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => self.finish_module_scope(activation_depth, outcome),
            };
            record_profile_runtime_path_timing(
                runtime_profile_resume,
                runtime_profile_outcome_for_expr_result(&result),
            );
            return result;
        }

        let (symbol, init) = items[index].clone();
        if let Some(init) = init {
            let profile_start = lexical_profile_start();
            let eval_cache_profile_start =
                evaluation_cache_profile_source_timing_start("module_init_substitution_by_source");
            let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
            record_profile_module_init_substitution(profile_start);
            record_profile_module_init_substitution_by_source(eval_cache_profile_start);
            let runtime_profile_init = runtime_path_timing_start("module.initializer.evaluate");
            let init_result = self.evaluate(renamed_init);
            record_profile_runtime_path_timing(
                runtime_profile_init,
                runtime_profile_outcome_for_expr_result(&init_result),
            );
            let outcome = self.suspend_with_result_frame_lazy(init_result, || {
                let items = items.clone();
                let body = body.clone();
                let renames = renames.clone();
                let symbol = symbol.clone();
                move |session, outcome| {
                    session.resume_module_after_initializer(
                        items,
                        body,
                        renames,
                        activation_depth,
                        index,
                        symbol,
                        outcome,
                    )
                }
            });

            let result = match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => self.resume_module_after_initializer(
                    items,
                    body,
                    renames,
                    activation_depth,
                    index,
                    symbol,
                    outcome,
                ),
            };
            record_profile_runtime_path_timing(
                runtime_profile_resume,
                runtime_profile_outcome_for_expr_result(&result),
            );
            return result;
        }

        let result =
            self.resume_module_initializers(items, body, renames, activation_depth, index + 1);
        record_profile_runtime_path_timing(
            runtime_profile_resume,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    fn resume_module_after_initializer(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        renames: SymbolExprMap,
        activation_depth: usize,
        index: usize,
        symbol: Symbol,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                self.store_module_initializer_value(&renames, &symbol, value);
                self.resume_module_initializers(items, body, renames, activation_depth, index + 1)
            }
            Err(error) => self.finish_module_scope(activation_depth, Err(error)),
        }
    }

    fn store_module_initializer_value(
        &mut self,
        renames: &SymbolExprMap,
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
        self.bump_evaluation_epoch_for_symbol(&fresh_symbol);
    }

    fn finish_module_scope(
        &mut self,
        activation_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let activation = self.module_activations[activation_depth].clone();
        let escaped =
            activation.escaped || self.current_side_table_stamp() != activation.side_table_stamp;

        let mut reclaim = Vec::new();
        if let Ok(result) = &outcome {
            if !escaped {
                let mut retained: BTreeSet<Symbol> = activation
                    .gensyms
                    .iter()
                    .filter(|gensym| expr_contains_symbol(result, gensym))
                    .cloned()
                    .collect();

                loop {
                    let retained_snapshot: Vec<Symbol> = retained.iter().cloned().collect();
                    let mut grew = false;

                    for retained_gensym in retained_snapshot {
                        let Some(definition) = self.definitions.get(&retained_gensym) else {
                            continue;
                        };

                        for gensym in &activation.gensyms {
                            if retained.contains(gensym) {
                                continue;
                            }

                            if definition_references_symbol(definition, gensym) {
                                retained.insert(gensym.clone());
                                grew = true;
                            }
                        }
                    }

                    if !grew {
                        break;
                    }
                }

                reclaim = activation
                    .gensyms
                    .iter()
                    .filter(|gensym| !retained.contains(*gensym))
                    .cloned()
                    .collect();
            }
        }

        self.module_activations.truncate(activation_depth);
        for gensym in reclaim {
            self.clear_symbol_state(&gensym, true, true, true, true, true);
        }

        outcome
    }

    fn resume_block_initializers(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= items.len() {
            return self.resume_block_body(body, saved_state, dynamic_depth);
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
                        dynamic_depth,
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
                    dynamic_depth,
                    index,
                    symbol,
                    outcome,
                ),
            };
        }

        self.resume_block_initializers(items, body, saved_state, dynamic_depth, index + 1)
    }

    fn resume_block_after_initializer(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
        index: usize,
        symbol: Symbol,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(value);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
                self.resume_block_initializers(items, body, saved_state, dynamic_depth, index + 1)
            }
            Err(error) => {
                self.truncate_dynamic_own_value_bindings(dynamic_depth);
                self.restore_block_symbol_states(saved_state);
                Err(error)
            }
        }
    }

    fn resume_single_symbol_block_after_initializer(
        &mut self,
        symbol: Symbol,
        body: Expr,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(value);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
                self.resume_single_symbol_block_body(symbol, body, saved_state, dynamic_depth)
            }
            Err(error) => {
                self.truncate_dynamic_own_value_bindings(dynamic_depth);
                self.restore_single_block_symbol_state(symbol, saved_state);
                Err(error)
            }
        }
    }

    fn resume_single_symbol_block_body(
        &mut self,
        symbol: Symbol,
        body: Expr,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
    ) -> Result<Expr, EvalError> {
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let saved_state = saved_state.clone();
            move |session, outcome| {
                session.finish_single_symbol_block_scope(
                    symbol,
                    saved_state,
                    dynamic_depth,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => {
                self.finish_single_symbol_block_scope(symbol, saved_state, dynamic_depth, outcome)
            }
        }
    }

    fn finish_single_symbol_block_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.restore_single_block_symbol_state(symbol, saved_state);
        outcome
    }

    fn restore_single_block_symbol_state(&mut self, symbol: Symbol, saved_state: BlockSymbolState) {
        let current_had_up_values = self
            .definitions
            .get(&symbol)
            .is_some_and(|definition| !definition.up_values.is_empty());
        let restored_has_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        match saved_state.definition {
            Some(definition) => {
                self.definitions.insert(symbol, definition);
            }
            None => {
                self.definitions.remove(&symbol);
            }
        }
        self.bump_evaluation_epoch_for_symbol(&symbol);

        if current_had_up_values || restored_has_up_values {
            self.refresh_has_up_value_definitions();
        }

        let current_attributes = self.attributes.get(&symbol).copied();
        if current_attributes != saved_state.attributes {
            match saved_state.attributes {
                Some(attributes) => {
                    self.attributes.insert(symbol, attributes);
                }
                None => {
                    self.attributes.remove(&symbol);
                }
            }
            self.bump_attribute_revision();
        }

        match saved_state.options {
            Some(options) => {
                self.options.insert(symbol, options);
            }
            None => {
                self.options.remove(&symbol);
            }
        }

        match saved_state.syntax_information {
            Some(syntax_information) => {
                self.syntax_information.insert(symbol, syntax_information);
            }
            None => {
                self.syntax_information.remove(&symbol);
            }
        }

        let current_message_count = self
            .message_symbol_counts
            .get(&symbol)
            .copied()
            .unwrap_or_default();
        if current_message_count > 0 || !saved_state.messages.is_empty() {
            self.message_definitions
                .retain(|key, _| key.symbol != symbol);
            for (key, definition) in saved_state.messages {
                self.message_definitions.insert(key, definition);
            }
            self.rebuild_message_symbol_counts();
        }
    }

    fn resume_block_body(
        &mut self,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
    ) -> Result<Expr, EvalError> {
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let saved_state = saved_state.clone();
            move |session, outcome| session.finish_block_scope(saved_state, dynamic_depth, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_block_scope(saved_state, dynamic_depth, outcome),
        }
    }

    fn finish_block_scope(
        &mut self,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
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

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}
