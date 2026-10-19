//! Diagnostic-copy build only. Never included in a counter-off timing binary.
use std::panic::Location;
use std::sync::atomic::{AtomicUsize, Ordering};

static STACKS: AtomicUsize = AtomicUsize::new(0);

#[track_caller]
pub(crate) fn record(kind: &str, slots: usize, strong: usize, weak: usize) {
    if !std::env::var_os("RUM_TRACE_COPIES").is_some_and(|v| !v.is_empty() && v != "0") {
        return;
    }
    let site = Location::caller();
    eprintln!(
        "rum-copy event kind={kind} slots={slots} shallow_slot_bytes={} strong={strong} weak={weak} caller={}:{}",
        slots.saturating_mul(std::mem::size_of::<crate::Expr>()), site.file(), site.line()
    );
    if std::env::var_os("RUM_COPY_STACKS").is_some_and(|v| !v.is_empty() && v != "0")
        && STACKS.fetch_add(1, Ordering::Relaxed) < 128
    {
        eprintln!("rum-copy stack-begin\n{}\nrum-copy stack-end", std::backtrace::Backtrace::force_capture());
    }
}
