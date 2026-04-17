use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SymbolOptionEntry {
    name: Symbol,
    value: Expr,
    delayed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SymbolOptions {
    entries: Vec<SymbolOptionEntry>,
}

impl SymbolOptions {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(super) fn contains_key(&self, key: &Symbol) -> bool {
        self.entries.iter().any(|entry| entry.name == *key)
    }

    pub(super) fn contains_name(&self, key: &str) -> bool {
        self.entries.iter().any(|entry| entry.name.name() == key)
    }

    pub(super) fn get(&self, key: &Symbol) -> Option<&Expr> {
        self.entry(key).map(|entry| &entry.value)
    }

    pub(super) fn get_by_name(&self, key: &str) -> Option<&Expr> {
        self.entry_by_name(key).map(|entry| &entry.value)
    }

    fn entry(&self, key: &Symbol) -> Option<&SymbolOptionEntry> {
        self.entries.iter().find(|entry| entry.name == *key)
    }

    fn entry_by_name(&self, key: &str) -> Option<&SymbolOptionEntry> {
        self.entries.iter().find(|entry| entry.name.name() == key)
    }

    fn set_existing(&mut self, key: &Symbol, value: Expr, delayed: bool) -> bool {
        if let Some(current) = self.entries.iter_mut().find(|entry| entry.name == *key) {
            current.value = value;
            current.delayed = delayed;
            true
        } else {
            false
        }
    }

    fn set_existing_by_name(&mut self, key: &str, value: Expr, delayed: bool) -> bool {
        if let Some(current) = self
            .entries
            .iter_mut()
            .find(|entry| entry.name.name() == key)
        {
            current.value = value;
            current.delayed = delayed;
            true
        } else {
            false
        }
    }

    pub(super) fn from_entries(entries: Vec<SymbolOptionEntry>) -> Self {
        let mut normalized = Vec::new();

        for entry in entries {
            if let Some(existing) = normalized
                .iter_mut()
                .find(|existing: &&mut SymbolOptionEntry| existing.name == entry.name)
            {
                *existing = entry;
            } else {
                normalized.push(entry);
            }
        }

        Self {
            entries: normalized,
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&Symbol, &Expr, bool)> {
        self.entries
            .iter()
            .map(|entry| (&entry.name, &entry.value, entry.delayed))
    }

    pub(super) fn rule_exprs(&self) -> Vec<Expr> {
        self.entries
            .iter()
            .map(|entry| {
                Expr::call(
                    system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
                    vec![
                        Expr::Atom(Atom::Symbol(entry.name.clone())),
                        entry.value.clone(),
                    ],
                )
            })
            .collect()
    }

    pub(super) fn expr(&self) -> Expr {
        Expr::call(system_expr("List"), self.rule_exprs())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OptionValueScope {
    target: Symbol,
    explicit: SymbolOptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OptionValueQuery {
    Single { original: Expr, name: Symbol },
    Many(Vec<(Expr, Symbol)>),
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum OptionsPatternTarget {
    Automatic,
    Explicit(Symbol),
}

impl Session {
    pub(super) fn system_option_bool(
        &mut self,
        symbol_name: &str,
        option_name: &str,
        fallback: bool,
    ) -> Result<bool, EvalError> {
        let Some(value) = self
            .option_value_for_symbol(&Symbol::system(symbol_name), &Symbol::system(option_name))?
        else {
            return Ok(fallback);
        };

        Ok(is_system_symbol_name(&value, "True"))
    }

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

        let Some(targets) = self.normalize_symbol_target_exprs(&args[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                "SetOptions",
                args,
                OptionsNormalizationError::InvalidTarget.message(),
            ));
        };
        let preserve_target_list_shape =
            matches!(&args[0], Expr::Call { head, .. } if is_system_symbol_name(head, "List"));

        let tail = Expr::call(system_expr("List"), args[1..].to_vec());
        let original_args = args.to_vec();
        let resolved_result = self.evaluate(tail);
        let resolved = self.suspend_with_frame(resolved_result, {
            let targets = targets.clone();
            let original_args = original_args.clone();
            move |session, resolved| {
                session.finish_set_options_from_resolved(
                    targets,
                    preserve_target_list_shape,
                    original_args,
                    resolved,
                )
            }
        })?;

        self.finish_set_options_from_resolved(
            targets,
            preserve_target_list_shape,
            original_args,
            resolved,
        )
    }

    pub(super) fn eval_option_value(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [option_expr] => {
                let query = match self.parse_option_value_query(option_expr) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            "OptionValue",
                            args,
                            error.message(),
                        ));
                    }
                };
                let Some(scope) = self.option_value_scopes.last().cloned() else {
                    return Ok(self.invalid_call_with_arg_range("OptionValue", args, 2, 3));
                };

                self.resolve_option_value_query(scope.target, Some(scope.explicit), query)
            }
            [target_expr, option_expr] => {
                let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };
                let query = match self.parse_option_value_query(option_expr) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            "OptionValue",
                            args,
                            error.message(),
                        ));
                    }
                };

                self.resolve_option_value_query(target, None, query)
            }
            [target_expr, explicit_expr, option_expr] => {
                let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };
                let query = match self.parse_option_value_query(option_expr) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            "OptionValue",
                            args,
                            error.message(),
                        ));
                    }
                };

                let original_args = args.to_vec();
                let explicit_result = self.evaluate(explicit_expr.clone());
                let explicit = self.suspend_with_frame(explicit_result, {
                    let target = target.clone();
                    let query = query.clone();
                    let original_args = original_args.clone();
                    move |session, resolved| {
                        session.finish_option_value_from_explicit(
                            target,
                            query,
                            original_args,
                            resolved,
                        )
                    }
                })?;

                self.finish_option_value_from_explicit(target, query, original_args, explicit)
            }
            _ => Ok(self.invalid_call_with_arg_range("OptionValue", args, 1, 3)),
        }
    }

    pub(super) fn eval_arguments_options(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (call, spec, assoc) = match args {
            [call_expr, spec_expr] => (call_expr, spec_expr, None),
            [call_expr, spec_expr, assoc_expr] => (call_expr, spec_expr, Some(assoc_expr)),
            _ => return Ok(self.invalid_call_with_arg_range("ArgumentsOptions", args, 2, 3)),
        };

        let Some((head_symbol, call_args)) = self.arguments_options_target_call(call) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArgumentsOptions",
                args,
                "expected a held call expression whose head is a symbol",
            ));
        };

        let Some(spec) = parse_argument_count_spec(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArgumentsOptions",
                args,
                "expected an exact argument count or a {min, max} count range",
            ));
        };

        if let Some(assoc_expr) = assoc {
            if association_entries_from_expr(assoc_expr).is_none() {
                return Ok(self.invalid_call_with_arg_form(
                    "ArgumentsOptions",
                    args,
                    "expected an association as the optional third argument",
                ));
            }
        }

        Ok(
            match self.partition_arguments_and_options(&head_symbol, call, &call_args, spec) {
                ArgumentPartitionOutcome::Success {
                    positional,
                    options,
                } => Expr::call(
                    system_expr("List"),
                    vec![
                        Expr::call(system_expr("List"), positional),
                        Expr::call(system_expr("List"), options),
                    ],
                ),
                ArgumentPartitionOutcome::Failure { tag, details, text } => {
                    self.push_message(head_symbol.name(), tag, text);
                    build_argument_partition_failure(tag, details)
                }
            },
        )
    }

    pub(super) fn eval_check_arguments(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (call, spec, assoc) = match args {
            [call_expr, spec_expr] => (call_expr, spec_expr, None),
            [call_expr, spec_expr, assoc_expr] => (call_expr, spec_expr, Some(assoc_expr)),
            _ => return Ok(self.invalid_call_with_arg_range("CheckArguments", args, 2, 3)),
        };

        let Some((head_symbol, call_args)) = self.arguments_options_target_call(call) else {
            return Ok(self.invalid_call_with_arg_form(
                "CheckArguments",
                args,
                "expected a held call expression whose head is a symbol",
            ));
        };

        let Some(spec) = parse_argument_count_spec(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "CheckArguments",
                args,
                "expected an exact argument count or a {min, max} count range",
            ));
        };

        if let Some(assoc_expr) = assoc {
            if association_entries_from_expr(assoc_expr).is_none() {
                return Ok(self.invalid_call_with_arg_form(
                    "CheckArguments",
                    args,
                    "expected an association as the optional third argument",
                ));
            }
        }

        Ok(
            match self.partition_arguments_and_options(&head_symbol, call, &call_args, spec) {
                ArgumentPartitionOutcome::Success { .. } => system_expr("True"),
                ArgumentPartitionOutcome::Failure { tag, text, .. } => {
                    self.push_message(head_symbol.name(), tag, text);
                    system_expr("False")
                }
            },
        )
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

    pub(super) fn option_value_for_symbol(
        &mut self,
        symbol: &Symbol,
        option: &Symbol,
    ) -> Result<Option<Expr>, EvalError> {
        let options = self.options_for_symbol(symbol);
        let entry = options
            .entry(option)
            .or_else(|| options.entry_by_name(option.name()))
            .cloned();

        match entry {
            Some(entry) => self.resolve_option_entry_value(&entry).map(Some),
            None => Ok(None),
        }
    }

    pub(super) fn option_rule_expr_for_symbol(
        &mut self,
        symbol: &Symbol,
        option: &Symbol,
    ) -> Result<Option<Expr>, EvalError> {
        let options = self.options_for_symbol(symbol);
        let entry = options
            .entry(option)
            .or_else(|| options.entry_by_name(option.name()))
            .cloned();

        let Some(entry) = entry else {
            return Ok(None);
        };
        let value = self.resolve_option_entry_value(&entry)?;
        Ok(Some(Expr::call(
            system_expr("Rule"),
            vec![Expr::Atom(Atom::Symbol(option.clone())), value],
        )))
    }

    pub(super) fn option_rule_exprs_for_symbol(&self, symbol: &Symbol) -> Vec<Expr> {
        self.options_for_symbol(symbol).rule_exprs()
    }

    pub(super) fn merged_option_exprs_for_symbol(
        &mut self,
        symbol: &Symbol,
        explicit: &[Expr],
    ) -> Result<Vec<Expr>, EvalError> {
        let mut merged = Vec::new();
        for expr in self.option_rule_exprs_for_symbol(symbol) {
            merged.push(self.evaluate_runtime_option_expr(&expr)?);
        }
        for expr in explicit {
            merged.push(self.evaluate_runtime_option_expr(expr)?);
        }
        Ok(merged)
    }

    pub(super) fn merged_option_exprs_for_system_symbol(
        &mut self,
        symbol_name: &str,
        explicit: &[Expr],
    ) -> Result<Vec<Expr>, EvalError> {
        self.merged_option_exprs_for_symbol(&Symbol::system(symbol_name), explicit)
    }

    pub(super) fn compile_assigned_options_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Result<SymbolOptions, OptionsNormalizationError> {
        let entries = self.parse_option_entries_from_resolved(resolved)?;
        Ok(SymbolOptions::from_entries(entries))
    }

    fn resolve_option_entry_value(&mut self, entry: &SymbolOptionEntry) -> Result<Expr, EvalError> {
        if entry.delayed {
            self.evaluate(entry.value.clone())
        } else {
            Ok(entry.value.clone())
        }
    }

    fn evaluate_runtime_option_expr(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = expr else {
            return Ok(expr.clone());
        };

        if is_system_symbol_name(head, "RuleDelayed") && args.len() == 2 {
            let value = self.evaluate(args[1].clone())?;
            Ok(Expr::call(
                system_expr("Rule"),
                vec![args[0].clone(), value],
            ))
        } else {
            Ok(expr.clone())
        }
    }

    pub(super) fn evaluate_with_option_value_scope(
        &mut self,
        scope: Option<OptionValueScope>,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(scope) = scope else {
            return self.evaluate(expr);
        };

        self.option_value_scopes.push(scope);

        match self.evaluate(expr) {
            Ok(value) => {
                let _ = self.option_value_scopes.pop();
                Ok(value)
            }
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(|session, outcome| {
                    let _ = session.option_value_scopes.pop();
                    outcome
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => {
                let _ = self.option_value_scopes.pop();
                Err(error)
            }
        }
    }

    pub(super) fn option_value_scope_for_definition(
        &self,
        lhs: &Expr,
        candidate: &Expr,
    ) -> Option<OptionValueScope> {
        let Expr::Call { args: lhs_args, .. } = lhs else {
            return None;
        };
        let Expr::Call {
            args: candidate_args,
            ..
        } = candidate
        else {
            return None;
        };

        let automatic_target = automatic_options_pattern_target(candidate)?;
        let mut matched = None;

        for (index, arg) in lhs_args.iter().enumerate() {
            let Some(target) = self.options_pattern_target_for_arg(arg) else {
                continue;
            };
            if matched.is_some() {
                return None;
            }
            matched = Some((index, target));
        }

        let (index, target) = matched?;
        if index + 1 != lhs_args.len() || index > candidate_args.len() {
            return None;
        }

        let target = match target {
            OptionsPatternTarget::Automatic => automatic_target,
            OptionsPatternTarget::Explicit(symbol) => symbol,
        };
        let explicit_expr = Expr::call(system_expr("List"), candidate_args[index..].to_vec());
        let explicit = self
            .compile_assigned_options_from_resolved(&explicit_expr)
            .ok()?;

        Some(OptionValueScope { target, explicit })
    }

    fn finish_set_options_from_resolved(
        &mut self,
        targets: Vec<Symbol>,
        preserve_target_list_shape: bool,
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

        let results = targets
            .into_iter()
            .map(|target| self.apply_set_options_updates(&target, &original_args, &updates))
            .collect::<Vec<_>>();

        if preserve_target_list_shape {
            Ok(Expr::call(system_expr("List"), results))
        } else {
            Ok(results
                .into_iter()
                .next()
                .unwrap_or_else(|| self.invalid_call("SetOptions", &original_args)))
        }
    }

    fn apply_set_options_updates(
        &mut self,
        target: &Symbol,
        original_args: &[Expr],
        updates: &[SymbolOptionEntry],
    ) -> Expr {
        let mut current = self.options_for_symbol(target);

        for update in updates {
            if !current.contains_key(&update.name) && !current.contains_name(update.name.name()) {
                self.push_message(
                    "SetOptions",
                    "optnf",
                    format!("{} is not a known option for {target}.", update.name),
                );
                return self.invalid_call("SetOptions", original_args);
            }
        }

        for update in updates {
            let replaced = current.set_existing(&update.name, update.value.clone(), update.delayed);
            if !replaced {
                let _ = current.set_existing_by_name(
                    update.name.name(),
                    update.value.clone(),
                    update.delayed,
                );
            }
        }

        self.set_options_for_symbol(target.clone(), current.clone());
        current.expr()
    }

    fn finish_option_value_from_explicit(
        &mut self,
        target: Symbol,
        query: OptionValueQuery,
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

        self.resolve_option_value_query(target, Some(explicit), query)
    }

    fn resolve_option_value_query(
        &mut self,
        target: Symbol,
        explicit: Option<SymbolOptions>,
        query: OptionValueQuery,
    ) -> Result<Expr, EvalError> {
        match query {
            OptionValueQuery::Single { original, name } => {
                self.resolve_option_value_scalar(target, explicit, original, name)
            }
            OptionValueQuery::Many(items) => {
                let mut values = Vec::with_capacity(items.len());
                for (original, name) in items {
                    values.push(self.resolve_option_value_scalar(
                        target.clone(),
                        explicit.clone(),
                        original,
                        name,
                    )?);
                }
                Ok(Expr::call(system_expr("List"), values))
            }
        }
    }

    fn resolve_option_value_scalar(
        &mut self,
        target: Symbol,
        explicit: Option<SymbolOptions>,
        original_option_expr: Expr,
        option_name: Symbol,
    ) -> Result<Expr, EvalError> {
        let defaults = self.options_for_symbol(&target);

        if let Some(explicit) = explicit {
            if let Some(entry) = explicit
                .entry(&option_name)
                .or_else(|| explicit.entry_by_name(option_name.name()))
                .cloned()
            {
                if !defaults.contains_key(&option_name)
                    && !defaults.contains_name(option_name.name())
                {
                    self.push_message(
                        "OptionValue",
                        "nodef",
                        format!("Unknown option {option_name} for {target}."),
                    );
                }
                return self.resolve_option_entry_value(&entry);
            }
        }

        if let Some(entry) = defaults
            .entry(&option_name)
            .or_else(|| defaults.entry_by_name(option_name.name()))
            .cloned()
        {
            self.resolve_option_entry_value(&entry)
        } else {
            self.push_message(
                "OptionValue",
                "optnf",
                format!("Option name {option_name} not found in defaults for {target}."),
            );
            Ok(original_option_expr)
        }
    }

    fn parse_option_value_query(
        &self,
        expr: &Expr,
    ) -> Result<OptionValueQuery, OptionsNormalizationError> {
        if let Some(name) = self.normalize_symbol_target_expr(expr, true) {
            return Ok(OptionValueQuery::Single {
                original: expr.clone(),
                name,
            });
        }

        let Expr::Call { head, args } = expr else {
            return Err(OptionsNormalizationError::InvalidOptionName);
        };

        if !is_system_symbol_name(head, "List") {
            return Err(OptionsNormalizationError::InvalidOptionName);
        }

        let mut names = Vec::with_capacity(args.len());

        for item in args {
            let Some(name) = self.normalize_symbol_target_expr(item, true) else {
                return Err(OptionsNormalizationError::InvalidOptionName);
            };
            names.push((item.clone(), name));
        }

        Ok(OptionValueQuery::Many(names))
    }

    fn parse_option_entries_from_resolved_set_options(
        &self,
        expr: &Expr,
    ) -> Result<Vec<SymbolOptionEntry>, OptionsNormalizationError> {
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
    ) -> Result<Vec<SymbolOptionEntry>, OptionsNormalizationError> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                let mut entries = Vec::new();
                for item in args {
                    entries.extend(self.parse_option_entries_from_resolved(item)?);
                }
                Ok(entries)
            }
            Expr::Call { head, args }
                if (is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                let Some(name) = self.normalize_symbol_target_expr(&args[0], true) else {
                    return Err(OptionsNormalizationError::InvalidRules);
                };
                Ok(vec![SymbolOptionEntry {
                    name,
                    value: args[1].clone(),
                    delayed: is_system_symbol_name(head, "RuleDelayed"),
                }])
            }
            _ => Err(OptionsNormalizationError::InvalidRules),
        }
    }

    fn options_pattern_target_for_arg(&self, expr: &Expr) -> Option<OptionsPatternTarget> {
        let expr = strip_hold_pattern(expr);

        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Pattern") && args.len() == 2 =>
            {
                self.options_pattern_target_for_arg(&args[1])
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Optional") && !args.is_empty() =>
            {
                self.options_pattern_target_for_arg(&args[0])
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "OptionsPattern") => {
                match args.as_slice() {
                    [] => Some(OptionsPatternTarget::Automatic),
                    [target] => self
                        .normalize_symbol_target_expr(target, true)
                        .map(OptionsPatternTarget::Explicit),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn arguments_options_target_call(&self, expr: &Expr) -> Option<(Symbol, Vec<Expr>)> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        Some((expr_as_symbol(head)?.clone(), args.clone()))
    }

    fn partition_arguments_and_options(
        &self,
        head_symbol: &Symbol,
        held_call: &Expr,
        call_args: &[Expr],
        spec: ArgumentCountSpec,
    ) -> ArgumentPartitionOutcome {
        let known_options = self.options_for_symbol(head_symbol);

        if call_args.len() < spec.min() {
            return ArgumentPartitionOutcome::Failure {
                tag: spec.too_few_message_tag(),
                details: build_argument_count_failure_details(
                    head_symbol,
                    held_call.clone(),
                    call_args.len(),
                    spec,
                ),
                text: render_argument_count_failure_message(head_symbol, call_args.len(), spec),
            };
        }

        let max_count = spec.max().min(call_args.len());
        for positional_count in spec.min()..=max_count {
            let positional = call_args[..positional_count].to_vec();
            match classify_option_suffix(&call_args[positional_count..], &known_options) {
                OptionSuffixClassification::Valid(options) => {
                    return ArgumentPartitionOutcome::Success {
                        positional,
                        options,
                    };
                }
                OptionSuffixClassification::UnknownOption(option_name) => {
                    return ArgumentPartitionOutcome::Failure {
                        tag: "optx",
                        details: build_unknown_option_failure_details(
                            head_symbol,
                            held_call.clone(),
                            &option_name,
                        ),
                        text: render_unknown_option_failure_message(
                            head_symbol,
                            &option_name,
                            held_call,
                        ),
                    };
                }
                OptionSuffixClassification::NotOptionSyntax => {}
            }
        }

        let offending = call_args
            .last()
            .cloned()
            .unwrap_or_else(|| system_expr("Null"));
        let max_allowed = spec.max();

        ArgumentPartitionOutcome::Failure {
            tag: "nonopt",
            details: build_non_option_failure_details(
                head_symbol,
                held_call.clone(),
                offending.clone(),
                max_allowed,
            ),
            text: render_non_option_failure_message(
                head_symbol,
                &offending,
                max_allowed,
                held_call,
            ),
        }
    }
}

pub(super) fn options_pattern_sequence_matcher() -> CompiledSequencePattern {
    let option_rule_matcher = |head_name: &str| MatcherNode::Call {
        head: Box::new(MatcherNode::Exact(system_expr(head_name))),
        args: vec![
            MatcherArg::Single(MatcherNode::Blank {
                head_constraint: None,
            }),
            MatcherArg::Single(MatcherNode::Blank {
                head_constraint: None,
            }),
        ],
    };

    CompiledSequencePattern {
        min_len: 0,
        max_len: None,
        matcher: MatcherNode::Alternatives(vec![
            option_rule_matcher("Rule"),
            option_rule_matcher("RuleDelayed"),
        ]),
        name: None,
        checks: Vec::new(),
    }
}

pub(super) fn is_options_pattern_expr(expr: &Expr) -> bool {
    let expr = strip_hold_pattern(expr);

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            is_options_pattern_expr(&args[1])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Optional") && !args.is_empty() =>
        {
            is_options_pattern_expr(&args[0])
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "OptionsPattern") => {
            args.len() <= 1
        }
        _ => false,
    }
}

fn automatic_options_pattern_target(candidate: &Expr) -> Option<Symbol> {
    let Expr::Call { head, .. } = candidate else {
        return None;
    };

    base_head_symbol_of_nested_call(head)
        .cloned()
        .or_else(|| expr_as_symbol(head).cloned())
}

fn system_rule_option(name: &str, value: Expr) -> SymbolOptionEntry {
    SymbolOptionEntry {
        name: Symbol::system(name),
        value,
        delayed: false,
    }
}

fn bool_option(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgumentCountSpec {
    Exact(usize),
    Range { min: usize, max: usize },
}

impl ArgumentCountSpec {
    fn min(self) -> usize {
        match self {
            Self::Exact(count) => count,
            Self::Range { min, .. } => min,
        }
    }

    fn max(self) -> usize {
        match self {
            Self::Exact(count) => count,
            Self::Range { max, .. } => max,
        }
    }

    fn too_few_message_tag(self) -> &'static str {
        match self {
            Self::Exact(_) => "argx",
            Self::Range { .. } => "argt",
        }
    }
}

enum OptionSuffixClassification {
    Valid(Vec<Expr>),
    UnknownOption(Symbol),
    NotOptionSyntax,
}

enum ArgumentPartitionOutcome {
    Success {
        positional: Vec<Expr>,
        options: Vec<Expr>,
    },
    Failure {
        tag: &'static str,
        details: Expr,
        text: String,
    },
}

fn parse_argument_count_spec(expr: &Expr) -> Option<ArgumentCountSpec> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some(ArgumentCountSpec::Exact(count));
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    match items {
        [min, max] => {
            let min = expr_as_non_negative_usize(min)?;
            let max = expr_as_non_negative_usize(max)?;
            (min <= max).then_some(ArgumentCountSpec::Range { min, max })
        }
        _ => None,
    }
}

