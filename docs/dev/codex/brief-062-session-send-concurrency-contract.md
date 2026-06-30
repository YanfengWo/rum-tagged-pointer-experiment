# Brief 062 — Make HostSession `Send` + nail the concurrency/isolation contract

## Goal

Diagnosed 2026-07-01: `HostSession` is currently **neither `Send` nor `Sync`**. The
`Send` blocker is the single `Box<dyn SuspensionFrame>` (the suspension/resume
continuation trait object lacks a `+ Send` bound); `Cell<usize>` (memory proxy) and
`Box<dyn FnMut + Send>` (print sink) additionally block `Sync`. For the Tier-2
"long-running, multi-client host" use case, a session must at least be **movable to a
worker thread** (`Send`) — a thread pool / actor model can't hand a session to a worker
otherwise. When you are done: `HostSession` (and the inner `Session`) is `Send`, proven
by a static assertion and a real parallel-sessions isolation test; the contract (`Send`
yes, `Sync` no-by-design, plus the process-global interner) is explicit and tested.
`Sync` is deliberately NOT a goal — a session is single-threaded mutable state; sharing
it is wrong, so `!Sync` is the correct contract.

This is the Tier-2 "concurrency / isolation contract" item from
`docs/host-embedding-and-hardening.md`. Adding `+ Send` is a marker-trait bound — ZERO
runtime cost, no behavior change; it either compiles (done) or the compiler pinpoints a
`!Send` capture to fix.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-2 ("Concurrency /
  isolation contract. Is `HostSession` `Send`/`Sync`? … the global symbol interner is a
  `Mutex<HashSet>` shared process-wide").
- The Send blocker: `crates/rum-runtime/src/runtime/suspension/core.rs` — `SuspendedEvaluation.frames:
  VecDeque<Box<dyn SuspensionFrame>>` (:7), `trait SuspensionFrame` (:10), `struct
  ClosureFrame<F>` + its impl (:24-), `push_suspension_frame<F>(...)` (:195, the
  `Box::new(ClosureFrame { ... })` at :203). Add `+ Send` to the trait-object type and the
  generic `F` bounds so the boxed frames are `Send`.
- The frame closures are created at ~306 `drive_internal_value` / `suspend_with_frame*` /
  `push_suspension_frame` call sites across the runtime. They capture continuation state —
  `Expr` (Arc-shared = `Send`), `SymbolExprMap` bindings, `MatcherArg`s, `Symbol`s — which
  are Arc-based and almost certainly already `Send`. The compiler will flag any that are
  not.
- Session `!Sync` blockers (DO NOT try to remove — `!Sync` is the intended contract):
  `Cell<usize>` `peak_memory_proxy_bytes` (model.rs:372), `Box<dyn FnMut + Send>`
  `print_sink` (:371). A session is owned + mutated single-threaded; it must not be shared.
- The interner: a process-global `Mutex<HashSet>` (the memory line). N parallel sessions
  share it → it is a contention point AND a shared interned-NAME table (but definitions /
  ownvalues / contexts are per-`Session`, NOT shared — that is the isolation to prove).

## Design (do this; deviate only with a reported reason)

1. Add `+ Send` to the suspension-frame trait object: `VecDeque<Box<dyn SuspensionFrame + Send>>`
   and the `push_suspension_frame<F>` / `ClosureFrame` generic bound (`F: … + Send`).
   `cargo build -p rum-runtime`, then fix EACH `!Send`-capture the compiler reports. The
   expected fixes (if any) are small and local (a captured value that should be Arc/owned).
   **STOP-LOSS: if a reported `!Send` capture cannot be fixed cheaply / locally (e.g. it
   would require reshaping a core data structure or it reveals a genuinely thread-hostile
   value), STOP and report the specific type — do NOT force it. We will then document the
   `!Send` contract instead.**
2. Do NOT touch the `Cell` / `print_sink` — `Sync` stays (correctly) unavailable.
3. Static-assert test (rum-host): a test that calls `fn is_send<T: Send>() {}` with
   `HostSession` and `HostCancelHandle`, and (as a guard documenting intent) a comment that
   `Sync` is intentionally NOT provided.
4. Parallel-isolation test (rum-host): spawn N (e.g. 8) threads; **MOVE a freshly-built
   `HostSession` into each** (this only compiles once `Send` lands — it is the proof);
   each thread evaluates a DISTINCT definition + readback (e.g. thread i does
   `x = <i>; x * x`) and asserts it gets its own `i*i` with no cross-thread interference;
   join all and check every result. This proves Send + per-session definition isolation
   despite the shared interner.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/suspension/core.rs` (the `+ Send` bounds);
  ONLY the specific runtime files the compiler flags for a `!Send` capture (keep each fix
  minimal + local; list them in the report); `crates/rum-host/src/tests.rs` (the two
  tests). You MAY add `+ Send` bounds where required but do NOT otherwise reshape types.
- MUST NOT touch: anything under any `mma-hol` clone/standard; `crates/rum-core/src/expr.rs`
  `Expr`/`Symbol`/serde shape; the `Cell`/`print_sink` fields; `AGENTS.md`/`PROGRESS.md`/
  `TODO.md`/`docs/**` except your own report; `.git`, `Cargo.lock`.
- **No new `unsafe`** (no `unsafe impl Send` — make it Send HONESTLY via the bound; if it
  can't be honest, STOP and report). This is the whole point: a real `Send`, not an
  asserted one.

## rum / Rust pitfalls (read twice)

1. `+ Send` is a marker bound — ZERO runtime cost, no behavior change. No microbench needed
   (state that in the report). It is a compile-time contract change only.
2. Do NOT `unsafe impl Send for Session`. If the bound can't be satisfied honestly, that
   means a real `!Send` value lives in the session — STOP and report it, don't paper over.
3. Do NOT pursue `Sync`. `Cell` + `FnMut` + the single-threaded mutation model make `Sync`
   wrong; sessions are owned-not-shared. Leave them.
4. The parallel test must MOVE the session into the thread (that is the `Send` proof) and
   assert isolation (each session's definitions are its own). Use the process-global
   interner as-is — the test proves definitions don't leak across sessions even though the
   name table is shared.
5. Minimal diff; only add bounds, do not refactor.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # the +Send bound ripples through the whole runtime build
```
Paste the workspace summary AND the two new test results (the static-assert compiling is
itself the Send proof; the parallel test output). If the compiler flagged any `!Send`
capture, list each one and the minimal fix you applied (or STOP and report if unfixable).

No microbench (marker-trait change, zero runtime cost) — state that explicitly.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes for Claude's gate.
- No new `unsafe`; specifically NO `unsafe impl Send`. Honest `Send` or STOP+report.
- Do NOT touch `Cell`/`print_sink`; do NOT chase `Sync`.
- Never edit any `mma-hol` file. Minimal diff (bounds only, + compiler-forced local fixes).
- Stop-loss: an unfixable/non-local `!Send` capture, or the same failure twice → STOP and
  report the specific type so we can document `!Send` instead.
- Paste REAL verbatim output.

## Report format (final message)

1. What changed — per file, line ranges, one-line why. List every `!Send`-capture fix the
   compiler forced (or "none — only the trait-object + generic bounds").
2. Verification — verbatim workspace summary + the static-assert + parallel test output.
3. Finding — `HostSession` is now `Send` (proof: the parallel test moves it into threads);
   `Sync` intentionally not provided (Cell/FnMut); definitions proven isolated per session
   despite the shared interner. Note the interner Mutex as a known contention point.
4. Stop-loss / open questions (empty if none).
