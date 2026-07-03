# Brief 103 — Stage-3 batch move: integer_digits + number_theory → rum-stdlib

## Goal

Second Stage-3 move, first BATCH (`docs/crate-rebalance-plan.md`): run the
proven brief-102 pipeline over TWO leaf families in one pass. After this brief:
`rum-stdlib` owns the `integer_digits` and `number_theory` implementations,
their specs, and their behavioral tests; `rum-runtime` greps clean of both;
`cargo test --workspace` green; CLI smoke
`--eval "NextPrime[100] + FromDigits[{1,2,3}] + EulerPhi[10]"` → `228`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; the pattern is brief-102
  (committed 47716e6) — mirror its structure in rum-stdlib exactly
  (per-family `src/<family>.rs`, specs appended to `src/catalog.rs`,
  `install_core` unchanged in shape).
- Facts (verified 2026-07-04):
  - `runtime/integer_digits.rs` (387 lines): builtins `IntegerLength`,
    `IntegerDigits`, `IntegerReverse`, `FromDigits`, `DigitCount`. One of its
    `fn eval_*` is a local threading helper, not a builtin handler — determine
    handler-vs-helper by what `builtin_catalog/numeric.rs` references; helpers
    move along as private free fns.
  - `runtime/number_theory.rs` (733 lines): builtins `Divisors`, `EulerPhi`,
    `MoebiusMu`, `DivisorSigma`, `Prime`, `PrimePi`, `PrimeNu`, `PrimeOmega`,
    `NextPrime`. Its private helpers (`try_thread_number_theory_call`,
    `push_prime_intpp_message`, +1 more defined in-file) move along. The
    Prime/PrimePi small tables are plain code — move verbatim.
  - Both families' specs live in `builtin_catalog/numeric.rs`.
  - Kernel Session methods these families call (promote to `pub` with a
    one-line doc, per demand — the compiler is the referee):
    `invalid_call_with_arg_range` (session_helpers),
    `enforce_generated_element_limit_for_count`
    (debug_tools/resources/constraints), `evaluate` (the big one — it was
    always going to be the head of the kernel method API; doc it properly),
    `thread_call_args` / `eval_threaded_builtin_args` (listable_dispatch),
    `charge`, `push_message` — some may already be visible enough; promote
    only what rustc demands and list every promotion.
  - Tests: `runtime/tests/number_theory.rs` moves whole (like brief-102).
    `integer_digits` has NO dedicated file — its tests (46 name references)
    sit inside `runtime/tests/arithmetic.rs`. Migration rule for that case
    (NEW, this brief establishes it): after the move, run
    `cargo test -p rum-runtime`; every FAILING test function in
    `tests/arithmetic.rs` (they fail because the builtins are now inert in a
    runtime-only session) moves WHOLE to a new rum-stdlib test module
    (`src/tests/integer_digits.rs`), adapted to the stdlib session pattern
    (`Session::new()` + `install_core`). No test function may be deleted or
    weakened; mixed-purpose functions move whole (the stdlib session is a
    superset, they must pass there). Account for every moved function in the
    report.

## The move (per family; integer_digits first, then number_theory)

Identical to brief-102's pipeline:

1. Move the implementation file to `crates/rum-stdlib/src/<family>.rs`,
   converting `impl Session` methods → free fns (receiver-only, bodies
   VERBATIM), imports → explicit `use rum_runtime::kernel_api::{…}` +
   `use rum_core::{…}`; kernel Session-method calls stay method calls.
2. Append the family's specs to `crates/rum-stdlib/src/catalog.rs`
   (byte-identical modulo handler path; paste one before/after example per
   family). Register modules in `src/lib.rs`.
3. Delete from rum-runtime: the implementation file, its `mod` + glob lines in
   `runtime/mod.rs`, the spec entries in `builtin_catalog/numeric.rs`
   (report the catalog count 502 → expected 488).
4. Promote kernel items per demand (methods `pub` + doc; free items via
   `kernel_api` with the facade per-item `pub use` pattern where nested).
5. Migrate tests per the rules above (whole file for number_theory;
   failing-function migration for integer_digits).

## Scope

- MAY modify: `crates/rum-stdlib/**`; `Cargo.lock` (only if a new dep is truly
  forced — none is expected, rum-core is already a dep);
  `crates/rum-runtime/src/runtime/{integer_digits,number_theory}.rs` (DELETE);
  `runtime/mod.rs` (mod/glob removals); `builtin_catalog/numeric.rs` (spec
  removals); `runtime/tests/number_theory.rs` (MOVE out),
  `runtime/tests/arithmetic.rs` (failing-function removals),
  `runtime/tests/mod.rs`; promotion definition sites (`session_helpers.rs`,
  `listable_dispatch.rs`, `debug_tools/resources/constraints.rs`,
  `kernel_api.rs`, facades, and compiler-revealed — visibility + one-line doc
  ONLY, list each).
- MUST NOT touch: any other family, `eval_dispatch/`, other crates, any
  `mma-hol` clone, `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim**; message text/order identical; the Prime/PrimePi table
   bounds and `General::intpp`-style messages keep their exact behavior.
2. **Spec identity** modulo handler path; install order stays core-then-stdlib.
3. **Handler-vs-helper by catalog reference**, not by name pattern.
4. **Test migration is by measured failure**, not by grep guess — run the
   runtime suite to find the movers; after migration BOTH suites must be green
   and the workspace total accounted (1299 ± moved/added).
5. **`evaluate` going `pub` is intended** (kernel method API head); do not
   invent a wrapper — promote it directly with a doc comment.
6. No microbench (registration source only — state it).
7. Stop-loss: same failure twice, a non-target family's test breaking for a
   non-obvious reason, or promotion demands exploding → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "integer_digits\|number_theory\|NextPrime\|EulerPhi\|FromDigits" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                                                  # expect 25 (11+5+9)
cargo run -p rum-cli --release -- --eval "NextPrime[100] + FromDigits[{1,2,3}] + EulerPhi[10]" 2>/dev/null | tail -1      # expect 228
```

Paste all outputs verbatim + the workspace-totals accounting + the moved-test
inventory (function names, from → to).

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

1. What changed — per family: moved files, spec counts, promotion inventory
   (item → demander), moved-test inventory.
2. Verification — each command with exact verbatim output.
3. Verdict — batch pipeline health; anything the NEXT batch should know.
4. Stop-loss / open questions (empty if none).
