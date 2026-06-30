# Brief 059 — Host interrupt/cancel + wall-clock time budget (one shared eval-loop poll point)

## Goal

`HostSession::eval` is synchronous and uninterruptible, and there is no host-imposed
wall-clock cap — the #1 + #2 Tier-1 embedding gaps (`docs/host-embedding-and-hardening.md`).
When you are done: (1) a host can obtain a **cancel handle BEFORE calling `eval`** and,
from ANOTHER thread, request cancellation; the running `eval` stops at the next poll
point and returns a graceful `TerminatedEvaluation["Cancelled"]` (NOT a hang, NOT a
crash). (2) a host can run an eval with a **per-eval wall-clock timeout**; on expiry the
eval stops and returns a graceful `TerminatedEvaluation` (reuse the existing
`TimeConstraintExceeded` channel). BOTH are served by ONE amortized check point in the
eval loop, and the common path (no cancel handle, no timeout) must show **zero
microbench regression** (the no-default-path-cost hard line).

This is host-resilience / DoS-bounding, the natural continuation of brief-058. The
mechanism mirrors the existing per-eval-step guards (recursion_limit, brief-058's
match_depth).

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-1 (interrupt/cancel
  is "#1 gap"; session time budget is the next bullet — `HostResourceLimits` has
  recursion/replace_repeated/pass/generated_element/integer_work/iteration_work but NO
  time/memory).
- The eval-loop per-step chokepoint to mirror: `crates/rum-runtime/src/runtime/eval_core.rs:20-34`
  (`evaluate_once_unstable_after_input_check_with_cycle_tracking` — already checks
  `recursion_limit` and bumps `evaluation_depth` per recursive eval step; the
  resource-limit `Err` propagates cleanly from here). This is the poll point.
- Resource-limit → graceful value plumbing (mirror it for the new variant):
  `crates/rum-runtime/src/runtime/session_helpers.rs:168-205`
  (`runtime_message_for_non_loader_eval_error` + `terminated_evaluation_reason_for_error`,
  each switching on `EvalError`; the 63efadd specific-tag philosophy — give Cancelled
  its own tag). `EvalError::TimeConstraintExceeded` ALREADY EXISTS (model.rs Display
  ~271, session_helpers tag `"timelim"`) and already maps to `HostFailureCode::TimeConstraintExceeded`
  — the host time budget REUSES it; do NOT add a new error for timeout.
- `Session` struct + fields: `crates/rum-runtime/src/runtime/model.rs:316` (`evaluation_depth`
  is line 353, `resource_limits` line 352; brief-058 added `match_depth` here — add the
  new fields the same way). `EvalError` enum + Display: model.rs ~255-296 + variant defs
  ~630-650; the variant list in `session_state.rs:298`.
- Runtime setters live in `crates/rum-runtime/src/runtime/session_api.rs` (e.g.
  `set_recursion_limit` :35, `set_iteration_work_limit` :55) — add the deadline / cancel
  wiring methods here.
- Host surface: `crates/rum-host/src/session.rs` (`HostSession::eval` :107 calls
  `self.session.eval_request`); `crates/rum-host/src/model/outcome.rs:41` `HostFailureCode`
  (add `Cancelled`); the EvalError→SessionRequestError→HostFailureCode chain
  (`session_requests.rs:384`, outcome.rs mapping, `rum-jupyter/src/adapter/render.rs:161`)
  — same chain brief-058 threaded `MatchRecursionLimitExceeded` through.
- `panic = "unwind"` (no profile override) — irrelevant here but confirms later
  catch_unwind work is viable (separate brief).

## Design (do this; deviate only with a reported reason)

### A. The shared poll point (the hot-path core — get this right)
1. Add to `Session`: `interrupt_flag: Arc<AtomicBool>` (the cancel signal),
   `eval_deadline: Option<std::time::Instant>` (the timeout), an `interrupt_armed: bool`
   (true iff a cancel handle was handed out OR a deadline is set), and a
   `interrupt_poll_counter: u64`. Initialize in every `Session` construct/reset path.
2. At the top of `evaluate_once_unstable_*` (the eval_core.rs poll point), insert:
   ```
   if self.interrupt_armed {                       // false in the common case → one
       self.interrupt_poll_counter = self.interrupt_poll_counter.wrapping_add(1);
       if self.interrupt_poll_counter & POLL_MASK == 0 {   // amortize: ~every 1024 steps
           if self.interrupt_flag.load(Ordering::Relaxed) {
               return Err(EvalError::Cancelled);
           }
           if let Some(deadline) = self.eval_deadline {
               if Instant::now() >= deadline { return Err(EvalError::TimeConstraintExceeded); }
           }
       }
   }
   ```
   The `interrupt_armed` gate keeps the no-budget path to a single predictable branch.
   AMORTIZE the `Instant::now()` (it is not free) behind the counter mask — never call
   it every step. Tune `POLL_MASK` (start 1023) so the microbench is flat AND a cancel
   is observed promptly (sub-10ms on a tight loop). Report the chosen mask.
3. Reset per eval: at the top of `eval_request` (or wherever a top-level eval begins),
   clear `interrupt_flag` to false and recompute `eval_deadline`/`interrupt_armed` from
   the requested timeout, so a stale signal from a previous eval never carries over.

