use super::super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum TaggedDefinitionTarget {
    OwnValue(Symbol),
    DownValue(Symbol),
    SubValue(Symbol),
    UpValue(Symbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum TagDefinitionError {
    TagNotFound,
    TagPositionInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum UnsetDefinitionTarget {
    Message(MessageNameKey),
    Options(Symbol),
    SyntaxInformation(Symbol),
    OwnValue(Symbol),
    DownValue { symbol: Symbol, lhs: Expr },
    SubValue { symbol: Symbol, lhs: Expr },
    TaggedDefinition { tag: Symbol, lhs: Expr },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum DefinitionAssignmentTarget {
    Message(MessageNameKey),
    SyntaxInformation(Symbol),
    Definition(TaggedDefinitionTarget),
}

impl TaggedDefinitionTarget {
    pub(in crate::runtime) fn symbol(&self) -> &Symbol {
        match self {
            Self::OwnValue(symbol)
            | Self::DownValue(symbol)
            | Self::SubValue(symbol)
            | Self::UpValue(symbol) => symbol,
        }
    }
}

impl UnsetDefinitionTarget {
    pub(in crate::runtime) fn symbol(&self) -> &Symbol {
        match self {
            Self::Message(key) => &key.symbol,
            Self::Options(symbol) => symbol,
            Self::SyntaxInformation(symbol) => symbol,
            Self::OwnValue(symbol)
            | Self::DownValue { symbol, .. }
            | Self::SubValue { symbol, .. }
            | Self::TaggedDefinition { tag: symbol, .. } => symbol,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum DefinitionTargetNormalizationError {
    InvalidClearTarget,
    InvalidTagTarget,
    InvalidUnsetTarget,
}

impl DefinitionTargetNormalizationError {
    pub(in crate::runtime) fn message(self) -> &'static str {
        match self {
            Self::InvalidClearTarget => {
                "expected one or more symbols, symbol-name strings, or lists of them"
            }
            Self::InvalidTagTarget => {
                "expected a symbol or symbol-name string tag as the first argument"
            }
            Self::InvalidUnsetTarget => {
                "expected a symbol, definition lhs, message name, or tagged definition target"
            }
        }
    }
}

pub(in crate::runtime) fn collect_with_binding_symbols(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    let Expr::Call { head, args } = expr else {
        return;
    };

    if !is_system_symbol_name(head, "List") {
        return;
    }

    for item in args {
        if let Some((symbol, _)) = parse_module_binding_item(item) {
            locals.insert(symbol);
        }
    }
}

pub(in crate::runtime) fn default_value_lhs_targets_symbol(lhs: &Expr, symbol: &Symbol) -> bool {
    let Expr::Call { head, args } = lhs else {
        return false;
    };

    is_system_symbol_name(head, "Default")
        && matches!(args.first(), Some(Expr::Atom(Atom::Symbol(target))) if target == symbol)
}

impl Session {
    pub(in crate::runtime) fn normalize_tag_target(
        &self,
        expr: &Expr,
    ) -> Result<Symbol, DefinitionTargetNormalizationError> {
        self.normalize_symbol_target_expr(expr, true)
            .ok_or(DefinitionTargetNormalizationError::InvalidTagTarget)
    }

    pub(in crate::runtime) fn normalize_clear_targets(
        &self,
        args: &[Expr],
    ) -> Result<Vec<Symbol>, DefinitionTargetNormalizationError> {
        if args.is_empty() {
            return Err(DefinitionTargetNormalizationError::InvalidClearTarget);
        }

        let mut symbols = Vec::new();

        for arg in args {
            let targets = self
                .normalize_symbol_target_exprs(arg, true)
                .ok_or(DefinitionTargetNormalizationError::InvalidClearTarget)?;
            symbols.extend(targets);
        }

        Ok(symbols)
    }

    pub(in crate::runtime) fn normalize_unset_target(
        &self,
        expr: &Expr,
    ) -> Result<UnsetDefinitionTarget, DefinitionTargetNormalizationError> {
        if let Some(key) = self.message_name_key_from_expr(expr) {
            return Ok(UnsetDefinitionTarget::Message(key));
        }

        if let Some(symbol) = self.normalize_options_target_expr(expr) {
            return Ok(UnsetDefinitionTarget::Options(symbol));
        }

        if let Some(symbol) = self.normalize_syntax_information_target_expr(expr) {
            return Ok(UnsetDefinitionTarget::SyntaxInformation(symbol));
        }

        if let Some(symbol) = expr_as_symbol(expr).cloned() {
            return Ok(UnsetDefinitionTarget::OwnValue(symbol));
        }

        if let Expr::Call { head, args } = expr {
            if is_system_symbol_name(head, "TagSet") && args.len() == 2 {
                return Ok(UnsetDefinitionTarget::TaggedDefinition {
                    tag: self.normalize_tag_target(&args[0])?,
                    lhs: args[1].clone(),
                });
            }
        }

        if let Some(symbol) = downvalue_head_symbol(expr).cloned() {
            return Ok(UnsetDefinitionTarget::DownValue {
                symbol,
                lhs: expr.clone(),
            });
        }

        if let Some(symbol) = subvalue_base_symbol(expr).cloned() {
            return Ok(UnsetDefinitionTarget::SubValue {
                symbol,
                lhs: expr.clone(),
            });
        }

        Err(DefinitionTargetNormalizationError::InvalidUnsetTarget)
    }

    pub(in crate::runtime) fn normalize_definition_assignment_target(
        &self,
        lhs: &Expr,
    ) -> Option<DefinitionAssignmentTarget> {
        if let Some(key) = self.message_name_key_from_expr(lhs) {
            return Some(DefinitionAssignmentTarget::Message(key));
        }

        if let Some(symbol) = self.normalize_syntax_information_target_expr(lhs) {
            return Some(DefinitionAssignmentTarget::SyntaxInformation(symbol));
        }

        if let Some(symbol) = expr_as_symbol(lhs).cloned() {
            return Some(DefinitionAssignmentTarget::Definition(
                TaggedDefinitionTarget::OwnValue(symbol),
            ));
        }

        classify_definition_lhs(lhs).map(DefinitionAssignmentTarget::Definition)
    }

    /// Returns the immediate own value stored for a symbol.
    pub fn definition_own_value(&self, symbol: &Symbol) -> Option<Expr> {
        self.definitions
            .get(symbol)
            .and_then(|definition| definition.own_value.clone())
    }

    /// Stores an immediate value for a normalized definition assignment target.
    pub fn store_definition_assignment_target_immediate(&mut self, lhs: &Expr, rhs: Expr) {
        if let Some(DefinitionAssignmentTarget::Definition(target)) =
            self.normalize_definition_assignment_target(lhs)
        {
            self.store_definition_target(target, lhs.clone(), rhs, false);
        }
    }

    /// Returns the definition target symbol when it is protected or known-system.
    pub fn protected_or_known_definition_target_symbol(&self, lhs: &Expr) -> Option<Symbol> {
        let Some(DefinitionAssignmentTarget::Definition(target)) =
            self.normalize_definition_assignment_target(lhs)
        else {
            return None;
        };

        let symbol = target.symbol().clone();
        (self.symbol_is_protected(&symbol)
            || is_known_system_symbol(&symbol)
            || self.builtin_spec_for_symbol(&symbol).is_some())
        .then_some(symbol)
    }

    pub(in crate::runtime) fn normalize_options_target_expr(&self, expr: &Expr) -> Option<Symbol> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "Options") || args.len() != 1 {
            return None;
        }

        self.normalize_symbol_target_expr(&args[0], true)
    }

    pub(in crate::runtime) fn normalize_syntax_information_target_expr(
        &self,
        expr: &Expr,
    ) -> Option<Symbol> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "SyntaxInformation") || args.len() != 1 {
            return None;
        }

        self.normalize_symbol_target_expr(&args[0], true)
    }
}

pub(in crate::runtime) fn classify_definition_lhs(lhs: &Expr) -> Option<TaggedDefinitionTarget> {
    if let Some(symbol) = expr_as_symbol(lhs).cloned() {
        return Some(TaggedDefinitionTarget::OwnValue(symbol));
    }

    if let Some(symbol) = subvalue_base_symbol(lhs).cloned() {
        return Some(TaggedDefinitionTarget::SubValue(symbol));
    }

    downvalue_head_symbol(lhs)
        .cloned()
        .map(TaggedDefinitionTarget::DownValue)
}

pub(in crate::runtime) fn classify_tagged_definition(
    tag: &Symbol,
    lhs: &Expr,
) -> Result<TaggedDefinitionTarget, TagDefinitionError> {
    if let Expr::Call { head, args } = strip_hold_pattern(lhs) {
        if is_system_symbol_name(head, "Condition") && !args.is_empty() {
            return classify_tagged_definition(tag, &args[0]);
        }
    }

    if expr_as_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::OwnValue(tag.clone()));
    }

    if subvalue_base_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::SubValue(tag.clone()));
    }

    if downvalue_head_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::DownValue(tag.clone()));
    }

    if matches!(strip_hold_pattern(lhs), Expr::Call { args, .. } if args.iter().any(|arg| valid_upvalue_tag_position(arg, tag)))
    {
        return Ok(TaggedDefinitionTarget::UpValue(tag.clone()));
    }

    if expr_contains_symbol(lhs, tag) {
        Err(TagDefinitionError::TagPositionInvalid)
    } else {
        Err(TagDefinitionError::TagNotFound)
    }
}

