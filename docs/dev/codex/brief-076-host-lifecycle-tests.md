# brief-076 — host / kernel-lifecycle test buildout (the vacuum mma-hol doesn't touch)

## Goal

Fill the biggest test-coverage vacuum: the HOST / embedding / kernel-lifecycle surface
(`rum-host`), which mma-hol's stress corpus never exercises. rum has 1124 unit tests but
the LANGUAGE surface is double-covered while `rum-host` has only ~18 tests for a large API
(the Tier-1/Tier-2 host-hardening shipped in briefs 059-066). Write COMPREHENSIVE,
behavior-real tests for each lifecycle method + its error / recovery / concurrency paths,
asserting the documented contracts. This is a TEST-ONLY brief: NO product-code change. If a
test reveals a real behavior gap / broken contract, REPORT it (leave the failing test +
the finding) — do NOT paper over it or weaken the assertion; finding such gaps is the point.

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` (the CONTRACTS: Tier-1
  interrupt/cancel + wall-clock timeout, `catch_unwind` panic boundary + session poison,
  structured `HostValue`, memory-budget-as-host-watchdog; Tier-2 `reset()`, whole-session
  introspection `definitions()`/`contexts()`, snapshot `restore_from_bytes`, `Session` is
  `Send`/!`Sync`), and PROGRESS/memory `host-hardening-tier1` for the intended semantics.
- The API surface (all `pub` on `HostSession` unless noted), `crates/rum-host/src/session.rs`:
  `new`/`builder`/`with_config`/`with_tracing`/`with_core_builtins`; `reset`;
  `snapshot_to_bytes` + `restore_from_bytes`; `configuration`; `parse`; `eval`;
  `cancel_handle` (+ `HostCancelHandle::cancel`); `definitions`; `contexts`;
  `eval_with_timeout`; `resume_with_input`; `inspect`/`inspect_symbol`; `complete`;
  `is_complete`; `pending_input_request`. Config: `HostConfig`/`HostResourceLimits`
  (`crates/rum-host/src/config.rs`), typed output `HostValue`/`HostTyped`, `HostFailure`
  /`HostFailureCode` (incl. `Cancelled`).
- Existing tests (match their style): `crates/rum-host/src/tests.rs`,
  `crates/rum-host/tests/adapter_contract.rs`. Add new test files/modules alongside.

## Coverage to add (each an area; write multiple focused tests per area)

Prioritize the briefs-059-066 lifecycle vacuum:
1. **Wall-clock timeout** (`eval_with_timeout`): a bounded eval returns normally; a
   long/looping eval returns `HostFailureCode::Cancelled` within ~the budget; a fast eval
   is unaffected. Assert the outcome variant + that the session stays usable after.
2. **Interrupt / cancel** (`cancel_handle` + `HostCancelHandle::cancel`): cancel from
   another thread mid-eval stops it (returns Cancelled); cancel before/after eval is a
   no-op; the handle is cheap and `Send` (cross-thread). Assert the session is reusable
   after a cancel.
3. **Panic boundary + poison + recovery** (`catch_unwind`, `reset`): an eval that panics
   is caught (returns a failure, not a process abort — NOTE stack-overflow aborts are NOT
   caught; use a genuinely panicking path if one is reachable, else document the limit),
   the session is POISONED, and `reset()` recovers it to a usable state. Assert
   post-recovery eval works.
4. **reset()**: clears user definitions/state to a clean session (a def set before reset is
   gone after); recovers a poisoned session; is idempotent.
5. **Snapshot round-trip** (`snapshot_to_bytes` → `restore_from_bytes`): definitions +
   context state set before snapshot are present after restoring into a FRESH session;
   a later eval that depends on them works; `restore_from_bytes` on garbage bytes returns
   a `HostFailure` (not a panic). Byte-image is in-memory (no fs).
6. **Introspection** (`definitions`, `contexts`): reflect user definitions/contexts after
   some evals; empty on a fresh session; consistent with what was defined.
7. **Structured output** (`HostValue`/`HostTyped`): `.text` unchanged + the typed variant
   is correct for Integer/BigInteger/Rational/Symbol/Str/List/Other (depth-capped).
8. **Concurrency** (`Session`/`HostSession` is `Send`): a static `Send` assert + move a
   session to another thread, eval there, move back / join. (`!Sync` by design — one per
   thread.)
9. **Suspension resume** (`resume_with_input`, `pending_input_request`): an eval needing
   input suspends (pending request set), `resume_with_input` continues to the right result.
10. **Resource limits** (`HostResourceLimits` via `HostConfig`): an eval exceeding a
    configured iteration/recursion budget returns the specific `TerminatedEvaluation`
    outcome, and the session stays usable.

Do NOT re-test the LANGUAGE surface (arithmetic/patterns/etc. — already covered). Focus on
the lifecycle/host contracts above.

## Hard rules

- **Tests only — NO product-code change** (no edits under `crates/*/src` except adding test
  modules/`#[cfg(test)]` / files under `tests/`). If a test needs a `pub` accessor that
  doesn't exist, prefer testing through the existing public API; if genuinely blocked, STOP
  and report (do not add product API just to test).
- **Real assertions**: exercise actual behavior and assert the CONTRACT (variant, state,
  reusability), not trivial `is_ok()`. A test that can't fail is not a test.
- **Report contract gaps**: if a documented behavior is NOT actually held, leave the
  failing test and REPORT it as a finding (do NOT weaken the assertion to pass). This is a
  valuable outcome, not a failure of the brief.
- Do NOT `git commit`/branch/push. Never edit any `mma-hol` file. No new `unsafe`.
- No network (do NOT `cargo install`; `cargo-llvm-cov` coverage is a SEPARATE later step
  run by Claude after the human installs the tool).

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-host                 # new + existing host tests green (paste the count)
cargo test --workspace                 # no regression (paste rum-host + rum-runtime summaries)
```

Paste: the new test count (before 18 → after N), the `cargo test -p rum-host` summary, the
`--workspace` summary. If any new test FAILS because a contract isn't held, paste the
failure + your analysis (a finding, not a defect to hide).

## Report format

1. What changed — the new test files/modules, per area, with how many tests each; confirm
   NO product-code change.
2. Verification — `cargo test -p rum-host` + `--workspace` summaries, the before/after test
   count, all verbatim.
3. Findings — any contract gap / bug a test revealed (with the failing test + analysis), or
   "all covered contracts hold."
4. Coverage note — which of the 10 areas are now covered and any you could not reach
   through the public API (with why).

---

## Outcome (Claude gate, 2026-07-01) — DONE + KEEP. Host/lifecycle vacuum covered; all contracts hold.

Committed. TEST-ONLY: 12 public lifecycle tests (`tests/lifecycle_contract.rs`) + 1
`#[cfg(test)]` panic/poison/reset test (`src/lifecycle_contract_tests.rs`) + `lib.rs`
`#[cfg(test)] mod` registration (verified cfg-gated, zero product impact). All 10 brief
areas covered (timeout, cancel, panic→poison→reset, reset, snapshot round-trip,
introspection, typed HostValue depth-cap, Send/concurrency, input resume, resource limits).
rum-host package tests 22 → 35 (+13). **All covered host-hardening (059-066) contracts
HOLD — no bug/contract-gap found.** Gate (Claude reproduced): `cargo test -p rum-host`
19+4+12 all green; `cargo test --workspace` green (rum-runtime 723 unchanged). No unsafe,
no mma-hol edits. The only non-public path (deliberate panic injection) is covered via the
crate-internal `#[cfg(test)]` test (public API rightly exposes no panic trigger).

Next in the test-buildout line: measure the coverage delta with `cargo-llvm-cov` (now
installing) to confirm the vacuum is filled + find residual gaps; then error/resource-limit
paths + WK-conformance breadth.
