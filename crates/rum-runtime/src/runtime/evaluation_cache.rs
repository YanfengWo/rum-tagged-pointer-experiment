use super::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::panic::Location;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const EVALUATION_CACHE_PROFILE_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE";
const EVALUATION_CACHE_PROFILE_DEFER_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE_DEFER";
const EVALUATION_CACHE_PROFILE_INCREMENTAL_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL";
const CALL_STABILITY_CACHE_MAX_ENTRIES_ENV: &str = "RUM_CALL_STABILITY_CACHE_MAX_ENTRIES";
const STABILITY_DIRTY_BLOOM_MAX_BITS_ENV: &str = "RUM_STABILITY_DIRTY_BLOOM_MAX_BITS";

/// Entry-count bound for the pointer-keyed call-stability cache. Entries pin
/// their head/args trees alive, so the map must stay bounded now that
/// invalidation is generation-based instead of clearing on every write.
const CALL_STABILITY_CACHE_MAX_ENTRIES: usize = 1 << 14;

/// Dirty-bit population bound before bloom-based invalidation falls back to a
/// full call-cache generation bump.
const STABILITY_DIRTY_BLOOM_MAX_BITS: u32 = 96;

static EVALUATION_CACHE_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static EVALUATION_CACHE_PROFILE_ACTIVE: AtomicU8 = AtomicU8::new(0);
static EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED: AtomicU8 = AtomicU8::new(0);
static EPOCH_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
static CACHE_CLEAR_COUNT: AtomicU64 = AtomicU64::new(0);
static SYMBOL_INVALIDATION_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_GENERATION_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
static DIRTY_BLOOM_SATURATION_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_SIZE_CAP_CLEAR_COUNT: AtomicU64 = AtomicU64::new(0);
static STABILITY_CHECK_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_ABSENT_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_GENERATION_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_DIRTY_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_POINTER_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_INSERT_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT: AtomicU64 = AtomicU64::new(0);
static EVALUATE_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);
static DOWN_VALUE_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
static DOWN_VALUE_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
static SUB_VALUE_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
static BUILTIN_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
static EVAL_MODULE_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default)]
struct SymbolTimingProfile {
    count: u64,
    nanos: u64,
    self_nanos: u64,
}

