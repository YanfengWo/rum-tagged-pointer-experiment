use super::*;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

const EVALUATION_CACHE_PROFILE_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE";

static EVALUATION_CACHE_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static EPOCH_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
static CACHE_CLEAR_COUNT: AtomicU64 = AtomicU64::new(0);
static STABILITY_CHECK_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static CALL_STABILITY_MISS_COUNT: AtomicU64 = AtomicU64::new(0);

impl Session {
    pub(super) fn bump_evaluation_epoch(&mut self) {
        record_evaluation_cache_counter(&EPOCH_BUMP_COUNT, "epoch_bumps");
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);
        self.clear_evaluation_caches();
    }

    pub(super) fn clear_evaluation_caches(&mut self) {
        record_evaluation_cache_counter(&CACHE_CLEAR_COUNT, "cache_clears");
        self.stable_symbol_cache.clear();
        self.symbol_call_effect_cache.clear();
        self.call_stability_cache.clear();
    }

    pub(super) fn cached_expr_is_evaluation_stable(&mut self, expr: &Expr) -> bool {
        record_evaluation_cache_counter(&STABILITY_CHECK_COUNT, "stability_checks");
        match expr {
            Expr::Call { head, args } => self.call_is_evaluation_stable(head, args),
            Expr::Atom(_) => self.expr_is_evaluation_stable_uncached(expr),
        }
    }

    pub(super) fn call_is_evaluation_stable(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> bool {
        let key = CallStabilityCacheKey::new(head, args);
        if let Some(entry) = self.call_stability_cache.get(&key) {
            if entry.epoch == self.evaluation_epoch
                && SharedExpr::ptr_eq(&entry.head, head)
                && SharedArgs::ptr_eq(&entry.args, args)
            {
                record_evaluation_cache_counter(&CALL_STABILITY_HIT_COUNT, "call_stability_hits");
                return entry.stable;
            }
        }

        record_evaluation_cache_counter(&CALL_STABILITY_MISS_COUNT, "call_stability_misses");
        let stable = self.call_is_evaluation_stable_uncached(head, args);
        self.call_stability_cache.insert(
            key,
            CallStabilityCacheEntry {
                epoch: self.evaluation_epoch,
                head: head.clone(),
                args: args.clone(),
                stable,
            },
        );
        stable
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
}
