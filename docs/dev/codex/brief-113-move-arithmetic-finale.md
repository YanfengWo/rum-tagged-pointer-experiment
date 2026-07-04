# Brief 113 — Stage-3 FINALE: arithmetic → rum-stdlib (hottest handlers; A/B at the gate)

## Goal

The LAST Stage-3 family (pipeline + rules per briefs 106-112, incl. both
solution shapes and the amended mechanism-test rule). `arithmetic` owns the
kernel's HOTTEST handlers (Plus/Times/Power drive every evaluation) — choose
the shape by reference analysis exactly like brief-112, with a strong prior
for the WRAPPER shape (kernel iterator machinery references arithmetic
helpers; the canonicalization `combine_*` methods are Session methods used
from stdlib families). After this brief: every builtin family's SPEC
ownership lives in `rum-stdlib`; `cargo test --workspace` green; CLI smoke
`--eval "Plus[1, 2, 3] * Power[2, 3] - Mod[17, 5] + Max[1, 9]"` → `55`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror brief-112's
  report structure (shape decision + evidence FIRST).
- Facts (verified 2026-07-04):
  - Implementation: `runtime/arithmetic.rs` — ONE file, 1185 lines, 27
    `eval_*` handlers: plus, times, power, subtract, divide, abs, sign,
    floor, ceiling, integer_part, fractional_part, minus, round, factorial,
    factorial2, total, denominator, numerator, mod, quotient, gcd, lcm, max,
    min, min_max, exact_extremum (helper?), power_mod.
  - Specs: `builtin_catalog/numeric.rs` has EXACTLY 27 entries left — the
    family and the file are now aligned; the whole file goes (or empties).
  - Kernel-side references INTO arithmetic: `control_data` iterator machinery
    (`branching_loops/iteration.rs`) uses arithmetic helpers; `combine_additive_args`/
    `combine_multiplicative_args` are pub Session METHODS on arithmetic.rs
    used by stdlib families (calculus/combinatorics/polynomials) — under the
    wrapper shape they simply stay where they are; under a full move they
    would need relocation, which likely blows the coupling budget. Evidence
    decides; the prior says wrapper.
  - Under the wrapper shape: `runtime/arithmetic.rs` stays (rename to
    `runtime/arithmetic_data.rs` for the grep gate OR keep the name and gate
    on `builtin_catalog` emptiness — you choose, state it; path-precise gates
    only), specs move behind thin wrappers via an `ArithmeticDataBuiltin`-style
    facade (the 111/112 pattern), promotions ≤3.
  - Tests: `runtime/tests/arithmetic.rs` remnants (already partially
    migrated) — ratio rule; mechanism tests may stay with test-local
    registrations (Plus/Times will be needed by MANY runtime mechanism tests
    — the evaluation_cache/vm template tests especially; a SHARED test-helper
    module for common test-local registrations is allowed if it reduces
    duplication, but it must live under `#[cfg(test)]` and be reported).
  - Promotion stop line: ≤3 under wrapper; ≤10 under full move (STOP over).
- THE GATE WILL RUN A FULL MICROBENCH A/B REGARDLESS of shape — Plus/Times
  are the per-call hot core; you do NOT run it (probe budget), but your
  report must assert the dispatch-shape invariant (registered BuiltinSpec fn
  pointer path unchanged) and enumerate any code that moved between the
  handler entry and the arithmetic work.

## Scope

- MAY modify: `crates/rum-stdlib/**`; `crates/rum-runtime/src/runtime/arithmetic.rs`
  (rename/split per shape); `runtime/mod.rs`; `builtin_catalog/numeric.rs`
  (family-entry removals; delete or empty the file + catalog wiring);
  `runtime/tests/**` per rules + `runtime/tests/mod.rs`; `kernel_api.rs`
  retargets + promotion sites (visibility + one-line doc only); test-local
  registration sites incl. a shared `#[cfg(test)]` helper; `Cargo.lock` NOT
  expected to change.
- MUST NOT touch: `eval_core`/`eval_dispatch`/`hold_thread`/
  `listable_dispatch` beyond compiler-forced import retargets (list each —
  Plus/Times Orderless/Flat canonicalization ORDER is evaluator semantics),
  other families' stdlib code except catalog append, any `mma-hol` clone,
  `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Plus/Times bodies are the most-executed code in the engine.** Bodies
   verbatim; exact-rational arithmetic, overflow paths, and canonical
   ordering byte-stable. Any "simplification" is a defect.
2. **Numeric messages** (Power::infy, Divide-by-zero…) identical.
3. **Spec identity** modulo handler path; install order core-then-stdlib.
4. **Migration discipline** as amended; totals conserved (currently 1300/0);
   full table; test-local registrations (incl. the shared helper if used)
   listed.
5. Dispatch-shape invariant asserted explicitly in the report.
6. Stop-loss: promotion lines, canonicalization-order changes,
   unattributable non-target failures, same compile error twice → STOP and
   report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -c "BuiltinSpec::" crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs 2>/dev/null || echo FILE_GONE   # expect 0 or FILE_GONE
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                                           # report (318 + 27)
cargo run -p rum-cli --release -- --eval "Plus[1, 2, 3] * Power[2, 3] - Mod[17, 5] + Max[1, 9]" 2>/dev/null | tail -1   # expect 55
```

Paste all outputs verbatim + workspace accounting + the FULL migration table +
the shape decision evidence + the dispatch-shape invariant assertion.

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

1. Shape decision + evidence FIRST; dispatch-shape invariant assertion.
2. What changed — stay/move map, catalog split (numeric.rs 27 → 0), promotion
   inventory, FULL migration table, test-local registrations incl. shared
   helper.
3. Verification — each command with exact verbatim output.
4. Verdict — the line's END STATE: every family's spec ownership in
   rum-stdlib; what remains in the kernel catalogs (the evaluator-semantics
   residue inventory: Function/HoldForm/Part family/definition machinery/…).
5. Stop-loss / open questions (empty if none).
