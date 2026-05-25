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

fn unique_string_prefix_parts(prefix: &str, current_context: &str) -> Option<(String, String)> {
    let (context, name_prefix) = if let Some((context, name_prefix)) = split_symbol_name(prefix) {
        if !is_valid_unique_context(context) {
            return None;
        }
        (normalize_context_name(context), name_prefix)
    } else {
        (current_context.to_string(), prefix)
    };

    is_valid_unique_symbol_prefix(name_prefix).then(|| (context, name_prefix.to_string()))
}

fn is_valid_unique_context(context: &str) -> bool {
    context
        .strip_suffix('`')
        .is_some_and(|body| !body.is_empty() && body.split('`').all(is_valid_unique_context_part))
}

fn is_valid_unique_context_part(part: &str) -> bool {
    is_valid_unique_symbol_prefix(part)
}

fn is_valid_unique_symbol_prefix(prefix: &str) -> bool {
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '$')
}

fn generated_symbol_candidate(
    context: &str,
    name_prefix: &str,
    separator: &str,
    id: u64,
) -> Symbol {
    Symbol::new(context.to_string(), format!("{name_prefix}{separator}{id}"))
}

impl Session {
    pub(super) fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_unique(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (target, attributes) = match args {
            [] => (None, AttributeSet::empty()),
            [target] => (Some(target), AttributeSet::empty()),
            [target, attributes] => {
                let Some(attributes) = parse_attribute_set_expr(attributes) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Unique",
                        args,
                        "expected an attribute symbol or list of attribute symbols as the second argument",
                    ));
                };
                (Some(target), attributes)
            }
            _ => {
                return Ok(self.invalid_call_with_arg_range("Unique", args, 0, 2));
            }
        };

        let generated = match target {
            None => self
                .allocate_unique_string_symbol("$")
                .map(|symbol| Expr::Atom(Atom::Symbol(symbol))),
            Some(target) => self.allocate_unique_expr(target),
        };

        let Some(generated) = generated else {
            let offender = target.expect("invalid Unique target should have a target argument");
            return Ok(self.invalid_unique_arg(args, offender));
        };

        self.register_unique_generated_expr(&generated, attributes);
        Ok(generated)
    }

    pub(super) fn eval_block(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

        self.resume_block_initializers(items, args[1].clone(), saved_state, 0)
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
    }

    pub(super) fn try_apply_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let applied = if let Some(function) = parse_function(head) {
            match function {
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
            }
        } else if let Some(composition) = parse_composition(head) {
            apply_composition_to_args(&composition, args)
        } else if let Some(status) = self.apply_status_object_to_args(head, args)? {
            status
        } else if let Some(value) = apply_association_to_args(head, args) {
            value
        } else if let Some(invalid_key_application) =
            self.invalid_callable_key_application(head, args)
        {
            return Ok(Some(invalid_key_application));
        } else if let Some(wrapper) = apply_callable_operator_wrapper_to_args(head, args) {
            wrapper
        } else if let Some(symbol) = invalid_callable_operator_wrapper_argx_symbol(head, args) {
            self.push_message(
                symbol,
                "argx",
                format!(
                    "{head} called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return Ok(Some(Expr::call(head.clone(), args.to_vec())));
        } else if let Some(wrapper) = apply_callable_failsafe_wrapper_to_args(head, args) {
            wrapper
        } else {
            return Ok(None);
        };

        let applied_result = self.evaluate(applied);
        let outcome = self.suspend_with_result_frame(applied_result, |session, outcome| {
            session.finish_function_application(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_function_application(outcome).map(Some),
        }
    }

    fn finish_function_application(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        }
    }

    fn invalid_callable_key_application(&mut self, head: &Expr, args: &[Expr]) -> Option<Expr> {
        let Expr::Call {
            head: key_head,
            args: key_args,
        } = head
        else {
            return None;
        };
        if !is_system_symbol_name(key_head, "Key") || key_args.len() != 1 {
            return None;
        }

        let held = Expr::call(head.clone(), args.to_vec());
        let [expr] = args else {
            self.push_message(
                "Key",
                "argx",
                format!(
                    "{head} called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return Some(held);
        };

        if association_entries_from_expr(expr).is_none() {
            self.push_message("Key", "invak", invalid_association_message(expr));
            return Some(held);
        }

        None
    }

    pub(super) fn allocate_fresh_symbol(&mut self, symbol: &Symbol) -> Symbol {
        self.allocate_generated_symbol(symbol.context(), symbol.name(), "$")
    }

    fn allocate_unique_string_symbol(&mut self, prefix: &str) -> Option<Symbol> {
        let (context, name_prefix) = unique_string_prefix_parts(prefix, &self.current_context)?;
        Some(self.allocate_generated_symbol(&context, &name_prefix, ""))
    }

    fn allocate_unique_expr(&mut self, expr: &Expr) -> Option<Expr> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                Some(Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))))
            }
            Expr::Atom(Atom::String(prefix)) => self
                .allocate_unique_string_symbol(prefix)
                .map(|symbol| Expr::Atom(Atom::Symbol(symbol))),
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                if args.is_empty() {
                    return Some(Expr::call(system_expr("List"), Vec::new()));
                }

                if let Some(symbols) = args.iter().map(expr_as_symbol).collect::<Option<Vec<_>>>() {
                    let generated = self.allocate_fresh_symbol_batch(&symbols);
                    return Some(Expr::call(
                        system_expr("List"),
                        generated
                            .into_iter()
                            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
                            .collect(),
                    ));
                }

                Some(Expr::call(
                    system_expr("List"),
                    args.iter()
                        .map(|arg| self.allocate_unique_expr(arg))
                        .collect::<Option<Vec<_>>>()?,
                ))
            }
            _ => None,
        }
    }

    fn allocate_fresh_symbol_batch(&mut self, symbols: &[&Symbol]) -> Vec<Symbol> {
        loop {
            self.next_generated_symbol_id += 1;
            let candidates = symbols
                .iter()
                .map(|symbol| {
                    generated_symbol_candidate(
                        symbol.context(),
                        symbol.name(),
                        "$",
                        self.next_generated_symbol_id,
                    )
                })
                .collect::<Vec<_>>();

            if candidates
                .iter()
                .all(|candidate| !self.generated_symbol_exists(candidate))
            {
                return candidates;
            }
        }
    }

    fn allocate_generated_symbol(
        &mut self,
        context: &str,
        name_prefix: &str,
        separator: &str,
    ) -> Symbol {
        loop {
            self.next_generated_symbol_id += 1;
            let candidate = generated_symbol_candidate(
                context,
                name_prefix,
                separator,
                self.next_generated_symbol_id,
            );

            if !self.generated_symbol_exists(&candidate) {
                return candidate;
            }
        }
    }

    fn register_unique_generated_expr(&mut self, expr: &Expr, attributes: AttributeSet) {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                self.definitions.entry(symbol.clone()).or_default();
                if !attributes.is_empty() {
                    let mut current = self.attribute_set_for_symbol(symbol);
                    for attribute in attributes.attributes() {
                        current.insert(attribute);
                    }
                    self.set_attribute_set_for_symbol(symbol.clone(), current);
                }
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                for arg in args {
                    self.register_unique_generated_expr(arg, attributes);
                }
            }
            _ => {}
        }
    }

    pub(super) fn generated_symbol_exists(&self, symbol: &Symbol) -> bool {
        self.symbol_exists(symbol)
            || self
                .message_definitions
                .keys()
                .any(|key| key.symbol == *symbol)
    }

    fn invalid_unique_arg(&mut self, args: &[Expr], arg: &Expr) -> Expr {
        self.invalid_call_with_message(
            "Unique",
            args,
            "usym",
            format!(
                "{} is not a symbol or a valid symbol name.",
                render_input_like_expr(arg)
            ),
        )
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
