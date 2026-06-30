# Brief 060 — Host panic-safety boundary (catch_unwind + session poison)

## Goal

A Rust `panic!` inside evaluation currently unwinds straight through the `rum-host`
API and out of the embedding — a future FFI / pyo3 / WASM boundary MUST NOT let a panic
cross (it is UB across `extern "C"`, and it takes the host process down). When you are
done: every public `HostSession` entry point that runs guest code catches an unwinding
panic at the boundary, converts it to a graceful `HostEvalOutcome::Failure` with a new
`HostFailureCode::Panic` (carrying the panic message), and **poisons the session** so a
subsequent call on the now-possibly-inconsistent session returns a clean Poisoned
failure instead of operating on corrupted state. Zero eval-hot-path cost (this wraps the
OUTER entry, once per call).

This is the Tier-1 panic-safety boundary from `docs/host-embedding-and-hardening.md`.
NOTE it does NOT (and cannot) catch a stack overflow / SIGABRT — those are aborts, not
unwinds; the reachable ones are already guarded (parser `MAX_PARSE_DEPTH`, matcher
`MAX_MATCH_DEPTH`). This catches Rust `panic!`/`unwrap`/index-out-of-bounds/`unreachable!`
that a latent bug could trigger on adversarial input.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-1 ("Panic-safety
  boundary. The host API should wrap eval in `catch_unwind`").
- `panic = "unwind"` is in effect (no `[profile.*] panic` override in any `Cargo.toml`),
  so `std::panic::catch_unwind` works. CONFIRM this still holds before relying on it.
- Host entry points to harden: `crates/rum-host/src/session.rs` — `eval` (:107),
  `resume_with_input` (:116) run guest code and are the priority; `parse` (:93),
  `inspect`/`inspect_symbol`, `complete`, `is_complete` also call into the runtime and
  should be wrapped for completeness (lower risk). `HostSession { session, ... }` is the
  struct to add a `poisoned: bool` to.
- Outcome types: `crates/rum-host/src/model/outcome.rs` — `HostFailureCode` (:41, add
  `Panic` and `Poisoned` — or a single `Panic` reused for the poisoned-followup; your
  call, but a poisoned follow-up should be DISTINGUISHABLE from the original panic),
  `HostFailure`/`HostEvalOutcome` (the failure shape to construct without going through
  an `EvalError`). Look at how `eval_failure_from_runtime` builds a `HostEvalOutcome::Failure`
  and mirror its shape for a synthesized panic failure.
- `catch_unwind` requires `UnwindSafe`; `&mut HostSession`/`&mut Session` are NOT
  `UnwindSafe`, so wrap the closure in `std::panic::AssertUnwindSafe`. The poison flag is
  what makes that assertion sound — we explicitly do NOT trust post-panic session state.

## Design (do this; deviate only with a reported reason)

1. Add `poisoned: bool` to `HostSession` (init false in the builder).
2. A private helper on `HostSession`, e.g.
   `fn guard<T>(&mut self, poisoned_value: impl FnOnce() -> T, body: impl FnOnce(&mut Self) -> T) -> T`
   — actually simpler: a helper that, GIVEN the session is not poisoned, runs
   `catch_unwind(AssertUnwindSafe(|| body(self)))`; on `Err(payload)` sets
   `self.poisoned = true` and returns the caller-supplied panic outcome built from the
   downcast message. Each public method checks `self.poisoned` first (return the Poisoned
   outcome) then calls the helper. Because the return types differ (`HostEvalOutcome`,
   `Result<HostParseResponse, HostFailure>`, `HostInspectResponse`, …), the helper is
   generic over the outcome `T` with two closures: the body and the on-panic constructor.
3. Panic message extraction: from the `Box<dyn Any + Send>` payload, downcast to
   `&str` then `String` (the two common panic payload types); fall back to a generic
   "panic" string. Put the message in the failure's text.
4. `HostFailureCode::Panic` for a freshly-caught panic; the poisoned-session follow-up
   returns a failure that is clearly the poisoned state (either a distinct
   `HostFailureCode::Poisoned`, or `Panic` with a "session poisoned" message — pick one
   and be consistent; a distinct code is cleaner for an embedder switching on the code).
5. Wire `HostFailureCode::Panic` (and `Poisoned` if added) into the Jupyter render
   mapping (`crates/rum-jupyter/src/adapter/render.rs`) so the workspace compiles.
6. Do NOT install a process-global `std::panic::set_hook` — that is global mutable
   process state a library must not impose. (The default hook will still print the panic
   to stderr before `catch_unwind` returns; suppressing/capturing that cleanly needs a
   global hook and is therefore a documented follow-up, NOT this brief. Note it in the
   report.)

## Scope

- MAY modify: `crates/rum-host/src/session.rs`, `crates/rum-host/src/model/outcome.rs`,
  `crates/rum-host/src/model/mod.rs` (if the failure-constructor helpers live there),
  `crates/rum-host/src/lib.rs` (exports), `crates/rum-jupyter/src/adapter/render.rs`
  (the new code mapping), `crates/rum-host/src/tests.rs` (tests).
- MUST NOT touch: anything under any `mma-hol` clone/standard; `crates/rum-core` /
  `crates/rum-runtime` source (this is a HOST-layer boundary — the runtime is NOT
  changed; if you think a runtime change is needed, STOP and report); `Expr`/serde
  shape; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your own report; `.git`,
  `Cargo.lock`.
- **No new `unsafe`.** `catch_unwind` + `AssertUnwindSafe` are safe APIs.
- This is NOT a hot-path change (wraps the outer entry, once per call) — but DO confirm
  you did not put `catch_unwind` inside any per-step loop.

## Deliverable

- `poisoned` flag + the `catch_unwind`/`AssertUnwindSafe` guard on the public
  `HostSession` methods; `HostFailureCode::Panic` (+ `Poisoned`).
- Tests (rum-host): (a) a body that panics (you need a way to make the runtime panic
  from a test — if no input reliably panics, gate the test behind a tiny test-only hook
  or use a `HostSession` method that takes a closure; if there is genuinely no safe way
  to trigger a runtime panic from a host test WITHOUT a runtime change, STOP and report —
  do NOT add a runtime panic path just to test). Assert the outcome is the graceful Panic
  failure (not a process abort) AND that the session is now poisoned (a second eval
  returns the Poisoned outcome). (b) a normal eval still works and does NOT poison.

## rum / Rust pitfalls (read twice)

1. `catch_unwind` needs `UnwindSafe`; use `AssertUnwindSafe` around the `&mut self`
   closure — the poison flag is the justification.
2. Not a hot path, but do NOT wrap anything per eval step — wrap the OUTER public method
   body only.
3. After a caught panic the session is UNTRUSTED — poison and refuse, do not try to
   "continue". Do not attempt partial cleanup of session state.
4. Do not install a global panic hook (process-global state). Capture the payload
   message from the `catch_unwind` `Err` instead.
5. Minimal diff; no drive-by churn.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # new HostFailureCode ripples to rum-jupyter mapping
```
Paste the workspace test summary AND the new rum-host panic/poison test output. If you
could not trigger a runtime panic from a host test without a runtime change, say so
explicitly and describe how you tested the boundary instead (e.g. a unit test of the
guard helper with a closure that panics).

No microbench is required (not a hot-path change) — but state in the report that you
confirmed `catch_unwind` wraps only the outer entry, not any per-step path.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes in the tree for Claude's gate.
- Do NOT modify `rum-core`/`rum-runtime` source. Host layer only. If you think you must,
  STOP and report.
- No new `unsafe`. No global panic hook.
- Never edit any `mma-hol` file. Minimal diff.
- Stop-loss: if you cannot test the boundary without a runtime change, STOP and report
  rather than adding a runtime panic path.
- Paste REAL verbatim output.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim test summary + the panic/poison test output; the note that
   catch_unwind wraps only the outer entry; the stderr-default-hook follow-up note.
3. Finding — confirmation a panic is converted to a graceful failure + the session
   poisons.
4. Stop-loss / open questions (empty if none).
