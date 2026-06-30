use super::*;
use std::{
    hash::{BuildHasherDefault, Hash, Hasher},
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    pub recursion_limit: usize,
    pub replace_repeated_limit: usize,
    pub evaluation_pass_limit: usize,
    pub generated_element_limit: usize,
    pub integer_work_limit: usize,
    pub iteration_work_limit: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            recursion_limit: DEFAULT_RECURSION_LIMIT,
            replace_repeated_limit: DEFAULT_REPLACE_REPEATED_LIMIT,
            evaluation_pass_limit: DEFAULT_EVALUATION_PASS_LIMIT,
            generated_element_limit: DEFAULT_GENERATED_ELEMENT_LIMIT,
            integer_work_limit: DEFAULT_INTEGER_WORK_LIMIT,
            iteration_work_limit: DEFAULT_ITERATION_WORK_LIMIT,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionConfig {
    pub resource_limits: ResourceLimits,
    pub current_context: String,
    pub context_path: Vec<String>,
    pub package_search_paths: Vec<PathBuf>,
    pub tracing_enabled: bool,
    pub asserts_enabled: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            current_context: GLOBAL_CONTEXT.to_string(),
            context_path: vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()],
            package_search_paths: vec![PathBuf::from(".")],
            tracing_enabled: false,
            asserts_enabled: false,
        }
    }
}

impl SessionConfig {
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    pub fn with_current_context(mut self, context: impl Into<String>) -> Self {
        self.current_context = context.into();
        self
    }

    pub fn with_context_path<I, S>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.context_path = contexts.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_package_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.package_search_paths = paths.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }

    pub fn with_asserts_enabled(mut self, enabled: bool) -> Self {
        self.asserts_enabled = enabled;
        self
    }

    pub fn normalized(mut self) -> Self {
        self.current_context = normalize_context_name(self.current_context);

        self.context_path = if self.context_path.is_empty() {
            vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()]
        } else {
            let mut normalized = Vec::new();

            for context in self.context_path {
                let context = normalize_context_name(context);

                if !normalized.iter().any(|existing| existing == &context) {
                    normalized.push(context);
                }
            }

            normalized
        };

        self.package_search_paths = if self.package_search_paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            dedup_paths(self.package_search_paths)
        };

        self
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
    Replace,
    ReplaceList,
    ReplaceAt,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeInputKind {
    InputString,
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInputRequest {
    pub kind: RuntimeInputKind,
    pub prompt: String,
    pub password: bool,
}

impl RuntimeInputRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            kind: RuntimeInputKind::InputString,
            prompt: prompt.into(),
            password: false,
        }
    }

    pub fn input(mut self) -> Self {
        self.kind = RuntimeInputKind::Input;
        self
    }

    pub fn input_string(mut self) -> Self {
        self.kind = RuntimeInputKind::InputString;
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ModuleActivation {
    pub(super) gensyms: Vec<Symbol>,
    pub(super) escaped: bool,
    pub(super) side_table_stamp: SideTableStamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SideTableStamp {
    pub(super) once_values: usize,
    pub(super) monitor_scopes: usize,
    pub(super) option_value_scopes: usize,
    pub(super) dynamic_own_values: usize,
    pub(super) reap_bucket_values: usize,
}

impl fmt::Display for RuntimeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}: {}", self.symbol, self.tag, self.text)
    }
}

impl fmt::Display for RuntimeInputRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prompt.is_empty() {
            f.write_str("input required")
        } else {
            write!(f, "input required: {}", self.prompt)
        }
    }
}

