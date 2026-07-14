# Brief 232 — request-level memory ceiling with a classified outcome

REVISION 1 (2026-07-15): the first dispatch correctly stopped at the
scope stop-loss — adding a public field to `ResourceLimits` forces the
five exhaustive `ResourceLimits { … }` literals in
`crates/rum-runtime/src/runtime/tests/session_config.rs` to be updated,
and that file was missing from the MAY list. It is now listed (the only
such file in the tree; `HostResourceLimits` has no exhaustive literals).
No other change to the brief.

Implementation brief — host-embedding line, item b2 (see
`docs/dev/host-embedding-line.md`, concern (b) protocol ask 4). Today the
kernel's memory accounting activates only inside
`MemoryConstrained[body, bytes]`; there is no way for an embedder to say
"every request in this session runs under an N-byte ceiling". Add that
knob as a resource limit, reusing the existing accounting wholesale, and
classify a top-level breach through the brief-231 termination field. The
ceiling is a *proxy-bytes* ceiling (the modeled accounting that
`MemoryInUse`/`ByteCount` already expose), not an OS RSS measurement —
document it as such, honestly.

## Mechanism (pre-diagnosed; reuse, do not rebuild)

- Accounting + enforcement already exist: `current_memory_limit`
  (`model/session.rs`, init at `session_state.rs:423`) is checked in
  `enforce_active_resource_limits_for_expr` and
  `enforce_active_memory_limit_for_projected_bytes`
  (`debug_tools/resources/constraints.rs:126-167`) against
  `current_memory_proxy_bytes()` (`debug_tools/resources/memory.rs` —
  a FULL walk of session state per call), raising
  `EvalError::MemoryConstraintExceeded` → already mapped to
  `SessionRequestError::MemoryConstraintExceeded`
  (`session_requests.rs:401`) and `HostFailureCode::
  MemoryConstraintExceeded` (`rum-host/src/model/outcome.rs`).
  `General::memlim` is already mapped in
  `runtime_message_for_non_loader_eval_error` (`session_helpers.rs:194`).
- `MemoryConstrained` frames min-combine into `current_memory_limit` and
  restore the prior value on exit
  (`constraints.rs:94-124`) — a request-level base value composes with
  them UNCHANGED.
- Request start runs `begin_top_level_interrupt_budget`
  (`session_helpers.rs:30`) — the same place the wall-clock
  `eval_deadline` is armed per request. Arm the memory base there.
- Bulk-allocation prechecks (e.g. the `Table` count precheck at
  `kernel_data/control_data/branching_loops/iteration/spec.rs:279-291`)
  fire only when `current_memory_limit` is `Some` — they activate
  automatically under the new knob.

## Required change

1. **The knob.** `ResourceLimits` (`runtime/model/config.rs:4`) gains
   `pub eval_memory_limit: Option<usize>` (default `None` = today's
   behavior). Mirror it in `HostResourceLimits`
   (`rum-host/src/config.rs`) with both `From` impls and a doc comment
   stating the proxy-bytes semantics (modeled accounting, not RSS).
2. **Arming.** In `begin_top_level_interrupt_budget`, set
   `self.current_memory_limit = self.resource_limits.eval_memory_limit;`
   (plain assign — each top-level form re-arms; `MemoryConstrained`
   frames min-combine on top and restore back to this base). Reset the
   stride counter (below) there too.
3. **Top-level classification.** Do NOT touch
   `terminated_evaluation_reason_for_error` (brief-231's shared map —
   the `debug_tools` renderings depend on memory staying OUT of it; see
   Boundaries). Instead:
   - add variant `EvalTerminationReason::MemoryLimit` with tag
     `"MemoryLimit"` (`model/requests.rs`), and the
     `HostTerminationReason::MemoryLimit` mirror arm
     (`rum-host/src/model/outcome.rs`);
   - add a small helper next to the shared map (e.g.
     `terminated_evaluation_reason_for_top_level_error`): the shared map
     first, else `EvalError::MemoryConstraintExceeded { .. }` →
     `Some(EvalTerminationReason::MemoryLimit)`;
   - use that helper ONLY in `finalize_top_level_eval_value`
     (`session_eval.rs:22-38`; build the value expr from
     `reason.tag()`). `finalize_cycle_termination_value` (loader path)
     stays cycles-only. Result: an uncaught top-level breach returns the
     in-band `TerminatedEvaluation["MemoryLimit"]` + `General::memlim`
     message (free via `emit_non_loader_eval_message`) +
     `termination == Some(MemoryLimit)` — same graceful shape as
     cancel/timeout.