#[derive(Clone, Copy, Debug)]
struct SymbolTimingFrame {
    name: &'static str,
    symbol: Symbol,
    child_nanos: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct InvalidationSiteKey {
    event: &'static str,
    file: &'static str,
    line: u32,
    source: Option<Symbol>,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct InvalidationSiteStats {
    count: u64,
    target_with_identity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct InvalidationFamilyKey {
    event: &'static str,
    file: &'static str,
    line: u32,
    source: Option<Symbol>,
    target_family: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CallStabilityHeadShape {
    AtomSymbol(Symbol),
    AtomOther,
    CallHeadSymbol(Symbol),
    CallOther,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct CallStabilitySourceHeadKey {
    source: Symbol,
    head: CallStabilityHeadShape,
}

#[derive(Clone, Copy, Debug, Default)]
struct CallStabilitySourceHeadStats {
    checks: u64,
    hits: u64,
    misses: u64,
    miss_absent: u64,
    miss_generation: u64,
    miss_dirty: u64,
    miss_pointer: u64,
    insertions: u64,
    rewritten_dirty_skips: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallStabilityProfileOutcome {
    Hit,
    MissAbsent,
    MissGeneration,
    MissDirty,
    MissPointer,
}

thread_local! {
    static EVALUATION_PROFILE_SOURCE_STACK: RefCell<Vec<Symbol>> = const { RefCell::new(Vec::new()) };
    static EVALUATION_PROFILE_SOURCE_DEPTH: Cell<usize> = const { Cell::new(0) };
    static EVALUATION_PROFILE_TIMING_STACK: RefCell<Vec<SymbolTimingFrame>> = const { RefCell::new(Vec::new()) };
    static CALL_STABILITY_SOURCE_HEAD_STATS: RefCell<HashMap<CallStabilitySourceHeadKey, CallStabilitySourceHeadStats>> =
        RefCell::new(HashMap::new());
}

pub(super) struct EvaluationProfileSourceGuard {
    active: bool,
}

impl Drop for EvaluationProfileSourceGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        EVALUATION_PROFILE_SOURCE_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        EVALUATION_PROFILE_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

/// One bloom bit per written symbol, derived from the interned pair id plus
/// the generated identity so distinct `Module` locals map to distinct bits.
pub(super) fn symbol_stability_bloom_bit(symbol: &Symbol) -> u128 {
    let identity = match symbol.identity() {
        Some(value) => value.rotate_left(1) | 1,
        None => 0,
    };
    let mix = u64::from(symbol.interned_id()).wrapping_mul(0x9e3779b97f4a7c15) ^ identity;
    1u128 << (mix >> 57)
}

impl Session {
    pub(super) fn record_profile_evaluate_entry(&self) {
        record_evaluation_cache_counter(&EVALUATE_ENTRY_COUNT, "evaluate_entries");
    }

    pub(super) fn record_profile_down_value_scan(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&DOWN_VALUE_SCAN_COUNT, "down_value_scans");
        record_symbol_profile_counter("down_value_scans", symbol);
    }

    pub(super) fn record_profile_down_value_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&DOWN_VALUE_APPLICATION_COUNT, "down_value_applications");
        record_symbol_profile_counter("down_value_applications", symbol);
    }

    pub(super) fn record_profile_sub_value_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&SUB_VALUE_APPLICATION_COUNT, "sub_value_applications");
        record_symbol_profile_counter("sub_value_applications", symbol);
    }

    pub(super) fn record_profile_builtin_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&BUILTIN_APPLICATION_COUNT, "builtin_applications");
        record_symbol_profile_counter("builtin_applications", symbol);
    }

    pub(super) fn record_profile_eval_module_entry(&self) {
        record_evaluation_cache_counter(&EVAL_MODULE_ENTRY_COUNT, "eval_module_entries");
        if let Some(symbol) = current_evaluation_profile_source_symbol() {
            record_symbol_profile_counter("eval_module_entries_by_source", &symbol);
        }
    }

    /// Full invalidation: definition-state change without a single owning
    /// symbol (package loads, context-path mutation, attribute changes).
    pub(super) fn bump_evaluation_epoch(&mut self) {
        record_evaluation_cache_counter(&EPOCH_BUMP_COUNT, "epoch_bumps");
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);
        self.clear_evaluation_caches();
    }

    /// Surgical invalidation for a definition write to one symbol: the
    /// symbol-keyed caches drop that symbol's entries exactly, and the
    /// pointer-keyed call cache is invalidated through a dirty bloom over
    /// written symbols. Re-writing a symbol whose bloom bit is already dirty
    /// records that bit as rewritten instead of bumping the whole call-cache
    /// generation, so unrelated stable entries can still hit.
    #[track_caller]
    pub(super) fn bump_evaluation_epoch_for_symbol(&mut self, symbol: &Symbol) {
        record_evaluation_cache_counter(&SYMBOL_INVALIDATION_COUNT, "symbol_invalidations");
        let source = current_evaluation_profile_source_symbol();
        let location = Location::caller();
        record_invalidation_site("symbol_invalidation", source, Some(symbol), location);
        if let Some(source) = source {
            record_symbol_profile_counter("symbol_invalidations_by_source", &source);
        }
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);

        self.stable_symbol_cache.remove(symbol);
        self.symbol_call_effect_cache.remove(&SymbolCallEffectKey {
            symbol: *symbol,
            args_empty: true,
        });
        self.symbol_call_effect_cache.remove(&SymbolCallEffectKey {
            symbol: *symbol,
            args_empty: false,
        });

        self.record_symbol_stability_write(symbol, location);
    }

    fn record_symbol_stability_write(
        &mut self,
        symbol: &Symbol,
        location: &'static Location<'static>,
    ) {
        let bit = symbol_stability_bloom_bit(symbol);
        let already_dirty = self.stability_dirty_bloom & bit != 0;
        let source = current_evaluation_profile_source_symbol();
        record_invalidation_site("stability_dirty_write", source, Some(symbol), location);
        if let Some(source) = source {
            record_symbol_profile_counter("stability_dirty_writes_by_source", &source);
            if already_dirty {
                record_invalidation_site(
                    "stability_rewritten_dirty_write",
                    Some(source),
                    Some(symbol),
                    location,
                );
                record_symbol_profile_counter(
                    "stability_rewritten_dirty_writes_by_source",
                    &source,
                );
            } else {
                record_invalidation_site(
                    "stability_new_dirty_write",
                    Some(source),
                    Some(symbol),
                    location,
                );
            }
        } else if already_dirty {
            record_invalidation_site(
                "stability_rewritten_dirty_write",
                None,
                Some(symbol),
                location,
            );
        } else {
            record_invalidation_site("stability_new_dirty_write", None, Some(symbol), location);
        }
        if !already_dirty
            && (self.stability_dirty_bloom | bit).count_ones() > stability_dirty_bloom_max_bits()
        {
            record_evaluation_cache_counter(
                &DIRTY_BLOOM_SATURATION_COUNT,
                "dirty_bloom_saturations",
            );
            let source = current_evaluation_profile_source_symbol();
            record_invalidation_site("dirty_bloom_saturation", source, Some(symbol), location);
            if let Some(source) = source {
                record_symbol_profile_counter("dirty_bloom_saturations_by_source", &source);
            }
            self.bump_call_stability_generation(Some(symbol), location);
        }

        if already_dirty {
            self.stability_rewritten_dirty_bloom |= bit;
        }
        self.stability_dirty_bloom |= bit;
    }

    fn bump_call_stability_generation(
        &mut self,
        target: Option<&Symbol>,
        location: &'static Location<'static>,
    ) {
        record_evaluation_cache_counter(&CALL_GENERATION_BUMP_COUNT, "call_generation_bumps");
        let source = current_evaluation_profile_source_symbol();
        record_invalidation_site("call_generation_bump", source, target, location);
        if let Some(source) = source {
            record_symbol_profile_counter("call_generation_bumps_by_source", &source);
        }
        self.call_stability_generation = self.call_stability_generation.saturating_add(1);
        self.stability_dirty_bloom = 0;
        self.stability_rewritten_dirty_bloom = 0;
        if self.call_stability_cache.len() > call_stability_cache_max_entries() {
            self.clear_call_stability_cache_for_size_cap();
        }
    }

    pub(super) fn clear_evaluation_caches(&mut self) {
        record_evaluation_cache_counter(&CACHE_CLEAR_COUNT, "cache_clears");
        if let Some(source) = current_evaluation_profile_source_symbol() {
            record_symbol_profile_counter("cache_clears_by_source", &source);
            record_symbol_profile_counter("call_generation_bumps_by_source", &source);
        }
        self.symbol_cache_generation = self.symbol_cache_generation.saturating_add(1);
        self.call_stability_generation = self.call_stability_generation.saturating_add(1);
        self.stability_dirty_bloom = 0;
        self.stability_rewritten_dirty_bloom = 0;
        self.stable_symbol_cache.clear();
        self.symbol_call_effect_cache.clear();
        self.call_stability_cache.clear();
    }

    pub(super) fn cached_expr_is_evaluation_stable(&mut self, expr: &Expr) -> bool {
        self.cached_expr_stability_with_bloom(expr).0
    }

    pub(super) fn cached_expr_stability_with_bloom(&mut self, expr: &Expr) -> (bool, u128) {
        record_evaluation_cache_counter(&STABILITY_CHECK_COUNT, "stability_checks");
        match expr {
            Expr::Call { head, args } => self.call_stability_with_bloom(head, args),
            Expr::Atom(Atom::Symbol(symbol)) => (
                self.symbol_is_atom_evaluation_stable(symbol),
                symbol_stability_bloom_bit(symbol),
            ),
            Expr::Atom(_) => (true, 0),
        }
    }

    fn call_stability_with_bloom(&mut self, head: &SharedExpr, args: &SharedArgs) -> (bool, u128) {
        let profile_key = call_stability_source_head_key(head.as_ref());
        let key = CallStabilityCacheKey::new(head, args);
        if let Some(entry) = self.call_stability_cache.get(&key) {
            if entry.generation != self.call_stability_generation {
                record_evaluation_cache_counter(
                    &CALL_STABILITY_MISS_GENERATION_COUNT,
                    "call_stability_miss_generation",
                );
                record_call_stability_source_head_outcome(
                    profile_key,
                    CallStabilityProfileOutcome::MissGeneration,
                );
            } else if !SharedExpr::ptr_eq(&entry.head, head)
                || !SharedArgs::ptr_eq(&entry.args, args)
            {
                record_evaluation_cache_counter(
                    &CALL_STABILITY_MISS_POINTER_COUNT,
                    "call_stability_miss_pointer",
                );
                record_call_stability_source_head_outcome(
                    profile_key,
                    CallStabilityProfileOutcome::MissPointer,
                );
            } else {
                let dirty_delta = self.call_stability_entry_dirty_delta(entry);
                if entry.bloom & dirty_delta == 0 {
                    record_evaluation_cache_counter(
                        &CALL_STABILITY_HIT_COUNT,
                        "call_stability_hits",
                    );
                    record_call_stability_source_head_outcome(
                        profile_key,
                        CallStabilityProfileOutcome::Hit,
                    );
                    return (entry.stable, entry.bloom);
                }
                record_evaluation_cache_counter(
                    &CALL_STABILITY_MISS_DIRTY_COUNT,
                    "call_stability_miss_dirty",
                );
                record_call_stability_source_head_outcome(
                    profile_key,
                    CallStabilityProfileOutcome::MissDirty,
                );
            }
        } else {
            record_evaluation_cache_counter(
                &CALL_STABILITY_MISS_ABSENT_COUNT,
                "call_stability_miss_absent",
            );
            record_call_stability_source_head_outcome(
                profile_key,
                CallStabilityProfileOutcome::MissAbsent,
            );
        }

        record_evaluation_cache_counter(&CALL_STABILITY_MISS_COUNT, "call_stability_misses");
        let (stable, bloom) = self.call_stability_uncached_with_bloom(head, args);
        // Only stable verdicts are worth caching: an unstable expression is
        // about to be rewritten by evaluation, so its entry would pin a dead
        // tree and never be asked again, while stable subtrees are re-checked
        // by their parents for as long as they stay alive.
        let rewritten_dirty_overlap = bloom & self.stability_rewritten_dirty_bloom != 0;
        if stable && !rewritten_dirty_overlap {
            if self.call_stability_cache.len() > call_stability_cache_max_entries() {
                self.clear_call_stability_cache_for_size_cap();
            }
            let dirty_snapshot = self.stability_dirty_bloom;
            record_evaluation_cache_counter(
                &CALL_STABILITY_INSERT_COUNT,
                "call_stability_insertions",
            );
            record_call_stability_source_head_insertion(profile_key);
            self.call_stability_cache.insert(
                key,
                CallStabilityCacheEntry {
                    generation: self.call_stability_generation,
                    head: head.clone(),
                    args: args.clone(),
                    stable,
                    bloom,
                    dirty_snapshot,
                },
            );
        } else if stable {
            record_evaluation_cache_counter(
                &CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT,
                "call_stability_rewritten_dirty_skips",
            );
            record_call_stability_source_head_rewritten_dirty_skip(profile_key);
        }
        (stable, bloom)
    }

    fn call_stability_entry_dirty_delta(&self, entry: &CallStabilityCacheEntry) -> u128 {
        let newly_dirty = self.stability_dirty_bloom & !entry.dirty_snapshot;
        let rewritten_after_snapshot = self.stability_rewritten_dirty_bloom & entry.dirty_snapshot;
        newly_dirty | rewritten_after_snapshot
    }

    fn clear_call_stability_cache_for_size_cap(&mut self) {
        record_evaluation_cache_counter(
            &CALL_STABILITY_SIZE_CAP_CLEAR_COUNT,
            "call_stability_size_cap_clears",
        );
        self.call_stability_cache.clear();
    }
}

