# Brief 013 — Layer A3: reuse SharedArgs in down/up/sub-value candidate build

## Goal

The Pair-descent re-diagnosis (post-Layer-A) localized the dominant per-call
cost to the down/sub-value dispatch stage, and the cleanest reducible piece is
the **candidate build**. On every down/up/sub-value dispatch, the stage does:

```
let candidate = Expr::call(head.clone(), args.to_vec());
```

`args.to_vec()` deep-copies the argument slice into a fresh `Vec`, then
`Expr::call` allocates a new `SharedArgs` for it — on every call that reaches the
stage. Post-A2, the dispatch state already carries the arguments as a
`SharedArgs` (an `Arc<Vec<Expr>>`), so the candidate can reuse that Arc with a
refcount bump instead of cloning:

```
let candidate = Expr::call_with_shared_args(head.clone(), shared_args.clone());
```

This is Layer A3 from `docs/representation-rewrite-plan.md`. It removes the
per-dispatch argument clone + `SharedArgs` allocation for the candidate, on the
down/up/sub-value paths — which is exactly what the Pair conversion combinators
(`ORELSEC`/`REPEATC`/`TRYCONV`/`simpRuleConv`, all down/sub-value-defined) pay
repeatedly, and what the `downvalue-exact-hit` / `downvalue-pattern-hit`
microbench labels measure.

## Expected, measurable result (the gate)

Per 1M calls (`RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1`):
- `downvalue-exact-hit`: `shared_args_alloc` ~`1.00M` → ~`0` (the candidate stops
  allocating its args; `shared_expr_alloc` stays ~`1M` — the candidate head is
  still `SharedExpr::new`, not this brief).
- `downvalue-pattern-hit`: `shared_args_alloc` ~`2.00M` → ~`1.00M` (the
  candidate's `1M` goes away; the RHS rebuild `Plus[1,1]`→`2` keeps its `1M`).
- `into_vec_clone` stays `0` (A2 invariant preserved).

Microbench ratios for `downvalue-exact-hit`, `downvalue-pattern-hit`,
`downvalue-exact-miss`, `downvalue-pattern-miss` improve or hold; no same-tier
regression. If `shared_args_alloc` does not drop as above, the SharedArgs is not
being threaded to the candidate build — STOP and report.

## Context pointers

- Read `docs/representation-rewrite-plan.md` (Layer A, A3) and the
  `2026-06-21 Pair descent re-diagnosis` / `Layer A COMPLETE` entries in
  `PROGRESS.md`.
- The three candidate builds are in
  `crates/rum-runtime/src/runtime/definition_application.rs`:
  - `try_apply_down_values` (fn at line ~4), candidate at line ~69,
  - `try_apply_up_values` (fn at line ~382), candidate at line ~410,
  - `try_apply_sub_values` (fn at line ~575), candidate at line ~606.
  Each is `let candidate = Expr::call(head.clone(), args.to_vec());`.
- These three are called from
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:99-103` with
  `state.evaluated_args()` (a `&[Expr]`). After A2, `CallDispatchState`
  (`session_state.rs`) holds `evaluated_args: Option<SharedArgs>`; the
  `evaluated_args()` accessor returns `&[Expr]`.
- The constructor exists:
  `Expr::call_with_shared_args(head: Expr, args: SharedArgs) -> Expr` (rum-core
  `expr.rs`). `SharedArgs::clone()` is an `Arc` refcount bump (cheap).
  `SharedArgs` derefs/`as_ref`s to `[Expr]`.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definition_application.rs` (the three stage
    fns + their candidate builds),
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` (pass the shared args
    to the three stages),
  - `crates/rum-runtime/src/runtime/session_state.rs` (add a
    `evaluated_shared_args(&self) -> &SharedArgs` accessor if helpful),
  - a runtime test module under `crates/rum-runtime/src/runtime/tests/` if you
    add a regression.
- MUST NOT change: dispatch ordering, matcher/normalization behavior, the RHS
  substitution, the Listable/builtin/Function stages, `Expr`/`SharedArgs` public
  types, serde, any `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

1. Give `try_apply_down_values` / `try_apply_up_values` / `try_apply_sub_values`
   access to the `SharedArgs` (not just a slice). Cleanest: change their `args`
   parameter from `&[Expr]` to `&SharedArgs`, and at the internal sites that
   need a slice (`could_match_args(args)`, `try_apply_direct_default_value(args,
   ..)`, any `args.iter()/len()/index`) read `args.as_ref()` (or rely on
   `Deref`). Do NOT otherwise change their logic.
2. Replace each `let candidate = Expr::call(head.clone(), args.to_vec());` with
   `let candidate = Expr::call_with_shared_args(head.clone(), args.clone());`
   (the `args.clone()` is an Arc bump).
3. In `run_call_dispatch_stage` (eval_dispatch/mod.rs:99-103), pass the shared
   args. Add `CallDispatchState::evaluated_shared_args(&self) -> &SharedArgs`
   (returns the prepared `SharedArgs`, expecting it present like
   `evaluated_args()` does) and pass it to the three stages. The Listable stage
   (line 86-97) and the builtin/Function stages keep using `evaluated_args()`
   (`&[Expr]`) — do not change them.

**Why this is correct (state it in your report):** the candidate is structurally
identical — same head, same argument values — only its argument vector is now a
shared `Arc` rather than a freshly copied `Vec`. Matching, normalization, the
shallow-signature prefilter, and RHS handling all read the candidate the same
way. Nothing mutates the candidate's args in place (matching/normalization
produce new exprs), so sharing the Arc is safe.

## Tests

- Existing down/up/sub-value tests must stay green (they exercise match + RHS).
- Add or confirm: a ground downvalue (`fE[1]=2; fE[1]` → `2`), a pattern
  downvalue (`fP[x_]:=x+1; fP[5]` → `6`), an upvalue (`m/:g[m]:=ok; g[m]` →
  `ok`), a subvalue (`d[1][2]:=12; d[1][2]` → `12`) all still apply.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green. Then the counter gate:

```
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fExact[1]=2; Do[Do[fExact[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fPattern[x_Integer]:=x+1; Do[Do[fPattern[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
```

Report both lines. Gate: exact-hit `shared_args_alloc` ~`1M`→~`0`; pattern-hit
`shared_args_alloc` ~`2M`→~`1M`; `into_vec_clone` stays `0` for both. Then:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_013_micro
```

Paste the `ratios.tsv` rows for `downvalue-exact-hit`, `downvalue-pattern-hit`,
`downvalue-exact-miss`, `downvalue-pattern-miss`, `if-true`, `sameq-integer`.

## rum / Rust pitfalls (read twice)

1. The candidate-build change is `to_vec()` → shared `args.clone()` (Arc bump);
   do not deep-clone anywhere on this path.
2. Changing the param to `&SharedArgs` may need `.as_ref()` at slice-consuming
   call sites; fix by reading the slice, never by `to_vec()`.
3. Do not touch the matcher, normalization, or RHS substitution — only the
   candidate's argument carrier.
4. Measure the RELEASE binary, profiling OFF for ratios.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree.
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench script, `python3`. Remove any
  `.rum_history` you create.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, OR `shared_args_alloc` does not
  drop as specified → STOP and report.
- Minimal diff. Paste real test summary, the two counter lines, and the ratio
  rows; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument (candidate is structurally identical; args carrier
   only).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, the two `rum-profile repr` lines, and the ratio rows.
4. Stop-loss / open questions (empty if none).