impl fmt::Display for SessionRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(diagnostic) => f.write_str(&diagnostic.message),
            Self::InvalidInteger { literal } => write!(f, "invalid integer literal `{literal}`"),
            Self::Io { path, message } => write!(f, "could not read `{path}`: {message}"),
            Self::LoadPathNotFound { target, .. } => {
                write!(f, "could not resolve package or file `{target}`")
            }
            Self::EvaluationCycleDetected { expr } => {
                write!(f, "evaluation cycle detected while re-entering `{expr}`")
            }
            Self::RecursionLimitExceeded { limit } => {
                write!(f, "evaluation exceeded recursion limit ({limit})")
            }
            Self::MatchRecursionLimitExceeded { limit } => {
                write!(
                    f,
                    "evaluation exceeded pattern-match recursion limit ({limit})"
                )
            }
            Self::TimeConstraintExceeded => {
                f.write_str("evaluation exceeded TimeConstrained limit")
            }
            Self::MemoryConstraintExceeded { limit } => {
                write!(f, "evaluation exceeded MemoryConstrained limit ({limit})")
            }
            Self::GeneratedElementLimitExceeded { limit } => {
                write!(f, "evaluation exceeded generated element limit ({limit})")
            }
            Self::IntegerWorkLimitExceeded { limit } => {
                write!(f, "evaluation exceeded integer work limit ({limit})")
            }
            Self::IterationWorkLimitExceeded { limit } => {
                write!(f, "evaluation exceeded iteration work limit ({limit})")
            }
            Self::IterationLimitExceeded { limit } => {
                write!(f, "ReplaceRepeated exceeded iteration limit ({limit})")
            }
            Self::EvaluationPassLimitExceeded { limit } => {
                write!(f, "evaluation exceeded fixed-point pass limit ({limit})")
            }
            Self::Exit { code } => write!(f, "Exit[{code}]"),
            Self::InputRequired { request } => write!(f, "{request}"),
            Self::Other { message } => f.write_str(message),
        }
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

    pub(super) fn covers(&self, other: &MessageSelector) -> bool {
        self.symbol == other.symbol
            && self.tag.as_ref().map_or(true, |tag| {
                other.tag.as_ref().is_some_and(|other| other == tag)
            })
    }
}

pub struct Session {
    pub(super) builtins: BuiltinSpecMap,
    pub(super) attributes: AttributeMap,
    pub(super) attribute_revision: u64,
    pub(super) attribute_set_cache: AttributeSetCache,
    pub(super) builtin_dispatch_spec_cache: BuiltinDispatchSpecCache,
    pub(super) evaluation_epoch: u64,
    pub(super) symbol_cache_generation: u64,
    pub(super) call_stability_generation: u64,
    pub(super) stability_dirty_bloom: StabilityBloom,
    pub(super) stability_rewritten_dirty_bloom: StabilityBloom,
    pub(super) stable_symbol_cache: SymbolStabilityCache,
    pub(super) symbol_call_effect_cache: SymbolCallEffectCache,
    pub(super) call_stability_cache: CallStabilityCache,
    pub(super) options: BTreeMap<Symbol, SymbolOptions>,
    pub(super) syntax_information: BTreeMap<Symbol, Expr>,
    pub(super) declared_stubs: SymbolSet,
    pub(super) removed_symbols: SymbolSet,
    pub(super) symbol_generations: BTreeMap<(String, String), u64>,
    pub(super) once_values: BTreeMap<String, Expr>,
    pub(super) option_value_scopes: Vec<OptionValueScope>,
    pub(super) dynamic_own_values: Vec<DynamicOwnValueFrame>,
    pub(super) module_activations: Vec<ModuleActivation>,
    pub(super) gc_candidate_gensyms: std::collections::HashSet<Symbol>,
    pub(super) in_gensym_sweep: bool,
    pub(super) current_context: String,
    pub(super) context_path: Vec<String>,
    pub(super) context_stack: Vec<ContextFrame>,
    pub(super) package_search_paths: Vec<PathBuf>,
    pub(super) loaded_packages: Vec<String>,
    pub(super) load_stack: Vec<PathBuf>,
    pub(super) definitions: DefinitionMap,
    pub(super) has_up_value_definitions: bool,
    pub(super) message_definitions: BTreeMap<MessageNameKey, MessageDefinition>,
    pub(super) message_symbol_counts: SymbolCountMap,
    pub(super) next_generated_symbol_id: u64,
    pub(super) resource_limits: ResourceLimits,
    pub(super) evaluation_depth: usize,
    pub(super) match_depth: usize,
    pub(super) evaluation_stack: Vec<EvaluationStackEntry>,
    pub(super) tracing_enabled: bool,
    pub(super) trace_events: Vec<TraceEvent>,
    pub(super) trace_event_depths: Vec<usize>,
    pub(super) disabled_message_selectors: Vec<MessageSelector>,
    pub(super) messages: Vec<RuntimeMessage>,
    pub(super) prints: Vec<String>,
    pub(super) print_sink: Option<Box<dyn FnMut(&str) + Send>>,
    pub(super) peak_memory_proxy_bytes: std::cell::Cell<usize>,
    pub(super) last_message_list: Vec<RuntimeMessage>,
    pub(super) current_message_list_checkpoint: Option<usize>,
    pub(super) current_message_list_dirty: bool,
    pub(super) handled_non_loader_eval_errors: Vec<RuntimeMessage>,
    pub(super) reap_scopes: Vec<ReapScope>,
    pub(super) monitor_scopes: Vec<MonitorScope>,
    pub(super) monitor_callbacks_enabled: bool,
    pub(super) current_time_deadline: Option<Instant>,
    pub(super) current_memory_limit: Option<usize>,
    pub(super) asserts_enabled: bool,
    pub(super) abort_protect_depth: usize,
    pub(super) abort_protect_pending: usize,
    pub(super) queued_input_responses: VecDeque<String>,
    pub(super) pending_input_request: Option<RuntimeInputRequest>,
    pub(super) pending_eval_request: Option<PendingEvalRequest>,
    pub(super) pending_suspension: Option<SuspendedEvaluation>,
}

