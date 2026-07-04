use super::*;

#[inline]
pub(in crate::runtime) fn lexical_profile_enabled() -> bool {
    lexical_profile_configured() && lexical_profile_active()
}

#[inline]
pub(super) fn lexical_profile_configured() -> bool {
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
pub(in crate::runtime) fn lexical_profile_detail_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_DETAIL_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_detail_enabled(),
        }
}

#[inline]
pub(super) fn lexical_profile_incremental_enabled() -> bool {
    match LEXICAL_PROFILE_INCREMENTAL_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_incremental_enabled(),
    }
}

#[inline]
pub(super) fn lexical_profile_shapes_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_SHAPES_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_shapes_enabled(),
        }
}

#[inline]
pub(super) fn lexical_profile_bindings_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_BINDINGS_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_bindings_enabled(),
        }
}

#[inline]
pub(super) fn lexical_profile_sources_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_SOURCES_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_sources_enabled(),
        }
}

#[inline]
pub(super) fn lexical_profile_source_fingerprints_enabled() -> bool {
    lexical_profile_sources_enabled()
        && match LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_source_fingerprints_enabled(),
        }
}

#[inline]
pub(super) fn lexical_profile_provenance_enabled() -> bool {
    lexical_profile_enabled()
        && match LEXICAL_PROFILE_PROVENANCE_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_lexical_profile_provenance_enabled(),
        }
}

#[inline]
pub(super) fn rule_rhs_timing_enabled() -> bool {
    lexical_profile_enabled()
        && match RULE_RHS_TIMING_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_rule_rhs_timing_enabled(),
        }
}

#[inline]
pub(super) fn definition_rule_timing_enabled() -> bool {
    lexical_profile_enabled()
        && match DEFINITION_RULE_TIMING_ENABLED.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => initialize_definition_rule_timing_enabled(),
        }
}

#[inline]
pub(super) fn builtin_timing_enabled() -> bool {
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
