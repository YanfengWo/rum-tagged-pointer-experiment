# Brief 111 — Stage-3 batch 10: control_flow → rum-stdlib (the signal machinery stays)

## Goal

Tenth Stage-3 move (pipeline + rules per briefs 106-110). The `control_flow`
BUILTIN family (If/Which/Switch/Do/While/For/Table/Sum/Product/
CompoundExpression/Throw/Catch/Sow/Reap/TimeConstrained/…) moves to
`rum-stdlib`; the kernel keeps everything the evaluator itself depends on —
non-local-exit SIGNAL machinery (Return/Break/Continue/Throw/Abort
propagation through the eval loop), iterator/Block wrappers if
kernel-referenced, the Sow/Reap side-table plumbing in the session model —
per reference analysis. After this brief: runtime greps clean of the family
module; `cargo test --workspace` green; CLI smoke
`--eval "Module[{s = 0}, Do[s = s + i, {i, 1, 5}]; s] + Catch[Throw[7]]"` → `22`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror briefs 108-110.
  All standing rules apply (verbatim bodies + in-transit free-fn conversion,
  spec identity, path-precise gates, Option-1 test migration with ratio rule,
  probes never rewritten, test-local registrations listed and minimized,
  behavior-equivalence adaptations reported individually, stays are verbatim
  relocations with the stay-list pasted).
- Facts (verified 2026-07-04):
  - Implementation: `runtime/control_flow/` — 14 files, ~4.3k lines, 41
    `eval_*` fns. Specs: `builtin_catalog/control.rs` (33 references;
    ownership by definition site).
  - This is the HIGHEST-COUPLING family (audited 25 kernel imports). Known
    hot boundaries — decide each by reference analysis, and expect a
    substantial stay-list (a large stay-list is a correct outcome, not a
    failure):
    - Non-local exits: the eval loop and suspension machinery consume
      Return/Break/Continue/Throw/Abort signals — whatever parts of that
      signal path live in `control_flow/` and are referenced by kernel code
      STAY (`runtime/control_data.rs` naming precedent).
    - Iterator machinery (Do/Table iterator specs, the Block wrapper): if
      kernel code references it, it stays; otherwise it moves with the
      family.
    - Sow/Reap touch the session side tables (`reap_bucket_values` in the
      model); the side-table itself is kernel state — handlers move, state
      stays, promote narrow accessors if the compiler demands.
    - `WithCleanup`/`AbortProtect`/`CheckAbort` interact with the
      interrupt/cancel Tier-1 machinery — that machinery is kernel; handlers
      move.
  - Promotion stop line: ≤12 genuinely new items (sized to the audited 25
    kernel imports; this family legitimately needs more doors). STOP AT 13,
    even mid-edit.
  - Tests: `runtime/tests/holding_and_control*` (many files, several already
    partially migrated), `tests/session_requests/*` resume files — expect a
    LARGE migration; ratio rule per file; persist raw logs first. The VM
    template tests may need test-local `If`/`Do` registrations (the SameQ
    precedent) — list each.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/control_flow/**` (DELETE after the split) +
  NEW `runtime/control_data.rs` (or module dir; verbatim stays);
  `runtime/mod.rs`; `builtin_catalog/control.rs` (family-entry removals);
  `runtime/tests/**` per migration rules + `runtime/tests/mod.rs`;
  `kernel_api.rs` retargets + promotion sites/facades (visibility + one-line
  doc only, list each); `vm_rhs_template.rs` tests ONLY for test-local
  registrations; `Cargo.lock` NOT expected to change.
- MUST NOT touch: other families, `eval_dispatch/`, `eval_core.rs`/
  `suspension/**` beyond compiler-forced import retargets (list each; if
  signal-propagation LOGIC needs changes the split is wrong — STOP), other
  crates, any `mma-hol` clone, `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Signal semantics are the crown jewels**: Return/Break/Continue/Throw
   propagation, `TerminatedEvaluation` tags, and abort/cleanup ordering must
   be byte-stable. Any change to how a signal crosses the eval loop is a
   defect even with green tests.
2. **Suspension density**: Do/While/For/Table resumable shapes move VERBATIM.
3. **Bodies verbatim**; message text/order identical (iteration limit
   messages, `General::argct`-style diagnostics).
4. **Spec identity** modulo handler path; install order core-then-stdlib.
5. **Mass migration discipline**: ratio rule, nothing deleted/weakened,
   totals conserved (currently 1300/0), full table, test-local registrations
   listed.
6. No microbench (ownership move only — state it). NOTE: control_flow
   handlers ARE hot-path adjacent (Do/CompoundExpression), but the dispatch
   shape (fn pointer via registered spec) is IDENTICAL before and after the
   move — assert this in the report; if you change ANY dispatch-relevant
   code shape, that voids the no-microbench waiver and you STOP.
7. Stop-loss: >12 genuinely new promotions (STOP AT 13), signal-logic
   changes, unattributable non-target failures, same compile error twice →
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod control_flow\|control_flow::" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                   # report (276 + moved)
cargo run -p rum-cli --release -- --eval "Module[{s = 0}, Do[s = s + i, {i, 1, 5}]; s] + Catch[Throw[7]]" 2>/dev/null | tail -1   # expect 22
```

Paste all outputs verbatim + workspace accounting + the FULL migration table +
the stay-list (item → kernel referencer).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; changes stay in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Full machine access, but nothing outside this repo, no network; only file
  edits in Scope, `cargo`, `grep`, `python3`.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Clean-check probes: `git status --short -- crates/`.
- Minimal diff; paste real verbatim outputs; never fake a green.

## Report format (your final message)

1. What changed — stay-list, moved file map, catalog split, promotion
   inventory, FULL migration table, test-local registrations.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health; the no-dispatch-shape-change assertion; advice for
   rewrite_builtins and arithmetic (the last two).
4. Stop-loss / open questions (empty if none).
