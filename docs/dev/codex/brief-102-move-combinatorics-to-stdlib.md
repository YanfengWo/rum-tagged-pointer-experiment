# Brief 102 — MOVE pilot: the combinatorics family leaves rum-runtime for rum-stdlib

## Goal

Stage 3 begins (`docs/crate-rebalance-plan.md`): physically move ONE leaf builtin
family end-to-end, proving the whole extraction pipeline. After this brief:
`crates/rum-stdlib/src/` OWNS the combinatorics implementation (11 handlers),
its 11 catalog specs, and its behavioral tests; `rum-runtime` contains NO
combinatorics code; `install_core` = runtime catalog + stdlib-owned specs; a
fresh `Session::new()` + `install_core` session evaluates `Fibonacci[10]` → `55`.
`cargo test --workspace` green. Combinatorics was chosen because it has ZERO
incidental usage in other crates'/families' tests (measured) — the kernel test
tree survives its departure.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`, and skim
  `docs/dev/kernel-api-audit-2026-07-03.md` (combinatorics row) first.
- Landed pattern briefs: 097 (fn-pointer dispatch), 099 (kernel_api + facade
  per-item `pub use` lesson), 100 (registration-only dispatch; `install_core`
  is real), 101 (method→free-fn conversion, receiver-only, bodies verbatim).
- Facts (verified 2026-07-03):
  - `runtime/combinatorics.rs` (562 lines) owns EXACTLY these 11 handlers:
    `eval_binomial`, `eval_fibonacci`, `eval_partitions_p`, `eval_partitions_q`,
    `eval_bernoulli_b`, `eval_subfactorial`, `eval_stirling_s1`,
    `eval_stirling_s2`, `eval_catalan_number`, `eval_multinomial`,
    `eval_pochhammer` — plus PRIVATE helpers
    (`nonnegative_combinatorics_index`, `enforce_integer_work_limit_for_binomial`,
    `enforce_combinatorics_work_units`) that move with it. `eval_factorial2`'s
    spec sits among the same catalog entries but its handler is arithmetic's —
    it does NOT move. Re-verify the inventory with
    `grep "fn eval_" crates/rum-runtime/src/runtime/combinatorics.rs`.
  - The 11 specs live in `runtime/builtin_catalog/numeric.rs` — split them out;
    the file keeps everything else (incl. Factorial/Factorial2).
  - Kernel Session methods combinatorics calls (to make `pub` with a one-line
    doc): `invalid_call_with_exact_arg_count` (session_helpers),
    `combine_additive_args` + `combine_multiplicative_args` (arithmetic.rs,
    audit fan-out 3 — legit kernel canonicalization API), plus whatever the
    compiler reveals from inside the private enforce helpers (likely the
    resource/charge API) — list every promotion in the report.
  - Free-item imports: combinatorics' audit demand is ~7 items (e.g.
    `expr_as_exact_integer`) — mostly already in `kernel_api`; extend
    `kernel_api` per demand (099 policy; facade per-item `pub use` where
    needed).
  - Tests: `runtime/tests/combinatorics.rs` (209 lines) + its `mod` line at
    `runtime/tests/mod.rs:70`. Zero combinatorics names in any other test file
    (measured) — if the tree proves otherwise, STOP and report.
  - `rum-stdlib` today: 21-line `install_core` + `src/tests.rs`
    (`Session::new()` + `install_core` pattern); Cargo deps = rum-runtime only.

## Step 0 — the resolution probe (do this FIRST, before any move)

Unknown to settle: bare-name resolution for a builtin REGISTERED but absent
from the static catalog. `Session::symbol_exists` consults registered specs
(fine), but the static vocabulary path
(`expr_helpers::resolve_unqualified_symbol` → `is_known_system_symbol_name`,
used by `lowering.rs:5` among others) predates registration-only dispatch.

Write a runtime test: `register_builtin` a spec with a name NOT in the static
catalog (e.g. `"ZzzProbe"`, trivial handler fn returning an Expr) on a fresh
session, then `eval_text("ZzzProbe[1]")` — assert it dispatches (does NOT stay
an inert `Global`-context symbol).

- If it PASSES: no vocabulary work; keep the test as a regression guard.
- If it FAILS: fix the vocabulary path so lowering/resolution treats
  SESSION-REGISTERED specs like catalog ones (expected touch points:
  `lowering.rs` / `expr_helpers.rs` / `resolution.rs`; keep the static path for
  session-free callers like ToExpression string parsing, and report the exact
  semantics you chose). This fix also benefits ALL user-registered builtins —
  it is in scope. If the fix balloons past those files, STOP and report.

## The move (after step 0 is green)

1. `crates/rum-stdlib/Cargo.toml`: add the `rum-core` path dependency
   (mirror rum-runtime's dep style). `Cargo.lock` will update — that is the ONE
   sanctioned lock change.
2. Layout: `crates/rum-stdlib/src/lib.rs` keeps `install_core` (now: register
   `rum_runtime::core_builtin_specs()` THEN the stdlib's own specs);
   new `crates/rum-stdlib/src/combinatorics.rs` = the moved implementation;
   new `crates/rum-stdlib/src/catalog.rs` = the 11 `BuiltinSpec` entries
   (constructors/AttributeSet are `pub` via rum_runtime) referencing the local
   free fns.
3. Convert while moving (the 101 pattern): every `impl Session` method in the
   moved file becomes `fn eval_x(session: &mut Session, …)` / private helpers
   likewise; bodies VERBATIM; imports become explicit
   `use rum_runtime::kernel_api::{…};` + `use rum_core::{…};` + the
   now-`pub` Session methods stay method calls.
4. Kernel removals: delete `runtime/combinatorics.rs`, its `mod` + glob lines
   in `runtime/mod.rs`, and the 11 spec entries from
   `builtin_catalog/numeric.rs` (count before/after: 513 → 502 entries;
   auto-install shrinks accordingly — assert the fresh-session builtin count
   test from brief-100 still passes, adjusting its expectation ONLY by reading
   `core_builtin_specs().len()` as it already does).
5. Kernel promotions: the Session methods listed above become `pub fn` (doc
   one-liner each); any additional free item goes through `kernel_api`
   (per-item facade `pub use` where nested — 099 lesson).
6. Test migration: move `runtime/tests/combinatorics.rs` into rum-stdlib
   (e.g. `src/tests/combinatorics.rs` or merged into its tests module),
   adapting session setup to `Session::new()` + `install_core` (the existing
   stdlib test pattern); delete the runtime `mod` line. Every one of the ~30
   test functions must survive — no test deletions.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/**` (Cargo.toml, src — this is the destination)
  - `Cargo.lock` (the rum-core dep only)
  - `crates/rum-runtime/src/runtime/combinatorics.rs` (DELETE)
  - `crates/rum-runtime/src/runtime/mod.rs` (mod/glob removals)
  - `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs` (the 11 spec
    removals only)
  - `crates/rum-runtime/src/runtime/session_helpers.rs`,
    `crates/rum-runtime/src/runtime/arithmetic.rs`, and compiler-revealed
    definition sites — VISIBILITY-ONLY promotions + one-line docs
  - `crates/rum-runtime/src/runtime/kernel_api.rs` + facades for per-item
    re-exports (099 pattern)
  - `crates/rum-runtime/src/runtime/tests/mod.rs` (drop the mod line) +
    `crates/rum-runtime/src/runtime/tests/combinatorics.rs` (MOVE out)
  - Step-0 probe test location + (only if the probe fails)
    `crates/rum-runtime/src/runtime/{lowering,expr_helpers,resolution}.rs`
