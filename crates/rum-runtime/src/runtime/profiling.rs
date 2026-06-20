use super::{Atom, Expr, Symbol, evaluation_profile_current_source_symbol};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

const LEXICAL_PROFILE_ENV: &str = "RUM_PROFILE_LEXICAL_SUBSTITUTION";
const LEXICAL_PROFILE_DEFER_ENV: &str = "RUM_PROFILE_LEXICAL_DEFER";
const LEXICAL_PROFILE_DETAIL_ENV: &str = "RUM_PROFILE_LEXICAL_DETAIL";
const LEXICAL_PROFILE_INCREMENTAL_ENV: &str = "RUM_PROFILE_LEXICAL_INCREMENTAL";
const LEXICAL_PROFILE_SHAPES_ENV: &str = "RUM_PROFILE_LEXICAL_SHAPES";
const LEXICAL_PROFILE_BINDINGS_ENV: &str = "RUM_PROFILE_LEXICAL_BINDINGS";
const LEXICAL_PROFILE_SOURCES_ENV: &str = "RUM_PROFILE_LEXICAL_SOURCES";
const LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENV: &str = "RUM_PROFILE_LEXICAL_SOURCE_FINGERPRINTS";
const LEXICAL_PROFILE_PROVENANCE_ENV: &str = "RUM_PROFILE_LEXICAL_PROVENANCE";
const RULE_RHS_TIMING_ENV: &str = "RUM_PROFILE_RULE_RHS_TIMING";
const DEFINITION_RULE_TIMING_ENV: &str = "RUM_PROFILE_DEFINITION_RULE_TIMING";
const BUILTIN_TIMING_ENV: &str = "RUM_PROFILE_BUILTIN_TIMING";
const PRESENCE_REUSE_MIN_NODES: u64 = 1024;
const RULE_RHS_ALPHA_ID_MARKER: u64 = 1_u64 << 63;

static LEXICAL_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_ACTIVE: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_DETAIL_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_INCREMENTAL_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_SHAPES_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_BINDINGS_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_SOURCES_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_PROVENANCE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RULE_RHS_TIMING_ENABLED: AtomicU8 = AtomicU8::new(0);
static DEFINITION_RULE_TIMING_ENABLED: AtomicU8 = AtomicU8::new(0);
static BUILTIN_TIMING_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_SUMMARY_PRINTED: AtomicU8 = AtomicU8::new(0);