fn classify_option_suffix(
    args: &[Expr],
    known_options: &SymbolOptions,
) -> OptionSuffixClassification {
    let mut options = Vec::new();

    for arg in args {
        match classify_option_expr(arg, known_options) {
            Some(Ok(mut entries)) => options.append(&mut entries),
            Some(Err(option_name)) => {
                return OptionSuffixClassification::UnknownOption(option_name);
            }
            None => return OptionSuffixClassification::NotOptionSyntax,
        }
    }

    OptionSuffixClassification::Valid(options)
}

fn classify_option_expr(
    expr: &Expr,
    known_options: &SymbolOptions,
) -> Option<Result<Vec<Expr>, Symbol>> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let name = match &args[0] {
                Expr::Atom(Atom::Symbol(symbol)) => symbol.clone(),
                Expr::Atom(Atom::String(name)) => resolve_unqualified_symbol(name)?,
                _ => return None,
            };

            if known_options.contains_key(&name) || known_options.contains_name(name.name()) {
                Some(Ok(vec![expr.clone()]))
            } else {
                Some(Err(name))
            }
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut flattened = Vec::new();
            for item in args {
                match classify_option_expr(item, known_options)? {
                    Ok(mut entries) => flattened.append(&mut entries),
                    Err(option_name) => return Some(Err(option_name)),
                }
            }
            Some(Ok(flattened))
        }
        _ => None,
    }
}