- MUST NOT touch: any other family, `eval_dispatch/`, other crates
  (cli/repl/host/jupyter must compile & pass UNCHANGED — they use
  `install_core` already), any `mma-hol` clone, `docs/**` except your report,
  `.git`.

## Deliverable

- The move in the working tree (no commit); step-0 probe verdict + (if
  applicable) the vocabulary fix description.
- Promotion inventory: every kernel item made `pub` (method or kernel_api
  re-export), with its demander.
- Verbatim gates below.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim** — the move is relocation + receiver change, not a
   rewrite. Message text/order identical; the work-limit enforce helpers keep
   their exact thresholds and messages.
2. **Spec identity**: the moved specs must be byte-identical in
   name/summary/attributes/dispatch/capability to the deleted ones — only the
   handler path changes. Paste one before/after example in the report.
3. **Install order**: core specs first, stdlib specs second (register_builtin
   merge semantics from brief-100 assume prior = catalog).
4. **dev-dependency direction**: rum-stdlib already depends on rum-runtime; do
   NOT add any rum-runtime dependency on rum-stdlib in this brief (the probe
   test lives in rum-runtime using only runtime APIs; the moved tests live in
   rum-stdlib).
5. Zero behavior change for everything NOT combinatorics; combinatorics itself
   changes only its crate of residence. `cargo test --workspace` totals must
   match 1298/0 modulo the moved/added tests — account for every delta in the
   report.
6. No microbench: dispatch/eval machinery untouched; only the registration
   source of 11 specs moved (state this in the report).
7. Stop-loss: same failure twice, or the probe fix ballooning, or any
   non-combinatorics test breaking for a non-obvious reason → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "combinatorics\|Fibonacci\|CatalanNumber" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                          # expect 11
cargo run -p rum-cli --release -- eval "Fibonacci[10] + CatalanNumber[4]" 2>/dev/null | tail -2   # expect 69
```

Paste all outputs verbatim, plus the step-0 probe test name + result, and the
workspace totals accounting (runtime lost N tests, stdlib gained N+probe…).

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

1. Step-0 probe verdict (+ vocabulary fix if made, with chosen semantics).
2. What changed — moved files, kernel deletions, promotion inventory, spec
   before/after example, test migration accounting.
3. Verification — each command with exact verbatim output.
4. Verdict — the pipeline works end-to-end / what the next family needs.
5. Stop-loss / open questions (empty if none).