#[inline]
fn call_stability_cache_max_entries() -> usize {
    static MAX_ENTRIES: OnceLock<usize> = OnceLock::new();
    *MAX_ENTRIES.get_or_init(|| {
        std::env::var(CALL_STABILITY_CACHE_MAX_ENTRIES_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(CALL_STABILITY_CACHE_MAX_ENTRIES)
    })
}

#[inline]
fn stability_dirty_bloom_max_bits() -> u32 {
    static MAX_BITS: OnceLock<u32> = OnceLock::new();
    *MAX_BITS.get_or_init(|| {
        std::env::var(STABILITY_DIRTY_BLOOM_MAX_BITS_ENV)
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value >= 1 && *value <= 128)
            .unwrap_or(STABILITY_DIRTY_BLOOM_MAX_BITS)
    })
}

#[inline]
fn evaluation_cache_profile_enabled() -> bool {
    evaluation_cache_profile_configured() && evaluation_cache_profile_active()
}

#[inline]
fn evaluation_cache_profile_configured() -> bool {
    match EVALUATION_CACHE_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_enabled(),
    }
}

#[inline]
fn evaluation_cache_profile_active() -> bool {
    match EVALUATION_CACHE_PROFILE_ACTIVE.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_active(),
    }
}

#[inline]
fn evaluation_cache_profile_incremental_enabled() -> bool {
    match EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_incremental_enabled(),
    }
}