pub(in crate::runtime) fn valid_upvalue_tag_position(expr: &Expr, tag: &Symbol) -> bool {
    expr_as_symbol(expr).is_some_and(|symbol| symbol == tag)
        || match expr {
            Expr::Call { head, .. } => expr_as_symbol(head).is_some_and(|symbol| symbol == tag),
            _ => false,
        }
}

pub(in crate::runtime) fn expr_contains_symbol(expr: &Expr, target: &Symbol) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol == target,
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_contains_symbol(head, target)
                || args.iter().any(|arg| expr_contains_symbol(arg, target))
        }
    }
}

pub(in crate::runtime) fn definition_references_symbol(
    definition: &SymbolDefinition,
    target: &Symbol,
) -> bool {
    definition
        .own_value
        .as_ref()
        .is_some_and(|value| expr_contains_symbol(value, target))
        || definition
            .delayed_own_value
            .as_ref()
            .is_some_and(|value| expr_contains_symbol(value, target))
        || definition.up_values.iter().any(|value| {
            expr_contains_symbol(&value.lhs, target) || expr_contains_symbol(&value.rhs, target)
        })
        || definition.down_values.iter().any(|value| {
            expr_contains_symbol(&value.lhs, target) || expr_contains_symbol(&value.rhs, target)
        })
        || definition.sub_values.iter().any(|value| {
            expr_contains_symbol(&value.lhs, target) || expr_contains_symbol(&value.rhs, target)
        })
}

