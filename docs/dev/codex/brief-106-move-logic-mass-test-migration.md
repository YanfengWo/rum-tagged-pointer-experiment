# Brief 106 — Stage-3 batch 5: logic → rum-stdlib (first mass test migration)

## Goal

Fifth Stage-3 move (pipeline per briefs 102-105). The `logic` family
(truth ops + the COMPARISON operators) moves to `rum-stdlib`. This family's
conversion work is ALREADY DONE (brief-099 de-glob, brief-101 free-fn) and its
kernel-method needs are already `pub` — the increment is the physical move plus
the first MASS test migration: comparisons appear all over the runtime test
tree, and by the user's decision (2026-07-04) those tests now MIGRATE with the
family. Hundreds of moving test fns is the expected outcome, not a stop signal.
After this brief: runtime greps clean of the logic family;
`cargo test --workspace` green; CLI smoke
`--eval "If[Xor[True, False] && 1 <= 1, 42, 0]"` → `42`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror briefs 102-105's
  structure (commits 47716e6/5566c5c/a0cfa6a/a943624). All standing rules
  apply (verbatim bodies, spec identity, promotion per compiler demand,
  handler-vs-helper by catalog reference, no flattening).
- Facts (verified this line):
  - Implementation: `runtime/logic.rs` (facade with per-item `pub use`) +
    `runtime/logic/{truth,comparison,shared}.rs` — handlers are ALREADY free
    fns `fn eval_x(session: &mut Session, …)`. The move is file relocation +
    import rewrites (`crate::runtime::kernel_api::…` →
    `rum_runtime::kernel_api::…`, `rum_core` direct) — bodies untouched.
  - The family's 16 fns: truth (and/or/nand/nor/xor/equivalent/implies/not) +
    comparison (equal/unequal/less/greater/less_equal/greater_equal/
    comparison_operator_wrapper + private eval_comparison_subset) + the
    private truth continuation helpers. `Boole`/`TrueQ`-style predicates whose
    handlers live OUTSIDE the logic module do NOT move (catalog-reference rule).
  - Specs: in `builtin_catalog/logic.rs` — that file references ~69 handlers
    but only the logic-module-owned entries move (expected ~15; report the
    split; runtime catalog 421 → report measured).
  - Kernel needs: `suspend_with_frame`, `evaluate`,
    `invalid_call_with_exact_arg_count` — ALL already pub;
    `truthiness_symbol`/`system_expr`/`is_system_symbol_name` already in
    kernel_api. Expected NEW promotions: ~0. If the compiler demands more than
    ~5 new items, something is off — STOP and report.
- **Mass test migration rules (Option 1, user-decided):**
  1. Move the dedicated files first: `runtime/tests/logic/{comparison,truth}.rs`,
     `runtime/tests/logic.rs`, `runtime/tests/logic_part2.rs`, and
     `runtime/tests/session_requests/matcher_resume/logic.rs` → stdlib test
     modules (adapt to the `Session::new()` + `install_core` pattern).
  2. Then run `cargo test -p rum-runtime` and apply the ratio rule to every
     file with failures: **≥50% of a file's test fns failing → move the WHOLE
     file** to a same-named stdlib test module; **<50% → move only the failing
     fns** into per-source `incidental_*.rs` stdlib modules. The stdlib session
     is a superset, so moved non-failing fns must pass there too.
  3. NO test fn may be deleted or weakened; every move accounted in the report
     (file → whole/partial → fn count). Empty runtime test files are removed;
     `runtime/tests/mod.rs` updated.
  4. Iterate (a file that only fails after another file moved is normal) until
     `cargo test -p rum-runtime` is green, then the full workspace.
- Note: tests that exercise kernel MECHANISM but happen to use a comparison in
  their probe expressions migrate like any other — that is the accepted cost;
  do not rewrite probes to avoid migration.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/logic.rs` +
  `crates/rum-runtime/src/runtime/logic/**` (DELETE); `runtime/mod.rs`;
  `builtin_catalog/logic.rs` (family-entry removals; other families' entries
  stay); `runtime/tests/**` per the migration rules (moves/removals only) +
  `runtime/tests/mod.rs`; `kernel_api.rs`/facades/definition sites ONLY if a
  genuinely new promotion is demanded (expected ~0; list each);
  `Cargo.lock` NOT expected to change.
- MUST NOT touch: any other family's implementation or specs,
  `eval_dispatch/`, other crates, any `mma-hol` clone, `docs/**` except your
  report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim** — the `Evaluate`-style suspension shapes in the truth
   continuations moved verbatim in brief-101; keep them byte-stable through
   the relocation too.
2. **Import rewrite only**: `use crate::runtime::kernel_api::{…}` becomes
   `use rum_runtime::kernel_api::{…}`; family-internal `use super::shared::…`
   stays as-is.
3. **Spec identity** modulo handler path; install order core-then-stdlib.
4. **The ratio rule needs real counts**: compute per-file failing/total from
   the actual `cargo test` output (persist the raw log to /tmp first — never
   inline-grep and lose it), not from grep guesses.
5. Workspace totals accounted (currently 1300/0): runtime loses N, stdlib
   gains N (+ any new), zero net loss of test fns.
6. No microbench (ownership move only — state it).
7. Stop-loss (reframed per Option 1): >5 genuinely NEW kernel promotions, any
   NON-logic-related test failure you cannot attribute to the move, or the
   same compile error twice → STOP and report. Mass test movement itself is
   NOT a stop condition.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod logic\|logic::\|eval_comparison_subset" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                             # report (expect 92 + ~15)
cargo run -p rum-cli --release -- --eval "If[Xor[True, False] && 1 <= 1, 42, 0]" 2>/dev/null | tail -1   # expect 42
```

Paste all outputs verbatim + the workspace-totals accounting + the full
migration table (file → whole/partial → fn count → destination).

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

1. What changed — moved implementation files, catalog split, promotion
   inventory (expected ~0), the FULL test-migration table.
2. Verification — each command with exact verbatim output.
3. Verdict — how the mass migration went: total fns moved, whole-file vs
   partial counts, what the runtime test tree looks like now (mechanism-only
   trend), advice for predicates (the next mass batch).
4. Stop-loss / open questions (empty if none).