#[cold]
fn initialize_evaluation_cache_profile_enabled() -> bool {
    let enabled = std::env::var_os(EVALUATION_CACHE_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    EVALUATION_CACHE_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_evaluation_cache_profile_active() -> bool {
    let deferred = std::env::var_os(EVALUATION_CACHE_PROFILE_DEFER_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    let active = !deferred;
    EVALUATION_CACHE_PROFILE_ACTIVE.store(if active { 1 } else { 2 }, Ordering::Relaxed);
    active
}

#[inline]
pub(super) fn evaluation_profile_definition_source(symbol: Symbol) -> EvaluationProfileSourceGuard {
    if !evaluation_cache_profile_enabled() {
        return EvaluationProfileSourceGuard { active: false };
    }

    EVALUATION_PROFILE_SOURCE_STACK.with(|stack| {
        stack.borrow_mut().push(symbol);
    });
    EVALUATION_PROFILE_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    EvaluationProfileSourceGuard { active: true }
}

#[inline]
pub(super) fn evaluation_cache_profile_source_timing_start(name: &'static str) -> Option<Instant> {
    if !evaluation_cache_profile_enabled() {
        return None;
    }

    let symbol = current_evaluation_profile_source_symbol()?;
    let start = Instant::now();
    EVALUATION_PROFILE_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(SymbolTimingFrame {
            name,
            symbol,
            child_nanos: 0,
        });
    });
    Some(start)
}

pub(super) fn evaluation_profile_current_source_symbol() -> Option<Symbol> {
    current_evaluation_profile_source_symbol()
}

#[cold]
fn initialize_evaluation_cache_profile_incremental_enabled() -> bool {
    let enabled = std::env::var_os(EVALUATION_CACHE_PROFILE_INCREMENTAL_ENV)
        .map(|value| !value.is_empty() && value != "0")
        .unwrap_or(true);
    EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED
        .store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[inline]
fn record_evaluation_cache_counter(counter: &AtomicU64, name: &str) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_evaluation_cache_counter_slow(counter, name);
}

#[cold]
#[inline(never)]
fn record_evaluation_cache_counter_slow(counter: &AtomicU64, name: &str) {
    let value = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if evaluation_cache_profile_incremental_enabled() && value.is_power_of_two() {
        eprintln!("rum-profile evaluation-cache {name}={value}");
    }
}

fn symbol_profile_counters() -> &'static Mutex<HashMap<(&'static str, Symbol), u64>> {
    static SYMBOL_PROFILE_COUNTERS: OnceLock<Mutex<HashMap<(&'static str, Symbol), u64>>> =
        OnceLock::new();
    SYMBOL_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn invalidation_site_profile_counters()
-> &'static Mutex<HashMap<InvalidationSiteKey, InvalidationSiteStats>> {
    static INVALIDATION_SITE_PROFILE_COUNTERS: OnceLock<
        Mutex<HashMap<InvalidationSiteKey, InvalidationSiteStats>>,
    > = OnceLock::new();
    INVALIDATION_SITE_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn invalidation_family_profile_counters()
-> &'static Mutex<HashMap<InvalidationFamilyKey, InvalidationSiteStats>> {
    static INVALIDATION_FAMILY_PROFILE_COUNTERS: OnceLock<
        Mutex<HashMap<InvalidationFamilyKey, InvalidationSiteStats>>,
    > = OnceLock::new();
    INVALIDATION_FAMILY_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn symbol_timing_profile_counters()
-> &'static Mutex<HashMap<(&'static str, Symbol), SymbolTimingProfile>> {
    static SYMBOL_TIMING_PROFILE_COUNTERS: OnceLock<
        Mutex<HashMap<(&'static str, Symbol), SymbolTimingProfile>>,
    > = OnceLock::new();
    SYMBOL_TIMING_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn record_symbol_profile_counter(name: &'static str, symbol: &Symbol) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_symbol_profile_counter_slow(name, *symbol);
}

#[cold]
#[inline(never)]
fn record_symbol_profile_counter_slow(name: &'static str, symbol: Symbol) {
    let mut counters = symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned");
    let value = counters.entry((name, symbol)).or_insert(0);
    *value += 1;
    if evaluation_cache_profile_incremental_enabled() && value.is_power_of_two() {
        eprintln!("rum-profile evaluation-cache {name}[{symbol}]={value}");
    }
}

fn record_invalidation_site(
    event: &'static str,
    source: Option<Symbol>,
    target: Option<&Symbol>,
    location: &'static Location<'static>,
) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    let target_text = target.map(|symbol| Symbol::new(symbol.context(), symbol.name()));
    let target_family = target.map(symbol_family_name);
    let target_has_identity = target.and_then(Symbol::identity).is_some();
    let key = InvalidationSiteKey {
        event,
        file: location.file(),
        line: location.line(),
        source,
        target_text,
    };
    let mut counters = invalidation_site_profile_counters()
        .lock()
        .expect("profile invalidation-site counter lock should not be poisoned");
    let entry = counters.entry(key).or_default();
    entry.count = entry.count.saturating_add(1);
    if target_has_identity {
        entry.target_with_identity = entry.target_with_identity.saturating_add(1);
    }

    let family_key = InvalidationFamilyKey {
        event,
        file: location.file(),
        line: location.line(),
        source,
        target_family,
    };
    let mut family_counters = invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned");
    let family_entry = family_counters.entry(family_key).or_default();
    family_entry.count = family_entry.count.saturating_add(1);
    if target_has_identity {
        family_entry.target_with_identity = family_entry.target_with_identity.saturating_add(1);
    }
}

pub(super) fn record_profile_definition_rhs_eval_by_source(start: Option<Instant>) {
    record_current_source_timing("definition_rhs_eval_by_source", start);
}

pub(super) fn record_profile_eval_module_by_source(start: Option<Instant>) {
    record_current_source_timing("eval_module_by_source", start);
}

pub(super) fn record_profile_module_init_substitution_by_source(start: Option<Instant>) {
    record_current_source_timing("module_init_substitution_by_source", start);
}

pub(super) fn record_profile_module_body_substitution_by_source(start: Option<Instant>) {
    record_current_source_timing("module_body_substitution_by_source", start);
}

fn record_current_source_timing(name: &'static str, start: Option<Instant>) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    let Some(symbol) = current_evaluation_profile_source_symbol() else {
        return;
    };
    record_symbol_timing_profile_counter(name, symbol, start);
}

fn current_evaluation_profile_source_symbol() -> Option<Symbol> {
    if !evaluation_cache_profile_enabled() {
        return None;
    }

    EVALUATION_PROFILE_SOURCE_STACK.with(|stack| stack.borrow().last().copied())
}

fn record_symbol_timing_profile_counter(
    name: &'static str,
    symbol: Symbol,
    start: Option<Instant>,
) {
    let Some(start) = start else {
        return;
    };
    let elapsed_nanos = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let child_nanos = EVALUATION_PROFILE_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop();
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }

        frame
            .filter(|frame| frame.name == name && frame.symbol == symbol)
            .map(|frame| frame.child_nanos)
            .unwrap_or(0)
    });
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);

    let mut counters = symbol_timing_profile_counters()
        .lock()
        .expect("profile timing counter lock should not be poisoned");
    let entry = counters.entry((name, symbol)).or_default();
    entry.count = entry.count.saturating_add(1);
    entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
    entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
}

