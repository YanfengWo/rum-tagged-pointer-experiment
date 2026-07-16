# Brief 236 — host failure-classification honesty (exit status, reset contract, termination-field adoption)

Implementation brief — embedding API line, item a1 (see
`docs/dev/embedding-api-line.md`, findings F3/F4/F7). Three small,
related honesty fixes to the `rum-host` failure-classification surface
and its one in-tree consumer. Host-layer only: **zero changes in
`rum-runtime`/`rum-core`/`rum-syntax`** — every needed datum is already
available at the existing mapping seams.

Background: `rum-host` is the embedding contract. The survey found three
places where the contract loses or under-publishes information it
already has, forcing consumers to work around it:

1. `HostFailureCode::Exit` discards the exit status.
   `crates/rum-host/src/model/outcome.rs:493` maps
   `SessionRequestError::Exit { .. } => HostFailureCode::Exit` — the
   `code: i64` is dropped, so a contract-level host cannot implement
   `Exit[7]` → process exit status 7.
2. Which failures require a session reset is consumer folklore.
   `rum-jupyter` hard-codes the answer
   (`crates/rum-jupyter/src/adapter/worker.rs:145-151`:
   `Panic | Poisoned` ⇒ reset). That classification belongs to the
   contract as a documented method.
3. The brief-231 termination discriminator has zero adoption.
   `rum-jupyter` still string-matches the rendered value
   (`worker.rs:130-132`:
   `response.value.text == "TerminatedEvaluation[\"Cancelled\"]"`),
   which is exactly what the `HostEvalSuccess.termination` field was
   built to replace — and the string match misclassifies a program
   whose ordinary RESULT VALUE happens to be that literal.

## Required change

1. **Exit status field.** `HostFailure`
   (`crates/rum-host/src/model/outcome.rs`) gains
   `pub exit_status: Option<i64>`, documented (the crate has
   `#![warn(missing_docs)]`). It is `Some(code)` iff the failure maps
   from `SessionRequestError::Exit { code }` (populate at the existing
   mapping seam near `outcome.rs:493`); `None` on every other
   construction. Keep `HostFailureCode::Exit` a unit variant — the
   classification enum stays flat; structured payload rides the struct,
   same pattern as brief-231's `termination` field on success.
2. **Reset contract method.** `HostFailureCode` gains
   `pub const fn requires_reset(self) -> bool`, documented: `true` for
   `Panic | Poisoned`, `false` otherwise. Before writing it, verify the
   truth from `crates/rum-host/src/session.rs` (the `guard`/`poisoned`
   machinery): confirm exactly which outcomes leave the session poisoned
   (and report if any other path does — e.g. check what a failed
   `restore_from_bytes` leaves behind). Then port
   `outcome_requires_session_reset`
   (`rum-jupyter/src/adapter/worker.rs:145-151`) onto the method so the
   knowledge lives in one place.
3. **Termination-field adoption.** Replace `is_cancelled_success`
   (`worker.rs:130-132`) with a check of the structured field:
   `response.termination == Some(HostTerminationReason::Cancelled)`.
   NOTE the deliberate behavior fix: a program whose ordinary result
   value is the literal `TerminatedEvaluation["Cancelled"]` was
   previously converted by the jupyter adapter into a cancelled
   failure; with the provenance-tested field (brief-231 sets it only
   for runtime-synthesized terminations) it is now an ordinary Success.
   That is the point of the field — pin it with a test.
4. **Construction ripple (MECHANICAL).** Adding the `exit_status` field
   breaks every `HostFailure { … }` struct literal. Known sites:
   `rum-host/src/model/outcome.rs` (5), `rum-host/src/session.rs` (5),
   `rum-jupyter/src/adapter/render.rs` (2),
   `rum-jupyter/src/adapter/worker.rs` (4). Set `exit_status: None`
   everywhere except the Exit mapping seam; if further literals surface
   in test code, set the honest value and list each in the report.

## Tests

- `rum-host/src/tests.rs`:
  a. evaluating `Exit[7]` on a session whose policy allows
     `Capability::Process` (existing Exit tests from brief-183 show the
     setup) yields a Failure with `code == HostFailureCode::Exit` AND
     `exit_status == Some(7)`;
  b. any non-Exit failure (reuse an existing failure fixture) has
     `exit_status == None`;
  c. `requires_reset` truth table: `Panic` and `Poisoned` true; at
     minimum `Cancelled`, `TimeConstraintExceeded`, `SyntaxError`,
     `Exit` false.
- `rum-jupyter` (existing test files under `src/tests/` — name the file
  you extend):
  d. the existing cancel/interrupt flow tests stay green UNCHANGED
     (the structured field and the old string agree on real
     cancellations — brief-231 pinned the in-band value byte-identical);
  e. NEW: executing the literal source `TerminatedEvaluation["Cancelled"]`
     yields an ordinary successful execute response (NOT a cancelled
     failure) — the provenance fix of change 3.

## Scope

MAY modify:
- `crates/rum-host/src/model/outcome.rs`
- `crates/rum-host/src/lib.rs` (re-export/docs only, if needed)
- `crates/rum-host/src/session.rs` (MECHANICAL literal ripple only)
- `crates/rum-host/src/tests.rs`
- `crates/rum-host/src/lifecycle_contract_tests.rs` (MECHANICAL only)
- `crates/rum-host/tests/adapter_contract.rs`,
  `crates/rum-host/tests/lifecycle_buildout.rs`,
  `crates/rum-host/tests/lifecycle_contract.rs` (MECHANICAL only)
- `crates/rum-jupyter/src/adapter/worker.rs`
- `crates/rum-jupyter/src/adapter/render.rs` (MECHANICAL literal ripple only)
- `crates/rum-jupyter/src/adapter/mod.rs` (import/callsite ripple only)
- `rum-jupyter` test files that exercise cancel/execute flows or
  construct `HostFailure` literally (list each in the report)
- `docs/guide/embedding.md` (a short "Failure classification" note:
  `exit_status`, `requires_reset`, the termination field as the correct
  cancellation check)

MUST NOT touch: `HostFailureCode` variant set (no additions/removals/
payloads), `HostTyped`, `HostEvalRequest`, anything in `rum-runtime`/
`rum-core`/`rum-syntax`/`rum-stdlib`/`rum-cli`/`rum-repl`/`rum-terminal`,
rendered value/message text, `docs/**` beyond the one listed file.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

- **No microbench A/B**: every touched path is failure-path or
  adapter-side classification, none is per-evaluation-step work; state
  this in the report.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Write your findings for change 2's poison-truth verification into the
  report BEFORE running the full test suite.
- Additive metadata only on the rum-host side: if any existing rum-host
  test needs its EXPECTED value changed (beyond adding the new field to
  a literal or a new assertion), STOP and report. The ONLY intended
  behavior change is jupyter's classification of the literal-value case
  (change 3 / test e).
- Stop-loss: if the literal ripple forces edits outside the MAY list,
  STOP and report the file list instead of expanding scope.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Change-2 poison-truth findings (which paths poison the session,
   evidence from session.rs).
3. Test survey — each new/extended assertion, named.
4. Verification — the exact commands with verbatim output (test totals).
5. The no-A/B justification (one sentence).
6. Deviations / open questions (empty if none).
