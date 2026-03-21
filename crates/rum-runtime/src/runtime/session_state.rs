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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CallDispatchPlan {
    pub(super) attributes: AttributeSet,
    pub(super) stages: Vec<CallDispatchStage>,
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
    pub(super) messages: Vec<(MessageNameKey, MessageDefinition)>,
}

#[derive(Clone, Debug)]
pub(super) struct LoaderSnapshot {
    pub(super) attributes: BTreeMap<Symbol, AttributeSet>,
    pub(super) current_context: String,
    pub(super) context_path: Vec<String>,
    pub(super) context_stack: Vec<ContextFrame>,
    pub(super) loaded_packages: Vec<String>,
    pub(super) definitions: BTreeMap<Symbol, SymbolDefinition>,
    pub(super) message_definitions: BTreeMap<MessageNameKey, MessageDefinition>,
    pub(super) next_generated_symbol_id: u64,
    pub(super) trace_events: Vec<TraceEvent>,
    pub(super) messages: Vec<RuntimeMessage>,
    pub(super) last_message_list: Vec<RuntimeMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MatcherNode {
    Exact(Expr),
    Blank {
        head_constraint: Option<Expr>,
    },
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledSequencePattern {
    pub(super) min_len: usize,
    pub(super) max_len: Option<usize>,
    pub(super) matcher: MatcherNode,
    pub(super) name: Option<Symbol>,
    pub(super) checks: Vec<SequencePatternCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SequencePatternCheck {
    Condition(Expr),
    PatternTest(Expr),
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
    #[error("ReplaceRepeated exceeded iteration limit ({limit})")]
    IterationLimitExceeded { limit: usize },
    #[error("evaluation exceeded fixed-point pass limit ({limit})")]
    EvaluationPassLimitExceeded { limit: usize },
    #[error("{request}")]
    InputRequired { request: RuntimeInputRequest },
}

impl Default for Session {
    fn default() -> Self {
        Self {
            builtins: BTreeMap::new(),
            attributes: BTreeMap::new(),
            current_context: GLOBAL_CONTEXT.to_string(),
            context_path: vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()],
            context_stack: Vec::new(),
            package_search_paths: vec![PathBuf::from(".")],
            loaded_packages: Vec::new(),
            load_stack: Vec::new(),
            definitions: BTreeMap::new(),
            message_definitions: BTreeMap::new(),
            next_generated_symbol_id: 0,
            resource_limits: ResourceLimits::default(),
            evaluation_depth: 0,
            evaluation_stack: Vec::new(),
            tracing_enabled: false,
            trace_events: Vec::new(),
            messages: Vec::new(),
            last_message_list: Vec::new(),
            handled_non_loader_eval_errors: Vec::new(),
            queued_input_responses: VecDeque::new(),
            pending_input_request: None,
            pending_eval_request: None,
            pending_suspension: None,
        }
    }
}