fn call_stability_source_head_key(head: &Expr) -> Option<CallStabilitySourceHeadKey> {
    if !evaluation_cache_profile_enabled() {
        return None;
    }

    let source = current_evaluation_profile_source_symbol()?;
    Some(CallStabilitySourceHeadKey {
        source,
        head: call_stability_head_shape(head),
    })
}

fn call_stability_head_shape(head: &Expr) -> CallStabilityHeadShape {
    match head {
        Expr::Atom(Atom::Symbol(symbol)) => CallStabilityHeadShape::AtomSymbol(*symbol),
        Expr::Atom(_) => CallStabilityHeadShape::AtomOther,
        Expr::Call {
            head: nested_head, ..
        } => match nested_head.as_ref() {
            Expr::Atom(Atom::Symbol(symbol)) => CallStabilityHeadShape::CallHeadSymbol(*symbol),
            _ => CallStabilityHeadShape::CallOther,
        },
    }
}

fn record_call_stability_source_head_outcome(
    key: Option<CallStabilitySourceHeadKey>,
    outcome: CallStabilityProfileOutcome,
) {
    let Some(key) = key else {
        return;
    };

    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.checks = entry.checks.saturating_add(1);
        match outcome {
            CallStabilityProfileOutcome::Hit => {
                entry.hits = entry.hits.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissAbsent => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_absent = entry.miss_absent.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissGeneration => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_generation = entry.miss_generation.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissDirty => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_dirty = entry.miss_dirty.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissPointer => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_pointer = entry.miss_pointer.saturating_add(1);
            }
        }
    });
}

fn record_call_stability_source_head_insertion(key: Option<CallStabilitySourceHeadKey>) {
    let Some(key) = key else {
        return;
    };

    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.insertions = entry.insertions.saturating_add(1);
    });
}

fn record_call_stability_source_head_rewritten_dirty_skip(key: Option<CallStabilitySourceHeadKey>) {
    let Some(key) = key else {
        return;
    };

    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.rewritten_dirty_skips = entry.rewritten_dirty_skips.saturating_add(1);
    });
}