pub(in crate::runtime) fn collect_upvalue_candidate_symbols(args: &[Expr]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for arg in args {
        collect_expr_symbols_unique(arg, &mut symbols);
    }

    symbols
}

pub(in crate::runtime) fn collect_upset_target_symbols(lhs: &Expr) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    collect_upset_target_symbols_from_lhs(strip_hold_pattern(lhs), &mut symbols);
    symbols
}

fn collect_upset_target_symbols_from_lhs(expr: &Expr, symbols: &mut Vec<Symbol>) {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, "Condition") && !args.is_empty() {
            collect_upset_target_symbols_from_lhs(&args[0], symbols);
            return;
        }

        for arg in args {
            collect_upset_target_symbol_from_arg(arg, symbols);
        }
    }
}

fn collect_upset_target_symbol_from_arg(expr: &Expr, symbols: &mut Vec<Symbol>) {
    if let Some((_, inner)) = pattern_named_pattern(expr) {
        collect_upset_target_symbol_from_arg(inner, symbols);
        return;
    }

    if let Some((inner, _)) = pattern_condition(expr) {
        collect_upset_target_symbol_from_arg(inner, symbols);
        return;
    }

    if let Some((inner, _)) = pattern_optional(expr) {
        collect_upset_target_symbol_from_arg(inner, symbols);
        return;
    }

    if pattern_blank_head(expr).is_some() {
        return;
    }

    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => push_unique_upset_target(symbols, symbol.clone()),
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "BlankSequence")
                || is_system_symbol_name(head, "BlankNullSequence") => {}
        Expr::Call { head, .. } => {
            if let Some(symbol) = expr_as_symbol(head) {
                push_unique_upset_target(symbols, symbol.clone());
            }
        }
        Expr::Atom(_) => {}
    }
}