4. **Active-mode check cost (the one design decision).** When the
   ceiling is armed, `enforce_active_resource_limits_for_expr` currently
   pays a full session walk per evaluation step. Add stride
   amortization INSIDE the `Some` arm: a `Cell<u32>` counter on the
   session (follow the `peak_memory_proxy_bytes` `Cell` precedent),
   full walk only every `K = 64` calls (a named const). The
   projected-bytes bulk checks (`enforce_active_memory_limit_for_
   projected_bytes` call sites) stay UNSTRIDED — they guard single
   large allocations. Overshoot between strided walks is bounded by the
   existing per-step limits (element/integer/iteration budgets); say so
   in a code-adjacent doc comment.
   - **Measure before you keep K=64**: with a build at `K = 1` and one
     at `K = 64`, run the SAME armed workload
     (`MemoryConstrained[<loop of small Table builds>, 10^9]` through
     the release `rum-cli`, best-of-2, e.g.
     `MemoryConstrained[Do[Length[Table[k, {k, 200}]], {200000}], 10^9]`)
     and report both wall times. Decision rule: keep the stride if it
     improves the armed wall time by >20%; otherwise ship WITHOUT the
     counter (plain per-call walk, i.e. today's behavior) and say so.
   - The DEFAULT path (`current_memory_limit == None`) must stay
     instruction-identical: the counter is only touched inside the
     `Some` arm.
5. **Default-off proof.** The enforce seam is on the hot path, so run
   the microbench A/B (base vs change, ceiling unset — the default) and
   paste the per-label rows for the eval-core labels plus same-tier
   neighbors. Zero-regression is the gate.

## Boundaries (read carefully — these pin existing behavior)

- `debug_tools/evaluation_data.rs:103` has an explicit reachable arm
  rendering a memory breach as `$Aborted` + `"MemoryConstraint"` (an
  outer `MemoryConstrained` can push the error into `EvaluationData`'s
  finish). Adding memory to the SHARED reason map would silently reroute
  that arm through the terminated guard at `:86` and change rendered
  output — that is WHY change 3 uses a finalize-only helper. Do not
  modify `evaluation_data.rs` or `debug_tools/test_report/evaluation.rs`
  at all; their tests must pass unchanged.
- `MemoryConstrained` semantics are UNTOUCHED: same in-band
  `$Aborted`/fallback on its own breach, min-combining with the request
  base. When the request-level base fires INSIDE a user
  `MemoryConstrained` frame, the frame's handler catches it (in-band
  `$Aborted`, request continues) — that mirrors how nested
  time-constraint frames layer, the ceiling itself still holds (every
  further allocation re-trips), and the shim still sees any TOP-LEVEL
  breach classified. Document this layering in the knob's doc comment;
  do not "fix" it.
- The stride may coarsen WHERE inside a computation an armed breach is
  detected (up to 63 steps later). Existing `MemoryConstrained` tests
  must stay green UNCHANGED; if any test pins exact abort granularity
  and fails, STOP and report (do not weaken the test).

## Tests (name each in the report)

- Runtime (session request tests): with
  `eval_memory_limit = Some(<small>)`, an eval that allocates past the
  ceiling (pick a workload that trips memory BEFORE the element/work
  limits; verify by the assertion itself) returns Ok with value
  `TerminatedEvaluation["MemoryLimit"]`, `termination ==
  Some(EvalTerminationReason::MemoryLimit)`, and a `General::memlim`
  message; the SAME session evaluates `1 + 1` normally afterwards.
- Interplay: request base high + `MemoryConstrained[<huge>, <lower>]`
  inside → the frame catches first ($Aborted in-band, `termination ==
  None`, request continues).
- Host (`rum-host/src/tests.rs`): `HostResourceLimits` with
  `eval_memory_limit` set → `HostEvalOutcome::Success` with
  `termination == Some(HostTerminationReason::MemoryLimit)` and the
  in-band value text; default config → suite unchanged.
- Config plumbing: `HostConfig`/`HostSession::configuration()`
  round-trips the new field (extend an existing config test).

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/model/config.rs`
- `crates/rum-runtime/src/runtime/model/requests.rs` (the new enum
  variant + its `tag()` arm only)
- `crates/rum-runtime/src/runtime/model/session.rs` (stride `Cell`
  field)
- `crates/rum-runtime/src/runtime/session/session_state.rs` (field init)
- `crates/rum-runtime/src/runtime/session/session_helpers.rs` (arming +
  the top-level reason helper)
- `crates/rum-runtime/src/runtime/session/session_eval.rs`
  (`finalize_top_level_eval_value` only)
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs`
  (stride inside the `Some` arm)
- `crates/rum-host/src/config.rs`, `crates/rum-host/src/model/outcome.rs`
  (mirror field / mirror variant)
- `crates/rum-runtime/src/runtime/tests/session_config.rs` — MECHANICAL
  ONLY: the five exhaustive `ResourceLimits { … }` literals must gain
  the new field (Rust requires it for a same-crate exhaustive literal).
  Set it to `None` except where a test deliberately round-trips a value;
  extend the config round-trip assertion to cover the new field. Do not
  otherwise restructure this file.
- Tests: the runtime session-request test module(s) you extend,
  `crates/rum-host/src/tests.rs`

MUST NOT touch: `terminated_evaluation_reason_for_error`'s existing
arms, `debug_tools/evaluation_data.rs`,
`debug_tools/test_report/evaluation.rs`, `memory.rs` (the walk itself),
the `MemoryConstrained`/`TimeConstrained` builtins, `eval_core/mod.rs`,
`docs/**`, anything else.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

Performance gates:

```
# 1) default-off A/B (ceiling unset):
bash scripts/core-language-microbench.sh --output-dir <base-dir>   # on BASE
bash scripts/core-language-microbench.sh --output-dir <new-dir>    # on the change
# paste per-label rum_seconds rows for the eval-core labels + same-tier neighbors

# 2) armed-mode stride decision probe (K=1 vs K=64 builds, release rum-cli,
#    best-of-2 each, the MemoryConstrained workload above): paste both wall
#    times and state the decision per the >20% rule.
```

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Default path stays instruction-identical when the knob is `None`; the
  microbench A/B is the proof, not an assertion.
- All rendered values, messages, and existing test expectations stay
  byte-identical; only NEW behavior behind the new `Option` field.
- Probe budget: best-of-2 per timing, full env set up front, raw outputs
  saved to files before parsing; timeouts via
  `scripts/run-bounded.py` sized from prior runs.
- Stop-loss: if the stride cannot live inside the `Some` arm without
  touching the `None` path, or an existing test pins behavior this
  change must alter, STOP and report.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Test survey — each new/extended test, named.
3. Verification — commands + verbatim outputs (test totals, A/B
   per-label rows, the K=1 vs K=64 wall times and the decision).
4. The layering note you wrote for the knob's doc comment (quoted).
5. Deviations / open questions (empty if none).
