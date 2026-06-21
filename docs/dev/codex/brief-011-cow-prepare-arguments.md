# Brief 011 — Layer A2: copy-on-write argument preparation (implementation)

## Goal

Layer A2 of the representation rewrite. A1 (`bfa7ba4`) threaded `SharedArgs`
through dispatch, but `prepare_arguments` still unconditionally materializes an
owned `Vec` (via `unwrap_shared_args`), so the `into_vec` clone just moved from
the dispatch entry into the `PrepareArguments` stage — still ~`1M` clones for
`if-true`. The fix: make argument preparation **copy-on-write**. Evaluate every
argument as today, but when *no argument actually changes* (the steady-state
case for already-evaluated literals and held args), return the **same**
`SharedArgs` Arc instead of allocating a new vector. Then the held / builtin /
downvalue paths read args by borrow and never deep-clone.

`If[True,1,2]` (arg0 `True`→`True`, args held), `fExact[1]` (`1`→`1`),
`fPattern[1]` (`1`→`1`): in every case prepared args == source args, so COW
returns the input Arc and `into_vec` is never called.

## Expected, measurable result (this is the gate)

Per 1M calls, `into_vec_clone` (and `into_vec_total`):
- `if-true`: ~`1.00M` → ~`0`.
- `downvalue-exact-hit`: ~`1.00M` → ~`0`.
- `downvalue-pattern-hit`: ~`2.00M` → near `0` (the RHS-rebuild allocations —
  `shared_*_alloc` — stay; those are flaw 2 / Layer B, NOT this brief).

Microbench ratios for `if-true`, `downvalue-exact-hit`,
`downvalue-pattern-hit`, `sameq-integer`, `length-list`, `head-list` should
improve or hold; none may regress beyond cross-run noise. If `into_vec_clone`
does NOT collapse for `if-true`/`exact-hit`, the COW is not firing — STOP and
report (do not paper over with a partial win).

## Why this is correct (the safety argument — state it in your report)

Every argument is still evaluated exactly once, in order, with identical side
effects — the COW logic changes only the *carrier*, never whether/when an arg
is evaluated. The input `SharedArgs` is reused **only if** every prepared arg is
structurally equal to its source arg AND no sequence splice changed the arity
AND attribute normalization (`Flat`/`Orderless`) is a no-op. In that case the
reused vector is byte-for-byte the vector we would have built, so reuse is
indistinguishable from materialization. The comparison can never cause a wrong
result: if it misjudges "unchanged," storing the (equal) original is identical;
if an arg truly changed, the comparison sees the inequality and materializes.

## Context pointers

- Read `docs/representation-rewrite-plan.md` (Layer A) and the Phase-0 baseline
  entry in `PROGRESS.md`.
