use super::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

const EVALUATION_CACHE_PROFILE_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE";

/// Entry-count bound for the pointer-keyed call-stability cache. Entries pin
/// their head/args trees alive, so the map must stay bounded now that
/// invalidation is generation-based instead of clearing on every write.
const CALL_STABILITY_CACHE_MAX_ENTRIES: usize = 1 << 14;

/// Dirty-bit population bound before bloom-based invalidation falls back to a
/// full call-cache generation bump.
const STABILITY_DIRTY_BLOOM_MAX_BITS: u32 = 96;

static EVALUATION_CACHE_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static EPOCH_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
static CACHE_CLEAR_COUNT: AtomicU64 = AtomicU64::new(0);
static SYMBOL_INVALIDATION_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_GENERATION_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
static STABILITY_CHECK_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
static EVALUATE_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);
static DOWN_VALUE_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
static DOWN_VALUE_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
static BUILTIN_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);

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

    pub(super) fn record_profile_builtin_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&BUILTIN_APPLICATION_COUNT, "builtin_applications");
        record_symbol_profile_counter("builtin_applications", symbol);
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
    pub(super) fn bump_evaluation_epoch_for_symbol(&mut self, symbol: &Symbol) {
        record_evaluation_cache_counter(&SYMBOL_INVALIDATION_COUNT, "symbol_invalidations");
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

        self.record_symbol_stability_write(symbol);
    }

    fn record_symbol_stability_write(&mut self, symbol: &Symbol) {
        let bit = symbol_stability_bloom_bit(symbol);
        let already_dirty = self.stability_dirty_bloom & bit != 0;
        if !already_dirty
            && (self.stability_dirty_bloom | bit).count_ones() > STABILITY_DIRTY_BLOOM_MAX_BITS
        {
            self.bump_call_stability_generation();
        }

        if already_dirty {
            self.stability_rewritten_dirty_bloom |= bit;
        }
        self.stability_dirty_bloom |= bit;
    }

    fn bump_call_stability_generation(&mut self) {
        record_evaluation_cache_counter(&CALL_GENERATION_BUMP_COUNT, "call_generation_bumps");
        self.call_stability_generation = self.call_stability_generation.saturating_add(1);
        self.stability_dirty_bloom = 0;
        self.stability_rewritten_dirty_bloom = 0;
        if self.call_stability_cache.len() > CALL_STABILITY_CACHE_MAX_ENTRIES {
            self.call_stability_cache.clear();
        }
    }

    pub(super) fn clear_evaluation_caches(&mut self) {
        record_evaluation_cache_counter(&CACHE_CLEAR_COUNT, "cache_clears");
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
        let key = CallStabilityCacheKey::new(head, args);
        if let Some(entry) = self.call_stability_cache.get(&key) {
            if entry.generation == self.call_stability_generation
                && SharedExpr::ptr_eq(&entry.head, head)
                && SharedArgs::ptr_eq(&entry.args, args)
                && entry.bloom & self.call_stability_entry_dirty_delta(entry) == 0
            {
                record_evaluation_cache_counter(&CALL_STABILITY_HIT_COUNT, "call_stability_hits");
                return (entry.stable, entry.bloom);
            }
        }

        record_evaluation_cache_counter(&CALL_STABILITY_MISS_COUNT, "call_stability_misses");
        let (stable, bloom) = self.call_stability_uncached_with_bloom(head, args);
        // Only stable verdicts are worth caching: an unstable expression is
        // about to be rewritten by evaluation, so its entry would pin a dead
        // tree and never be asked again, while stable subtrees are re-checked
        // by their parents for as long as they stay alive.
        if stable && bloom & self.stability_rewritten_dirty_bloom == 0 {
            if self.call_stability_cache.len() > CALL_STABILITY_CACHE_MAX_ENTRIES {
                self.call_stability_cache.clear();
            }
            let dirty_snapshot = self.stability_dirty_bloom;
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
        }
        (stable, bloom)
    }

    fn call_stability_entry_dirty_delta(&self, entry: &CallStabilityCacheEntry) -> u128 {
        let newly_dirty = self.stability_dirty_bloom & !entry.dirty_snapshot;
        let rewritten_after_snapshot = self.stability_rewritten_dirty_bloom & entry.dirty_snapshot;
        newly_dirty | rewritten_after_snapshot
    }
}

#[inline]
fn evaluation_cache_profile_enabled() -> bool {
    match EVALUATION_CACHE_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_enabled(),
    }
}

#[cold]
fn initialize_evaluation_cache_profile_enabled() -> bool {
    let enabled = std::env::var_os(EVALUATION_CACHE_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    EVALUATION_CACHE_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
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
    if value.is_power_of_two() {
        eprintln!("rum-profile evaluation-cache {name}={value}");
    }
}

fn symbol_profile_counters() -> &'static Mutex<HashMap<(&'static str, Symbol), u64>> {
    static SYMBOL_PROFILE_COUNTERS: OnceLock<Mutex<HashMap<(&'static str, Symbol), u64>>> =
        OnceLock::new();
    SYMBOL_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
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
    if value.is_power_of_two() {
        eprintln!("rum-profile evaluation-cache {name}[{symbol}]={value}");
    }
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
