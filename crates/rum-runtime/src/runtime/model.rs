use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    pub recursion_limit: usize,
    pub replace_repeated_limit: usize,
    pub evaluation_pass_limit: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            recursion_limit: DEFAULT_RECURSION_LIMIT,
            replace_repeated_limit: DEFAULT_REPLACE_REPEATED_LIMIT,
            evaluation_pass_limit: DEFAULT_EVALUATION_PASS_LIMIT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    OwnValue,
    DelayedOwnValue,
    UpValue,
    DownValue,
    SubValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleApplicationSource {
    UpValue,
    DownValue,
    SubValue,
    ReplaceAll,
    ReplaceRepeated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    EvaluationPass {
        before: String,
        after: String,
    },
    DefinitionStored {
        symbol: String,
        kind: DefinitionKind,
        value: String,
    },
    RuleApplied {
        source: RuleApplicationSource,
        input: String,
        rule: String,
        result: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMessage {
    pub symbol: String,
    pub tag: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MessageSelector {
    pub(super) symbol: String,
    pub(super) tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MessageNameKey {
    pub(super) symbol: Symbol,
    pub(super) tag: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MessageDefinition {
    pub(super) immediate: Option<Expr>,
    pub(super) delayed: Option<Expr>,
}

impl fmt::Display for RuntimeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}: {}", self.symbol, self.tag, self.text)
    }
}

impl MessageSelector {
    pub(super) fn matches(&self, message: &RuntimeMessage) -> bool {
        self.symbol == message.symbol
            && self
                .tag
                .as_ref()
                .map_or(true, |tag| message.tag.as_str() == tag.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub(super) builtins: BTreeMap<String, BuiltinSpec>,
    pub(super) attributes: BTreeMap<Symbol, AttributeSet>,
    pub(super) current_context: String,
    pub(super) context_path: Vec<String>,
    pub(super) context_stack: Vec<ContextFrame>,
    pub(super) package_search_paths: Vec<PathBuf>,
    pub(super) loaded_packages: Vec<String>,
    pub(super) load_stack: Vec<PathBuf>,
    pub(super) definitions: BTreeMap<Symbol, SymbolDefinition>,
    pub(super) message_definitions: BTreeMap<MessageNameKey, MessageDefinition>,
    pub(super) next_generated_symbol_id: u64,
    pub(super) resource_limits: ResourceLimits,
    pub(super) evaluation_depth: usize,
    pub(super) evaluation_stack: Vec<Expr>,
    pub(super) tracing_enabled: bool,
    pub(super) trace_events: Vec<TraceEvent>,
    pub(super) messages: Vec<RuntimeMessage>,
    pub(super) last_message_list: Vec<RuntimeMessage>,
    pub(super) handled_non_loader_eval_errors: Vec<RuntimeMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalOutcome {
    pub value: Expr,
    pub messages: Vec<RuntimeMessage>,
}

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
    pub sub_values: Vec<SubValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
}

impl SymbolDefinition {
    pub(super) fn is_empty(&self) -> bool {
        self.own_value.is_none()
            && self.delayed_own_value.is_none()
            && self.up_values.is_empty()
            && self.down_values.is_empty()
            && self.sub_values.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RewriteRule {
    pub(super) lhs: Expr,
    pub(super) rhs: Expr,
    pub(super) delayed: bool,
    pub(super) matcher: MatcherNode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CasesSpec {
    Pattern(Expr),
    Rule(RewriteRule),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CasesLevelSpec {
    pub(super) min: usize,
    pub(super) max: Option<usize>,
}

impl CasesLevelSpec {
    pub(super) fn all() -> Self {
        Self { min: 0, max: None }
    }

    pub(super) fn exact(level: usize) -> Self {
        Self {
            min: level,
            max: Some(level),
        }
    }

    pub(super) fn range(min: usize, max: Option<usize>) -> Self {
        Self { min, max }
    }

    pub(super) fn contains(self, level: usize) -> bool {
        level >= self.min && self.max.is_none_or(|max| level <= max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CasesTraversalConfig {
    pub(super) levels: CasesLevelSpec,
    pub(super) include_heads: bool,
}
