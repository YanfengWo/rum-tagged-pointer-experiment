use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SymbolOptions {
    entries: Vec<(Symbol, Expr)>,
}

impl SymbolOptions {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(super) fn contains_key(&self, key: &Symbol) -> bool {
        self.entries.iter().any(|(name, _)| name == key)
    }

    pub(super) fn get(&self, key: &Symbol) -> Option<&Expr> {
        self.entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    pub(super) fn set_existing(&mut self, key: &Symbol, value: Expr) -> bool {
        if let Some((_, current)) = self.entries.iter_mut().find(|(name, _)| name == key) {
            *current = value;
            true
        } else {
            false
        }
    }

    pub(super) fn from_entries(entries: Vec<(Symbol, Expr)>) -> Self {
        let mut normalized = Vec::new();

        for (name, value) in entries {
            if let Some((_, existing)) = normalized
                .iter_mut()
                .find(|(existing, _)| *existing == name)
            {
                *existing = value;
            } else {
                normalized.push((name, value));
            }
        }

        Self {
            entries: normalized,
        }
    }

    pub(super) fn expr(&self) -> Expr {
        Expr::call(
            system_expr("List"),
            self.entries
                .iter()
                .map(|(name, value)| {
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::Atom(Atom::Symbol(name.clone())), value.clone()],
                    )
                })
                .collect(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OptionsNormalizationError {
    InvalidTarget,
    InvalidRules,
    InvalidOptionName,
}

impl OptionsNormalizationError {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::InvalidTarget => "expected a symbol or symbol-name string argument",
            Self::InvalidRules => {
                "expected a rule or list of rules using symbol or symbol-name-string option names"
            }
            Self::InvalidOptionName => "expected an option symbol or symbol-name string",
        }
    }
}

impl Session {
    pub(super) fn eval_options(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target] => {
                let Some(symbol) = self.normalize_symbol_target_expr(target, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Options",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };

                Ok(self.options_for_symbol(&symbol).expr())
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("Options", args, 1)),
        }
    }

