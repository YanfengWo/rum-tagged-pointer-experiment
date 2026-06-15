use super::{Atom, Expr, Symbol};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

const LEXICAL_PROFILE_ENV: &str = "RUM_PROFILE_LEXICAL_SUBSTITUTION";
const LEXICAL_PROFILE_DETAIL_ENV: &str = "RUM_PROFILE_LEXICAL_DETAIL";
const LEXICAL_PROFILE_INCREMENTAL_ENV: &str = "RUM_PROFILE_LEXICAL_INCREMENTAL";
const LEXICAL_PROFILE_SHAPES_ENV: &str = "RUM_PROFILE_LEXICAL_SHAPES";
const LEXICAL_PROFILE_BINDINGS_ENV: &str = "RUM_PROFILE_LEXICAL_BINDINGS";

static LEXICAL_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_DETAIL_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_INCREMENTAL_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_SHAPES_ENABLED: AtomicU8 = AtomicU8::new(0);
static LEXICAL_PROFILE_BINDINGS_ENABLED: AtomicU8 = AtomicU8::new(0);
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

thread_local! {
    static PRESENCE_SHAPE_STATS: RefCell<HashMap<PresenceShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    static PRESENCE_BINDING_STATS: RefCell<HashMap<PresenceBindingKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
}

#[inline]
pub(super) fn lexical_profile_enabled() -> bool {
    match LEXICAL_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_enabled(),
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

#[inline]
pub(super) fn lexical_profile_start() -> Option<Instant> {
    if lexical_profile_enabled() {
        Some(Instant::now())
    } else {
        None
    }
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
    record_presence_shape(PresenceScanKind::Single, expr, result, nodes, elapsed_nanos);
    record_presence_binding(expr, target, result, nodes, elapsed_nanos);
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
}

pub(super) fn print_lexical_profile_summary_once() {
    if !lexical_profile_enabled() {
        return;
    }
    if LEXICAL_PROFILE_SUMMARY_PRINTED.swap(1, Ordering::Relaxed) == 1 {
        return;
    }

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
    print_presence_binding_summary();
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

fn profile_stats_average(total: u64, count: u64) -> u64 {
    if count == 0 { 0 } else { total / count }
}

fn profile_symbol_name(symbol: Symbol) -> String {
    match symbol.identity() {
        Some(identity) => format!("{symbol}#{}", identity),
        None => symbol.to_string(),
    }
}

fn presence_scan_kind_name(kind: PresenceScanKind) -> &'static str {
    match kind {
        PresenceScanKind::Single => "single",
        PresenceScanKind::Multi => "multi",
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