pub(super) fn reset_evaluation_cache_profile_counters() {
    if !evaluation_cache_profile_configured() {
        return;
    }

    macro_rules! reset {
        ($($counter:ident),+ $(,)?) => {
            $( $counter.store(0, Ordering::Relaxed); )+
        };
    }

    reset!(
        EPOCH_BUMP_COUNT,
        CACHE_CLEAR_COUNT,
        SYMBOL_INVALIDATION_COUNT,
        CALL_GENERATION_BUMP_COUNT,
        DIRTY_BLOOM_SATURATION_COUNT,
        CALL_STABILITY_SIZE_CAP_CLEAR_COUNT,
        STABILITY_CHECK_COUNT,
        CALL_STABILITY_HIT_COUNT,
        CALL_STABILITY_MISS_COUNT,
        CALL_STABILITY_MISS_ABSENT_COUNT,
        CALL_STABILITY_MISS_GENERATION_COUNT,
        CALL_STABILITY_MISS_DIRTY_COUNT,
        CALL_STABILITY_MISS_POINTER_COUNT,
        CALL_STABILITY_INSERT_COUNT,
        CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT,
        EVALUATE_ENTRY_COUNT,
        DOWN_VALUE_SCAN_COUNT,
        DOWN_VALUE_APPLICATION_COUNT,
        SUB_VALUE_APPLICATION_COUNT,
        BUILTIN_APPLICATION_COUNT,
        EVAL_MODULE_ENTRY_COUNT,
    );

    symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned")
        .clear();
    invalidation_site_profile_counters()
        .lock()
        .expect("profile invalidation-site counter lock should not be poisoned")
        .clear();
    invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned")
        .clear();
    symbol_timing_profile_counters()
        .lock()
        .expect("profile timing counter lock should not be poisoned")
        .clear();
    EVALUATION_PROFILE_SOURCE_STACK.with(|stack| stack.borrow_mut().clear());
    EVALUATION_PROFILE_SOURCE_DEPTH.with(|depth| depth.set(0));
    EVALUATION_PROFILE_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| stats.borrow_mut().clear());
    EVALUATION_CACHE_PROFILE_ACTIVE.store(1, Ordering::Relaxed);
}

pub(super) fn print_evaluation_cache_profile_snapshot(label: &str) {
    if !evaluation_cache_profile_configured() {
        return;
    }

    eprintln!("rum-profile evaluation-cache snapshot label={label}");
    eprintln!(
        "rum-profile evaluation-cache summary epoch_bumps={} cache_clears={} symbol_invalidations={} call_generation_bumps={} dirty_bloom_saturations={} call_stability_size_cap_clears={}",
        load(&EPOCH_BUMP_COUNT),
        load(&CACHE_CLEAR_COUNT),
        load(&SYMBOL_INVALIDATION_COUNT),
        load(&CALL_GENERATION_BUMP_COUNT),
        load(&DIRTY_BLOOM_SATURATION_COUNT),
        load(&CALL_STABILITY_SIZE_CAP_CLEAR_COUNT),
    );
    eprintln!(
        "rum-profile evaluation-cache summary stability_checks={} call_stability_hits={} call_stability_misses={} miss_absent={} miss_generation={} miss_dirty={} miss_pointer={} insertions={} rewritten_dirty_skips={}",
        load(&STABILITY_CHECK_COUNT),
        load(&CALL_STABILITY_HIT_COUNT),
        load(&CALL_STABILITY_MISS_COUNT),
        load(&CALL_STABILITY_MISS_ABSENT_COUNT),
        load(&CALL_STABILITY_MISS_GENERATION_COUNT),
        load(&CALL_STABILITY_MISS_DIRTY_COUNT),
        load(&CALL_STABILITY_MISS_POINTER_COUNT),
        load(&CALL_STABILITY_INSERT_COUNT),
        load(&CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT),
    );
    eprintln!(
        "rum-profile evaluation-cache summary evaluate_entries={} down_value_scans={} down_value_applications={} sub_value_applications={} builtin_applications={} eval_module_entries={}",
        load(&EVALUATE_ENTRY_COUNT),
        load(&DOWN_VALUE_SCAN_COUNT),
        load(&DOWN_VALUE_APPLICATION_COUNT),
        load(&SUB_VALUE_APPLICATION_COUNT),
        load(&BUILTIN_APPLICATION_COUNT),
        load(&EVAL_MODULE_ENTRY_COUNT),
    );
    print_symbol_profile_summary();
    print_symbol_invalidation_profile_summary();
    print_invalidation_site_profile_summary();
    print_invalidation_family_profile_summary();
    print_invalidation_critical_profile_summary();
    print_symbol_timing_profile_summary();
    print_call_stability_source_head_summary();
    print_call_stability_source_head_invalidation_summary();
}

fn print_symbol_profile_summary() {
    let counters = symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|((name, symbol), count)| (*name, *symbol, *count))
        .collect::<Vec<_>>();

    entries.sort_by(
        |(left_name, left_symbol, left_count), (right_name, right_symbol, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, count)) in entries.into_iter().take(32).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache symbol rank={} name={} symbol={} count={}",
            rank + 1,
            name,
            symbol,
            count,
        );
    }
}

fn print_symbol_invalidation_profile_summary() {
    let counters = symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .filter(|((name, _), _)| {
            matches!(
                *name,
                "symbol_invalidations_by_source"
                    | "stability_dirty_writes_by_source"
                    | "stability_rewritten_dirty_writes_by_source"
                    | "dirty_bloom_saturations_by_source"
                    | "call_generation_bumps_by_source"
                    | "cache_clears_by_source"
            )
        })
        .map(|((name, symbol), count)| (*name, *symbol, *count))
        .collect::<Vec<_>>();

    entries.sort_by(
        |(left_name, left_symbol, left_count), (right_name, right_symbol, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, count)) in entries.into_iter().take(64).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-source rank={} name={} source={} count={}",
            rank + 1,
            name,
            symbol,
            count,
        );
    }
}