- `crates/rum-runtime/src/runtime/eval_call_prepare.rs`:
  - `prepare_arguments_for_call` (line ~79) → `resume_prepared_call_arguments`
    (line ~95) builds `evaluated_args: Vec` by evaluating each arg via
    `evaluate_atom_fast_path`. The `InputRequired` arm (lines ~123-145)
    suspends with `resume_call_after_argument` as the continuation.
  - `resume_call_after_argument` (line ~165) and `resume_call_from_prepared_args`
    (line ~192) are the post-suspension continuation; they work on `Vec` and
    build the `CallDispatchState`.
  - `normalize_call_args_by_attributes` (line ~234): for a head WITHOUT `Flat`
    it returns `args` unchanged (line ~248, no clone), and without `Orderless`
    it does not sort. So for non-Flat/non-Orderless heads it is already a pure
    passthrough.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:77-83`: the
  `PrepareArguments` stage. Currently `unwrap_shared_args(mem::take(raw_args))`
  → `prepare_arguments_for_call(Vec)` → `normalize` → `state.evaluated_args =
  Some(Vec)`.
- `crates/rum-runtime/src/runtime/session_state.rs:80-101`: `CallDispatchState`
  with `evaluated_args: Option<Vec<Expr>>`, the `evaluated_args()` accessor
  (returns `&[Expr]`), and `into_final_args`.
- Constructors in `crates/rum-core/src/expr.rs`: `Expr::call_with_shared_args`,
  `Expr::call_shared`, `Expr::unwrap_shared_args`, `SharedArgs::new`,
  `SharedArgs::default`. `SharedArgs` impls `AsRef<[Expr]>` / `Deref<Target=Vec>`.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/eval_call_prepare.rs` (COW prepare + a
    shared-args normalize variant),
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` (the
    `PrepareArguments` stage + the Listable rebuild site at line ~94-97),
  - `crates/rum-runtime/src/runtime/session_state.rs` (`evaluated_args` field
    type + `evaluated_args()` accessor + `into_final_args`),
  - `crates/rum-runtime/src/runtime/eval_core.rs` (final rebuild at line ~695),
  - a runtime test module under `crates/rum-runtime/src/runtime/tests/` if you
    add regressions.
- MUST NOT change: argument-evaluation order/semantics, the `Flat`/`Orderless`
  normalization results, the suspension/resume contract, the existing
  `Vec`-taking `normalize_call_args_by_attributes` signature (its OTHER callers
  — `arithmetic.rs`, `resolution.rs` — must keep compiling unchanged; add a new
  shared variant instead), `Expr`/`SharedArgs` public types, serde, any
  `mma-hol` clone, `.git`.

## Design (implement this shape; idiomatic Rust is yours)

1. **`CallDispatchState.evaluated_args: Option<SharedArgs>`** (was
   `Option<Vec<Expr>>`). Update:
   - `evaluated_args(&self) -> &[Expr]`: `self.evaluated_args.as_ref().expect(..).as_ref()`.
   - `into_final_args(self) -> Vec<Expr>`: prefer the evaluated shared args, else
     the raw; both via `Expr::unwrap_shared_args`. (Or add an
     `into_final_shared_args(self) -> SharedArgs` and rebuild the node with
     `Expr::call_with_shared_args` at `eval_core.rs:695` to avoid a final
     re-wrap — preferred.)
   - `resume_call_from_prepared_args` (eval_call_prepare.rs:198) builds the state
     with `evaluated_args: Some(<SharedArgs>)` now.

2. **COW `prepare_arguments_for_call(head, attributes, args: SharedArgs) ->
   Result<SharedArgs, EvalError>`**. Keep `resume_prepared_call_arguments` /
   `resume_call_after_argument` (the Vec-based post-suspension continuation) as
   they are — they only run after a suspension has already materialized. Add the
   COW entry walk:
   - `let mut built: Option<Vec<Expr>> = None;` (lazy accumulator; `None` =
     still reusing `args`).
   - For `i in 0..args.len()`: compute `plan =
     plan_call_argument_preparation(attributes, i, args[i].clone())` (the clone
     is an Arc bump). Produce `prepared: Expr` exactly as the current loop does
     (`evaluate_atom_fast_path` when `plan.should_evaluate`, else `plan.expr`).
   - **Suspension:** if `evaluate_atom_fast_path` returns
     `Err(InputRequired{..})`, materialize the prefix
     (`built.take().unwrap_or_else(|| args[..i].to_vec())`), collect the
     remaining source args (`args[i+1..].to_vec()`), and suspend with the SAME
     `resume_call_after_argument` continuation the current code uses (it returns
     the final call `Expr` and drives `resume_call_from_prepared_args`). Return
     the propagated `Err` (COW never returns `Ok` on the suspend path).
   - **Change detection:** let `splice = plan.allow_sequence_splice &&
     !attributes.sequence_hold() && <prepared is a Sequence[..] call>` (reuse
     the existing splice predicate the current `append_prepared_argument` path
     relies on — match how `splice_sequence_argument` decides). `changed =
     splice || prepared != args[i]` (structural `Expr` `PartialEq`; cheap for
     atoms).
     - if `built.is_none() && !changed`: continue (still COW).
     - if `built.is_none() && changed`: `let mut v = args[..i].to_vec();
       append_prepared_argument(&mut v, prepared, plan.allow_sequence_splice,
       sequence_hold); built = Some(v);`
     - if `built.is_some()`: `append_prepared_argument(built.as_mut().unwrap(),
       ..)`.
   - After the loop: `Ok(built.map(SharedArgs::new).unwrap_or(args))`.

3. **COW normalize.** Add `normalize_call_args_shared(head, args: SharedArgs,
   attributes) -> SharedArgs`: if `attributes` contains neither `Flat` nor
   `Orderless`, return `args` unchanged (no clone). Otherwise materialize
   (`Expr::unwrap_shared_args`), call the existing
   `normalize_call_args_by_attributes`, and re-wrap with `SharedArgs::new`. Do
   NOT change the existing `Vec` normalize fn.

4. **`PrepareArguments` stage** (eval_dispatch/mod.rs:77-84): pass the shared
   args straight in — `let prepared = self.prepare_arguments_for_call(head.clone(),
   attributes, std::mem::take(&mut state.raw_args))?; state.evaluated_args =
   Some(self.normalize_call_args_shared(head, prepared, attributes));`. No
   `unwrap_shared_args` here anymore.

5. **Listable rebuild** (eval_dispatch/mod.rs:94-97): keep correct; `to_vec()`
   there is fine (it is the arity-mismatch terminal rebuild, rare) — or use
   `Expr::call_with_shared_args` with the shared evaluated args. Your call;
   minimal.

## Tests (add focused regressions)

- Arg that DOES change still rebuilds correctly: `g[1+1]` → `g[2]` (arg
  evaluates, COW materializes); `h[x]` with `x = 5` → `h[5]`.
- Held args unchanged: `Hold[1+1]` stays `Hold[1 + 1]`; `If[True, a, b]` → `a`.
- Sequence splice still works: `f[Sequence[1, 2], 3]` → `f[1, 2, 3]`.
- `Flat`/`Orderless` still normalize: with `SetAttributes[ff, Orderless];
  ff[2, 1]` → `ff[1, 2]`; a `Flat` head still flattens.
- An arg evaluation that suspends (InputString-style) still resumes and
  completes (reuse/extend the existing input-resume regression).

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-core
cargo test -p rum-runtime
```