### B. New error variant + graceful value
4. `EvalError::Cancelled` (no payload). Wire it: Display ("evaluation cancelled by
   host"), `session_state.rs` variant list, the two `session_helpers.rs` tag functions
   (`terminated_evaluation_reason_for_error` → `Some("Cancelled")`,
   `runtime_message_for_non_loader_eval_error` → e.g. `"cancel"`), the
   SessionRequestError/host/jupyter mapping → new `HostFailureCode::Cancelled`. Mirror
   brief-058's `MatchRecursionLimitExceeded` wiring at every site. `TimeConstraintExceeded`
   already has all of this — only Cancelled is new.

### C. Runtime + host API
5. Runtime (`session_api.rs`): a method to fetch/clone the interrupt flag
   (`pub fn interrupt_handle(&self) -> Arc<AtomicBool>` — sets `interrupt_armed = true`),
   and to set a per-eval timeout (e.g. `pub fn set_eval_timeout(&mut self, d: Option<Duration>)`
   storing the Instant + arming). Keep `eval_request` the same signature; the deadline is
   read from the field at eval start.
6. Host (`rum-host`): `HostSession::cancel_handle(&self) -> HostCancelHandle`, where
   `HostCancelHandle` wraps the `Arc<AtomicBool>` and exposes `pub fn cancel(&self)`
   (sets it `Relaxed`). It is `Clone + Send + Sync` so the host can move a clone to
   another thread. Add `HostSession::eval_with_timeout(&mut self, request, Duration)`
   (or a `HostEvalRequest` timeout field) that sets the deadline then calls the existing
   eval path. Keep `eval` (no timeout) unchanged.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/eval_core.rs`, `model.rs`,
  `session_state.rs`, `session_helpers.rs`, `session_requests.rs`, `session_api.rs`,
  and wherever `Session` is constructed/reset (initialize the new fields);
  `crates/rum-host/src/session.rs`, `crates/rum-host/src/model/outcome.rs`, and the
  minimal `crates/rum-jupyter/src/adapter/render.rs` mapping needed to compile the new
  `HostFailureCode::Cancelled`; a new/existing test module in rum-host and rum-runtime.
- MUST NOT touch: any `mma-hol` clone/standard; `crates/rum-core/src/expr.rs`
  `Expr`/`Symbol`/serde shape; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your
  own report; `.git`, `Cargo.lock`.
- **No new `unsafe`** (rum-core/rum-runtime stay zero-unsafe). `Arc<AtomicBool>` is safe.
- Do NOT make `Session: Send`/`Sync` or change its thread contract — only the
  `Arc<AtomicBool>` crosses threads, not `Session`. If you think `Session` must become
  `Send`, STOP and report.

## rum / Rust pitfalls (read twice)

1. **This IS a hot-path change** (the eval-loop poll point) → `scripts/core-language-microbench.sh`
   base-vs-new is MANDATORY; paste per-label ratios. The no-budget common path must be
   flat — if any label regresses, the `interrupt_armed` gate / amortization is wrong.
2. `Instant::now()` is NOT free — it must sit behind the `POLL_MASK` counter, never per
   step. Prove the armed-path overhead is acceptable too (a loop with a timeout set).
3. Default-path cost: when `interrupt_armed == false`, the only cost is one branch +
   (ideally) nothing else. No atomic load, no clock read, no counter bump when disarmed.
4. Suspension-safe: do not collapse any `suspend_with_frame*` / `drive_internal_value`;
   the poll-point `Err` propagates like the existing recursion-limit `Err`.
5. The reset-per-eval (clear flag, recompute deadline) must happen on the top-level eval
   entry only, NOT per recursive step — otherwise a mid-eval cancel is lost.
6. One variable per measurement; stop-loss on a stuck step or an immovable regression.

## Verification (MANDATORY — run, iterate, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # new EvalError + HostFailureCode ripple cross-crate
```
A rum-host integration test that actually exercises cancel across threads:
spawn the eval on the main thread inside a closure that, from another thread, sleeps
briefly then calls `cancel()` on a pre-obtained handle, against a non-terminating input
(e.g. `While[True, 1]` or a deep recursion); assert the outcome is the graceful
Cancelled value within a bound (not a hang). And a timeout test:
`eval_with_timeout("While[True, 1]", Duration::from_millis(200))` returns the graceful
timeout value in well under, say, 2 s. Plus a positive test: a normal eval with a handle
obtained but never cancelled, and with a generous timeout, returns the correct value.

Then the hot-path gate:
```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_059_micro
# paste ratios for loop/eval-heavy labels (loop-empty, do-count-null, table-*,
# downvalue-*, matchq-*) base vs new — confirm flat.
```
Paste: the `cargo test --workspace` summary, the cancel + timeout + positive test
results, and the microbench base-vs-new rows.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes in the tree for Claude's gate.
- No new `unsafe` anywhere in rum-core/rum-runtime.
- Hot-path change → microbench A/B mandatory; a regression on any common-path label is a
  defect to fix or report, not to ship.
- Never edit any `mma-hol` file. Minimal diff, no drive-by churn.
- Stop-loss: same failure twice, or the common-path microbench regresses and you can't
  recover it, or a cancel does not interrupt within a bound → STOP and report verbatim.
- Paste REAL verbatim output; if a run didn't reach green, say so.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim: test summary, the cancel/timeout/positive test outputs, the
   microbench base-vs-new rows, the chosen `POLL_MASK`.
3. Finding — cancel latency observed, timeout latency observed, and confirmation the
   no-budget path is microbench-flat.
4. Stop-loss / open questions (empty if none).
