use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

const REPR_PROFILE_ENV: &str = "RUM_PROFILE_REPR";

static REPR_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static INTO_VEC_TOTAL: AtomicU64 = AtomicU64::new(0);
static INTO_VEC_CLONE: AtomicU64 = AtomicU64::new(0);
static INTO_OWNED_TOTAL: AtomicU64 = AtomicU64::new(0);
static INTO_OWNED_CLONE: AtomicU64 = AtomicU64::new(0);
static SHARED_ARGS_ALLOC: AtomicU64 = AtomicU64::new(0);
static SHARED_EXPR_ALLOC: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReprProfile {
    pub into_vec_total: u64,
    pub into_vec_clone: u64,
    pub into_owned_total: u64,
    pub into_owned_clone: u64,
    pub shared_args_alloc: u64,
    pub shared_expr_alloc: u64,
}

#[inline]
pub fn repr_profile_enabled() -> bool {
    match REPR_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_repr_profile_enabled(),
    }
}

#[cold]
fn initialize_repr_profile_enabled() -> bool {
    let enabled =
        std::env::var_os(REPR_PROFILE_ENV).is_some_and(|value| !value.is_empty() && value != "0");
    REPR_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[inline]
pub fn record_into_vec(cloned: bool) {
    if !repr_profile_enabled() {
        return;
    }

    record_into_vec_slow(cloned);
}

#[cold]
#[inline(never)]
fn record_into_vec_slow(cloned: bool) {
    INTO_VEC_TOTAL.fetch_add(1, Ordering::Relaxed);
    if cloned {
        INTO_VEC_CLONE.fetch_add(1, Ordering::Relaxed);
    }
}

#[inline]
pub fn record_into_owned(cloned: bool) {
    if !repr_profile_enabled() {
        return;
    }

    record_into_owned_slow(cloned);
}

#[cold]
#[inline(never)]
fn record_into_owned_slow(cloned: bool) {
    INTO_OWNED_TOTAL.fetch_add(1, Ordering::Relaxed);
    if cloned {
        INTO_OWNED_CLONE.fetch_add(1, Ordering::Relaxed);
    }
}

#[inline]
pub fn record_shared_args_alloc() {
    if !repr_profile_enabled() {
        return;
    }

    record_shared_args_alloc_slow();
}

#[cold]
#[inline(never)]
fn record_shared_args_alloc_slow() {
    SHARED_ARGS_ALLOC.fetch_add(1, Ordering::Relaxed);
}

#[inline]
pub fn record_shared_expr_alloc() {
    if !repr_profile_enabled() {
        return;
    }

    record_shared_expr_alloc_slow();
}

#[cold]
#[inline(never)]
fn record_shared_expr_alloc_slow() {
    SHARED_EXPR_ALLOC.fetch_add(1, Ordering::Relaxed);
}

pub fn repr_profile_snapshot() -> ReprProfile {
    ReprProfile {
        into_vec_total: load(&INTO_VEC_TOTAL),
        into_vec_clone: load(&INTO_VEC_CLONE),
        into_owned_total: load(&INTO_OWNED_TOTAL),
        into_owned_clone: load(&INTO_OWNED_CLONE),
        shared_args_alloc: load(&SHARED_ARGS_ALLOC),
        shared_expr_alloc: load(&SHARED_EXPR_ALLOC),
    }
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}