All green. Then the counter gate:

```
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'Do[Do[If[True,1,2],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fExact[1]=2; Do[Do[fExact[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fPattern[x_Integer]:=x+1; Do[Do[fPattern[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
```

Report all three lines. Gate: `if-true` and `exact-hit` `into_vec_clone` ~`0`;
`pattern-hit` `into_vec_clone` near `0` (its `shared_args_alloc` /
`shared_expr_alloc` stay ~`2M` — that is the RHS rebuild, expected). Then:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_011_micro
```

Paste the `ratios.tsv` rows for `if-true`, `downvalue-exact-hit`,
`downvalue-pattern-hit`, `sameq-integer`, `length-list`, `head-list`.

## rum / Rust pitfalls (read twice)

1. EVERY arg must still be evaluated once, in order. The COW accumulator decides
   only carrier reuse, never evaluation.
2. The structural `prepared != args[i]` compare must be on the FINAL prepared
   arg (post-unwrap/eval), not the raw plan input.
3. `evaluated_args` MUST become `SharedArgs`, or you will COW in prepare then
   immediately re-clone on store — defeating the change.
4. Do not alter the existing `Vec`-based `normalize_call_args_by_attributes` or
   the suspension continuation; add alongside.
5. Measure the RELEASE binary, profiling OFF for ratios.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree.
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench script, `python3`. Remove any
  `.rum_history` you create.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, OR `into_vec_clone` does not
  collapse for `if-true`/`exact-hit` → STOP and report.
- Minimal diff. Paste real test summaries, the three counter lines, and the
  ratio rows; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The safety argument (every arg evaluated; Arc reused only when all equal).
3. Verification — verbatim `cargo fmt --check`, both `cargo test` summaries, the
   three `rum-profile repr` lines, and the ratio rows.
4. Stop-loss / open questions (empty if none).