fn hold_form_expr(expr: Expr) -> Expr {
    Expr::call(system_expr("HoldForm"), vec![expr])
}

fn held_expr(expr: Expr) -> Expr {
    Expr::call(system_expr("Hold"), vec![expr])
}

fn build_argument_partition_failure(tag: &str, details: Expr) -> Expr {
    Expr::call(system_expr("Failure"), vec![Expr::string(tag), details])
}

fn build_argument_count_failure_details(
    symbol: &Symbol,
    held_input: Expr,
    count: usize,
    spec: ArgumentCountSpec,
) -> Expr {
    let mut entries = vec![
        (
            Expr::string("MessageTemplate"),
            message_name_expr(symbol.clone(), spec.too_few_message_tag().to_string()),
        ),
        (Expr::string("HeldInput"), held_expr(held_input.clone())),
    ];

    let params = match spec {
        ArgumentCountSpec::Exact(expected) => vec![
            hold_form_expr(Expr::Atom(Atom::Symbol(symbol.clone()))),
            hold_form_expr(Expr::integer(BigInt::from(count))),
            hold_form_expr(Expr::integer(BigInt::from(expected))),
        ],
        ArgumentCountSpec::Range { min, max } => vec![
            hold_form_expr(Expr::Atom(Atom::Symbol(symbol.clone()))),
            hold_form_expr(Expr::integer(BigInt::from(count))),
            hold_form_expr(Expr::integer(BigInt::from(min))),
            hold_form_expr(Expr::integer(BigInt::from(max))),
        ],
    };

    entries.push((
        Expr::string("MessageParameters"),
        Expr::call(system_expr("List"), params),
    ));
    render_association(entries)
}