fn print_invalidation_site_profile_summary() {
    let counters = invalidation_site_profile_counters()
        .lock()
        .expect("profile invalidation-site counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|(key, stats)| (*key, *stats))
        .collect::<Vec<_>>();

    entries.sort_by(|(left_key, left_stats), (right_key, right_stats)| {
        right_stats
            .count
            .cmp(&left_stats.count)
            .then_with(|| left_key.event.cmp(right_key.event))
            .then_with(|| left_key.file.cmp(right_key.file))
            .then_with(|| left_key.line.cmp(&right_key.line))
            .then_with(|| {
                optional_symbol_sort_name(left_key.source)
                    .cmp(&optional_symbol_sort_name(right_key.source))
            })
            .then_with(|| {
                optional_symbol_sort_name(left_key.target_text)
                    .cmp(&optional_symbol_sort_name(right_key.target_text))
            })
    });

    for (rank, (key, stats)) in entries.into_iter().take(80).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-site rank={} event={} file={} line={} source={} target={} count={} target_with_identity={}",
            rank + 1,
            key.event,
            key.file,
            key.line,
            optional_symbol_profile_name(key.source),
            optional_symbol_profile_name(key.target_text),
            stats.count,
            stats.target_with_identity,
        );
    }
}

fn print_invalidation_family_profile_summary() {
    let counters = invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|(key, stats)| (key.clone(), *stats))
        .collect::<Vec<_>>();

    entries.sort_by(|(left_key, left_stats), (right_key, right_stats)| {
        right_stats
            .count
            .cmp(&left_stats.count)
            .then_with(|| left_key.event.cmp(right_key.event))
            .then_with(|| left_key.file.cmp(right_key.file))
            .then_with(|| left_key.line.cmp(&right_key.line))
            .then_with(|| {
                optional_symbol_sort_name(left_key.source)
                    .cmp(&optional_symbol_sort_name(right_key.source))
            })
            .then_with(|| {
                optional_string_sort_name(&left_key.target_family)
                    .cmp(&optional_string_sort_name(&right_key.target_family))
            })
    });

    for (rank, (key, stats)) in entries.into_iter().take(80).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-family rank={} event={} file={} line={} source={} target_family={} count={} target_with_identity={}",
            rank + 1,
            key.event,
            key.file,
            key.line,
            optional_symbol_profile_name(key.source),
            optional_string_profile_name(&key.target_family),
            stats.count,
            stats.target_with_identity,
        );
    }
}

fn print_invalidation_critical_profile_summary() {
    let counters = invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .filter(|(key, _)| matches!(key.event, "dirty_bloom_saturation" | "call_generation_bump"))
        .map(|(key, stats)| (key.clone(), *stats))
        .collect::<Vec<_>>();

    entries.sort_by(|(left_key, left_stats), (right_key, right_stats)| {
        right_stats
            .count
            .cmp(&left_stats.count)
            .then_with(|| left_key.event.cmp(right_key.event))
            .then_with(|| left_key.file.cmp(right_key.file))
            .then_with(|| left_key.line.cmp(&right_key.line))
            .then_with(|| {
                optional_symbol_sort_name(left_key.source)
                    .cmp(&optional_symbol_sort_name(right_key.source))
            })
            .then_with(|| {
                optional_string_sort_name(&left_key.target_family)
                    .cmp(&optional_string_sort_name(&right_key.target_family))
            })
    });

    for (rank, (key, stats)) in entries.into_iter().take(80).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-critical rank={} event={} file={} line={} source={} target_family={} count={} target_with_identity={}",
            rank + 1,
            key.event,
            key.file,
            key.line,
            optional_symbol_profile_name(key.source),
            optional_string_profile_name(&key.target_family),
            stats.count,
            stats.target_with_identity,
        );
    }
}

fn print_symbol_timing_profile_summary() {
    let counters = symbol_timing_profile_counters()
        .lock()
        .expect("profile timing counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|((name, symbol), stats)| (*name, *symbol, *stats))
        .collect::<Vec<_>>();

    entries.sort_by(
        |(left_name, left_symbol, left_stats), (right_name, right_symbol, right_stats)| {
            right_stats
                .nanos
                .cmp(&left_stats.nanos)
                .then_with(|| right_stats.count.cmp(&left_stats.count))
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, stats)) in entries.into_iter().take(32).enumerate() {
        let average = if stats.count == 0 {
            0
        } else {
            stats.nanos / stats.count
        };
        let self_average = if stats.count == 0 {
            0
        } else {
            stats.self_nanos / stats.count
        };
        eprintln!(
            "rum-profile evaluation-cache symbol-timing rank={} name={} symbol={} count={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={}",
            rank + 1,
            name,
            symbol,
            stats.count,
            stats.nanos,
            average,
            stats.self_nanos,
            self_average,
        );
    }

    let mut self_entries = counters
        .iter()
        .map(|((name, symbol), stats)| (*name, *symbol, *stats))
        .collect::<Vec<_>>();
    self_entries.sort_by(
        |(left_name, left_symbol, left_stats), (right_name, right_symbol, right_stats)| {
            right_stats
                .self_nanos
                .cmp(&left_stats.self_nanos)
                .then_with(|| right_stats.count.cmp(&left_stats.count))
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, stats)) in self_entries.into_iter().take(32).enumerate() {
        let self_average = if stats.count == 0 {
            0
        } else {
            stats.self_nanos / stats.count
        };
        eprintln!(
            "rum-profile evaluation-cache symbol-self-timing rank={} name={} symbol={} count={} self_total_ns={} self_avg_ns={} total_ns={}",
            rank + 1,
            name,
            symbol,
            stats.count,
            stats.self_nanos,
            self_average,
            stats.nanos,
        );
    }
}

fn print_call_stability_source_head_summary() {
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.checks
                .cmp(&lhs.checks)
                .then_with(|| rhs.misses.cmp(&lhs.misses))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile evaluation-cache call-stability-source-head rank={} source={} head={} checks={} hits={} misses={} miss_absent={} miss_generation={} miss_dirty={} miss_pointer={} insertions={} rewritten_dirty_skips={}",
                rank + 1,
                key.source,
                call_stability_head_shape_name(key.head),
                stats.checks,
                stats.hits,
                stats.misses,
                stats.miss_absent,
                stats.miss_generation,
                stats.miss_dirty,
                stats.miss_pointer,
                stats.insertions,
                stats.rewritten_dirty_skips,
            );
        }
    });
}