static DEFINITION_RHS_EVAL_COUNT: AtomicU64 = AtomicU64::new(0);
static DEFINITION_RHS_EVAL_NANOS: AtomicU64 = AtomicU64::new(0);
static EVAL_MODULE_COUNT: AtomicU64 = AtomicU64::new(0);
static EVAL_MODULE_NANOS: AtomicU64 = AtomicU64::new(0);
static MODULE_RESUME_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);
static MODULE_RESUME_MAX_INDEX: AtomicU64 = AtomicU64::new(0);
static MODULE_INIT_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
static MODULE_INIT_SUBSTITUTION_NANOS: AtomicU64 = AtomicU64::new(0);
static MODULE_BODY_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
static MODULE_BODY_SUBSTITUTION_NANOS: AtomicU64 = AtomicU64::new(0);
static LEXICAL_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_SUBSTITUTION_TIME_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_SUBSTITUTION_NANOS: AtomicU64 = AtomicU64::new(0);
static LEXICAL_SUBSTITUTION_CHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_SUBSTITUTION_UNCHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_BINDING_EMPTY_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_BINDING_SINGLE_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_BINDING_MULTI_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_BINDING_MAX_COUNT: AtomicU64 = AtomicU64::new(0);
static LEXICAL_SHADOWED_NONEMPTY_COUNT: AtomicU64 = AtomicU64::new(0);
static ARG_LIST_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
static ARG_LIST_TOTAL_ARGS: AtomicU64 = AtomicU64::new(0);
static ARG_LIST_MAX_LEN: AtomicU64 = AtomicU64::new(0);
static ARG_PREFILTER_SKIP_COUNT: AtomicU64 = AtomicU64::new(0);
static ARG_PREFILTER_MAYBE_COUNT: AtomicU64 = AtomicU64::new(0);
static ARG_SUBSTITUTION_ATTEMPT_COUNT: AtomicU64 = AtomicU64::new(0);
static ARG_SUBSTITUTION_ATTEMPT_NANOS: AtomicU64 = AtomicU64::new(0);
static ARG_SUBSTITUTION_CHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
static ARG_SUBSTITUTION_UNCHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_SINGLE_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_SINGLE_SCAN_NODES: AtomicU64 = AtomicU64::new(0);
static PRESENCE_SINGLE_SCAN_NANOS: AtomicU64 = AtomicU64::new(0);
static PRESENCE_SINGLE_SCAN_TIME_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_SINGLE_SCAN_TRUE_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_SINGLE_SCAN_FALSE_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_MULTI_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_MULTI_SCAN_NODES: AtomicU64 = AtomicU64::new(0);
static PRESENCE_MULTI_SCAN_NANOS: AtomicU64 = AtomicU64::new(0);
static PRESENCE_MULTI_SCAN_TIME_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_MULTI_SCAN_TRUE_COUNT: AtomicU64 = AtomicU64::new(0);
static PRESENCE_MULTI_SCAN_FALSE_COUNT: AtomicU64 = AtomicU64::new(0);
static REWRITE_RULE_CANDIDATE_COUNT: AtomicU64 = AtomicU64::new(0);
static REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT: AtomicU64 = AtomicU64::new(0);
static REWRITE_RULE_WOULD_MATCHER_COUNT: AtomicU64 = AtomicU64::new(0);
static REWRITE_RULE_MATCHER_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
static REWRITE_RULE_MATCH_COUNT: AtomicU64 = AtomicU64::new(0);
static REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceScanKind {
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceRootShape {
    AtomSymbol(Symbol),
    AtomOther,
    CallHeadSymbol(Symbol),
    CallHeadAtomOther,
    CallHeadCallSymbol(Symbol),
    CallHeadCallOther,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceShapeKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceContextKey {
    kind: PresenceScanKind,
    context: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceContextShapeKey {
    kind: PresenceScanKind,
    context: &'static str,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceArgSite {
    parent: PresenceRootShape,
    index: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceArgSiteKey {
    kind: PresenceScanKind,
    site: PresenceArgSite,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceShapeStats {
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceBindingKey {
    shape: PresenceRootShape,
    target: Symbol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceBindingNameKey {
    shape: PresenceRootShape,
    target_text: Symbol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceKey {
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceShapeKey {
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceTargetShapeKey {
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceSourceFingerprintKind {
    Text,
    Exact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceFingerprintKey {
    fingerprint_kind: PresenceSourceFingerprintKind,
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceFingerprintGroupKey {
    fingerprint_kind: PresenceSourceFingerprintKind,
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceSourceFingerprintGroupStats {
    unique_fingerprints: u64,
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    max_scans_per_fingerprint: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceReuseKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    head_ptr: usize,
    args_ptr: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceReuseGroupKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceReuseGroupStats {
    unique_ptrs: u64,
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    max_scans_per_ptr: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceFingerprintKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceFingerprintGroupKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceFingerprintGroupStats {
    unique_fingerprints: u64,
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    max_scans_per_fingerprint: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceExactFingerprintKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExpressionRebuildKey {
    source: &'static str,
    source_symbol: Option<Symbol>,
    shape: PresenceRootShape,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExpressionRebuildGroupKey {
    source: &'static str,
    source_symbol: Option<Symbol>,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, Default)]
struct ExpressionRebuildStats {
    rebuilds: u64,
    nodes: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct ExpressionRebuildGroupStats {
    unique_fingerprints: u64,
    rebuilds: u64,
    nodes: u64,
    max_rebuilds_per_fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuleRhsAlphaRenameKey {
    source_symbol: Option<Symbol>,
    local_text: Symbol,
}

#[derive(Clone, Copy, Debug, Default)]
struct RuleRhsAlphaRenameStats {
    renames: u64,
    fresh_with_identity: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuleRhsTimingKey {
    phase: &'static str,
    source_symbol: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RuleRhsTimingStats {
    calls: u64,
    nanos: u64,
    changed: u64,
    unchanged: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DefinitionRuleTimingKey {
    kind: &'static str,
    symbol: Symbol,
    index: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct DefinitionRuleTimingStats {
    calls: u64,
    nanos: u64,
    ok: u64,
    err: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DefinitionDispatchKey {
    kind: &'static str,
    symbol: Symbol,
    source_symbol: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct DefinitionDispatchStats {
    candidates: u64,
    prefilter_skips: u64,
    fast_matches: u64,
    fast_misses: u64,
    matcher_matches: u64,
    matcher_misses: u64,
    prefilter_nanos: u64,
    fast_match_nanos: u64,
    compiled_matcher_nanos: u64,
    rhs_substitution_nanos: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BuiltinTimingKey {
    symbol: Symbol,
    source_symbol: Option<Symbol>,
    arg_count: u16,
}

#[derive(Clone, Copy, Debug, Default)]
struct BuiltinTimingStats {
    calls: u64,
    nanos: u64,
    self_nanos: u64,
    ok: u64,
    err: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuntimePathTimingKey {
    phase: &'static str,
    source_symbol: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RuntimePathTimingStats {
    calls: u64,
    nanos: u64,
    self_nanos: u64,
    ok: u64,
    err: u64,
    some: u64,
    none: u64,
    return_err: u64,
    throw_err: u64,
    input_required: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeProfileOutcome {
    Ok,
    Some,
    None,
    Return,
    Throw,
    InputRequired,
    Error,
}

#[derive(Clone, Copy, Debug, Default)]
struct RewriteRulePrefilterStats {
    candidates: u64,
    would_reject: u64,
    would_matcher: u64,
    matcher_misses: u64,
    matches: u64,
    false_rejects: u64,
}

#[derive(Clone, Copy, Debug)]
struct BuiltinTimingFrame {
    key: BuiltinTimingKey,
    child_nanos: u64,
}

#[derive(Clone, Copy, Debug)]
struct RuntimePathTimingFrame {
    key: RuntimePathTimingKey,
    child_nanos: u64,
}

pub(super) struct BuiltinTimingToken {
    start: Instant,
    key: BuiltinTimingKey,
}

pub(super) struct RuntimePathTimingToken {
    start: Instant,
    key: RuntimePathTimingKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceTargetOrigin {
    None,
    PlainSymbol,
    GeneratedDollarName,
    RuleRhsAlphaIdentity,
    OtherIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExpressionProvenanceKind {
    RuleBindingValue,
    RuleRhsRebuild,
    OtherRebuild,
    Unknown,
}

impl ExpressionProvenanceKind {
    fn for_rebuild_source(source: &'static str) -> Self {
        if source.starts_with("rule_rhs_") {
            Self::RuleRhsRebuild
        } else {
            Self::OtherRebuild
        }
    }

    fn priority(self) -> u8 {
        match self {
            Self::RuleBindingValue => 3,
            Self::RuleRhsRebuild => 2,
            Self::OtherRebuild => 1,
            Self::Unknown => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ExpressionProvenanceValue {
    kind: ExpressionProvenanceKind,
    source: &'static str,
    source_symbol: Option<Symbol>,
    max_rule_rhs_alpha_id: Option<u64>,
    records: u64,
    nodes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceAlphaRelation {
    NoTarget,
    TargetNotRuleRhsAlpha,
    ExprHasNoRuleRhsAlpha,
    ExprOlderThanTarget,
    ExprSameOrNewerThanTarget,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceProvenanceKey {
    kind: PresenceScanKind,
    source: Option<Symbol>,
    context: &'static str,
    target_text: Option<Symbol>,
    target_origin: PresenceTargetOrigin,
    expr_origin: ExpressionProvenanceKind,
    expr_origin_source: &'static str,
    expr_origin_symbol: Option<Symbol>,
    alpha_relation: PresenceAlphaRelation,
    shape: PresenceRootShape,
}

thread_local! {
    static PRESENCE_SHAPE_STATS: RefCell<HashMap<PresenceShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_CONTEXT_STATS: RefCell<HashMap<PresenceContextKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_CONTEXT_SHAPE_STATS: RefCell<HashMap<PresenceContextShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_ARG_SITE_STATS: RefCell<HashMap<PresenceArgSiteKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_BINDING_STATS: RefCell<HashMap<PresenceBindingKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_BINDING_NAME_STATS: RefCell<HashMap<PresenceBindingNameKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_SOURCE_STATS: RefCell<HashMap<PresenceSourceKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_SOURCE_SHAPE_STATS: RefCell<HashMap<PresenceSourceShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_SOURCE_TARGET_SHAPE_STATS: RefCell<HashMap<PresenceSourceTargetShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_SOURCE_FINGERPRINT_STATS: RefCell<HashMap<PresenceSourceFingerprintKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_REUSE_STATS: RefCell<HashMap<PresenceReuseKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_FINGERPRINT_STATS: RefCell<HashMap<PresenceFingerprintKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_EXACT_FINGERPRINT_STATS: RefCell<HashMap<PresenceExactFingerprintKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static EXPRESSION_REBUILD_STATS: RefCell<HashMap<ExpressionRebuildKey, ExpressionRebuildStats>> =
        RefCell::new(HashMap::new());
    static EXPRESSION_REBUILD_EXACT_STATS: RefCell<HashMap<ExpressionRebuildKey, ExpressionRebuildStats>> =
        RefCell::new(HashMap::new());
    static RULE_RHS_ALPHA_RENAME_STATS: RefCell<HashMap<RuleRhsAlphaRenameKey, RuleRhsAlphaRenameStats>> =
        RefCell::new(HashMap::new());
    static RULE_RHS_TIMING_STATS: RefCell<HashMap<RuleRhsTimingKey, RuleRhsTimingStats>> =
        RefCell::new(HashMap::new());
    static DEFINITION_RULE_TIMING_STATS: RefCell<HashMap<DefinitionRuleTimingKey, DefinitionRuleTimingStats>> =
        RefCell::new(HashMap::new());
    static DEFINITION_DISPATCH_STATS: RefCell<HashMap<DefinitionDispatchKey, DefinitionDispatchStats>> =
        RefCell::new(HashMap::new());
    static BUILTIN_TIMING_STATS: RefCell<HashMap<BuiltinTimingKey, BuiltinTimingStats>> =
        RefCell::new(HashMap::new());
    static BUILTIN_TIMING_STACK: RefCell<Vec<BuiltinTimingFrame>> = const { RefCell::new(Vec::new()) };
    static RUNTIME_PATH_TIMING_STATS: RefCell<HashMap<RuntimePathTimingKey, RuntimePathTimingStats>> =
        RefCell::new(HashMap::new());
    static RUNTIME_PATH_TIMING_STACK: RefCell<Vec<RuntimePathTimingFrame>> = const { RefCell::new(Vec::new()) };
    static REWRITE_RULE_PREFILTER_SOURCE_STATS: RefCell<HashMap<Symbol, RewriteRulePrefilterStats>> =
        RefCell::new(HashMap::new());
    static EXPRESSION_PROVENANCE_BY_EXACT: RefCell<HashMap<u64, ExpressionProvenanceValue>> =
        RefCell::new(HashMap::new());
    static PRESENCE_PROVENANCE_STATS: RefCell<HashMap<PresenceProvenanceKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static LEXICAL_CONTEXT_STACK: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    static LEXICAL_CONTEXT_DEPTH: Cell<usize> = const { Cell::new(0) };
    static ARG_PREFILTER_SITE_STACK: RefCell<Vec<PresenceArgSite>> = const { RefCell::new(Vec::new()) };
    static ARG_PREFILTER_SITE_DEPTH: Cell<usize> = const { Cell::new(0) };
    static EXPRESSION_REBUILD_SOURCE_STACK: RefCell<Vec<Symbol>> = const { RefCell::new(Vec::new()) };
    static EXPRESSION_REBUILD_SOURCE_DEPTH: Cell<usize> = const { Cell::new(0) };
}

pub(super) struct LexicalProfileContextGuard {
    active: bool,
}

impl Drop for LexicalProfileContextGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        LEXICAL_CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

pub(super) struct LexicalProfileArgSiteGuard {
    active: bool,
}

impl Drop for LexicalProfileArgSiteGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        ARG_PREFILTER_SITE_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

pub(super) struct ExpressionRebuildSourceGuard {
    active: bool,
}

impl Drop for ExpressionRebuildSourceGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[inline]
pub(super) fn lexical_profile_context(context: &'static str) -> LexicalProfileContextGuard {
    if !lexical_profile_shapes_enabled() && !lexical_profile_sources_enabled() {
        return LexicalProfileContextGuard { active: false };
    }

    LEXICAL_CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(context);
    });
    LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    LexicalProfileContextGuard { active: true }
}

#[inline]
pub(super) fn lexical_profile_arg_prefilter_site(
    parent_head: &Expr,
    index: usize,
) -> LexicalProfileArgSiteGuard {
    if !lexical_profile_shapes_enabled() {
        return LexicalProfileArgSiteGuard { active: false };
    }

    let site = PresenceArgSite {
        parent: presence_root_shape(parent_head),
        index: index.min(usize::from(u16::MAX)) as u16,
    };
    ARG_PREFILTER_SITE_STACK.with(|stack| {
        stack.borrow_mut().push(site);
    });
    ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    LexicalProfileArgSiteGuard { active: true }
}

#[inline]
pub(super) fn expression_rebuild_source_symbol(symbol: Symbol) -> ExpressionRebuildSourceGuard {
    if !lexical_profile_shapes_enabled()
        && !lexical_profile_provenance_enabled()
        && !rule_rhs_timing_enabled()
    {
        return ExpressionRebuildSourceGuard { active: false };
    }

    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| {
        stack.borrow_mut().push(symbol);
    });
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    ExpressionRebuildSourceGuard { active: true }
}

#[inline]
pub(super) fn lexical_profile_enabled() -> bool {
    lexical_profile_configured() && lexical_profile_active()
}

#[inline]
fn lexical_profile_configured() -> bool {
    match LEXICAL_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_enabled(),
    }
}

#[inline]
fn lexical_profile_active() -> bool {
    match LEXICAL_PROFILE_ACTIVE.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_active(),
    }
}

#[inline]
pub(super) fn lexical_profile_detail_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_DETAIL_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_detail_enabled(),
        }
}

#[inline]
fn lexical_profile_incremental_enabled() -> bool {
    match LEXICAL_PROFILE_INCREMENTAL_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_incremental_enabled(),
    }
}

#[inline]
fn lexical_profile_shapes_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_SHAPES_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_shapes_enabled(),
        }
}

#[inline]
fn lexical_profile_bindings_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_BINDINGS_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_bindings_enabled(),
        }
}

#[inline]
fn lexical_profile_sources_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_SOURCES_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_sources_enabled(),
        }
}

#[inline]
fn lexical_profile_source_fingerprints_enabled() -> bool {
    lexical_profile_sources_enabled()
        && match LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_source_fingerprints_enabled(),
        }
}

#[inline]
fn lexical_profile_provenance_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_PROVENANCE_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_provenance_enabled(),
        }
}

#[inline]
fn rule_rhs_timing_enabled() -> bool {
    lexical_profile_enabled()
        && match RULE_RHS_TIMING_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_rule_rhs_timing_enabled(),
        }
}

#[inline]
fn definition_rule_timing_enabled() -> bool {
    lexical_profile_enabled()
        && match DEFINITION_RULE_TIMING_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_definition_rule_timing_enabled(),
        }
}

#[inline]
fn builtin_timing_enabled() -> bool {
    lexical_profile_enabled()
        && match BUILTIN_TIMING_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_builtin_timing_enabled(),
        }
}

#[cold]
fn initialize_lexical_profile_detail_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_DETAIL_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_DETAIL_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_active() -> bool {
    let deferred = std::env::var_os(LEXICAL_PROFILE_DEFER_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    let active = !deferred;
    LEXICAL_PROFILE_ACTIVE.store(if active { 1 } else { 2 }, Ordering::Relaxed);
    active
}

#[cold]
fn initialize_lexical_profile_incremental_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_INCREMENTAL_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_INCREMENTAL_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_shapes_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_SHAPES_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_SHAPES_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_bindings_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_BINDINGS_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_BINDINGS_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_sources_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_SOURCES_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_SOURCES_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_source_fingerprints_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED
        .store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_lexical_profile_provenance_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_PROVENANCE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    LEXICAL_PROFILE_PROVENANCE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_rule_rhs_timing_enabled() -> bool {
    let enabled = std::env::var_os(RULE_RHS_TIMING_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    RULE_RHS_TIMING_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_definition_rule_timing_enabled() -> bool {
    let enabled = std::env::var_os(DEFINITION_RULE_TIMING_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    DEFINITION_RULE_TIMING_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_builtin_timing_enabled() -> bool {
    let enabled =
        std::env::var_os(BUILTIN_TIMING_ENV).is_some_and(|value| !value.is_empty() && value != "0");
    BUILTIN_TIMING_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[inline]
pub(super) fn lexical_profile_start() -> Option<Instant> {
    if lexical_profile_enabled() {
        Some(Instant::now())
    } else {
        None
    }
}

#[inline]
pub(super) fn rule_rhs_timing_start() -> Option<Instant> {
    if rule_rhs_timing_enabled() {
        Some(Instant::now())
    } else {
        None
    }
}

pub(super) fn definition_rule_timing_start() -> Option<Instant> {
    if definition_rule_timing_enabled() {
        Some(Instant::now())
    } else {
        None
    }
}

pub(super) fn definition_dispatch_timing_start() -> Option<Instant> {
    if definition_rule_timing_enabled() {
        Some(Instant::now())
    } else {
        None
    }
}

pub(super) fn builtin_timing_start(
    symbol: &Symbol,
    arg_count: usize,
) -> Option<BuiltinTimingToken> {
    if !builtin_timing_enabled() {
        return None;
    }

    let key = BuiltinTimingKey {
        symbol: Symbol::new(symbol.context(), symbol.name()),
        source_symbol: evaluation_profile_current_source_symbol()
            .map(|symbol| Symbol::new(symbol.context(), symbol.name())),
        arg_count: arg_count.min(usize::from(u16::MAX)) as u16,
    };
    BUILTIN_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(BuiltinTimingFrame {
            key,
            child_nanos: 0,
        });
    });
    Some(BuiltinTimingToken {
        start: Instant::now(),
        key,
    })
}

pub(super) fn runtime_path_timing_start(phase: &'static str) -> Option<RuntimePathTimingToken> {
    if !lexical_profile_enabled() {
        return None;
    }

    let key = RuntimePathTimingKey {
        phase,
        source_symbol: evaluation_profile_current_source_symbol()
            .map(|symbol| Symbol::new(symbol.context(), symbol.name())),
    };
    RUNTIME_PATH_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(RuntimePathTimingFrame {
            key,
            child_nanos: 0,
        });
    });
    Some(RuntimePathTimingToken {
        start: Instant::now(),
        key,
    })
}

pub(super) fn record_profile_rewrite_rule_prefilter_candidate(would_reject: bool) {
    if !lexical_profile_enabled() {
        return;
    }

    REWRITE_RULE_CANDIDATE_COUNT.fetch_add(1, Ordering::Relaxed);
    if would_reject {
        REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT.fetch_add(1, Ordering::Relaxed);
    } else {
        REWRITE_RULE_WOULD_MATCHER_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    let Some(source_symbol) = evaluation_profile_current_source_symbol() else {
        return;
    };
    let source_symbol = Symbol::new(source_symbol.context(), source_symbol.name());
    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_symbol).or_default();
        entry.candidates = entry.candidates.saturating_add(1);
        if would_reject {
            entry.would_reject = entry.would_reject.saturating_add(1);
        } else {
            entry.would_matcher = entry.would_matcher.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_rewrite_rule_match_outcome(would_reject: bool, matched: bool) {
    if !lexical_profile_enabled() {
        return;
    }

    if matched {
        REWRITE_RULE_MATCH_COUNT.fetch_add(1, Ordering::Relaxed);
        if would_reject {
            REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        REWRITE_RULE_MATCHER_MISS_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    let Some(source_symbol) = evaluation_profile_current_source_symbol() else {
        return;
    };
    let source_symbol = Symbol::new(source_symbol.context(), source_symbol.name());
    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_symbol).or_default();
        if matched {
            entry.matches = entry.matches.saturating_add(1);
            if would_reject {
                entry.false_rejects = entry.false_rejects.saturating_add(1);
            }
        } else {
            entry.matcher_misses = entry.matcher_misses.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_definition_rhs_eval(start: Option<Instant>) {
    record_timing(
        &DEFINITION_RHS_EVAL_COUNT,
        &DEFINITION_RHS_EVAL_NANOS,
        "definition_rhs_eval",
        start,
    );
}

pub(super) fn record_profile_eval_module(start: Option<Instant>) {
    record_timing(&EVAL_MODULE_COUNT, &EVAL_MODULE_NANOS, "eval_module", start);
}

pub(super) fn record_profile_module_resume_entry(index: usize) {
    if !lexical_profile_enabled() {
        return;
    }

    record_counter(&MODULE_RESUME_ENTRY_COUNT, "module_resume_entries");
    record_max(
        &MODULE_RESUME_MAX_INDEX,
        "module_resume_max_index",
        index as u64,
    );
}

pub(super) fn record_profile_module_init_substitution(start: Option<Instant>) {
    record_timing(
        &MODULE_INIT_SUBSTITUTION_COUNT,
        &MODULE_INIT_SUBSTITUTION_NANOS,
        "module_init_substitution",
        start,
    );
}

pub(super) fn record_profile_module_body_substitution(start: Option<Instant>) {
    record_timing(
        &MODULE_BODY_SUBSTITUTION_COUNT,
        &MODULE_BODY_SUBSTITUTION_NANOS,
        "module_body_substitution",
        start,
    );
}

pub(super) fn record_profile_lexical_substitution_start(
    binding_count: usize,
    shadowed_count: usize,
) {
    if !lexical_profile_enabled() {
        return;
    }

    record_counter(&LEXICAL_SUBSTITUTION_COUNT, "lexical_substitutions");
    if shadowed_count > 0 {
        record_counter(
            &LEXICAL_SHADOWED_NONEMPTY_COUNT,
            "lexical_substitutions_shadowed_nonempty",
        );
    }
    match binding_count {
        0 => record_counter(
            &LEXICAL_BINDING_EMPTY_COUNT,
            "lexical_substitutions_binding_empty",
        ),
        1 => record_counter(
            &LEXICAL_BINDING_SINGLE_COUNT,
            "lexical_substitutions_binding_single",
        ),
        _ => record_counter(
            &LEXICAL_BINDING_MULTI_COUNT,
            "lexical_substitutions_binding_multi",
        ),
    }
    record_max(
        &LEXICAL_BINDING_MAX_COUNT,
        "lexical_binding_max_count",
        binding_count as u64,
    );
}

pub(super) fn record_profile_lexical_substitution_finish(start: Option<Instant>, changed: bool) {
    if changed {
        record_counter(
            &LEXICAL_SUBSTITUTION_CHANGED_COUNT,
            "lexical_substitutions_changed",
        );
    } else {
        record_counter(
            &LEXICAL_SUBSTITUTION_UNCHANGED_COUNT,
            "lexical_substitutions_unchanged",
        );
    }
    record_profile_lexical_timing(
        &LEXICAL_SUBSTITUTION_COUNT,
        &LEXICAL_SUBSTITUTION_TIME_COUNT,
        &LEXICAL_SUBSTITUTION_NANOS,
        "lexical_substitution_time",
        start,
    );
}

pub(super) fn record_profile_arg_list_start(arg_count: usize) {
    if !lexical_profile_enabled() {
        return;
    }

    record_counter(&ARG_LIST_SUBSTITUTION_COUNT, "arg_list_substitutions");
    record_add(
        &ARG_LIST_TOTAL_ARGS,
        "arg_list_total_args",
        arg_count as u64,
    );
    record_max(&ARG_LIST_MAX_LEN, "arg_list_max_len", arg_count as u64);
}

pub(super) fn record_profile_arg_prefilter(maybe_contains_binding: bool) {
    if maybe_contains_binding {
        record_counter(&ARG_PREFILTER_MAYBE_COUNT, "arg_prefilter_maybe");
    } else {
        record_counter(&ARG_PREFILTER_SKIP_COUNT, "arg_prefilter_skip");
    }
}

pub(super) fn record_profile_arg_substitution_attempt(start: Option<Instant>, changed: bool) {
    if changed {
        record_counter(&ARG_SUBSTITUTION_CHANGED_COUNT, "arg_substitution_changed");
    } else {
        record_counter(
            &ARG_SUBSTITUTION_UNCHANGED_COUNT,
            "arg_substitution_unchanged",
        );
    }
    record_timing(
        &ARG_SUBSTITUTION_ATTEMPT_COUNT,
        &ARG_SUBSTITUTION_ATTEMPT_NANOS,
        "arg_substitution_attempt",
        start,
    );
}

pub(super) fn record_profile_expression_rebuild(source: &'static str, expr: &Expr) {
    if !lexical_profile_shapes_enabled() && !lexical_profile_provenance_enabled() {
        return;
    }

    let (fingerprint, exact_fingerprint, nodes) =
        structural_text_exact_fingerprints_and_nodes(expr);
    if nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }
    record_expression_provenance_from_fingerprint(
        exact_fingerprint,
        ExpressionProvenanceKind::for_rebuild_source(source),
        source,
        current_expression_rebuild_source_symbol(),
        max_rule_rhs_alpha_id(expr),
        nodes,
    );

    if !lexical_profile_shapes_enabled() {
        return;
    }

    let key = ExpressionRebuildKey {
        source,
        source_symbol: current_expression_rebuild_source_symbol(),
        shape: presence_root_shape(expr),
        fingerprint,
    };
    let exact_key = ExpressionRebuildKey {
        fingerprint: exact_fingerprint,
        ..key
    };
    EXPRESSION_REBUILD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.rebuilds = entry.rebuilds.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
    });
    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(exact_key).or_default();
        entry.rebuilds = entry.rebuilds.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
    });
}

pub(super) fn record_profile_rule_rhs_binding_value(expr: &Expr) {
    if !lexical_profile_provenance_enabled() {
        return;
    }

    let (_, exact_fingerprint, nodes) = structural_text_exact_fingerprints_and_nodes(expr);
    record_expression_provenance_from_fingerprint(
        exact_fingerprint,
        ExpressionProvenanceKind::RuleBindingValue,
        "rule_binding_value",
        current_expression_rebuild_source_symbol(),
        max_rule_rhs_alpha_id(expr),
        nodes,
    );
}

pub(super) fn record_profile_rule_rhs_alpha_rename(local: &Symbol, fresh: &Symbol) {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    let key = RuleRhsAlphaRenameKey {
        source_symbol: current_expression_rebuild_source_symbol(),
        local_text: Symbol::new(local.context(), local.name()),
    };
    RULE_RHS_ALPHA_RENAME_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.renames = entry.renames.saturating_add(1);
        if fresh.identity().is_some() {
            entry.fresh_with_identity = entry.fresh_with_identity.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_rule_rhs_timing(
    phase: &'static str,
    start: Option<Instant>,
    changed: Option<bool>,
) {
    if !rule_rhs_timing_enabled() {
        return;
    }

    let Some(start) = start else {
        return;
    };
    let elapsed_nanos = elapsed_nanos_since(start);
    let key = RuleRhsTimingKey {
        phase,
        source_symbol: current_expression_rebuild_source_symbol(),
    };
    RULE_RHS_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        match changed {
            Some(true) => entry.changed = entry.changed.saturating_add(1),
            Some(false) => entry.unchanged = entry.unchanged.saturating_add(1),
            None => {}
        }
    });
}

pub(super) fn record_profile_definition_rule_timing(
    kind: &'static str,
    symbol: &Symbol,
    index: usize,
    start: Option<Instant>,
    result_is_ok: bool,
) {
    if !definition_rule_timing_enabled() {
        return;
    }

    let Some(start) = start else {
        return;
    };
    let elapsed_nanos = elapsed_nanos_since(start);
    let key = DefinitionRuleTimingKey {
        kind,
        symbol: Symbol::new(symbol.context(), symbol.name()),
        index: index.min(u32::MAX as usize) as u32,
    };
    DEFINITION_RULE_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        if result_is_ok {
            entry.ok = entry.ok.saturating_add(1);
        } else {
            entry.err = entry.err.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_definition_dispatch_candidate(kind: &'static str, symbol: &Symbol) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        stats.candidates = stats.candidates.saturating_add(1);
    });
}

pub(super) fn record_profile_definition_dispatch_prefilter_skip(
    kind: &'static str,
    symbol: &Symbol,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        stats.prefilter_skips = stats.prefilter_skips.saturating_add(1);
    });
}

pub(super) fn record_profile_definition_dispatch_fast_outcome(
    kind: &'static str,
    symbol: &Symbol,
    matched: bool,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        if matched {
            stats.fast_matches = stats.fast_matches.saturating_add(1);
        } else {
            stats.fast_misses = stats.fast_misses.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_definition_dispatch_matcher_outcome(
    kind: &'static str,
    symbol: &Symbol,
    matched: bool,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        if matched {
            stats.matcher_matches = stats.matcher_matches.saturating_add(1);
        } else {
            stats.matcher_misses = stats.matcher_misses.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_definition_dispatch_phase_timing(
    kind: &'static str,
    symbol: &Symbol,
    phase: &'static str,
    start: Option<Instant>,
) {
    if !definition_rule_timing_enabled() {
        return;
    }

    let Some(start) = start else {
        return;
    };
    let elapsed_nanos = elapsed_nanos_since(start);
    record_profile_definition_dispatch(kind, symbol, |stats| match phase {
        "prefilter" => {
            stats.prefilter_nanos = stats.prefilter_nanos.saturating_add(elapsed_nanos);
        }
        "fast_match" => {
            stats.fast_match_nanos = stats.fast_match_nanos.saturating_add(elapsed_nanos);
        }
        "compiled_matcher" => {
            stats.compiled_matcher_nanos =
                stats.compiled_matcher_nanos.saturating_add(elapsed_nanos);
        }
        "rhs_substitution" => {
            stats.rhs_substitution_nanos =
                stats.rhs_substitution_nanos.saturating_add(elapsed_nanos);
        }
        _ => {}
    });
}

fn record_profile_definition_dispatch<F>(kind: &'static str, symbol: &Symbol, update: F)
where
    F: FnOnce(&mut DefinitionDispatchStats),
{
    if !lexical_profile_enabled() {
        return;
    }

    let key = DefinitionDispatchKey {
        kind,
        symbol: Symbol::new(symbol.context(), symbol.name()),
        source_symbol: evaluation_profile_current_source_symbol()
            .map(|symbol| Symbol::new(symbol.context(), symbol.name())),
    };
    DEFINITION_DISPATCH_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        update(stats.entry(key).or_default());
    });
}

pub(super) fn record_profile_builtin_timing(token: Option<BuiltinTimingToken>, result_is_ok: bool) {
    if !builtin_timing_enabled() {
        return;
    }

    let Some(token) = token else {
        return;
    };
    let elapsed_nanos = elapsed_nanos_since(token.start);
    let child_nanos = BUILTIN_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop();
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }
        frame
            .filter(|frame| frame.key == token.key)
            .map(|frame| frame.child_nanos)
            .unwrap_or(0)
    });
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);
    BUILTIN_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(token.key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
        if result_is_ok {
            entry.ok = entry.ok.saturating_add(1);
        } else {
            entry.err = entry.err.saturating_add(1);
        }
    });
}

pub(super) fn record_profile_runtime_path_timing(
    token: Option<RuntimePathTimingToken>,
    outcome: RuntimeProfileOutcome,
) {
    if !lexical_profile_enabled() {
        return;
    }

    let Some(token) = token else {
        return;
    };
    let elapsed_nanos = elapsed_nanos_since(token.start);
    let child_nanos = RUNTIME_PATH_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop();
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }
        frame
            .filter(|frame| frame.key == token.key)
            .map(|frame| frame.child_nanos)
            .unwrap_or(0)
    });
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);
    RUNTIME_PATH_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(token.key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
        match outcome {
            RuntimeProfileOutcome::Ok => {
                entry.ok = entry.ok.saturating_add(1);
            }
            RuntimeProfileOutcome::Some => {
                entry.ok = entry.ok.saturating_add(1);
                entry.some = entry.some.saturating_add(1);
            }
            RuntimeProfileOutcome::None => {
                entry.ok = entry.ok.saturating_add(1);
                entry.none = entry.none.saturating_add(1);
            }
            RuntimeProfileOutcome::Return => {
                entry.err = entry.err.saturating_add(1);
                entry.return_err = entry.return_err.saturating_add(1);
            }
            RuntimeProfileOutcome::Throw => {
                entry.err = entry.err.saturating_add(1);
                entry.throw_err = entry.throw_err.saturating_add(1);
            }
            RuntimeProfileOutcome::InputRequired => {
                entry.err = entry.err.saturating_add(1);
                entry.input_required = entry.input_required.saturating_add(1);
            }
            RuntimeProfileOutcome::Error => {
                entry.err = entry.err.saturating_add(1);
            }
        }
    });
}

pub(super) fn record_profile_single_presence_scan(
    start: Option<Instant>,
    expr: &Expr,
    target: &Symbol,
    result: bool,
    nodes: u64,
) {
    let elapsed_nanos = start.map(elapsed_nanos_since);
    record_counter(&PRESENCE_SINGLE_SCAN_COUNT, "presence_single_scans");
    record_profile_lexical_timing_elapsed(
        &PRESENCE_SINGLE_SCAN_COUNT,
        &PRESENCE_SINGLE_SCAN_TIME_COUNT,
        &PRESENCE_SINGLE_SCAN_NANOS,
        "presence_single_scan_time",
        elapsed_nanos,
    );
    record_add(
        &PRESENCE_SINGLE_SCAN_NODES,
        "presence_single_scan_nodes",
        nodes,
    );
    if result {
        record_counter(
            &PRESENCE_SINGLE_SCAN_TRUE_COUNT,
            "presence_single_scan_true",
        );
    } else {
        record_counter(
            &PRESENCE_SINGLE_SCAN_FALSE_COUNT,
            "presence_single_scan_false",
        );
    }
    let target_text = Symbol::new(target.context(), target.name());
    record_presence_shape(PresenceScanKind::Single, expr, result, nodes, elapsed_nanos);
    record_presence_source(
        PresenceScanKind::Single,
        expr,
        Some(target_text),
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_binding(expr, target, result, nodes, elapsed_nanos);
    record_presence_reuse(
        PresenceScanKind::Single,
        expr,
        Some(target_text),
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_provenance(
        PresenceScanKind::Single,
        expr,
        Some(target),
        Some(target_text),
        result,
        nodes,
        elapsed_nanos,
    );
}

pub(super) fn record_profile_multi_presence_scan(
    start: Option<Instant>,
    expr: &Expr,
    result: bool,
    nodes: u64,
) {
    let elapsed_nanos = start.map(elapsed_nanos_since);
    record_counter(&PRESENCE_MULTI_SCAN_COUNT, "presence_multi_scans");
    record_profile_lexical_timing_elapsed(
        &PRESENCE_MULTI_SCAN_COUNT,
        &PRESENCE_MULTI_SCAN_TIME_COUNT,
        &PRESENCE_MULTI_SCAN_NANOS,
        "presence_multi_scan_time",
        elapsed_nanos,
    );
    record_add(
        &PRESENCE_MULTI_SCAN_NODES,
        "presence_multi_scan_nodes",
        nodes,
    );
    if result {
        record_counter(&PRESENCE_MULTI_SCAN_TRUE_COUNT, "presence_multi_scan_true");
    } else {
        record_counter(
            &PRESENCE_MULTI_SCAN_FALSE_COUNT,
            "presence_multi_scan_false",
        );
    }
    record_presence_shape(PresenceScanKind::Multi, expr, result, nodes, elapsed_nanos);
    record_presence_source(
        PresenceScanKind::Multi,
        expr,
        None,
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_reuse(
        PresenceScanKind::Multi,
        expr,
        None,
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_provenance(
        PresenceScanKind::Multi,
        expr,
        None,
        None,
        result,
        nodes,
        elapsed_nanos,
    );
}

pub(super) fn print_lexical_profile_summary_once() {
    if !lexical_profile_enabled() {
        return;
    }
    if LEXICAL_PROFILE_SUMMARY_PRINTED.swap(1, Ordering::Relaxed) == 1 {
        return;
    }

    print_lexical_profile_summary_body();
}

pub(super) fn reset_lexical_profile_counters() {
    if !lexical_profile_configured() {
        return;
    }

    macro_rules! reset {
        ($($counter:ident),+ $(,)?) => {
            $( $counter.store(0, Ordering::Relaxed); )+
        };
    }

    reset!(
        DEFINITION_RHS_EVAL_COUNT,
        DEFINITION_RHS_EVAL_NANOS,
        EVAL_MODULE_COUNT,
        EVAL_MODULE_NANOS,
        MODULE_RESUME_ENTRY_COUNT,
        MODULE_RESUME_MAX_INDEX,
        MODULE_INIT_SUBSTITUTION_COUNT,
        MODULE_INIT_SUBSTITUTION_NANOS,
        MODULE_BODY_SUBSTITUTION_COUNT,
        MODULE_BODY_SUBSTITUTION_NANOS,
        LEXICAL_SUBSTITUTION_COUNT,
        LEXICAL_SUBSTITUTION_TIME_COUNT,
        LEXICAL_SUBSTITUTION_NANOS,
        LEXICAL_SUBSTITUTION_CHANGED_COUNT,
        LEXICAL_SUBSTITUTION_UNCHANGED_COUNT,
        LEXICAL_BINDING_EMPTY_COUNT,
        LEXICAL_BINDING_SINGLE_COUNT,
        LEXICAL_BINDING_MULTI_COUNT,
        LEXICAL_BINDING_MAX_COUNT,
        LEXICAL_SHADOWED_NONEMPTY_COUNT,
        ARG_LIST_SUBSTITUTION_COUNT,
        ARG_LIST_TOTAL_ARGS,
        ARG_LIST_MAX_LEN,
        ARG_PREFILTER_SKIP_COUNT,
        ARG_PREFILTER_MAYBE_COUNT,
        ARG_SUBSTITUTION_ATTEMPT_COUNT,
        ARG_SUBSTITUTION_ATTEMPT_NANOS,
        ARG_SUBSTITUTION_CHANGED_COUNT,
        ARG_SUBSTITUTION_UNCHANGED_COUNT,
        PRESENCE_SINGLE_SCAN_COUNT,
        PRESENCE_SINGLE_SCAN_NODES,
        PRESENCE_SINGLE_SCAN_NANOS,
        PRESENCE_SINGLE_SCAN_TIME_COUNT,
        PRESENCE_SINGLE_SCAN_TRUE_COUNT,
        PRESENCE_SINGLE_SCAN_FALSE_COUNT,
        PRESENCE_MULTI_SCAN_COUNT,
        PRESENCE_MULTI_SCAN_NODES,
        PRESENCE_MULTI_SCAN_NANOS,
        PRESENCE_MULTI_SCAN_TIME_COUNT,
        PRESENCE_MULTI_SCAN_TRUE_COUNT,
        PRESENCE_MULTI_SCAN_FALSE_COUNT,
        REWRITE_RULE_CANDIDATE_COUNT,
        REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT,
        REWRITE_RULE_WOULD_MATCHER_COUNT,
        REWRITE_RULE_MATCHER_MISS_COUNT,
        REWRITE_RULE_MATCH_COUNT,
        REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT,
    );

    PRESENCE_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_CONTEXT_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_CONTEXT_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_ARG_SITE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_BINDING_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_BINDING_NAME_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_TARGET_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_REUSE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_FINGERPRINT_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_EXACT_FINGERPRINT_STATS.with(|stats| stats.borrow_mut().clear());
    EXPRESSION_REBUILD_STATS.with(|stats| stats.borrow_mut().clear());
    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| stats.borrow_mut().clear());
    RULE_RHS_ALPHA_RENAME_STATS.with(|stats| stats.borrow_mut().clear());
    RULE_RHS_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    DEFINITION_RULE_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    DEFINITION_DISPATCH_STATS.with(|stats| stats.borrow_mut().clear());
    BUILTIN_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    BUILTIN_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    RUNTIME_PATH_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    RUNTIME_PATH_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| stats.borrow_mut().clear());
    EXPRESSION_PROVENANCE_BY_EXACT.with(|stats| stats.borrow_mut().clear());
    PRESENCE_PROVENANCE_STATS.with(|stats| stats.borrow_mut().clear());
    LEXICAL_CONTEXT_STACK.with(|stack| stack.borrow_mut().clear());
    LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(0));
    ARG_PREFILTER_SITE_STACK.with(|stack| stack.borrow_mut().clear());
    ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(0));
    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| stack.borrow_mut().clear());
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(0));
    LEXICAL_PROFILE_SUMMARY_PRINTED.store(0, Ordering::Relaxed);
    LEXICAL_PROFILE_ACTIVE.store(1, Ordering::Relaxed);
}

pub(super) fn print_lexical_profile_snapshot(label: &str) {
    if !lexical_profile_configured() {
        return;
    }

    eprintln!("rum-profile lexical snapshot label={label}");
    print_lexical_profile_summary_body();
    LEXICAL_PROFILE_SUMMARY_PRINTED.store(1, Ordering::Relaxed);
}

fn print_lexical_profile_summary_body() {
    eprintln!(
        "rum-profile lexical summary definition_rhs_eval_count={} definition_rhs_eval_total_ns={} definition_rhs_eval_avg_ns={}",
        load(&DEFINITION_RHS_EVAL_COUNT),
        load(&DEFINITION_RHS_EVAL_NANOS),
        average(&DEFINITION_RHS_EVAL_NANOS, &DEFINITION_RHS_EVAL_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary eval_module_count={} eval_module_total_ns={} eval_module_avg_ns={}",
        load(&EVAL_MODULE_COUNT),
        load(&EVAL_MODULE_NANOS),
        average(&EVAL_MODULE_NANOS, &EVAL_MODULE_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary module_resume_entries={} module_resume_max_index={}",
        load(&MODULE_RESUME_ENTRY_COUNT),
        load(&MODULE_RESUME_MAX_INDEX),
    );
    eprintln!(
        "rum-profile lexical summary module_init_substitution_count={} module_init_substitution_total_ns={} module_init_substitution_avg_ns={}",
        load(&MODULE_INIT_SUBSTITUTION_COUNT),
        load(&MODULE_INIT_SUBSTITUTION_NANOS),
        average(
            &MODULE_INIT_SUBSTITUTION_NANOS,
            &MODULE_INIT_SUBSTITUTION_COUNT
        ),
    );
    eprintln!(
        "rum-profile lexical summary module_body_substitution_count={} module_body_substitution_total_ns={} module_body_substitution_avg_ns={}",
        load(&MODULE_BODY_SUBSTITUTION_COUNT),
        load(&MODULE_BODY_SUBSTITUTION_NANOS),
        average(
            &MODULE_BODY_SUBSTITUTION_NANOS,
            &MODULE_BODY_SUBSTITUTION_COUNT
        ),
    );
    eprintln!(
        "rum-profile lexical summary lexical_substitutions={} lexical_substitution_timed={} lexical_substitution_total_ns={} lexical_substitution_avg_ns={}",
        load(&LEXICAL_SUBSTITUTION_COUNT),
        load(&LEXICAL_SUBSTITUTION_TIME_COUNT),
        load(&LEXICAL_SUBSTITUTION_NANOS),
        average(
            &LEXICAL_SUBSTITUTION_NANOS,
            &LEXICAL_SUBSTITUTION_TIME_COUNT
        ),
    );
    eprintln!(
        "rum-profile lexical summary lexical_changed={} lexical_unchanged={} binding_empty={} binding_single={} binding_multi={} shadowed_nonempty={} binding_max_count={}",
        load(&LEXICAL_SUBSTITUTION_CHANGED_COUNT),
        load(&LEXICAL_SUBSTITUTION_UNCHANGED_COUNT),
        load(&LEXICAL_BINDING_EMPTY_COUNT),
        load(&LEXICAL_BINDING_SINGLE_COUNT),
        load(&LEXICAL_BINDING_MULTI_COUNT),
        load(&LEXICAL_SHADOWED_NONEMPTY_COUNT),
        load(&LEXICAL_BINDING_MAX_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary arg_list_substitutions={} arg_list_total_args={} arg_list_max_len={} arg_prefilter_skip={} arg_prefilter_maybe={}",
        load(&ARG_LIST_SUBSTITUTION_COUNT),
        load(&ARG_LIST_TOTAL_ARGS),
        load(&ARG_LIST_MAX_LEN),
        load(&ARG_PREFILTER_SKIP_COUNT),
        load(&ARG_PREFILTER_MAYBE_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary arg_substitution_attempts={} arg_substitution_total_ns={} arg_substitution_avg_ns={} arg_substitution_changed={} arg_substitution_unchanged={}",
        load(&ARG_SUBSTITUTION_ATTEMPT_COUNT),
        load(&ARG_SUBSTITUTION_ATTEMPT_NANOS),
        average(
            &ARG_SUBSTITUTION_ATTEMPT_NANOS,
            &ARG_SUBSTITUTION_ATTEMPT_COUNT
        ),
        load(&ARG_SUBSTITUTION_CHANGED_COUNT),
        load(&ARG_SUBSTITUTION_UNCHANGED_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary presence_single_scans={} presence_single_nodes={} presence_single_timed={} presence_single_total_ns={} presence_single_avg_ns={} presence_single_true={} presence_single_false={}",
        load(&PRESENCE_SINGLE_SCAN_COUNT),
        load(&PRESENCE_SINGLE_SCAN_NODES),
        load(&PRESENCE_SINGLE_SCAN_TIME_COUNT),
        load(&PRESENCE_SINGLE_SCAN_NANOS),
        average(
            &PRESENCE_SINGLE_SCAN_NANOS,
            &PRESENCE_SINGLE_SCAN_TIME_COUNT
        ),
        load(&PRESENCE_SINGLE_SCAN_TRUE_COUNT),
        load(&PRESENCE_SINGLE_SCAN_FALSE_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary presence_multi_scans={} presence_multi_nodes={} presence_multi_timed={} presence_multi_total_ns={} presence_multi_avg_ns={} presence_multi_true={} presence_multi_false={}",
        load(&PRESENCE_MULTI_SCAN_COUNT),
        load(&PRESENCE_MULTI_SCAN_NODES),
        load(&PRESENCE_MULTI_SCAN_TIME_COUNT),
        load(&PRESENCE_MULTI_SCAN_NANOS),
        average(&PRESENCE_MULTI_SCAN_NANOS, &PRESENCE_MULTI_SCAN_TIME_COUNT),
        load(&PRESENCE_MULTI_SCAN_TRUE_COUNT),
        load(&PRESENCE_MULTI_SCAN_FALSE_COUNT),
    );
    print_presence_shape_summary();
    print_presence_context_summary();
    print_presence_context_shape_summary();
    print_presence_arg_site_summary();
    print_presence_source_summary();
    print_presence_source_shape_summary();
    print_presence_source_target_shape_summary();
    print_presence_source_fingerprint_summary();
    print_presence_source_fingerprint_group_summary();
    print_presence_provenance_summary();
    print_presence_reuse_summary();
    print_presence_reuse_group_summary();
    print_presence_fingerprint_summary();
    print_presence_fingerprint_group_summary();
    print_presence_exact_fingerprint_group_summary();
    print_expression_rebuild_summary();
    print_expression_rebuild_group_summary();
    print_expression_rebuild_exact_summary();
    print_expression_rebuild_exact_group_summary();
    print_rule_rhs_alpha_rename_summary();
    print_rule_rhs_timing_summary();
    print_definition_rule_timing_summary();
    print_definition_dispatch_summary();
    print_runtime_path_timing_summary();
    print_builtin_timing_summary();
    print_rewrite_rule_prefilter_summary();
    print_presence_binding_summary();
    print_presence_binding_name_summary();
}

fn record_timing(count: &AtomicU64, total_nanos: &AtomicU64, name: &str, start: Option<Instant>) {
    let Some(start) = start else {
        return;
    };

    let elapsed_nanos = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let count_value = count.fetch_add(1, Ordering::Relaxed) + 1;
    let total_value = total_nanos
        .fetch_add(elapsed_nanos, Ordering::Relaxed)
        .saturating_add(elapsed_nanos);
    if lexical_profile_incremental_enabled() && count_value.is_power_of_two() {
        let avg = total_value / count_value;
        eprintln!(
            "rum-profile lexical {name}_count={count_value} {name}_total_ns={total_value} {name}_avg_ns={avg}"
        );
    }
}

fn record_profile_lexical_timing(
    total_count: &AtomicU64,
    timing_count: &AtomicU64,
    total_nanos: &AtomicU64,
    name: &str,
    start: Option<Instant>,
) {
    let Some(start) = start else {
        return;
    };

    let elapsed_nanos = elapsed_nanos_since(start);
    record_profile_lexical_timing_elapsed(
        total_count,
        timing_count,
        total_nanos,
        name,
        Some(elapsed_nanos),
    );
}

fn record_profile_lexical_timing_elapsed(
    total_count: &AtomicU64,
    timing_count: &AtomicU64,
    total_nanos: &AtomicU64,
    name: &str,
    elapsed_nanos: Option<u64>,
) {
    let Some(elapsed_nanos) = elapsed_nanos else {
        return;
    };

    let timing_value = timing_count.fetch_add(1, Ordering::Relaxed) + 1;
    let total_value = total_nanos
        .fetch_add(elapsed_nanos, Ordering::Relaxed)
        .saturating_add(elapsed_nanos);
    if lexical_profile_incremental_enabled() && timing_value.is_power_of_two() {
        let total_calls = total_count.load(Ordering::Relaxed);
        let avg = total_value / timing_value;
        eprintln!(
            "rum-profile lexical {name}_timed={timing_value} {name}_calls={total_calls} {name}_total_ns={total_value} {name}_avg_ns={avg}"
        );
    }
}

fn record_counter(counter: &AtomicU64, name: &str) {
    if !lexical_profile_enabled() {
        return;
    }

    let value = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if lexical_profile_incremental_enabled() && value.is_power_of_two() {
        eprintln!("rum-profile lexical {name}={value}");
    }
}

fn record_add(counter: &AtomicU64, name: &str, amount: u64) {
    if !lexical_profile_enabled() {
        return;
    }

    let previous = counter.fetch_add(amount, Ordering::Relaxed);
    let value = previous.saturating_add(amount);
    if lexical_profile_incremental_enabled() && crossed_power_of_two(previous, value) {
        eprintln!("rum-profile lexical {name}={value}");
    }
}

fn record_max(counter: &AtomicU64, name: &str, value: u64) {
    let mut previous = counter.load(Ordering::Relaxed);
    while value > previous {
        match counter.compare_exchange_weak(previous, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {
                if lexical_profile_incremental_enabled() {
                    eprintln!("rum-profile lexical {name}={value}");
                }
                return;
            }
            Err(actual) => previous = actual,
        }
    }
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

fn average(total: &AtomicU64, count: &AtomicU64) -> u64 {
    let count = load(count);
    if count == 0 { 0 } else { load(total) / count }
}

fn crossed_power_of_two(previous: u64, current: u64) -> bool {
    if current == 0 {
        return false;
    }

    let previous_next = if previous == 0 {
        0
    } else {
        previous.next_power_of_two()
    };
    current.is_power_of_two() || current.next_power_of_two() != previous_next
}

fn elapsed_nanos_since(start: Instant) -> u64 {
    start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn record_presence_shape(
    kind: PresenceScanKind,
    expr: &Expr,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    let key = PresenceShapeKey {
        kind,
        shape: presence_root_shape(expr),
    };
    let context = current_lexical_profile_context();
    let context_key = PresenceContextKey { kind, context };
    let context_shape_key = PresenceContextShapeKey {
        kind,
        context,
        shape: key.shape,
    };
    PRESENCE_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.scans = entry.scans.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
        if let Some(elapsed_nanos) = elapsed_nanos {
            entry.timed = entry.timed.saturating_add(1);
            entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        }
        if result {
            entry.true_count = entry.true_count.saturating_add(1);
        } else {
            entry.false_count = entry.false_count.saturating_add(1);
        }
    });
    PRESENCE_CONTEXT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(context_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_CONTEXT_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(context_shape_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    if let Some(site) = current_arg_prefilter_site() {
        let site_key = PresenceArgSiteKey {
            kind,
            site,
            shape: key.shape,
        };
        PRESENCE_ARG_SITE_STATS.with(|stats| {
            let mut stats = stats.borrow_mut();
            let entry = stats.entry(site_key).or_default();
            record_presence_stats(entry, result, nodes, elapsed_nanos);
        });
    }
}

fn record_presence_source(
    kind: PresenceScanKind,
    expr: &Expr,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_sources_enabled() {
        return;
    }

    let Some(source) = evaluation_profile_current_source_symbol() else {
        return;
    };

    let context = current_lexical_profile_context();
    let shape = presence_root_shape(expr);
    let source_key = PresenceSourceKey {
        kind,
        source,
        context,
    };
    let source_shape_key = PresenceSourceShapeKey {
        kind,
        source,
        context,
        shape,
    };
    let source_target_shape_key = PresenceSourceTargetShapeKey {
        kind,
        source,
        context,
        shape,
        target_text,
    };
    PRESENCE_SOURCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_SOURCE_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_shape_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_SOURCE_TARGET_SHAPE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_target_shape_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    record_presence_source_fingerprint(
        kind,
        source,
        context,
        shape,
        target_text,
        expr,
        result,
        nodes,
        elapsed_nanos,
    );
}

fn record_presence_source_fingerprint(
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    expr: &Expr,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_source_fingerprints_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let (text_fingerprint, exact_fingerprint) = structural_text_and_exact_fingerprints(expr);
    for (fingerprint_kind, fingerprint) in [
        (PresenceSourceFingerprintKind::Text, text_fingerprint),
        (PresenceSourceFingerprintKind::Exact, exact_fingerprint),
    ] {
        let key = PresenceSourceFingerprintKey {
            fingerprint_kind,
            kind,
            source,
            context,
            shape,
            target_text,
            fingerprint,
        };
        PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| {
            let mut stats = stats.borrow_mut();
            let entry = stats.entry(key).or_default();
            record_presence_stats(entry, result, nodes, elapsed_nanos);
        });
    }
}

fn record_expression_provenance_from_fingerprint(
    exact_fingerprint: u64,
    kind: ExpressionProvenanceKind,
    source: &'static str,
    source_symbol: Option<Symbol>,
    max_rule_rhs_alpha_id: Option<u64>,
    nodes: u64,
) {
    if !lexical_profile_provenance_enabled() {
        return;
    }

    EXPRESSION_PROVENANCE_BY_EXACT.with(|map| {
        let mut map = map.borrow_mut();
        let entry = map
            .entry(exact_fingerprint)
            .or_insert(ExpressionProvenanceValue {
                kind,
                source,
                source_symbol,
                max_rule_rhs_alpha_id,
                records: 0,
                nodes: 0,
            });
        entry.records = entry.records.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
        entry.max_rule_rhs_alpha_id = match (entry.max_rule_rhs_alpha_id, max_rule_rhs_alpha_id) {
            (Some(lhs), Some(rhs)) => Some(lhs.max(rhs)),
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            (None, None) => None,
        };
        if kind.priority() > entry.kind.priority() {
            entry.kind = kind;
            entry.source = source;
            entry.source_symbol = source_symbol;
        }
    });
}

fn record_presence_provenance(
    kind: PresenceScanKind,
    expr: &Expr,
    target: Option<&Symbol>,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_provenance_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let (_, exact_fingerprint) = structural_text_and_exact_fingerprints(expr);
    let provenance = EXPRESSION_PROVENANCE_BY_EXACT.with(|map| {
        map.borrow()
            .get(&exact_fingerprint)
            .copied()
            .unwrap_or(ExpressionProvenanceValue {
                kind: ExpressionProvenanceKind::Unknown,
                source: "-",
                source_symbol: None,
                max_rule_rhs_alpha_id: None,
                records: 0,
                nodes: 0,
            })
    });
    let key = PresenceProvenanceKey {
        kind,
        source: evaluation_profile_current_source_symbol(),
        context: current_lexical_profile_context(),
        target_text,
        target_origin: presence_target_origin(target),
        expr_origin: provenance.kind,
        expr_origin_source: provenance.source,
        expr_origin_symbol: provenance.source_symbol,
        alpha_relation: presence_alpha_relation(target, provenance),
        shape: presence_root_shape(expr),
    };
    PRESENCE_PROVENANCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
}

fn record_presence_binding(
    expr: &Expr,
    target: &Symbol,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_bindings_enabled() {
        return;
    }

    let key = PresenceBindingKey {
        shape: presence_root_shape(expr),
        target: *target,
    };
    let name_key = PresenceBindingNameKey {
        shape: key.shape,
        target_text: Symbol::new(target.context(), target.name()),
    };
    PRESENCE_BINDING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.scans = entry.scans.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
        if let Some(elapsed_nanos) = elapsed_nanos {
            entry.timed = entry.timed.saturating_add(1);
            entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        }
        if result {
            entry.true_count = entry.true_count.saturating_add(1);
        } else {
            entry.false_count = entry.false_count.saturating_add(1);
        }
    });
    PRESENCE_BINDING_NAME_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(name_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
}

fn record_presence_reuse(
    kind: PresenceScanKind,
    expr: &Expr,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_shapes_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let Expr::Call { head, args } = expr else {
        return;
    };

    let key = PresenceReuseKey {
        kind,
        shape: presence_root_shape(expr),
        target_text,
        head_ptr: head.ptr_key(),
        args_ptr: args.ptr_key(),
    };
    PRESENCE_REUSE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    record_presence_fingerprint(kind, expr, target_text, result, nodes, elapsed_nanos);
}

fn record_presence_fingerprint(
    kind: PresenceScanKind,
    expr: &Expr,
    target_text: Option<Symbol>,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    if !lexical_profile_shapes_enabled() || nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }

    let (text_fingerprint, exact_fingerprint) = structural_text_and_exact_fingerprints(expr);
    let key = PresenceFingerprintKey {
        kind,
        shape: presence_root_shape(expr),
        target_text,
        fingerprint: text_fingerprint,
    };
    let exact_key = PresenceExactFingerprintKey {
        kind,
        shape: key.shape,
        target_text,
        fingerprint: exact_fingerprint,
    };
    PRESENCE_FINGERPRINT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
    PRESENCE_EXACT_FINGERPRINT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(exact_key).or_default();
        record_presence_stats(entry, result, nodes, elapsed_nanos);
    });
}

fn presence_root_shape(expr: &Expr) -> PresenceRootShape {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::AtomSymbol(*symbol),
        Expr::Atom(_) => PresenceRootShape::AtomOther,
        Expr::Call { head, .. } => match head.as_ref() {
            Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::CallHeadSymbol(*symbol),
            Expr::Atom(_) => PresenceRootShape::CallHeadAtomOther,
            Expr::Call {
                head: nested_head, ..
            } => match nested_head.as_ref() {
                Expr::Atom(Atom::Symbol(symbol)) => PresenceRootShape::CallHeadCallSymbol(*symbol),
                _ => PresenceRootShape::CallHeadCallOther,
            },
        },
    }
}

fn current_lexical_profile_context() -> &'static str {
    if LEXICAL_CONTEXT_DEPTH.with(Cell::get) == 0 {
        return "root";
    }

    LEXICAL_CONTEXT_STACK.with(|stack| stack.borrow().last().copied().unwrap_or("root"))
}

fn current_arg_prefilter_site() -> Option<PresenceArgSite> {
    if ARG_PREFILTER_SITE_DEPTH.with(Cell::get) == 0 {
        return None;
    }

    ARG_PREFILTER_SITE_STACK.with(|stack| stack.borrow().last().copied())
}

fn current_expression_rebuild_source_symbol() -> Option<Symbol> {
    if EXPRESSION_REBUILD_SOURCE_DEPTH.with(Cell::get) == 0 {
        return None;
    }

    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| stack.borrow().last().copied())
}

fn record_presence_stats(
    stats: &mut PresenceShapeStats,
    result: bool,
    nodes: u64,
    elapsed_nanos: Option<u64>,
) {
    stats.scans = stats.scans.saturating_add(1);
    stats.nodes = stats.nodes.saturating_add(nodes);
    if let Some(elapsed_nanos) = elapsed_nanos {
        stats.timed = stats.timed.saturating_add(1);
        stats.nanos = stats.nanos.saturating_add(elapsed_nanos);
    }
    if result {
        stats.true_count = stats.true_count.saturating_add(1);
    } else {
        stats.false_count = stats.false_count.saturating_add(1);
    }
}

fn print_presence_shape_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_SHAPE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(16).enumerate() {
            eprintln!(
                "rum-profile lexical shape rank={} kind={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_context_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_CONTEXT_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(16).enumerate() {
            eprintln!(
                "rum-profile lexical context rank={} kind={} context={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.context,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_context_shape_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_CONTEXT_SHAPE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical context-shape rank={} kind={} context={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.context,
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_arg_site_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_ARG_SITE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical arg-site rank={} kind={} parent={} index={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.site.parent),
                key.site.index,
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_source_summary() {
    if !lexical_profile_sources_enabled() {
        return;
    }

    PRESENCE_SOURCE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile lexical source rank={} kind={} source={} context={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_source_shape_summary() {
    if !lexical_profile_sources_enabled() {
        return;
    }

    PRESENCE_SOURCE_SHAPE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile lexical source-shape rank={} kind={} source={} context={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_source_target_shape_summary() {
    if !lexical_profile_sources_enabled() {
        return;
    }

    PRESENCE_SOURCE_TARGET_SHAPE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile lexical source-target-shape rank={} kind={} source={} context={} target={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_source_fingerprint_summary() {
    if !lexical_profile_source_fingerprints_enabled() {
        return;
    }

    PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile lexical source-fingerprint rank={} fingerprint_kind={} kind={} source={} context={} target={} shape={} fingerprint={:#x} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_source_fingerprint_kind_name(key.fingerprint_kind),
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_root_shape_name(key.shape),
                key.fingerprint,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_source_fingerprint_group_summary() {
    if !lexical_profile_source_fingerprints_enabled() {
        return;
    }

    PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| {
        let mut groups =
            HashMap::<PresenceSourceFingerprintGroupKey, PresenceSourceFingerprintGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceSourceFingerprintGroupKey {
                    fingerprint_kind: key.fingerprint_kind,
                    kind: key.kind,
                    source: key.source,
                    context: key.context,
                    shape: key.shape,
                    target_text: key.target_text,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.scans = group.scans.saturating_add(stats.scans);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.timed = group.timed.saturating_add(stats.timed);
            group.nanos = group.nanos.saturating_add(stats.nanos);
            group.max_scans_per_fingerprint = group.max_scans_per_fingerprint.max(stats.scans);
            group.true_count = group.true_count.saturating_add(stats.true_count);
            group.false_count = group.false_count.saturating_add(stats.false_count);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile lexical source-fingerprint-group rank={} fingerprint_kind={} kind={} source={} context={} target={} shape={} unique_fingerprints={} scans={} max_scans_per_fingerprint={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_source_fingerprint_kind_name(key.fingerprint_kind),
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_root_shape_name(key.shape),
                stats.unique_fingerprints,
                stats.scans,
                stats.max_scans_per_fingerprint,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_provenance_summary() {
    if !lexical_profile_provenance_enabled() {
        return;
    }

    PRESENCE_PROVENANCE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile lexical provenance rank={} kind={} source={} context={} target={} target_origin={} expr_origin={} expr_source={} alpha_relation={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                profile_optional_symbol_name(key.source),
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_target_origin_name(key.target_origin),
                expression_provenance_kind_name(key.expr_origin),
                expression_rebuild_source_name(key.expr_origin_source, key.expr_origin_symbol),
                presence_alpha_relation_name(key.alpha_relation),
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_reuse_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_REUSE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical reuse rank={} kind={} shape={} target={} head_ptr={:#x} args_ptr={:#x} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                key.head_ptr,
                key.args_ptr,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_reuse_group_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_REUSE_STATS.with(|stats| {
        let mut groups = HashMap::<PresenceReuseGroupKey, PresenceReuseGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceReuseGroupKey {
                    kind: key.kind,
                    shape: key.shape,
                    target_text: key.target_text,
                })
                .or_default();
            group.unique_ptrs = group.unique_ptrs.saturating_add(1);
            group.scans = group.scans.saturating_add(stats.scans);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.timed = group.timed.saturating_add(stats.timed);
            group.nanos = group.nanos.saturating_add(stats.nanos);
            group.max_scans_per_ptr = group.max_scans_per_ptr.max(stats.scans);
            group.true_count = group.true_count.saturating_add(stats.true_count);
            group.false_count = group.false_count.saturating_add(stats.false_count);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical reuse-group rank={} kind={} shape={} target={} unique_ptrs={} scans={} max_scans_per_ptr={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                stats.unique_ptrs,
                stats.scans,
                stats.max_scans_per_ptr,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_fingerprint_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_FINGERPRINT_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical fingerprint rank={} kind={} shape={} target={} fingerprint={:#x} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                key.fingerprint,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_fingerprint_group_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_FINGERPRINT_STATS.with(|stats| {
        let mut groups =
            HashMap::<PresenceFingerprintGroupKey, PresenceFingerprintGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceFingerprintGroupKey {
                    kind: key.kind,
                    shape: key.shape,
                    target_text: key.target_text,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.scans = group.scans.saturating_add(stats.scans);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.timed = group.timed.saturating_add(stats.timed);
            group.nanos = group.nanos.saturating_add(stats.nanos);
            group.max_scans_per_fingerprint = group.max_scans_per_fingerprint.max(stats.scans);
            group.true_count = group.true_count.saturating_add(stats.true_count);
            group.false_count = group.false_count.saturating_add(stats.false_count);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical fingerprint-group rank={} kind={} shape={} target={} unique_fingerprints={} scans={} max_scans_per_fingerprint={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                stats.unique_fingerprints,
                stats.scans,
                stats.max_scans_per_fingerprint,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_exact_fingerprint_group_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    PRESENCE_EXACT_FINGERPRINT_STATS.with(|stats| {
        let mut groups =
            HashMap::<PresenceFingerprintGroupKey, PresenceFingerprintGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceFingerprintGroupKey {
                    kind: key.kind,
                    shape: key.shape,
                    target_text: key.target_text,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.scans = group.scans.saturating_add(stats.scans);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.timed = group.timed.saturating_add(stats.timed);
            group.nanos = group.nanos.saturating_add(stats.nanos);
            group.max_scans_per_fingerprint = group.max_scans_per_fingerprint.max(stats.scans);
            group.true_count = group.true_count.saturating_add(stats.true_count);
            group.false_count = group.false_count.saturating_add(stats.false_count);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical exact-fingerprint-group rank={} kind={} shape={} target={} unique_fingerprints={} scans={} max_scans_per_fingerprint={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                stats.unique_fingerprints,
                stats.scans,
                stats.max_scans_per_fingerprint,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_expression_rebuild_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    EXPRESSION_REBUILD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild rank={} source={} shape={} fingerprint={:#x} rebuilds={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                key.fingerprint,
                stats.rebuilds,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

fn print_expression_rebuild_group_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    EXPRESSION_REBUILD_STATS.with(|stats| {
        let mut groups = HashMap::<ExpressionRebuildGroupKey, ExpressionRebuildGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(ExpressionRebuildGroupKey {
                    source: key.source,
                    source_symbol: key.source_symbol,
                    shape: key.shape,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.rebuilds = group.rebuilds.saturating_add(stats.rebuilds);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.max_rebuilds_per_fingerprint =
                group.max_rebuilds_per_fingerprint.max(stats.rebuilds);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild-group rank={} source={} shape={} unique_fingerprints={} rebuilds={} max_rebuilds_per_fingerprint={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                stats.unique_fingerprints,
                stats.rebuilds,
                stats.max_rebuilds_per_fingerprint,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

fn print_expression_rebuild_exact_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild-exact rank={} source={} shape={} fingerprint={:#x} rebuilds={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                key.fingerprint,
                stats.rebuilds,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

fn print_expression_rebuild_exact_group_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| {
        let mut groups = HashMap::<ExpressionRebuildGroupKey, ExpressionRebuildGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(ExpressionRebuildGroupKey {
                    source: key.source,
                    source_symbol: key.source_symbol,
                    shape: key.shape,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.rebuilds = group.rebuilds.saturating_add(stats.rebuilds);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.max_rebuilds_per_fingerprint =
                group.max_rebuilds_per_fingerprint.max(stats.rebuilds);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild-exact-group rank={} source={} shape={} unique_fingerprints={} rebuilds={} max_rebuilds_per_fingerprint={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                stats.unique_fingerprints,
                stats.rebuilds,
                stats.max_rebuilds_per_fingerprint,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

fn print_rule_rhs_alpha_rename_summary() {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    RULE_RHS_ALPHA_RENAME_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.renames
                .cmp(&lhs.renames)
                .then_with(|| rhs.fresh_with_identity.cmp(&lhs.fresh_with_identity))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile rule-rhs-alpha-rename rank={} source={} local={} renames={} fresh_with_identity={}",
                rank + 1,
                profile_optional_symbol_name(key.source_symbol),
                key.local_text,
                stats.renames,
                stats.fresh_with_identity,
            );
        }
    });
}

fn print_rule_rhs_timing_summary() {
    if !rule_rhs_timing_enabled() {
        return;
    }

    RULE_RHS_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile rule-rhs-timing rank={} phase={} source={} calls={} total_ns={} avg_ns={} changed={} unchanged={}",
                rank + 1,
                key.phase,
                profile_optional_symbol_name(key.source_symbol),
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.changed,
                stats.unchanged,
            );
        }
    });
}

fn print_definition_rule_timing_summary() {
    if !definition_rule_timing_enabled() {
        return;
    }

    DEFINITION_RULE_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile definition-rule-timing rank={} kind={} symbol={} index={} calls={} total_ns={} avg_ns={} ok={} err={}",
                rank + 1,
                key.kind,
                key.symbol,
                key.index,
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.ok,
                stats.err,
            );
        }
    });
}

fn print_definition_dispatch_summary() {
    if !lexical_profile_enabled() {
        return;
    }

    DEFINITION_DISPATCH_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return;
        }

        let totals =
            entries
                .iter()
                .fold(DefinitionDispatchStats::default(), |mut acc, (_, stats)| {
                    acc.candidates = acc.candidates.saturating_add(stats.candidates);
                    acc.prefilter_skips = acc.prefilter_skips.saturating_add(stats.prefilter_skips);
                    acc.fast_matches = acc.fast_matches.saturating_add(stats.fast_matches);
                    acc.fast_misses = acc.fast_misses.saturating_add(stats.fast_misses);
                    acc.matcher_matches = acc.matcher_matches.saturating_add(stats.matcher_matches);
                    acc.matcher_misses = acc.matcher_misses.saturating_add(stats.matcher_misses);
                    acc.prefilter_nanos = acc.prefilter_nanos.saturating_add(stats.prefilter_nanos);
                    acc.fast_match_nanos =
                        acc.fast_match_nanos.saturating_add(stats.fast_match_nanos);
                    acc.compiled_matcher_nanos = acc
                        .compiled_matcher_nanos
                        .saturating_add(stats.compiled_matcher_nanos);
                    acc.rhs_substitution_nanos = acc
                        .rhs_substitution_nanos
                        .saturating_add(stats.rhs_substitution_nanos);
                    acc
                });
        eprintln!(
            "rum-profile definition-dispatch summary candidates={} prefilter_skips={} fast_matches={} fast_misses={} matcher_matches={} matcher_misses={} prefilter_skip_rate_per_mille={} matcher_miss_rate_per_mille={}",
            totals.candidates,
            totals.prefilter_skips,
            totals.fast_matches,
            totals.fast_misses,
            totals.matcher_matches,
            totals.matcher_misses,
            profile_stats_per_mille(totals.prefilter_skips, totals.candidates),
            profile_stats_per_mille(totals.matcher_misses, totals.candidates),
        );

        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.matcher_misses
                .cmp(&lhs.matcher_misses)
                .then_with(|| rhs.prefilter_skips.cmp(&lhs.prefilter_skips))
                .then_with(|| rhs.candidates.cmp(&lhs.candidates))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile definition-dispatch rank={} kind={} symbol={} source={} candidates={} prefilter_skips={} fast_matches={} fast_misses={} matcher_matches={} matcher_misses={} prefilter_skip_rate_per_mille={} matcher_miss_rate_per_mille={}",
                rank + 1,
                key.kind,
                key.symbol,
                profile_optional_symbol_name(key.source_symbol),
                stats.candidates,
                stats.prefilter_skips,
                stats.fast_matches,
                stats.fast_misses,
                stats.matcher_matches,
                stats.matcher_misses,
                profile_stats_per_mille(stats.prefilter_skips, stats.candidates),
                profile_stats_per_mille(stats.matcher_misses, stats.candidates),
            );
        }

        if definition_rule_timing_enabled() {
            eprintln!(
                "rum-profile definition-dispatch-time summary prefilter_ns={} fast_match_ns={} compiled_matcher_ns={} rhs_substitution_ns={}",
                totals.prefilter_nanos,
                totals.fast_match_nanos,
                totals.compiled_matcher_nanos,
                totals.rhs_substitution_nanos,
            );

            let mut entries = DEFINITION_DISPATCH_STATS.with(|stats| {
                stats
                    .borrow()
                    .iter()
                    .map(|(key, stats)| (*key, *stats))
                    .collect::<Vec<_>>()
            });
            entries.sort_by(|(_, lhs), (_, rhs)| {
                definition_dispatch_timing_total(rhs)
                    .cmp(&definition_dispatch_timing_total(lhs))
                    .then_with(|| rhs.candidates.cmp(&lhs.candidates))
            });

            for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
                let fast_attempts = stats.fast_matches.saturating_add(stats.fast_misses);
                let matcher_attempts = stats
                    .matcher_matches
                    .saturating_add(stats.matcher_misses);
                let rhs_substitutions = stats.fast_matches.saturating_add(stats.matcher_matches);
                eprintln!(
                    "rum-profile definition-dispatch-time rank={} kind={} symbol={} source={} total_ns={} candidates={} prefilter_ns={} prefilter_avg_ns={} fast_attempts={} fast_match_ns={} fast_match_avg_ns={} matcher_attempts={} compiled_matcher_ns={} compiled_matcher_avg_ns={} rhs_substitutions={} rhs_substitution_ns={} rhs_substitution_avg_ns={}",
                    rank + 1,
                    key.kind,
                    key.symbol,
                    profile_optional_symbol_name(key.source_symbol),
                    definition_dispatch_timing_total(&stats),
                    stats.candidates,
                    stats.prefilter_nanos,
                    profile_stats_average(stats.prefilter_nanos, stats.candidates),
                    fast_attempts,
                    stats.fast_match_nanos,
                    profile_stats_average(stats.fast_match_nanos, fast_attempts),
                    matcher_attempts,
                    stats.compiled_matcher_nanos,
                    profile_stats_average(stats.compiled_matcher_nanos, matcher_attempts),
                    rhs_substitutions,
                    stats.rhs_substitution_nanos,
                    profile_stats_average(stats.rhs_substitution_nanos, rhs_substitutions),
                );
            }
        }
    });
}

fn definition_dispatch_timing_total(stats: &DefinitionDispatchStats) -> u64 {
    stats
        .prefilter_nanos
        .saturating_add(stats.fast_match_nanos)
        .saturating_add(stats.compiled_matcher_nanos)
        .saturating_add(stats.rhs_substitution_nanos)
}

fn print_runtime_path_timing_summary() {
    if !lexical_profile_enabled() {
        return;
    }

    RUNTIME_PATH_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile runtime-path-timing rank={} phase={} source={} calls={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={} ok={} err={} some={} none={} return_err={} throw_err={} input_required={}",
                rank + 1,
                key.phase,
                profile_optional_symbol_name(key.source_symbol),
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.ok,
                stats.err,
                stats.some,
                stats.none,
                stats.return_err,
                stats.throw_err,
                stats.input_required,
            );
        }
    });

    RUNTIME_PATH_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.self_nanos
                .cmp(&lhs.self_nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile runtime-path-self-timing rank={} phase={} source={} calls={} self_total_ns={} self_avg_ns={} total_ns={} avg_ns={} ok={} err={} some={} none={} return_err={} throw_err={} input_required={}",
                rank + 1,
                key.phase,
                profile_optional_symbol_name(key.source_symbol),
                stats.calls,
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.ok,
                stats.err,
                stats.some,
                stats.none,
                stats.return_err,
                stats.throw_err,
                stats.input_required,
            );
        }
    });
}

fn print_builtin_timing_summary() {
    if !builtin_timing_enabled() {
        return;
    }

    BUILTIN_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile builtin-timing rank={} symbol={} source={} arg_count={} calls={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={} ok={} err={}",
                rank + 1,
                key.symbol,
                profile_optional_symbol_name(key.source_symbol),
                key.arg_count,
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.ok,
                stats.err,
            );
        }
    });

    BUILTIN_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.self_nanos
                .cmp(&lhs.self_nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile builtin-self-timing rank={} symbol={} source={} arg_count={} calls={} self_total_ns={} self_avg_ns={} total_ns={} avg_ns={} ok={} err={}",
                rank + 1,
                key.symbol,
                profile_optional_symbol_name(key.source_symbol),
                key.arg_count,
                stats.calls,
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.ok,
                stats.err,
            );
        }
    });
}

fn print_rewrite_rule_prefilter_summary() {
    let candidates = load(&REWRITE_RULE_CANDIDATE_COUNT);
    if candidates == 0 {
        return;
    }

    let would_reject = load(&REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT);
    let would_matcher = load(&REWRITE_RULE_WOULD_MATCHER_COUNT);
    let matcher_misses = load(&REWRITE_RULE_MATCHER_MISS_COUNT);
    let matches = load(&REWRITE_RULE_MATCH_COUNT);
    let false_rejects = load(&REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT);
    eprintln!(
        "rum-profile rewrite-rule prefilter summary candidates={} would_reject={} would_matcher={} matcher_misses={} matches={} false_rejects={} reject_rate_per_mille={} false_reject_rate_per_mille={}",
        candidates,
        would_reject,
        would_matcher,
        matcher_misses,
        matches,
        false_rejects,
        profile_stats_per_mille(would_reject, candidates),
        profile_stats_per_mille(false_rejects, candidates),
    );

    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(source, stats)| (*source, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.would_reject
                .cmp(&lhs.would_reject)
                .then_with(|| rhs.candidates.cmp(&lhs.candidates))
                .then_with(|| rhs.matcher_misses.cmp(&lhs.matcher_misses))
        });

        for (rank, (source, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile rewrite-rule prefilter-source rank={} source={} candidates={} would_reject={} would_matcher={} matcher_misses={} matches={} false_rejects={} reject_rate_per_mille={} false_reject_rate_per_mille={}",
                rank + 1,
                source,
                stats.candidates,
                stats.would_reject,
                stats.would_matcher,
                stats.matcher_misses,
                stats.matches,
                stats.false_rejects,
                profile_stats_per_mille(stats.would_reject, stats.candidates),
                profile_stats_per_mille(stats.false_rejects, stats.candidates),
            );
        }
    });
}

fn print_presence_binding_summary() {
    if !lexical_profile_bindings_enabled() {
        return;
    }

    PRESENCE_BINDING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical binding rank={} shape={} target={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_root_shape_name(key.shape),
                profile_symbol_name(key.target),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn print_presence_binding_name_summary() {
    if !lexical_profile_bindings_enabled() {
        return;
    }

    PRESENCE_BINDING_NAME_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical binding-name rank={} shape={} target={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_root_shape_name(key.shape),
                key.target_text,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

fn profile_stats_average(total: u64, count: u64) -> u64 {
    if count == 0 { 0 } else { total / count }
}

fn profile_stats_per_mille(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        0
    } else {
        numerator.saturating_mul(1000) / denominator
    }
}

fn profile_symbol_name(symbol: Symbol) -> String {
    match symbol.identity() {
        Some(identity) => format!("{symbol}#{}", identity),
        None => symbol.to_string(),
    }
}

fn profile_optional_symbol_name(symbol: Option<Symbol>) -> String {
    symbol.map_or_else(|| "-".to_string(), |symbol| symbol.to_string())
}

fn expression_rebuild_source_name(source: &'static str, symbol: Option<Symbol>) -> String {
    symbol.map_or_else(|| source.to_string(), |symbol| format!("{source}:{symbol}"))
}

fn presence_target_origin(symbol: Option<&Symbol>) -> PresenceTargetOrigin {
    let Some(symbol) = symbol else {
        return PresenceTargetOrigin::None;
    };

    if let Some(identity) = symbol.identity() {
        if identity & RULE_RHS_ALPHA_ID_MARKER != 0 {
            PresenceTargetOrigin::RuleRhsAlphaIdentity
        } else {
            PresenceTargetOrigin::OtherIdentity
        }
    } else if is_generated_dollar_name(symbol.name()) {
        PresenceTargetOrigin::GeneratedDollarName
    } else {
        PresenceTargetOrigin::PlainSymbol
    }
}

fn is_generated_dollar_name(name: &str) -> bool {
    let Some((_, suffix)) = name.rsplit_once('$') else {
        return false;
    };
    !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
}

fn rule_rhs_alpha_id(symbol: &Symbol) -> Option<u64> {
    symbol
        .identity()
        .filter(|identity| identity & RULE_RHS_ALPHA_ID_MARKER != 0)
        .map(|identity| identity & !RULE_RHS_ALPHA_ID_MARKER)
}

fn max_rule_rhs_alpha_id(expr: &Expr) -> Option<u64> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => rule_rhs_alpha_id(symbol),
        Expr::Atom(_) => None,
        Expr::Call { head, args } => {
            let mut max_id = max_rule_rhs_alpha_id(head);
            for arg in args.iter() {
                if let Some(arg_id) = max_rule_rhs_alpha_id(arg) {
                    max_id = Some(max_id.map_or(arg_id, |current| current.max(arg_id)));
                }
            }
            max_id
        }
    }
}

fn presence_alpha_relation(
    target: Option<&Symbol>,
    provenance: ExpressionProvenanceValue,
) -> PresenceAlphaRelation {
    let Some(target) = target else {
        return PresenceAlphaRelation::NoTarget;
    };
    let Some(target_id) = rule_rhs_alpha_id(target) else {
        return PresenceAlphaRelation::TargetNotRuleRhsAlpha;
    };

    match provenance.max_rule_rhs_alpha_id {
        Some(max_id) if max_id < target_id => PresenceAlphaRelation::ExprOlderThanTarget,
        Some(_) => PresenceAlphaRelation::ExprSameOrNewerThanTarget,
        None if provenance.kind == ExpressionProvenanceKind::Unknown => {
            PresenceAlphaRelation::Unknown
        }
        None => PresenceAlphaRelation::ExprHasNoRuleRhsAlpha,
    }
}

fn presence_target_origin_name(origin: PresenceTargetOrigin) -> &'static str {
    match origin {
        PresenceTargetOrigin::None => "none",
        PresenceTargetOrigin::PlainSymbol => "plain",
        PresenceTargetOrigin::GeneratedDollarName => "generated-dollar",
        PresenceTargetOrigin::RuleRhsAlphaIdentity => "rule-rhs-alpha",
        PresenceTargetOrigin::OtherIdentity => "other-identity",
    }
}

fn presence_alpha_relation_name(relation: PresenceAlphaRelation) -> &'static str {
    match relation {
        PresenceAlphaRelation::NoTarget => "no-target",
        PresenceAlphaRelation::TargetNotRuleRhsAlpha => "target-not-rule-rhs-alpha",
        PresenceAlphaRelation::ExprHasNoRuleRhsAlpha => "expr-has-no-rule-rhs-alpha",
        PresenceAlphaRelation::ExprOlderThanTarget => "expr-older-than-target",
        PresenceAlphaRelation::ExprSameOrNewerThanTarget => "expr-same-or-newer-than-target",
        PresenceAlphaRelation::Unknown => "unknown",
    }
}

fn expression_provenance_kind_name(kind: ExpressionProvenanceKind) -> &'static str {
    match kind {
        ExpressionProvenanceKind::RuleBindingValue => "rule-binding-value",
        ExpressionProvenanceKind::RuleRhsRebuild => "rule-rhs-rebuild",
        ExpressionProvenanceKind::OtherRebuild => "other-rebuild",
        ExpressionProvenanceKind::Unknown => "unknown",
    }
}

fn structural_text_and_exact_fingerprints(expr: &Expr) -> (u64, u64) {
    let mut text_hasher = DefaultHasher::new();
    let mut exact_hasher = DefaultHasher::new();
    let _nodes = hash_expr_text_and_exact(expr, &mut text_hasher, &mut exact_hasher);
    (text_hasher.finish(), exact_hasher.finish())
}

fn structural_text_exact_fingerprints_and_nodes(expr: &Expr) -> (u64, u64, u64) {
    let mut text_hasher = DefaultHasher::new();
    let mut exact_hasher = DefaultHasher::new();
    let nodes = hash_expr_text_and_exact(expr, &mut text_hasher, &mut exact_hasher);
    (text_hasher.finish(), exact_hasher.finish(), nodes)
}

fn hash_expr_text_and_exact(
    expr: &Expr,
    text_state: &mut impl Hasher,
    exact_state: &mut impl Hasher,
) -> u64 {
    match expr {
        Expr::Atom(atom) => {
            hash_atom_text_and_exact(atom, text_state, exact_state);
            1
        }
        Expr::Call { head, args } => {
            4_u8.hash(text_state);
            4_u8.hash(exact_state);
            let mut nodes =
                1_u64.saturating_add(hash_expr_text_and_exact(head, text_state, exact_state));
            args.len().hash(text_state);
            args.len().hash(exact_state);
            for arg in args.iter() {
                nodes =
                    nodes.saturating_add(hash_expr_text_and_exact(arg, text_state, exact_state));
            }
            nodes
        }
    }
}

fn hash_atom_text_and_exact(
    atom: &Atom,
    text_state: &mut impl Hasher,
    exact_state: &mut impl Hasher,
) {
    match atom {
        Atom::Integer(value) => {
            0_u8.hash(text_state);
            value.hash(text_state);
            0_u8.hash(exact_state);
            value.hash(exact_state);
        }
        Atom::Rational(value) => {
            1_u8.hash(text_state);
            value.hash(text_state);
            1_u8.hash(exact_state);
            value.hash(exact_state);
        }
        Atom::String(value) => {
            2_u8.hash(text_state);
            value.hash(text_state);
            2_u8.hash(exact_state);
            value.hash(exact_state);
        }
        Atom::Symbol(symbol) => {
            3_u8.hash(text_state);
            symbol.context().hash(text_state);
            symbol.name().hash(text_state);
            3_u8.hash(exact_state);
            symbol.hash(exact_state);
        }
    }
}

fn presence_scan_kind_name(kind: PresenceScanKind) -> &'static str {
    match kind {
        PresenceScanKind::Single => "single",
        PresenceScanKind::Multi => "multi",
    }
}

fn presence_source_fingerprint_kind_name(kind: PresenceSourceFingerprintKind) -> &'static str {
    match kind {
        PresenceSourceFingerprintKind::Text => "text",
        PresenceSourceFingerprintKind::Exact => "exact",
    }
}

fn presence_root_shape_name(shape: PresenceRootShape) -> String {
    match shape {
        PresenceRootShape::AtomSymbol(symbol) => format!("atom:{symbol}"),
        PresenceRootShape::AtomOther => "atom:<other>".to_string(),
        PresenceRootShape::CallHeadSymbol(symbol) => format!("call-head:{symbol}"),
        PresenceRootShape::CallHeadAtomOther => "call-head:<atom-other>".to_string(),
        PresenceRootShape::CallHeadCallSymbol(symbol) => format!("call-head-call:{symbol}"),
        PresenceRootShape::CallHeadCallOther => "call-head-call:<other>".to_string(),
    }
}