fn build_non_option_failure_details(
    symbol: &Symbol,
    held_input: Expr,
    offender: Expr,
    positional_count: usize,
) -> Expr {
    render_association(vec![
        (
            Expr::string("MessageTemplate"),
            message_name_expr(symbol.clone(), "nonopt".to_string()),
        ),
        (
            Expr::string("MessageParameters"),
            Expr::call(
                system_expr("List"),
                vec![
                    hold_form_expr(offender),
                    hold_form_expr(Expr::integer(BigInt::from(positional_count))),
                    hold_form_expr(held_input.clone()),
                ],
            ),
        ),
        (Expr::string("HeldInput"), held_expr(held_input)),
    ])
}

fn build_unknown_option_failure_details(
    symbol: &Symbol,
    held_input: Expr,
    option_name: &Symbol,
) -> Expr {
    render_association(vec![
        (
            Expr::string("MessageTemplate"),
            message_name_expr(symbol.clone(), "optx".to_string()),
        ),
        (
            Expr::string("MessageParameters"),
            Expr::call(
                system_expr("List"),
                vec![
                    hold_form_expr(Expr::Atom(Atom::Symbol(option_name.clone()))),
                    hold_form_expr(held_input.clone()),
                ],
            ),
        ),
        (Expr::string("HeldInput"), held_expr(held_input)),
    ])
}

