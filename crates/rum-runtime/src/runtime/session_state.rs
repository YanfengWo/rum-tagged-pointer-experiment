use super::*;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ContextFrameKind {
    Begin,
    BeginPackage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ContextFrame {
    pub(super) kind: ContextFrameKind,
    pub(super) previous_context: String,
    pub(super) previous_context_path: Vec<String>,
    pub(super) opened_context: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CallDispatchStage {
    SpecialForm,
    PrepareArguments,
    Listable,
    UpValues,
    DownValues,
    SubValues,
    OrdinaryBuiltin,
    Function,
}

const MAX_CALL_DISPATCH_STAGES: usize = 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CallDispatchPlan {
    pub(super) attributes: AttributeSet,
    pub(super) builtin_handler: Option<BuiltinHandler>,
    stages: [CallDispatchStage; MAX_CALL_DISPATCH_STAGES],
    stage_count: usize,
}

impl CallDispatchPlan {
    pub(super) fn new(attributes: AttributeSet, builtin_handler: Option<BuiltinHandler>) -> Self {
        Self {
            attributes,
            builtin_handler,
            stages: [CallDispatchStage::SpecialForm; MAX_CALL_DISPATCH_STAGES],
            stage_count: 0,
        }
    }

    pub(super) fn special_form(
        attributes: AttributeSet,
        builtin_handler: Option<BuiltinHandler>,
    ) -> Self {
        let mut plan = Self::new(attributes, builtin_handler);
        plan.push(CallDispatchStage::SpecialForm);
        plan
    }

    pub(super) fn push(&mut self, stage: CallDispatchStage) {
        debug_assert!(self.stage_count < MAX_CALL_DISPATCH_STAGES);
        self.stages[self.stage_count] = stage;
        self.stage_count += 1;
    }

    pub(super) fn len(&self) -> usize {
        self.stage_count
    }

    pub(super) fn stage(&self, index: usize) -> CallDispatchStage {
        self.stages[index]
    }

    #[cfg(test)]
    pub(super) fn stages(&self) -> &[CallDispatchStage] {
        &self.stages[..self.stage_count]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CallDispatchState {
    pub(super) raw_args: Vec<Expr>,
    pub(super) evaluated_args: Option<Vec<Expr>>,
}

impl CallDispatchState {
    pub(super) fn new(raw_args: Vec<Expr>) -> Self {
        Self {
            raw_args,
            evaluated_args: None,
        }
    }

    pub(super) fn evaluated_args(&self) -> &[Expr] {
        self.evaluated_args
            .as_deref()
            .expect("call arguments should be prepared before this stage")
    }

    pub(super) fn into_final_args(self) -> Vec<Expr> {
        self.evaluated_args.unwrap_or(self.raw_args)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ThreadingDispatch {
    NotApplicable,
    Threaded(Expr),
    Mismatch,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct BlockSymbolState {
    pub(super) definition: Option<SymbolDefinition>,
    pub(super) attributes: Option<AttributeSet>,
    pub(super) options: Option<SymbolOptions>,
    pub(super) syntax_information: Option<Expr>,
    pub(super) messages: Vec<(MessageNameKey, MessageDefinition)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DynamicOwnValueFrame {
    pub(super) symbol: Symbol,
    pub(super) value: Option<Expr>,
}

#[derive(Clone, Debug)]
pub(super) struct LoaderSnapshot {
    pub(super) attributes: AttributeMap,
    pub(super) options: BTreeMap<Symbol, SymbolOptions>,
    pub(super) syntax_information: BTreeMap<Symbol, Expr>,
    pub(super) declared_stubs: SymbolSet,
    pub(super) removed_symbols: SymbolSet,
    pub(super) symbol_generations: BTreeMap<(String, String), u64>,
    pub(super) once_values: BTreeMap<String, Expr>,
    pub(super) current_context: String,
    pub(super) context_path: Vec<String>,
    pub(super) context_stack: Vec<ContextFrame>,
    pub(super) package_search_paths: Vec<PathBuf>,
    pub(super) loaded_packages: Vec<String>,
    pub(super) definitions: DefinitionMap,
    pub(super) has_up_value_definitions: bool,
    pub(super) message_definitions: BTreeMap<MessageNameKey, MessageDefinition>,
    pub(super) next_generated_symbol_id: u64,
    pub(super) trace_events: Vec<TraceEvent>,
    pub(super) trace_event_depths: Vec<usize>,
    pub(super) disabled_message_selectors: Vec<MessageSelector>,
    pub(super) messages: Vec<RuntimeMessage>,
    pub(super) last_message_list: Vec<RuntimeMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReapBucket {
    pub(super) tag: Expr,
    pub(super) values: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReapScope {
    pub(super) patterns: Vec<Expr>,
    pub(super) pattern_list_form: bool,
    pub(super) handler: Option<Expr>,
    pub(super) buckets_by_pattern: Vec<Vec<ReapBucket>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MonitorScope {
    pub(super) display_expr: Expr,
    pub(super) evaluation_monitor: Option<Expr>,
    pub(super) step_monitor: Option<Expr>,
    pub(super) last_display: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MatcherNode {
    Exact(Expr),
    Blank {
        head_constraint: Option<Expr>,
    },
    KeyValuePattern(Vec<(MatcherNode, MatcherNode, bool)>),
    Named {
        name: Symbol,
        inner: Box<MatcherNode>,
    },
    Condition {
        inner: Box<MatcherNode>,
        condition: Expr,
    },
    PatternTest {
        inner: Box<MatcherNode>,
        test: Expr,
    },
    Except {
        excluded: Box<MatcherNode>,
        inner: Box<MatcherNode>,
    },
    Alternatives(Vec<MatcherNode>),
    Call {
        head: Box<MatcherNode>,
        args: Vec<MatcherArg>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MatcherArg {
    Single(MatcherNode),
    Sequence(CompiledSequencePattern),
    Optional(CompiledOptionalPattern),
    PatternSequenceGroup(Vec<MatcherArg>),
    OrderlessPatternSequenceGroup(Vec<MatcherArg>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledSequencePattern {
    pub(super) min_len: usize,
    pub(super) max_len: Option<usize>,
    pub(super) matcher: MatcherNode,
    pub(super) name: Option<Symbol>,
    pub(super) checks: Vec<SequencePatternCheck>,
    pub(super) greediness: SequenceGreediness,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SequencePatternCheck {
    Condition(Expr),
    PatternTest(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SequenceGreediness {
    Shortest,
    Longest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledOptionalPattern {
    pub(super) inner: Box<MatcherArg>,
    pub(super) default: CompiledOptionalDefault,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CompiledOptionalDefault {
    Explicit(Expr),
    Implicit { head: Expr, position: usize },
    UnavailableImplicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MessageCatalogQuery {
    All,
    Symbol(Symbol),
    Key(MessageNameKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AttributeUpdate {
    Insert,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ThreadArgSpec {
    All,
    None,
    Prefix(usize),
    Suffix(usize),
    Single(isize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error("invalid integer literal `{literal}`")]
    InvalidInteger { literal: String },
    #[error("could not read `{path}`: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not resolve package or file `{target}`")]
    LoadPathNotFound {
        target: String,
        searched: Vec<String>,
    },
    #[error("evaluation cycle detected while re-entering `{expr}`")]
    EvaluationCycleDetected { expr: String },
    #[error("evaluation exceeded recursion limit ({limit})")]
    RecursionLimitExceeded { limit: usize },
    #[error("evaluation exceeded TimeConstrained limit")]
    TimeConstraintExceeded,
    #[error("evaluation exceeded MemoryConstrained limit ({limit})")]
    MemoryConstraintExceeded { limit: usize },
    #[error("evaluation exceeded generated element limit ({limit})")]
    GeneratedElementLimitExceeded { limit: usize },
    #[error("evaluation exceeded integer work limit ({limit})")]
    IntegerWorkLimitExceeded { limit: usize },
    #[error("evaluation exceeded iteration work limit ({limit})")]
    IterationWorkLimitExceeded { limit: usize },
    #[error("ReplaceRepeated exceeded iteration limit ({limit})")]
    IterationLimitExceeded { limit: usize },
    #[error("evaluation exceeded fixed-point pass limit ({limit})")]
    EvaluationPassLimitExceeded { limit: usize },
    #[error("nonlocal return: {value}")]
    Return { value: Expr },
    #[error("uncaught throw: {value} with tag {tag}")]
    Thrown {
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    },
    #[error(
        "uncaught throw: {cleanup_value} with tag {cleanup_tag}; uncaught throw: {value} with tag {tag}"
    )]
    ThrownBypassingCatch {
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    },
    #[error("uncaught Abort[]")]
    Abort,
    #[error("uncaught protected Abort[]")]
    ProtectedAbort,
    #[error("uncaught Break[]")]
    Break,
    #[error("uncaught Continue[]")]
    Continue,
    #[error("Exit[{code}]")]
    Exit { code: i64 },
    #[error("{request}")]
    InputRequired { request: RuntimeInputRequest },
}

impl Default for Session {
    fn default() -> Self {
        Self {
            builtins: BuiltinSpecMap::default(),
            attributes: AttributeMap::default(),
            attribute_revision: 0,
            attribute_set_cache: AttributeSetCache::default(),
            builtin_dispatch_spec_cache: BuiltinDispatchSpecCache::default(),
            evaluation_epoch: 0,
            symbol_cache_generation: 0,
            call_stability_generation: 0,
            stability_dirty_bloom: StabilityBloom::default(),
            stability_rewritten_dirty_bloom: StabilityBloom::default(),
            stable_symbol_cache: SymbolStabilityCache::default(),
            symbol_call_effect_cache: SymbolCallEffectCache::default(),
            call_stability_cache: CallStabilityCache::default(),
            options: BTreeMap::new(),
            syntax_information: BTreeMap::new(),
            declared_stubs: SymbolSet::default(),
            removed_symbols: SymbolSet::default(),
            symbol_generations: BTreeMap::new(),
            once_values: BTreeMap::new(),
            option_value_scopes: Vec::new(),
            dynamic_own_values: Vec::new(),
            current_context: GLOBAL_CONTEXT.to_string(),
            context_path: vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()],
            context_stack: Vec::new(),
            package_search_paths: vec![PathBuf::from(".")],
            loaded_packages: Vec::new(),
            load_stack: Vec::new(),
            definitions: DefinitionMap::default(),
            has_up_value_definitions: false,
            message_definitions: BTreeMap::new(),
            message_symbol_counts: SymbolCountMap::default(),
            next_generated_symbol_id: 0,
            resource_limits: ResourceLimits::default(),
            evaluation_depth: 0,
            evaluation_stack: Vec::new(),
            tracing_enabled: false,
            trace_events: Vec::new(),
            trace_event_depths: Vec::new(),
            disabled_message_selectors: Vec::new(),
            messages: Vec::new(),
            prints: Vec::new(),
            last_message_list: Vec::new(),
            current_message_list_checkpoint: None,
            current_message_list_dirty: false,
            handled_non_loader_eval_errors: Vec::new(),
            reap_scopes: Vec::new(),
            monitor_scopes: Vec::new(),
            monitor_callbacks_enabled: true,
            current_time_deadline: None,
            current_memory_limit: None,
            asserts_enabled: false,
            abort_protect_depth: 0,
            abort_protect_pending: 0,
            queued_input_responses: VecDeque::new(),
            pending_input_request: None,
            pending_eval_request: None,
            pending_suspension: None,
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        print_lexical_profile_summary_once();
    }
}