impl Session {
    pub fn set_print_sink(&mut self, sink: Box<dyn FnMut(&str) + Send>) {
        self.print_sink = Some(sink);
    }

    pub fn clear_print_sink(&mut self) {
        self.print_sink = None;
    }

    pub(in crate::runtime) fn current_side_table_stamp(&self) -> SideTableStamp {
        SideTableStamp {
            once_values: self.once_values.len(),
            monitor_scopes: self.monitor_scopes.len(),
            option_value_scopes: self.option_value_scopes.len(),
            dynamic_own_values: self.dynamic_own_values.len(),
            reap_bucket_values: self
                .reap_scopes
                .iter()
                .flat_map(|scope| scope.buckets_by_pattern.iter())
                .flat_map(|buckets| buckets.iter())
                .map(|bucket| bucket.values.len())
                .sum(),
        }
    }

    pub(in crate::runtime) fn note_active_modules_escaped(&mut self) {
        if self.module_activations.is_empty() {
            return;
        }

        for activation in &mut self.module_activations {
            activation.escaped = true;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvaluationStackEntry {
    pub(super) expr: Expr,
    pub(super) epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SymbolStabilityCacheEntry {
    pub(super) generation: u64,
    pub(super) stable: bool,
}

pub(super) type SymbolCountMap = HashMap<Symbol, u32, BuildHasherDefault<RuntimeCacheHasher>>;
pub(in crate::runtime) type SymbolExprMap =
    HashMap<Symbol, Expr, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type SymbolSet =
    std::collections::HashSet<Symbol, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type DefinitionMap =
    HashMap<Symbol, SymbolDefinition, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type BuiltinSpecMap =
    HashMap<Symbol, BuiltinSpec, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type AttributeMap =
    HashMap<Symbol, AttributeSet, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type SymbolStabilityCache =
    HashMap<Symbol, SymbolStabilityCacheEntry, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type AttributeSetCache =
    HashMap<Symbol, AttributeSet, BuildHasherDefault<RuntimeCacheHasher>>;

pub(super) type BuiltinDispatchSpecCache =
    HashMap<Symbol, Option<BuiltinSpec>, BuildHasherDefault<RuntimeCacheHasher>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct SymbolCallEffectKey {
    pub(super) symbol: Symbol,
    pub(super) args_empty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CallEffectCacheEntry {
    pub(super) generation: u64,
    pub(super) has_effect: bool,
}

pub(super) type SymbolCallEffectCache =
    HashMap<SymbolCallEffectKey, CallEffectCacheEntry, BuildHasherDefault<RuntimeCacheHasher>>;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RuntimeCacheHasher(u64);

impl Hasher for RuntimeCacheHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325_u64
        } else {
            self.0
        };
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

pub(super) type CallStabilityCache =
    HashMap<CallStabilityCacheKey, CallStabilityCacheEntry, BuildHasherDefault<RuntimeCacheHasher>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CallStabilityCacheKey {
    head: usize,
    args: usize,
}

impl Hash for CallStabilityCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let head = self.head as u64;
        let args = self.args as u64;
        state.write_u64(head.rotate_left(17) ^ args.wrapping_mul(0x9e3779b97f4a7c15));
    }
}

impl CallStabilityCacheKey {
    pub(super) fn new(head: &SharedExpr, args: &SharedArgs) -> Self {
        Self {
            head: head.ptr_key(),
            args: args.ptr_key(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct CallStabilityCacheEntry {
    pub(super) generation: u64,
    pub(super) head: SharedExpr,
    pub(super) args: SharedArgs,
    pub(super) stable: bool,
    pub(super) bloom: StabilityBloom,
    pub(super) dirty_snapshot: StabilityBloom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct StabilityBloom {
    pub(super) regular: u128,
    pub(super) generated: u128,
}

impl StabilityBloom {
    pub(super) fn regular(bit: u128) -> Self {
        Self {
            regular: bit,
            generated: 0,
        }
    }

    pub(super) fn generated(bit: u128) -> Self {
        Self {
            regular: 0,
            generated: bit,
        }
    }

    pub(super) fn union_assign(&mut self, rhs: Self) {
        self.regular |= rhs.regular;
        self.generated |= rhs.generated;
    }

    pub(super) fn intersects(self, rhs: Self) -> bool {
        (self.regular & rhs.regular) != 0 || (self.generated & rhs.generated) != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalOutcome {
    pub value: Expr,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SessionParseResponse {
    pub parsed: ParsedSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionCompletionResponse {
    pub prefix: String,
    pub matches: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionCompletenessStatus {
    Complete,
    Incomplete,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionIsCompleteResponse {
    pub status: SessionCompletenessStatus,
    pub indent: Option<String>,
    pub diagnostic: Option<StructuredSourceDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionLowerResponse {
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionEvalResponse {
    pub value: Expr,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<TraceEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionInspectResponse {
    pub info: SymbolInfo,
    pub definitions: Vec<DefinitionCatalogEntry>,
    pub message_catalog: Vec<MessageCatalogEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionRequestError {
    Syntax(StructuredSourceDiagnostic),
    InvalidInteger {
        literal: String,
    },
    Io {
        path: String,
        message: String,
    },
    LoadPathNotFound {
        target: String,
        searched: Vec<String>,
    },
    EvaluationCycleDetected {
        expr: String,
    },
    RecursionLimitExceeded {
        limit: usize,
    },
    MatchRecursionLimitExceeded {
        limit: usize,
    },
    TimeConstraintExceeded,
    MemoryConstraintExceeded {
        limit: usize,
    },
    GeneratedElementLimitExceeded {
        limit: usize,
    },
    IntegerWorkLimitExceeded {
        limit: usize,
    },
    IterationWorkLimitExceeded {
        limit: usize,
    },
    IterationLimitExceeded {
        limit: usize,
    },
    EvaluationPassLimitExceeded {
        limit: usize,
    },
    Exit {
        code: i64,
    },
    InputRequired {
        request: RuntimeInputRequest,
    },
    Other {
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRequestFailure {
    pub error: SessionRequestError,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<TraceEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PendingEvalRequest {
    pub(super) message_checkpoint: usize,
    pub(super) print_checkpoint: usize,
    pub(super) handled_error_checkpoint: usize,
    pub(super) trace_checkpoint: usize,
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
    pub(super) down_value_shallow_signatures: Option<Vec<DownValueShallowSignature>>,
    pub sub_values: Vec<SubValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(super) rhs_may_splice_without_bindings: bool,
    pub(super) matcher: MatcherNode,
    pub(super) matcher_attribute_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(super) lhs_is_ground: bool,
    pub(super) rhs_may_splice_without_bindings: bool,
    pub(super) matcher: MatcherNode,
    pub(super) matcher_attribute_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubValue {
    pub lhs: Expr,
    pub rhs: Expr,
    pub delayed: bool,
    pub(super) rhs_may_splice_without_bindings: bool,
    pub(super) matcher: MatcherNode,
    pub(super) matcher_attribute_revision: u64,
}

impl SymbolDefinition {
    pub(super) fn is_empty(&self) -> bool {
        self.own_value.is_none()
            && self.delayed_own_value.is_none()
            && self.up_values.is_empty()
            && self.down_values.is_empty()
            && self.sub_values.is_empty()
    }

    pub(super) fn refresh_down_value_shallow_signatures(&mut self, symbol: &Symbol) {
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
pub(super) struct DownValueShallowSignature {
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

    pub(super) fn could_match_args(&self, args: &[Expr]) -> bool {
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

pub(super) fn expr_contains_pattern_syntax(expr: &Expr) -> bool {
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
pub(super) struct RewriteRule {
    pub(super) lhs: Expr,
    pub(super) rhs: Expr,
    pub(super) delayed: bool,
    pub(super) rhs_may_splice_without_bindings: bool,
    pub(super) matcher: MatcherNode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CasesSpec {
    Pattern(Expr),
    Rule(RewriteRule),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CasesLevelSpec {
    pub(super) min: isize,
    pub(super) max: Option<isize>,
}

impl CasesLevelSpec {
    pub(super) fn default_cases() -> Self {
        Self::exact(1)
    }

    pub(super) fn exact(level: isize) -> Self {
        Self {
            min: level,
            max: Some(level),
        }
    }

    pub(super) fn range(min: isize, max: Option<isize>) -> Self {
        Self { min, max }
    }

    pub(super) fn matches(self, level: usize, depth: usize) -> bool {
        let level = level_as_signed(level);
        let depth = level_as_signed(depth);

        if self.spans_negative_and_nonnegative_levels() {
            let positive_level_matches = self.max.is_none_or(|max| level <= max);
            let negative_depth_matches = self
                .min
                .checked_abs()
                .is_none_or(|max_depth| depth <= max_depth);
            return positive_level_matches && negative_depth_matches;
        }

        self.contains_signed(level) || self.contains_signed(-depth)
    }

    fn contains_signed(self, level: isize) -> bool {
        level >= self.min && self.max.is_none_or(|max| level <= max)
    }

    fn spans_negative_and_nonnegative_levels(self) -> bool {
        self.min < 0 && self.max.is_none_or(|max| max >= 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CasesTraversalConfig {
    pub(super) levels: CasesLevelSpec,
    pub(super) max_matches: Option<usize>,
    pub(super) include_heads: bool,
}

impl CasesTraversalConfig {
    pub(super) fn default_cases() -> Self {
        Self {
            levels: CasesLevelSpec::default_cases(),
            max_matches: None,
            include_heads: false,
        }
    }

    pub(super) fn default_member_q() -> Self {
        Self {
            max_matches: Some(1),
            ..Self::default_cases()
        }
    }

    pub(super) fn default_free_q() -> Self {
        Self {
            levels: CasesLevelSpec::range(0, None),
            max_matches: Some(1),
            include_heads: true,
        }
    }

    pub(super) fn default_position() -> Self {
        Self {
            levels: CasesLevelSpec::range(0, None),
            max_matches: None,
            include_heads: true,
        }
    }

    pub(super) fn limit_reached(self, current_count: usize) -> bool {
        self.max_matches.is_some_and(|limit| current_count >= limit)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CasesCollectionOutcome {
    pub(super) matches: Vec<Expr>,
    pub(super) depth: usize,
    pub(super) exhausted: bool,
}

impl CasesCollectionOutcome {
    pub(super) fn new(matches: Vec<Expr>, depth: usize, exhausted: bool) -> Self {
        Self {
            matches,
            depth,
            exhausted,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PositionCollectionOutcome {
    pub(super) positions: Vec<Expr>,
    pub(super) depth: usize,
    pub(super) exhausted: bool,
}

impl PositionCollectionOutcome {
    pub(super) fn new(positions: Vec<Expr>, depth: usize, exhausted: bool) -> Self {
        Self {
            positions,
            depth,
            exhausted,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TraversalCountOutcome {
    pub(super) count: usize,
    pub(super) depth: usize,
    pub(super) exhausted: bool,
}

impl TraversalCountOutcome {
    pub(super) fn new(count: usize, depth: usize, exhausted: bool) -> Self {
        Self {
            count,
            depth,
            exhausted,
        }
    }
}

fn level_as_signed(level: usize) -> isize {
    isize::try_from(level).unwrap_or(isize::MAX)
}