fn print_call_stability_source_head_invalidation_summary() {
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.miss_dirty
                .saturating_add(rhs.miss_generation)
                .cmp(&lhs.miss_dirty.saturating_add(lhs.miss_generation))
                .then_with(|| rhs.misses.cmp(&lhs.misses))
                .then_with(|| rhs.checks.cmp(&lhs.checks))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            let invalidation_misses = stats.miss_dirty.saturating_add(stats.miss_generation);
            if invalidation_misses == 0 {
                continue;
            }
            eprintln!(
                "rum-profile evaluation-cache call-stability-source-head-invalidated rank={} source={} head={} invalidation_misses={} miss_generation={} miss_dirty={} misses={} checks={} hits={} insertions={} rewritten_dirty_skips={}",
                rank + 1,
                key.source,
                call_stability_head_shape_name(key.head),
                invalidation_misses,
                stats.miss_generation,
                stats.miss_dirty,
                stats.misses,
                stats.checks,
                stats.hits,
                stats.insertions,
                stats.rewritten_dirty_skips,
            );
        }
    });
}

fn call_stability_head_shape_name(shape: CallStabilityHeadShape) -> String {
    match shape {
        CallStabilityHeadShape::AtomSymbol(symbol) => symbol.to_string(),
        CallStabilityHeadShape::AtomOther => "atom-other".to_string(),
        CallStabilityHeadShape::CallHeadSymbol(symbol) => {
            format!("call-head:{symbol}")
        }
        CallStabilityHeadShape::CallOther => "call-head-other".to_string(),
    }
}

fn optional_symbol_profile_name(symbol: Option<Symbol>) -> String {
    symbol.map_or_else(|| "-".to_string(), |symbol| symbol.to_string())
}

fn optional_symbol_sort_name(symbol: Option<Symbol>) -> String {
    optional_symbol_profile_name(symbol)
}

fn optional_string_profile_name(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

fn optional_string_sort_name(value: &Option<String>) -> String {
    optional_string_profile_name(value).to_string()
}

fn symbol_family_name(symbol: &Symbol) -> String {
    let name = symbol.name();
    let base_name = name
        .rsplit_once('$')
        .filter(|(_, suffix)| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map(|(base_name, _)| base_name)
        .unwrap_or(name);
    format!("{}{base_name}", symbol.context())
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_cache_is_invalidated_by_new_down_value() {
        let mut session = Session::default();

        assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
        assert!(!session.symbol_call_effect_cache.is_empty());

        session.eval_text("f[x_] := x + 1").unwrap();

        assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "2");
    }

    #[test]
    fn evaluation_cache_is_invalidated_by_attribute_change() {
        let mut session = Session::default();

        assert_eq!(
            session.eval_text("f[{a, b}]").unwrap().value.to_string(),
            "f[List[a, b]]"
        );
        assert!(!session.symbol_call_effect_cache.is_empty());

        session.eval_text("SetAttributes[f, Listable]").unwrap();

        assert_eq!(
            session.eval_text("f[{a, b}]").unwrap().value.to_string(),
            "List[f[a], f[b]]"
        );
    }

    #[test]
    fn repeated_writes_to_one_symbol_keep_invalidation_sound() {
        let mut session = Session::default();

        session.eval_text("g[] := (s = s + 1; s)").unwrap();
        session.eval_text("s = 0").unwrap();
        assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "1");
        assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "2");
        assert_eq!(session.eval_text("s").unwrap().value.to_string(), "2");
    }

    #[test]
    fn stability_cache_survives_unrelated_symbol_writes() {
        let mut session = Session::default();

        // Populate the call-stability cache with a stable expression.
        assert_eq!(
            session.eval_text("h[1, 2]").unwrap().value.to_string(),
            "h[1, 2]"
        );

        // Writing an unrelated symbol must not change the stable verdict...
        session.eval_text("unrelated = 5").unwrap();
        assert_eq!(
            session.eval_text("h[1, 2]").unwrap().value.to_string(),
            "h[1, 2]"
        );

        // ...while a write to the observed head must flip it.
        session.eval_text("h[x_, y_] := x + y").unwrap();
        assert_eq!(session.eval_text("h[1, 2]").unwrap().value.to_string(), "3");
    }

    #[test]
    fn repeated_unrelated_writes_do_not_bump_call_stability_generation() {
        let mut session = Session::default();

        session.eval_text("stableExpr = h[1, 2, 3]").unwrap();
        session.clear_evaluation_caches();
        assert_eq!(
            session.eval_text("stableExpr").unwrap().value.to_string(),
            "h[1, 2, 3]"
        );
        assert!(!session.call_stability_cache.is_empty());

        let generation = session.call_stability_generation;
        session.eval_text("unrelatedWrite = 1").unwrap();
        session.eval_text("unrelatedWrite = 2").unwrap();
        session.eval_text("unrelatedWrite = 3").unwrap();

        assert_eq!(session.call_stability_generation, generation);
        assert_eq!(
            session.eval_text("stableExpr").unwrap().value.to_string(),
            "h[1, 2, 3]"
        );
    }
}
