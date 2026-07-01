# Brief 066 — rum-jupyter real interrupt (execute-on-worker + interrupt→cancel + poison-restart)

## Goal

`KernelAdapter::interrupt` is a STUB — it sets a lifecycle flag and returns ok but does NOT
stop a running `execute_request`; and the transport is a single-threaded poll loop where a
blocking synchronous execute prevents even RECEIVING the interrupt. When you are done, an
`interrupt_request` on the Control channel actually STOPS the running cell: execute runs on
a WORKER thread (the `Send` `HostSession` from brief-062 moves into it), the main pump stays
responsive to Control, and `interrupt_request` calls the brief-059 `cancel_handle().cancel()`
so the eval loop's poll point stops the cell and the kernel returns an interrupted
`execute_reply`. A panic caught mid-execute (brief-060) surfaces as an error reply and the
poisoned session is `reset()` (brief-063) so the kernel keeps serving. All primitives exist;
this is the rum-jupyter wiring.

This is the highest-value item unlocked by the host-hardening round
(`docs/host-embedding-and-hardening.md` §2 "Downstream"). It is the BIGGEST, most
concurrency-sensitive change in this arc — precision + a hard stop-loss matter.

## Context pointers (STUDY these before touching anything)

- Read `AGENTS.md`; `docs/host-embedding-and-hardening.md` §2 "Downstream".
- Primitives already shipped: `HostSession::cancel_handle() -> HostCancelHandle`
  (Clone+Send+Sync, `.cancel()`), `eval` runs on any thread (`HostSession` is `Send`,
  brief-062), `HostSession::reset()` (brief-063, clears poison), `HostFailureCode::Cancelled`
  (a cancelled eval → a graceful outcome), `Poisoned` (brief-060).
- rum-jupyter execute flow (the seam is ALREADY split — exploit it):
  - `crates/rum-jupyter/src/adapter/mod.rs`: `KernelAdapter { session: HostSession, …,
    execution_state, lifecycle_state, pending_input, pending_execute, … }` (:~34).
    `run_execute_request(&request)` (:119) runs `self.session.eval(...)` → a
    `KernelExecuteOutcome` (JUST the eval). `dispatch_execute_request` (:374) sets Busy,
    calls run_execute_request, then either stdin-suspends (`pending_execute`) or
    `finish_execute_request(...)` builds the `KernelDispatchResult { reply, iopub, stdin }`
    from the outcome. `execute_prologue_iopub` (used in the stdin path) emits the
    busy+execution_input iopub. `interrupt` (the stub, ~:78) + `resume_pending_execute` (:422).
  - `crates/rum-jupyter/src/transport/event_loop.rs`: `KernelEventPump::pump_next` (:111)
    polls Control→Shell→Stdin; `pump_until_idle` (:142) loops pump_next. `handle_wire_request`
    (:47) calls `adapter.dispatch_request`.
  - `crates/rum-jupyter/src/launcher.rs`: `run_native_kernel_with_config` (:57) — the
    `loop { driver.pump_until_idle(transport); … }` real driver.
- No `thread::spawn` exists in rum-jupyter yet — the worker is the first. Sockets are in a
  `RefCell` (`!Sync`) and STAY on the main thread; ONLY the `HostSession` + `cancel_handle`
  cross to the worker.

## Design (the state machine — do this; deviate only with a reported reason)

**Core seam:** the eval (both initial `eval` and `resume_with_input`) moves to a worker; the
OUTCOME handling (finish / stdin-suspend / iopub / reply) stays on the main thread exactly as
today, just deferred until the worker returns. So `finish_execute_request` / the
stdin/`pending_execute` machinery is REUSED unchanged; only WHEN it runs changes.

1. Adapter execute state: replace the by-value `session` usage during execute with an
   explicit state, e.g. `enum ExecuteState { Idle, Running { worker: JoinHandle<(HostSession,
   KernelExecuteOutcome)>, cancel: HostCancelHandle, ctx: ExecuteContext } }` where
   `ExecuteContext` carries what `finish_execute_request` needs (parent_header, code,
   execution_count, routing). The `HostSession` lives in the adapter when Idle and in the
   worker when Running. (Model it so the borrow checker is satisfied without `unsafe` — e.g.
   `session: Option<HostSession>`, `Some` when Idle, `None` (moved to worker) when Running.)
2. On `execute_request` when Idle: take the session, `let cancel = session.cancel_handle();`
   spawn `thread::spawn(move || { let outcome = <run one eval segment>; (session, outcome) })`,
   store Running. Emit the PROLOGUE iopub NOW (busy + execution_input, via
   `execute_prologue_iopub`); reply = None (not done yet).
3. A new `poll_execute_completion(&mut self) -> Option<KernelDispatchResult>`: when Running,
   if `worker.is_finished()`, `join()` → `(session, outcome)`, restore `session` + set Idle,
   then run the EXISTING outcome handling (finish_execute_request → reply+epilogue iopub, OR
   stdin-suspend → pending_execute + stdin request; on a Poisoned/panic outcome, `reset()`
   the session and emit an error reply). A stdin resume (`resume_pending_execute`) spawns a
   NEW worker for `resume_with_input` the same way (so every segment is interruptible).