fn push_unique_upset_target(symbols: &mut Vec<Symbol>, symbol: Symbol) {
    if !symbols.iter().any(|existing| existing == &symbol) {
        symbols.push(symbol);
    }
}

pub(in crate::runtime) fn collect_expr_symbols_unique(expr: &Expr, symbols: &mut Vec<Symbol>) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            if !symbols.iter().any(|existing| existing == symbol) {
                symbols.push(symbol.clone());
            }
        }
        Expr::Atom(_) => {}
        Expr::Call { head, args } => {
            collect_expr_symbols_unique(head, symbols);

            for arg in args {
                collect_expr_symbols_unique(arg, symbols);
            }
        }
    }
}

pub(in crate::runtime) fn lhs_patterns_equivalent(lhs: &Expr, rhs: &Expr) -> bool {
    canonicalize_pattern_names(lhs) == canonicalize_pattern_names(rhs)
}

pub(in crate::runtime) fn canonicalize_pattern_names(expr: &Expr) -> Expr {
    let mut mapping = BTreeMap::new();
    let mut next_id = 0_u64;
    collect_pattern_name_mapping(expr, &mut mapping, &mut next_id);
    substitute_pattern_name_mapping(expr, &mapping)
}

pub(in crate::runtime) fn collect_pattern_name_mapping(
    expr: &Expr,
    mapping: &mut BTreeMap<Symbol, Symbol>,
    next_id: &mut u64,
) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            if let Some(symbol) = expr_as_symbol(&args[0]) {
                mapping.entry(symbol.clone()).or_insert_with(|| {
                    *next_id += 1;
                    Symbol::new("Pattern`", format!("p{next_id}"))
                });
            }

            collect_pattern_name_mapping(&args[1], mapping, next_id);
        }
        Expr::Call { head, args } => {
            collect_pattern_name_mapping(head, mapping, next_id);

            for arg in args {
                collect_pattern_name_mapping(arg, mapping, next_id);
            }
        }
        Expr::Atom(_) => {}
    }
}

pub(in crate::runtime) fn substitute_pattern_name_mapping(
    expr: &Expr,
    mapping: &BTreeMap<Symbol, Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => mapping
            .get(symbol)
            .cloned()
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => Expr::call(
            substitute_pattern_name_mapping(head, mapping),
            args.iter()
                .map(|arg| substitute_pattern_name_mapping(arg, mapping))
                .collect(),
        ),
    }
}

pub(in crate::runtime) fn downvalue_head_symbol(expr: &Expr) -> Option<&Symbol> {
    match strip_hold_pattern(expr) {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            downvalue_head_symbol(&args[0])
        }
        Expr::Call { head, .. } => expr_as_symbol(head),
        _ => None,
    }
}

pub(in crate::runtime) fn subvalue_base_symbol(expr: &Expr) -> Option<&Symbol> {
    match strip_hold_pattern(expr) {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            subvalue_base_symbol(&args[0])
        }
        Expr::Call { head, .. } => base_head_symbol_of_nested_call(head),
        _ => None,
    }
}

pub(in crate::runtime) fn base_head_symbol_of_nested_call(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Call { head, .. } => match &**head {
            Expr::Call { .. } => base_head_symbol_of_nested_call(head),
            _ => expr_as_symbol(head),
        },
        _ => None,
    }
}

pub(in crate::runtime) fn normalize_rewrite_rules(expr: &Expr) -> Option<Vec<RewriteRule>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut rules = Vec::new();

            for item in args {
                let rule = normalize_single_rewrite_rule(item)?;
                rules.push(rule);
            }

            Some(rules)
        }
        _ => normalize_single_rewrite_rule(expr).map(|rule| vec![rule]),
    }
}

pub(in crate::runtime) fn normalize_single_rewrite_rule(expr: &Expr) -> Option<RewriteRule> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let delayed = is_system_symbol_name(head, "RuleDelayed");

            Some(RewriteRule {
                lhs: args[0].clone(),
                rhs: args[1].clone(),
                delayed,
                rhs_may_splice_without_bindings: rule_rhs_may_splice_without_bindings(&args[1]),
                matcher: compile_matcher(&args[0]),
            })
        }
        _ => None,
    }
}
