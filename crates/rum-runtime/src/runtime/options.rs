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
            if let Some((_, existing_value)) = normalized
                .iter_mut()
                .find(|(existing_name, _)| *existing_name == name)
            {
                *existing_value = value;
            } else {
                normalized.push((name, value));
            }
        }

        Self {
            entries: normalized,
        }
    }

    pub(super) fn rule_exprs(&self) -> Vec<Expr> {
        self.entries
            .iter()
            .map(|(name, value)| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::Atom(Atom::Symbol(name.clone())), value.clone()],
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
            [option_expr] => {
                let Some(option_name) = self.normalize_symbol_target_expr(option_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidOptionName.message(),
                    ));
                };
                let Some(scope) = self.option_value_scopes.last().cloned() else {
                    return Ok(self.invalid_call_with_arg_range("OptionValue", args, 2, 3));
                };

                Ok(self.resolve_option_value(
                    scope.target,
                    Some(scope.explicit),
                    option_expr.clone(),
                    option_name,
                ))
            }
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
            _ => Ok(self.invalid_call_with_arg_range("OptionValue", args, 1, 3)),
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

    pub(super) fn option_value_for_symbol(&self, symbol: &Symbol, option: &Symbol) -> Option<Expr> {
        self.options_for_symbol(symbol).get(option).cloned()
    }

    pub(super) fn option_rule_expr_for_symbol(
        &self,
        symbol: &Symbol,
        option: &Symbol,
    ) -> Option<Expr> {
        self.option_value_for_symbol(symbol, option).map(|value| {
            Expr::call(
                system_expr("Rule"),
                vec![Expr::Atom(Atom::Symbol(option.clone())), value],
            )
        })
    }

    pub(super) fn option_rule_exprs_for_symbol(&self, symbol: &Symbol) -> Vec<Expr> {
        self.options_for_symbol(symbol).rule_exprs()
    }

    pub(super) fn merged_option_exprs_for_symbol(
        &self,
        symbol: &Symbol,
        explicit: &[Expr],
    ) -> Vec<Expr> {
        let mut merged = self.option_rule_exprs_for_symbol(symbol);
        merged.extend(explicit.iter().cloned());
        merged
    }

    pub(super) fn merged_option_exprs_for_system_symbol(
        &self,
        symbol_name: &str,
        explicit: &[Expr],
    ) -> Vec<Expr> {
        self.merged_option_exprs_for_symbol(&Symbol::system(symbol_name), explicit)
    }

    pub(super) fn compile_assigned_options_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Result<SymbolOptions, OptionsNormalizationError> {
        let entries = self.parse_option_entries_from_resolved(resolved)?;
        Ok(SymbolOptions::from_entries(entries))
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
}

pub(super) fn options_pattern_sequence_matcher() -> CompiledSequencePattern {
    let rule_matcher = MatcherNode::Call {
        head: Box::new(MatcherNode::Exact(system_expr("Rule"))),
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
        matcher: rule_matcher,
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

fn system_rule_option(name: &str, value: Expr) -> (Symbol, Expr) {
    (Symbol::system(name), value)
}

fn bool_option(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

pub(super) fn default_system_options(symbol: &Symbol) -> SymbolOptions {
    if symbol.context() != SYSTEM_CONTEXT {
        return SymbolOptions::default();
    }

    let entries = match symbol.name() {
        "ArrayFilter" => vec![system_rule_option("Padding", Expr::string("Fixed"))],
        "CheckAbort" => vec![system_rule_option(
            "PropagateAborts",
            system_expr("Automatic"),
        )],
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
