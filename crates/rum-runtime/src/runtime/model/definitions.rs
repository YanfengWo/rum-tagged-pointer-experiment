use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInfo {
    pub qualified_name: String,
    pub context: String,
    pub display_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub attribute_count: usize,
    pub attributes: Vec<String>,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
    pub has_up_values: bool,
    pub has_down_values: bool,
    pub has_sub_values: bool,
    pub message_count: usize,
    pub message_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionCatalogEntry {
    pub kind: DefinitionKind,
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRuleInfo {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageCatalogEntry {
    pub message_name: String,
    pub qualified_symbol_name: String,
    pub tag: String,
    pub template: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolDefinition {
    pub own_value: Option<Expr>,
    pub delayed_own_value: Option<Expr>,
    pub up_values: Vec<UpValue>,
    pub down_values: Vec<DownValue>,
    pub(in crate::runtime) down_value_shallow_signatures: Option<Vec<DownValueShallowSignature>>,
    pub sub_values: Vec<SubValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) matcher_attribute_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(in crate::runtime) lhs_is_ground: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) matcher_attribute_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) matcher_attribute_revision: u64,
}

impl SymbolDefinition {
    pub(in crate::runtime) fn is_empty(&self) -> bool {
        self.own_value.is_none()
            && self.delayed_own_value.is_none()
            && self.up_values.is_empty()
            && self.down_values.is_empty()
            && self.sub_values.is_empty()
    }

    pub(in crate::runtime) fn refresh_down_value_shallow_signatures(&mut self, symbol: &Symbol) {
        if self.down_values.is_empty() {
            self.down_value_shallow_signatures = None;
            return;
        }

        let mut signatures = Vec::with_capacity(self.down_values.len());
        for down_value in &self.down_values {
            let Some(signature) = DownValueShallowSignature::from_lhs(symbol, &down_value.lhs)
            else {
                self.down_value_shallow_signatures = None;
                return;
            };
            signatures.push(signature);
        }
        self.down_value_shallow_signatures = Some(signatures);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct DownValueShallowSignature {
    args: Vec<DownValueShallowArg>,
}

impl DownValueShallowSignature {
    fn from_lhs(symbol: &Symbol, lhs: &Expr) -> Option<Self> {
        let lhs = strip_hold_pattern(lhs);
        let Expr::Call { head, args } = lhs else {
            return None;
        };
        if expr_as_symbol(head).is_none_or(|head_symbol| head_symbol != symbol) {
            return None;
        }
        if shallow_pattern_args_may_have_variable_arity(args) {
            return None;
        }

        args.iter()
            .map(DownValueShallowArg::from_pattern_arg)
            .collect::<Option<Vec<_>>>()
            .map(|args| Self { args })
    }

    pub(in crate::runtime) fn could_match_args(&self, args: &[Expr]) -> bool {
        self.args.len() == args.len()
            && self
                .args
                .iter()
                .zip(args)
                .all(|(pattern, value)| pattern.could_match(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DownValueShallowArg {
    Any,
    Exact(Expr),
    Head(Symbol),
}

impl DownValueShallowArg {
    fn from_pattern_arg(pattern: &Expr) -> Option<Self> {
        let pattern = strip_hold_pattern(pattern);

        if let Some(inner) = pattern_verbatim_inner(pattern) {
            return Some(Self::Exact(inner.clone()));
        }

        if let Some((_, inner)) = pattern_named_pattern(pattern) {
            return Self::from_pattern_arg(inner);
        }

        if pattern_condition(pattern).is_some()
            || pattern_test(pattern).is_some()
            || pattern_except(pattern).is_some()
            || pattern_optional(pattern).is_some()
            || pattern_alternatives(pattern).is_some()
            || pattern_key_value_pattern_entries(pattern).is_some()
            || decompose_sequence_pattern(pattern).is_some()
            || pattern_sequence_items(pattern).is_some()
            || pattern_orderless_sequence_items(pattern).is_some()
        {
            return None;
        }

        if let Some(head_constraint) = pattern_blank_head(pattern) {
            return match head_constraint {
                None => Some(Self::Any),
                Some(head) => expr_as_symbol(head).map(|symbol| Self::Head(*symbol)),
            };
        }

        if expr_contains_pattern_syntax(pattern) {
            return None;
        }

        Some(Self::Exact(pattern.clone()))
    }

    fn could_match(&self, value: &Expr) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(pattern) => pattern == value,
            Self::Head(symbol) => expr_has_symbol_head(value, symbol),
        }
    }
}

pub(in crate::runtime) fn expr_contains_pattern_syntax(expr: &Expr) -> bool {
    if pattern_verbatim_inner(expr).is_some()
        || pattern_named_pattern(expr).is_some()
        || pattern_condition(expr).is_some()
        || pattern_test(expr).is_some()
        || pattern_except(expr).is_some()
        || pattern_optional(expr).is_some()
        || pattern_alternatives(expr).is_some()
        || pattern_key_value_pattern_entries(expr).is_some()
        || pattern_blank_head(expr).is_some()
        || decompose_sequence_pattern(expr).is_some()
        || pattern_sequence_items(expr).is_some()
        || pattern_orderless_sequence_items(expr).is_some()
    {
        return true;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_contains_pattern_syntax(head) || args.iter().any(expr_contains_pattern_syntax)
        }
    }
}

fn shallow_pattern_args_may_have_variable_arity(args: &[Expr]) -> bool {
    args.iter().any(|arg| {
        let arg = strip_hold_pattern(arg);
        decompose_sequence_pattern(arg).is_some()
            || pattern_sequence_items(arg).is_some()
            || pattern_orderless_sequence_items(arg).is_some()
            || pattern_optional(arg).is_some()
    })
}

fn expr_has_symbol_head(value: &Expr, expected_symbol: &Symbol) -> bool {
    match value {
        Expr::Atom(Atom::Integer(_)) => expected_symbol.name() == "Integer",
        Expr::Atom(Atom::Rational(_)) => expected_symbol.name() == "Rational",
        Expr::Atom(Atom::String(_)) => expected_symbol.name() == "String",
        Expr::Atom(Atom::Symbol(_)) => expected_symbol.name() == "Symbol",
        Expr::Call { head, .. } => expr_as_symbol(head)
            .map(|actual| actual == expected_symbol)
            .unwrap_or(false),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRule {
    pub(in crate::runtime) lhs: Expr,
    pub(in crate::runtime) rhs: Expr,
    pub(in crate::runtime) delayed: bool,
    pub(in crate::runtime) rhs_may_splice_without_bindings: bool,
    pub(in crate::runtime) matcher: MatcherNode,
}
