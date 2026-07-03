# Brief 104 — Stage-3 batch 3: calculus + polynomials → rum-stdlib

## Goal

Third Stage-3 move through the proven pipeline (briefs 102/103, commits
47716e6/5566c5c — mirror their structure exactly). After this brief:
`rum-stdlib` owns the `calculus` and `polynomials` families; `rum-runtime`
greps clean of both; `cargo test --workspace` green; CLI smoke
`--eval "Coefficient[Expand[(x+1)^4], x, 2] + D[x^3, x] /. x -> 2"` → `18`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`. The pipeline, promotion
  policy, facade `pub use` pattern, and test-migration rules are all
  established — see the two prior move briefs and their commits.
- Facts (verified 2026-07-04):
  - `runtime/calculus.rs` (276 lines): ONE handler (`eval_d` = `D`), plus
    private helpers. Its known kernel needs (`combine_multiplicative_args`,
    `expr_as_exact_*`) are ALREADY pub/kernel_api from prior batches.
  - `runtime/polynomials/` (directory: `mod.rs`, `candidates.rs`, `core.rs`,
    `core/`, `factorization.rs`, `rational_function.rs`): handlers `Expand`,
    `Coefficient`, `CoefficientList`, `Together`, `Factor` (verify by catalog
    reference as always — `eval_polynomial_q`-style predicates belong to the
    predicates family and MUST NOT move). The audit says most of its 38
    imported items are same-family locals — they move along; only genuinely
    kernel-owned demands get promoted.
  - Specs: find them by grepping the catalog files for the builtin names
    (expected in `builtin_catalog/numeric.rs`; count before/after —
    488 → expected 482).
  - Tests: `runtime/tests/calculus.rs` and `runtime/tests/polynomials.rs` move
    whole. `D[` also appears in 2 OTHER runtime test files — apply the
    measured-failure rule from brief-103: after the move, run
    `cargo test -p rum-runtime`; failing functions move whole to the stdlib
    test modules. Account for every moved function.
  - Directory move: preserve the polynomials submodule layout under
    `crates/rum-stdlib/src/polynomials/` (this is the first multi-file family
    move — keep intra-family `pub(crate)`/module structure working with the
    minimal visibility adjustments the compiler demands; do NOT flatten the
    directory).

## The move (calculus first, then polynomials)

Identical pipeline: move + convert (`impl Session` → free fns, receiver-only,
bodies VERBATIM; explicit `kernel_api`/`rum_core` imports), append specs to
`crates/rum-stdlib/src/catalog.rs` (byte-identical modulo handler path; one
before/after example per family), register modules in `src/lib.rs`, delete
runtime files + `mod`/glob lines + spec entries, promote kernel items strictly
per compiler demand (methods `pub` + one-line doc; free items via `kernel_api`
with facade per-item `pub use` where nested), migrate tests per the rules
above.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/calculus.rs` +
  `crates/rum-runtime/src/runtime/polynomials/**` (DELETE);
  `runtime/mod.rs`; the catalog file(s) holding the specs (removals only);
  `runtime/tests/{calculus,polynomials}.rs` (MOVE out), `runtime/tests/mod.rs`,
  and any runtime test file whose functions FAIL after the move
  (failing-function removals only); promotion definition sites +
  `kernel_api.rs` + facades (visibility + one-line doc only, list each);
  `Cargo.lock` only if a truly new dependency is forced (none expected).
- MUST NOT touch: any other family (especially `predicates` — `PolynomialQ`
  etc. stay), `eval_dispatch/`, other crates, any `mma-hol` clone, `docs/**`
  except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim**; message text/order identical; `Factor`'s
   structural-only scope and its `partial` fallback messages stay EXACTLY as
   they are (folk-implementation policy — do not "improve" anything in
   transit).
2. **Spec identity** modulo handler path; install order core-then-stdlib.
3. **Handler-vs-helper by catalog reference**; predicates' handlers that live
   near polynomial code do NOT move.
4. **Test migration by measured failure**, then both suites green; account for
   the workspace totals (currently 1299/0).
5. **Multi-file family**: keep the directory structure; intra-family imports
   may use `crate::polynomials::…` paths inside rum-stdlib — do not route
   family-internal items through kernel_api.
6. No microbench (registration/source ownership only — state it).
7. Stop-loss: same failure twice, promotion demands exploding past ~10 new
   items, or a non-target family's test failing non-obviously → STOP and
   report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "calculus\|polynomials\|eval_expand\|eval_factor\b\|eval_together" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                                                    # expect 31 (25+1+5)
cargo run -p rum-cli --release -- --eval "Coefficient[Expand[(x+1)^4], x, 2] + D[x^3, x] /. x -> 2" 2>/dev/null | tail -1   # expect 18
```

Paste all outputs verbatim + workspace-totals accounting + moved-test inventory
(function names, from → to).

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

1. What changed — per family: moved files (with the polynomials directory map),
   spec counts, promotion inventory (item → demander), moved-test inventory.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health; anything batch 4 should know (especially any
   surprise from the first multi-file directory move).
4. Stop-loss / open questions (empty if none).
