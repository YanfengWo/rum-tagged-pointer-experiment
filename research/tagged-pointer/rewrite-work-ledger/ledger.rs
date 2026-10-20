//! Diagnostic-only, synchronous rewrite work accounting.
//!
//! A `replace` scope opens a root; other scopes and events are ignored outside
//! that root. A suspended operation's later resumption is a separate lifetime:
//! these counters do not attribute resumed work to an already dropped scope.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};

// 0 = uninitialized, 1 = disabled, 2 = enabled. No clocks or global counters.
static ENABLED: AtomicU8 = AtomicU8::new(0);

type RowKey = (String, &'static str, String, usize);

#[derive(Default)]
struct State {
    next_id: u64,
    active_id: Option<u64>,
    stack: Vec<&'static str>,
    rows: BTreeMap<RowKey, u64>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Keeps one synchronous phase active until dropped on its originating thread.
#[must_use = "keep the scope guard alive for the entire diagnostic phase"]
pub struct Scope {
    root_id: Option<u64>,
    depth: usize,
    label: &'static str,
    _same_thread: PhantomData<Rc<()>>,
}

fn enabled() -> bool {
    match ENABLED.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => load_enabled(),
    }
}

#[cold]
fn load_enabled() -> bool {
    let value = if std::env::var("RUM_REWRITE_WORK_LEDGER").as_deref() == Ok("1") {
        2
    } else {
        1
    };
    ENABLED.store(value, Ordering::Relaxed);
    value == 2
}

impl State {
    fn record(&mut self, kind: &'static str, detail: &str, amount: usize) {
        let key = (self.stack.join("/"), kind, detail.to_owned(), amount);
        let calls = self.rows.entry(key).or_default();
        *calls = calls.checked_add(1).expect("rewrite ledger call overflow");
    }
}

/// Starts a phase, or an inactive guard if no rewrite root is active.
pub fn scope(label: &'static str) -> Scope {
    let mut guard = Scope {
        root_id: None,
        depth: 0,
        label,
        _same_thread: PhantomData,
    };
    if !enabled() {
        return guard;
    }
    STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.active_id.is_none() {
            if label != "replace" {
                return;
            }
            assert!(state.stack.is_empty(), "rewrite ledger stale phase stack");
            assert!(state.rows.is_empty(), "rewrite ledger stale rows");
            state.next_id = state
                .next_id
                .checked_add(1)
                .expect("rewrite ledger root ID overflow");
            state.active_id = Some(state.next_id);
            eprintln!("rum-work\troot\t{}\tbegin", state.next_id);
        }
        state.stack.push(label);
        guard.root_id = state.active_id;
        guard.depth = state.stack.len();
        state.record("scope", label, 0);
    });
    guard
}

/// Records one call with a work amount; callers define the amount's unit.
pub fn event(kind: &'static str, detail: &str, amount: usize) {
    if !enabled() {
        return;
    }
    STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.active_id.is_some() {
            state.record(kind, detail, amount);
        }
    });
}

fn hex_detail(detail: &str) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(detail.len() * 2);
    for byte in detail.bytes() {
        hex.push(char::from(DIGITS[usize::from(byte >> 4)]));
        hex.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    hex
}

impl Drop for Scope {
    fn drop(&mut self) {
        let Some(root_id) = self.root_id else {
            return;
        };
        STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            // A mismatched guard lifetime invalidates attribution. Fail loudly
            // instead of emitting apparently complete but misattributed rows.
            assert_eq!(
                state.active_id,
                Some(root_id),
                "rewrite ledger root mismatch"
            );
            assert_eq!(state.stack.len(), self.depth, "rewrite ledger scope order");
            assert_eq!(state.stack.last().copied(), Some(self.label));
            state.stack.pop();
            if self.depth != 1 {
                return;
            }
            for ((phase, kind, detail, amount), calls) in &state.rows {
                eprintln!(
                    "rum-work\trow\t{root_id}\t{phase}\t{kind}\t{}\t{amount}\t{calls}",
                    hex_detail(detail),
                );
            }
            eprintln!("rum-work\troot\t{root_id}\tend");
            state.rows.clear();
            state.active_id = None;
        });
    }
}
