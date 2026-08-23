use super::*;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::sync::OnceLock;

const DOWN_VALUE_GROUND_INDEX_THRESHOLD: usize = 32;
const NO_DOWN_VALUE_INDEX: usize = usize::MAX;

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
    down_value_dispatch: Option<Box<DownValueDispatchMetadata>>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::runtime) struct LinearDownValueScan {
    next_rule_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct IndexedDownValueScan {
    next_non_ground_position: usize,
    exact_rule_index: Option<usize>,
    next_linear_index: usize,
    fallback_rule_index: Option<usize>,
    evaluation_epoch: u64,
}

pub(in crate::runtime) trait DownValueScan: Copy + Send + 'static {
    fn next_rule_index(
        &mut self,
        definition: &SymbolDefinition,
        current_evaluation_epoch: u64,
    ) -> Option<usize>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DownValueDispatchMetadata {
    shallow_signatures: Option<Vec<DownValueShallowSignature>>,
    ground_count: usize,
    ground_index: DownValueGroundIndexCache,
}

#[derive(Debug, Default)]
struct DownValueGroundIndexCache {
    index: OnceLock<Box<DownValueGroundIndex>>,
}

impl Clone for DownValueGroundIndexCache {
    fn clone(&self) -> Self {
        // This is derived dispatch data. Definition snapshots and Block frames
        // should not duplicate it; the first wide lookup rebuilds it lazily.
        Self::default()
    }
}

impl PartialEq for DownValueGroundIndexCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DownValueGroundIndexCache {}

#[derive(Clone, Debug)]
struct DownValueGroundIndex {
    hash_builder: RandomState,
    first_rule_by_hash: HashMap<u64, usize>,
    next_collision: Vec<usize>,
    non_ground_rule_indices: Vec<usize>,
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
    pub(in crate::runtime) fn with_own_value(value: Expr) -> Self {
        Self {
            own_value: Some(value),
            ..Self::default()
        }
    }

    pub(in crate::runtime) fn is_empty(&self) -> bool {
        self.own_value.is_none()
            && self.delayed_own_value.is_none()
            && self.up_values.is_empty()
            && self.down_values.is_empty()
            && self.sub_values.is_empty()
    }

    pub(in crate::runtime) fn note_down_value_stored(
        &mut self,
        symbol: &Symbol,
        new_is_ground: bool,
        replaced_was_ground: Option<bool>,
    ) {
        let shallow_signatures = self.collect_down_value_shallow_signatures(symbol);
        let metadata = self
            .down_value_dispatch
            .get_or_insert_with(|| Box::new(DownValueDispatchMetadata::default()));
        match replaced_was_ground {
            Some(true) if !new_is_ground => {
                metadata.ground_count = metadata.ground_count.saturating_sub(1);
            }
            Some(false) if new_is_ground => {
                metadata.ground_count = metadata.ground_count.saturating_add(1);
            }
            None if new_is_ground => {
                metadata.ground_count = metadata.ground_count.saturating_add(1);
            }
            _ => {}
        }
        metadata.ground_index.clear();
        metadata.shallow_signatures = shallow_signatures;
    }

    pub(in crate::runtime) fn note_down_values_rebuilt(&mut self, symbol: &Symbol) {
        if self.down_values.is_empty() {
            self.down_value_dispatch = None;
            return;
        }

        let ground_count = self
            .down_values
            .iter()
            .filter(|down_value| down_value.lhs_is_ground)
            .count();
        let shallow_signatures = self.collect_down_value_shallow_signatures(symbol);
        let metadata = self
            .down_value_dispatch
            .get_or_insert_with(|| Box::new(DownValueDispatchMetadata::default()));
        metadata.ground_count = ground_count;
        metadata.ground_index.clear();
        metadata.shallow_signatures = shallow_signatures;
    }

    fn collect_down_value_shallow_signatures(
        &self,
        symbol: &Symbol,
    ) -> Option<Vec<DownValueShallowSignature>> {
        let mut signatures = Vec::with_capacity(self.down_values.len());
        for down_value in &self.down_values {
            let signature = DownValueShallowSignature::from_lhs(symbol, &down_value.lhs)?;
            signatures.push(signature);
        }
        Some(signatures)
    }

    pub(in crate::runtime) fn down_value_shallow_signatures(
        &self,
    ) -> Option<&[DownValueShallowSignature]> {
        self.down_value_dispatch
            .as_ref()?
            .shallow_signatures
            .as_deref()
    }

    pub(in crate::runtime) fn has_ground_down_value_index(&self) -> bool {
        self.down_value_dispatch
            .as_ref()
            .is_some_and(|metadata| metadata.ground_count >= DOWN_VALUE_GROUND_INDEX_THRESHOLD)
    }

    pub(in crate::runtime) fn start_indexed_down_value_scan(
        &self,
        candidate: &Expr,
        evaluation_epoch: u64,
    ) -> Option<IndexedDownValueScan> {
        if !self.has_ground_down_value_index() {
            return None;
        }

        let metadata = self
            .down_value_dispatch
            .as_ref()
            .expect("indexed DownValue scan requires dispatch metadata");
        let index = metadata.ground_index.get_or_init(&self.down_values);
        Some(IndexedDownValueScan {
            next_non_ground_position: 0,
            exact_rule_index: index.exact_rule_index(&self.down_values, candidate),
            next_linear_index: 0,
            fallback_rule_index: None,
            evaluation_epoch,
        })
    }

    pub(in crate::runtime) fn down_value_dispatch_memory_proxy_bytes(&self) -> usize {
        self.down_value_dispatch.as_ref().map_or(0, |metadata| {
            std::mem::size_of::<DownValueDispatchMetadata>()
                .saturating_add(metadata.ground_index.memory_proxy_bytes())
        })
    }
}

impl DownValueScan for LinearDownValueScan {
    #[inline]
    fn next_rule_index(
        &mut self,
        definition: &SymbolDefinition,
        _current_evaluation_epoch: u64,
    ) -> Option<usize> {
        let index = self.next_rule_index;
        if index >= definition.down_values.len() {
            return None;
        }
        self.next_rule_index += 1;
        Some(index)
    }
}

impl DownValueScan for IndexedDownValueScan {
    fn next_rule_index(
        &mut self,
        definition: &SymbolDefinition,
        current_evaluation_epoch: u64,
    ) -> Option<usize> {
        if self.fallback_rule_index.is_none() && self.evaluation_epoch != current_evaluation_epoch {
            self.fallback_rule_index = Some(self.next_linear_index);
        }

        if let Some(next_rule_index) = self.fallback_rule_index {
            if next_rule_index >= definition.down_values.len() {
                return None;
            }
            self.fallback_rule_index = Some(next_rule_index.saturating_add(1));
            return Some(next_rule_index);
        }

        let index = definition
            .down_value_dispatch
            .as_ref()
            .expect("indexed DownValue cursor requires dispatch metadata")
            .ground_index
            .get()
            .expect("indexed DownValue cursor requires a live index");
        let next_non_ground = index
            .non_ground_rule_indices
            .get(self.next_non_ground_position)
            .copied();
        let selected = match (next_non_ground, self.exact_rule_index) {
            (Some(non_ground), Some(exact)) if exact < non_ground => {
                self.exact_rule_index = None;
                Some(exact)
            }
            (Some(non_ground), _) => {
                self.next_non_ground_position += 1;
                Some(non_ground)
            }
            (None, Some(exact)) => {
                self.exact_rule_index = None;
                Some(exact)
            }
            (None, None) => None,
        };
        if let Some(index) = selected {
            self.next_linear_index = index.saturating_add(1);
        }
        selected
    }
}

impl DownValueGroundIndexCache {
    fn get_or_init(&self, down_values: &[DownValue]) -> &DownValueGroundIndex {
        self.index
            .get_or_init(|| Box::new(DownValueGroundIndex::new(down_values)))
            .as_ref()
    }

    fn get(&self) -> Option<&DownValueGroundIndex> {
        self.index.get().map(Box::as_ref)
    }

    fn clear(&mut self) {
        self.index.take();
    }

    fn memory_proxy_bytes(&self) -> usize {
        self.get().map_or(0, |index| {
            std::mem::size_of::<DownValueGroundIndex>().saturating_add(index.memory_proxy_bytes())
        })
    }
}

impl DownValueGroundIndex {
    fn new(down_values: &[DownValue]) -> Self {
        let ground_count = down_values
            .iter()
            .filter(|down_value| down_value.lhs_is_ground)
            .count();
        let hash_builder = RandomState::new();
        let mut first_rule_by_hash = HashMap::with_capacity(ground_count);
        let mut next_collision = vec![NO_DOWN_VALUE_INDEX; down_values.len()];
        let mut non_ground_rule_indices =
            Vec::with_capacity(down_values.len().saturating_sub(ground_count));

        for (index, down_value) in down_values.iter().enumerate() {
            if !down_value.lhs_is_ground {
                non_ground_rule_indices.push(index);
                continue;
            }

            let hash = hash_builder.hash_one(strip_hold_pattern(&down_value.lhs));
            let previous = first_rule_by_hash
                .insert(hash, index)
                .unwrap_or(NO_DOWN_VALUE_INDEX);
            next_collision[index] = previous;
        }

        Self {
            hash_builder,
            first_rule_by_hash,
            next_collision,
            non_ground_rule_indices,
        }
    }

    fn exact_rule_index(&self, down_values: &[DownValue], candidate: &Expr) -> Option<usize> {
        let hash = self.hash_builder.hash_one(candidate);
        let mut index = self.first_rule_by_hash.get(&hash).copied()?;

        while index != NO_DOWN_VALUE_INDEX {
            let down_value = down_values.get(index)?;
            if down_value.lhs_is_ground && strip_hold_pattern(&down_value.lhs) == candidate {
                return Some(index);
            }
            index = *self.next_collision.get(index)?;
        }
        None
    }

    fn memory_proxy_bytes(&self) -> usize {
        self.first_rule_by_hash
            .capacity()
            .saturating_mul(std::mem::size_of::<(u64, usize)>())
            .saturating_add(
                self.next_collision
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
            .saturating_add(
                self.non_ground_rule_indices
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
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
