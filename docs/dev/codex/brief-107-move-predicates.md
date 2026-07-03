# Brief 107 — Stage-3 batch 6: predicates → rum-stdlib

## Goal

Sixth Stage-3 move (pipeline per briefs 102-106; mass-test-migration rules per
brief-106, user's Option-1 decision). The `predicates` family moves to
`rum-stdlib`. After this brief: runtime greps clean of it;
`cargo test --workspace` green; CLI smoke
`--eval "Boole[IntegerQ[42]] + Boole[EvenQ[7]] + Boole[OrderedQ[{1, 2, 3}]]"` → `2`
(verify Boole's handler ownership first — if it is not predicates-owned, swap
in an equivalent all-predicates smoke and say so).

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror brief-106
  (commit d876645) exactly — including its test-migration rules (whole-file at
  ≥50% failing, else partial into `incidental_*.rs`; no deletions/weakening;
  full accounting; probes are never rewritten to dodge migration).
- Facts (verified 2026-07-04):
  - Implementation: `runtime/predicates.rs` (420 lines, includes the 3
    polynomial predicates folded in by brief-104) + `runtime/predicates/`
    (`arrays,basic,collections,numeric,ordering,truth`.rs). Handlers are still
    `impl Session` methods — this family needs the free-fn conversion IN
    TRANSIT (the 102-105 pattern: receiver-only, bodies verbatim).
  - Specs: `builtin_catalog/predicates.rs` (50 handler references — verify
    which are predicates-module-owned by definition site; entries whose
    handlers live in OTHER modules stay, and if any stay, leave them in the
    file; report the split; runtime catalog 400 → report measured).
  - Kernel needs (sampled from the facade): `invalid_call_with_arg_range`
    (already pub), `invalid_call_with_arg_form` (EXPECTED new promotion —
    audit-predicted, 67 call sites), family-internal
    `eval_polynomial_like_expression_q`. Expected new promotions ≤5; the
    compiler is the referee; list every one.
  - Tests: `runtime/tests/predicates.rs` + `predicates_part2.rs` +
    `incidental` leftovers — dedicated files move whole first, then the
    measured-failure pass. Note predicates (IntegerQ/EvenQ/ListQ/…) may be
    probe-incidental in more files than logic was — the brief-106 measured
    outcome (31 fns) suggests mild, but let the measurement decide; mass
    movement is NOT a stop condition.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/predicates.rs` +
  `crates/rum-runtime/src/runtime/predicates/**` (DELETE); `runtime/mod.rs`;
  `builtin_catalog/predicates.rs` (family-entry removals; file stays if other
  families' entries remain, else delete it + its mod line);
  `runtime/tests/**` per the migration rules + `runtime/tests/mod.rs`;
  promotion definition sites + `kernel_api.rs` + facades (visibility +
  one-line doc only, list each); `Cargo.lock` NOT expected to change.
- MUST NOT touch: any other family, `eval_dispatch/`, other crates, any
  `mma-hol` clone, `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim** through the free-fn conversion AND the move; message
   text/order identical (the `Positive`/`Negative`/`PossibleZeroQ` messaging
   and the folk `PrimeQ` trial-division stay exactly as-is).
2. **Handler-vs-helper by catalog reference**; anything in the predicates
   module NOT referenced by the catalog is a private helper and moves as one.
3. **Import rewrite**: `use rum_runtime::kernel_api::{…}` + `rum_core` direct;
   family-internal imports stay family-internal.
4. **Spec identity** modulo handler path; install order core-then-stdlib.
5. **Test migration by measured failure** with the ratio rule; persist raw
   logs to /tmp before parsing; totals accounted (currently 1300/0, zero net
   loss).
6. No microbench (ownership move only — state it).
7. Stop-loss: >5 genuinely NEW kernel promotions, unattributable non-target
   failures, or the same compile error twice → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod predicates\|predicates::" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                               # report (113 + moved)
cargo run -p rum-cli --release -- --eval "Boole[IntegerQ[42]] + Boole[EvenQ[7]] + Boole[OrderedQ[{1, 2, 3}]]" 2>/dev/null | tail -1   # expect 2
```

Paste all outputs verbatim + workspace accounting + the full migration table
(file → whole/partial → fn count → destination) + the catalog split.

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

1. What changed — moved files, catalog split, promotion inventory, the FULL
   test-migration table.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health; measured migration size vs logic's 31; advice for
   the remaining five (structure_ops, functional_ops, associations,
   control_flow, rewrite_builtins; arithmetic last).
4. Stop-loss / open questions (empty if none).