    pub(super) fn eval_set_options(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("SetOptions", args, 2, usize::MAX));
        }

        let Some(target) = self.normalize_symbol_target_expr(&args[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                "SetOptions",
                args,
                OptionsNormalizationError::InvalidTarget.message(),
            ));
        };

        let tail = Expr::call(system_expr("List"), args[1..].to_vec());
        let original_args = args.to_vec();
        let resolved_result = self.evaluate(tail);
        let resolved = self.suspend_with_frame(resolved_result, {
            let target = target.clone();
            let original_args = original_args.clone();
            move |session, resolved| {
                session.finish_set_options_from_resolved(target, original_args, resolved)
            }
        })?;

        self.finish_set_options_from_resolved(target, original_args, resolved)
    }

    pub(super) fn eval_option_value(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target_expr, option_expr] => {
                let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };
                let Some(option_name) = self.normalize_symbol_target_expr(option_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidOptionName.message(),
                    ));
                };

                Ok(self.resolve_option_value(target, None, option_expr.clone(), option_name))
            }
            [target_expr, explicit_expr, option_expr] => {
                let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };
                let Some(option_name) = self.normalize_symbol_target_expr(option_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidOptionName.message(),
                    ));
                };

                let original_args = args.to_vec();
                let explicit_result = self.evaluate(explicit_expr.clone());
                let explicit = self.suspend_with_frame(explicit_result, {
                    let target = target.clone();
                    let option_name = option_name.clone();
                    let option_expr = option_expr.clone();
                    let original_args = original_args.clone();
                    move |session, resolved| {
                        session.finish_option_value_from_explicit(
                            target,
                            option_name,
                            option_expr,
                            original_args,
                            resolved,
                        )
                    }
                })?;

                self.finish_option_value_from_explicit(
                    target,
                    option_name,
                    option_expr.clone(),
                    original_args,
                    explicit,
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("OptionValue", args, 2, 3)),
        }
    }

    pub(super) fn options_for_symbol(&self, symbol: &Symbol) -> SymbolOptions {
        self.options
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| default_system_options(symbol))
    }

    pub(super) fn set_options_for_symbol(&mut self, symbol: Symbol, options: SymbolOptions) {
        let default = default_system_options(&symbol);

        if options == default {
            self.options.remove(&symbol);
        } else {
            self.options.insert(symbol, options);
        }
    }

    pub(super) fn compile_assigned_options_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Result<SymbolOptions, OptionsNormalizationError> {
        let entries = self.parse_option_entries_from_resolved(resolved)?;
        Ok(SymbolOptions::from_entries(entries))
    }

    fn finish_set_options_from_resolved(
        &mut self,
        target: Symbol,
        original_args: Vec<Expr>,
        resolved: Expr,
    ) -> Result<Expr, EvalError> {
        let updates = match self.parse_option_entries_from_resolved_set_options(&resolved) {
            Ok(updates) => updates,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "SetOptions",
                    &original_args,
                    error.message(),
                ));
            }
        };

        let mut current = self.options_for_symbol(&target);

        for (name, _) in &updates {
            if !current.contains_key(name) {
                self.push_message(
                    "SetOptions",
                    "optnf",
                    format!("{name} is not a known option for {target}."),
                );
                return Ok(self.invalid_call("SetOptions", &original_args));
            }
        }

        for (name, value) in updates {
            let _ = current.set_existing(&name, value);
        }

        self.set_options_for_symbol(target, current.clone());
        Ok(current.expr())
    }

    fn finish_option_value_from_explicit(
        &mut self,
        target: Symbol,
        option_name: Symbol,
        original_option_expr: Expr,
        original_args: Vec<Expr>,
        explicit_resolved: Expr,
    ) -> Result<Expr, EvalError> {
        let explicit = match self.parse_option_entries_from_resolved(&explicit_resolved) {
            Ok(entries) => SymbolOptions::from_entries(entries),
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "OptionValue",
                    &original_args,
                    error.message(),
                ));
            }
        };

        Ok(self.resolve_option_value(target, Some(explicit), original_option_expr, option_name))
    }

    fn resolve_option_value(
        &mut self,
        target: Symbol,
        explicit: Option<SymbolOptions>,
        original_option_expr: Expr,
        option_name: Symbol,
    ) -> Expr {
        let defaults = self.options_for_symbol(&target);

        if let Some(explicit) = explicit {
            if let Some(value) = explicit.get(&option_name).cloned() {
                if !defaults.contains_key(&option_name) {
                    self.push_message(
                        "OptionValue",
                        "nodef",
                        format!("Unknown option {option_name} for {target}."),
                    );
                }
                return value;
            }
        }

        if let Some(value) = defaults.get(&option_name).cloned() {
            value
        } else {
            self.push_message(
                "OptionValue",
                "optnf",
                format!("Option name {option_name} not found in defaults for {target}."),
            );
            original_option_expr
        }
    }

    fn parse_option_entries_from_resolved_set_options(
        &self,
        expr: &Expr,
    ) -> Result<Vec<(Symbol, Expr)>, OptionsNormalizationError> {
        let Expr::Call { head, args } = expr else {
            return Err(OptionsNormalizationError::InvalidRules);
        };

        if !is_system_symbol_name(head, "List") {
            return Err(OptionsNormalizationError::InvalidRules);
        }

        let mut entries = Vec::new();

        for item in args {
            entries.extend(self.parse_option_entries_from_resolved(item)?);
        }

        if entries.is_empty() {
            return Err(OptionsNormalizationError::InvalidRules);
        }

        Ok(entries)
    }

    fn parse_option_entries_from_resolved(
        &self,
        expr: &Expr,
    ) -> Result<Vec<(Symbol, Expr)>, OptionsNormalizationError> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                let mut entries = Vec::new();
                for item in args {
                    entries.extend(self.parse_option_entries_from_resolved(item)?);
                }
                Ok(entries)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
                let Some(name) = self.normalize_symbol_target_expr(&args[0], true) else {
                    return Err(OptionsNormalizationError::InvalidRules);
                };
                Ok(vec![(name, args[1].clone())])
            }
            _ => Err(OptionsNormalizationError::InvalidRules),
        }
    }
}

pub(super) fn default_system_options(_symbol: &Symbol) -> SymbolOptions {
    SymbolOptions::default()
}