4. The pump / driver must (a) NOT pop Shell while Running (defer new execute/inspect/complete
   — the kernel is busy; leave them queued in the socket), (b) still process Control while
   Running, (c) call `poll_execute_completion` each iteration and send its dispatch. Wire this
   into `pump_next`/`pump_until_idle` and the `launcher.rs` loop (the loop already sleeps when
   idle — it must instead keep spinning + polling completion while Running).
5. `interrupt_request` while Running → `cancel.cancel()` (the eval loop stops at its poll
   point → the worker's eval returns a Cancelled outcome → poll_execute_completion emits an
   interrupted `execute_reply` with status error/aborted per the Jupyter protocol). While
   Idle, interrupt is a no-op ok (as today).
6. Keep the existing SYNCHRONOUS behavior for non-execute Control/Shell handling unchanged.

## Scope

- MAY modify: `crates/rum-jupyter/src/adapter/mod.rs` (+ its submodules if the execute
  handling is split across files), `crates/rum-jupyter/src/transport/event_loop.rs`,
  `crates/rum-jupyter/src/launcher.rs`, and the rum-jupyter test modules; a NEW small module
  under `crates/rum-jupyter/src/` for the worker/execute-state if that keeps the diff clean.
- MUST NOT touch: any `mma-hol` clone/standard; `crates/rum-core` / `crates/rum-runtime` /
  `crates/rum-host` SOURCE (all primitives exist — cancel_handle/eval/reset/Send are done; if
  you think a host/runtime change is needed, STOP and report); `Expr`/serde shape;
  `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your own report; `.git`, `Cargo.lock`.
- **No new `unsafe`** (model the session-move with `Option<HostSession>`, not raw pointers).
- Do NOT change the wire protocol / message shapes. The only behavior change is: execute is
  now asynchronous+interruptible; the SAME iopub/reply messages are produced (prologue at
  start, epilogue at completion).

## rum / Rust pitfalls (read twice)

1. **Concurrency correctness is the whole risk.** No shared mutable session across threads
   (the session MOVES; only the `Arc<AtomicBool>` cancel flag is shared). No busy-wait
   deadlock: while Running, the loop polls Control + worker completion and must make progress
   (do not block on `join()` — use `is_finished()` then `join()`). The worker must always
   return the session (even on panic-inside-eval — but note eval's panic is caught by
   brief-060's boundary inside `HostSession::eval`, so the worker gets a Failure outcome, not
   an unwind; confirm the worker closure itself cannot panic-unwind across the thread
   boundary — if it could, wrap it).
2. Deferring Shell while Running must not DROP messages — leave them in the incoming queue /
   socket buffer so they are processed once Idle (do not pop-and-discard).
3. The stdin-suspend path (`pending_execute`) must still work THROUGH the worker model (a
   segment that returns InputRequired → stdin round-trip → resume on a new worker).
4. No new default cost to the eval hot path — this is all in rum-jupyter, not the runtime.
   (No microbench needed; it is not an eval-hot-path change.)
5. Determinism in tests: drive the pump manually in the in-memory test transport; to test
   interrupt, start a non-terminating cell (e.g. `While[True, 1]`) on the worker, then feed an
   `interrupt_request` on Control and pump until the interrupted reply appears — assert it
   arrives (bounded) and the kernel returns to Idle + can run the next cell.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```
Then a real interrupt integration test (add it): a non-terminating execute is interrupted by
a Control `interrupt_request` and the kernel returns an interrupted reply + goes Idle + runs
a following cell correctly. AND a panic-in-execute test (reuse the brief-060 hook if
reachable through the adapter, else document) → error reply + the kernel keeps serving (reset).
AND re-run the rum-jupyter suite (incl. the native ZeroMQ tests) **5×** to check for
flakiness introduced by the new thread — paste the pass counts.

Paste: the workspace summary, the interrupt integration test output, and the 5× rum-jupyter
stability rerun counts.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes for Claude's gate.
- Do NOT modify rum-core/rum-runtime/rum-host source — the primitives are done. If you think
  you must, STOP and report.
- No new `unsafe`. Never edit any `mma-hol` file.
- **STOP-LOSS (important): if you cannot make the concurrency correct + non-racy + clean
  (deadlock, dropped messages, a session that doesn't return, flaky ZeroMQ reruns you can't
  stabilize), STOP and report the specific problem — do NOT ship a racy kernel.** A correct
  synchronous fallback that still can't interrupt is better than a racy async one.
- Paste REAL verbatim output; if a run didn't reach green / a rerun flaked, say so.

## Report format (final message)

1. What changed — per file, line ranges, one-line why; the execute state machine shape.
2. Verification — verbatim workspace summary + the interrupt integration test + the 5×
   rum-jupyter stability counts + the panic-recovery result.
3. Finding — interrupt now stops a running cell (latency observed); Shell deferred while
   busy; stdin-suspend still works; panic → error reply + reset keeps the kernel alive.
4. Stop-loss / open questions — anything racy/unresolved (be explicit; empty only if truly
   clean).