fn render_argument_count_failure_message(
    symbol: &Symbol,
    count: usize,
    spec: ArgumentCountSpec,
) -> String {
    match spec {
        ArgumentCountSpec::Exact(expected) => {
            let noun = if expected == 1 {
                "argument is"
            } else {
                "arguments are"
            };
            format!("{symbol} called with {count} arguments; {expected} {noun} expected.")
        }
        ArgumentCountSpec::Range { min, max } => format!(
            "{symbol} called with {count} arguments; {min} or {max} arguments are expected."
        ),
    }
}

fn render_non_option_failure_message(
    _symbol: &Symbol,
    offender: &Expr,
    positional_count: usize,
    held_input: &Expr,
) -> String {
    format!(
        "Options expected (instead of {offender}) beyond position {positional_count} in {held_input}. An option must be a rule or a list of rules."
    )
}

fn render_unknown_option_failure_message(
    _symbol: &Symbol,
    option_name: &Symbol,
    held_input: &Expr,
) -> String {
    format!("Unknown option {option_name} in {held_input}.")
}

pub(super) fn default_system_options(symbol: &Symbol) -> SymbolOptions {
    if symbol.context() != SYSTEM_CONTEXT {
        return SymbolOptions::default();
    }

    let entries = match symbol.name() {
        "ArrayFilter" => vec![system_rule_option("Padding", Expr::string("Fixed"))],
        "Cases" | "Count" | "MemberQ" | "Level" | "Depth" => {
            vec![system_rule_option("Heads", bool_option(false))]
        }
        "FreeQ" | "Position" => vec![system_rule_option("Heads", bool_option(true))],
        "CheckAbort" => vec![system_rule_option(
            "PropagateAborts",
            system_expr("Automatic"),
        )],
        "Monitor" => vec![
            system_rule_option("EvaluationMonitor", system_expr("None")),
            system_rule_option("StepMonitor", system_expr("None")),
        ],
        "Dialog" => vec![
            system_rule_option("DialogProlog", system_expr("Null")),
            system_rule_option("DialogSymbols", Expr::call(system_expr("List"), Vec::new())),
        ],
        "TestReport" => vec![
            system_rule_option("SameTest", system_expr("SameQ")),
            system_rule_option("TimeConstraint", system_expr("Infinity")),
            system_rule_option("MemoryConstraint", system_expr("Infinity")),
        ],
        "Save" => vec![
            system_rule_option("ExcludedContexts", system_expr("Automatic")),
            system_rule_option("IncludedContexts", system_expr("All")),
        ],
        "DumpSave" => vec![
            system_rule_option(
                "CompressionLevel",
                Expr::Atom(Atom::Integer(BigInt::from(1))),
            ),
            system_rule_option("SymbolAttributes", bool_option(true)),
        ],
        "Once" => vec![
            system_rule_option("ExpirationDate", system_expr("None")),
            system_rule_option("PersistenceTime", system_expr("None")),
        ],
        "StringContainsQ" | "StringStartsQ" | "StringEndsQ" | "StringMatchQ" | "StringFreeQ"
        | "StringDelete" | "StringReplace" | "StringCases" | "StringPosition" => {
            vec![system_rule_option("IgnoreCase", bool_option(false))]
        }
        "StringCount" => vec![
            system_rule_option("IgnoreCase", bool_option(false)),
            system_rule_option("Overlaps", bool_option(false)),
        ],
        _ => Vec::new(),
    };

    SymbolOptions::from_entries(entries)
}
