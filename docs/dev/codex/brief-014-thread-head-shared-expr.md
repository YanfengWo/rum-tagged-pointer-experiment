# Brief 014 — A4: thread the call head as SharedExpr (no-change head reuse)

## Goal

The Pair `34x` is the distributed per-call evaluation constant. After ruling out
the matcher, caching/stamp, Module frames, and (via A3) arg allocation, the
concrete reducible eval/dispatch cost that remains is the **head**: every call
does `head.into_owned()` at dispatch entry (the `into_owned_clone` counter shows
~`1M`/`2M` head clones per the microbench loops), and the no-change dispatch tail
rebuilds the node with `Expr::call_with_shared_args(evaluated_head, ..)`,
allocating a fresh `SharedExpr` for the head on every call — including the bulk
of failing Pair conversions, which return the call unchanged.

Mirror Layer A1's args work for the head: thread the head as `SharedExpr`
through `evaluate_call` / `resume_call_after_head`, only converting to owned when
the head actually needs evaluation (a non-symbol / unstable head). For the
common stable-symbol head (`If`, `fExact`, `ORELSEC`, every WL function), the
original head `Arc` flows through untouched and the final node rebuild reuses it
via `Expr::call_shared` — no clone, no head allocation.

## Honest framing (state it in your report)

The profiled "evaluate loop multiplicity" (~2 evaluate passes, ~4 stability
checks per call) was found to be largely INHERENT — the two evaluates are the
call and its argument, the stability checks are on distinct subexpressions, and
the profiled `evaluate.total` self-time is partly instrumentation inflation.
There is no redundant *pass* to delete. This brief instead removes the concrete
head clone + no-change rebuild allocation. Because A3 showed an allocation cut
need not translate to the Pair descent, **the Pair downstream re-measure is the
real test**, not just the microbench.

## Expected, measurable result (the gate)

Per the microbench counter loops (`RUM_PROFILE_EVALUATION_CACHE=1
RUM_PROFILE_REPR=1`):
- `if-true`: `into_owned_clone` ~`1M` → ~`0` (stable head no longer cloned);
  `shared_expr_alloc` stays ~`0` (already ~27).
- `downvalue-exact-hit`: `into_owned_clone` ~`1M` → ~`0`; `shared_expr_alloc`
  ~`1M` → near `0` IF the no-change/candidate path reuses the head Arc (report
  what you observe — exact-hit is a HIT so it also has the A3 candidate head).
- `downvalue-pattern-hit`: `into_owned_clone` ~`2M` → ~`1M` (the call head stops
  cloning; the RHS `Plus` head remains).
- `into_vec_clone` stays `0` (A1/A2 invariant).

Microbench ratios must not regress; `if-true` / `downvalue-*-hit` should improve.
**Then the Pair downstream:** run the bounded `basic-chain-orelsec-split` (rum)
and report whether the per-pass numbers move versus the post-A3 baseline
(manual-loop ~`3.72x`, full TRYCONV[REPEATC] ~`6.48x`). State plainly if it does
not move.

If `into_owned_clone` does not drop for `if-true`, the head is still being
cloned — STOP and report.

## Context pointers

- Read `docs/representation-rewrite-plan.md` and the `2026-06-21` Layer A /
  Pair-descent entries in `PROGRESS.md`. A1 (`bfa7ba4`) is the model: it threaded
  `SharedArgs` the same way for args.
- `crates/rum-runtime/src/runtime/eval_core.rs`:
  - line ~61: `Expr::Call { head, args } => self.evaluate_call(head.into_owned(),
    args)` — `head` here is a `&SharedExpr` (from the `Expr::Call` match);
    `args` is already passed as `SharedArgs`.
  - `evaluate_call(&mut self, head: Expr, args: SharedArgs)` (line ~626):
    `evaluate_atom_fast_path(head)` evaluates the head; `evaluated_head: Expr`
    flows to `resume_call_after_head`.
  - `resume_call_after_head(&mut self, evaluated_head: Expr, args: SharedArgs)`
    (line ~669): `build_call_dispatch_plan(&evaluated_head)`, then
    `resume_call_with_dispatch_plan`, whose final step
    (line ~695) is `Expr::call_with_shared_args(evaluated_head,
    state.into_final_shared_args())`.
  - `EvaluateAtomFastPathKind::classify(session, &expr)` (line ~741) already
    distinguishes `StableSymbol` / `Atom` (head unchanged) from
    `UnstableSymbol` / `Call` (head needs `evaluate`).
- `crates/rum-core/src/expr.rs`: `SharedExpr::new(Expr)` (alloc + counter),
  `SharedExpr::into_owned(self) -> Expr` (counter), `Expr::call_shared(head:
  SharedExpr, args: SharedArgs)`, `Expr::call_with_shared_args`. `SharedExpr`
  derefs/`as_ref`s to `Expr`.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/eval_core.rs` (the head arm at ~61,
    `evaluate_call`, `resume_call_after_head`, the final rebuild),
  - a runtime test module under `crates/rum-runtime/src/runtime/tests/` if you
    add a regression.
- MUST NOT change: dispatch ordering / stage behavior, the dispatch stages'
  `head: &Expr` parameters (they keep borrowing `&Expr` — obtain it from the
  `SharedExpr` via `as_ref()`/Deref), the candidate build (A3), matcher,
  `Expr`/`SharedExpr` public types, serde, any `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

1. line ~61: pass the head shared — `self.evaluate_call(head.clone(), args)`
   where `head.clone()` is a `SharedExpr` Arc bump (NOT `into_owned()`).
2. `evaluate_call(&mut self, head: SharedExpr, args: SharedArgs)`:
   classify the head by borrow (`EvaluateAtomFastPathKind::classify(self,
   head.as_ref())`):
   - `StableSymbol` / `Atom` → the head is unchanged; keep `head` (the
     `SharedExpr`) as `evaluated_head` WITHOUT `into_owned` or `evaluate`.
   - `UnstableSymbol` / `Call` → `let owned = self.evaluate(head.into_owned())?;`
     (preserve the existing `InputRequired` suspend handling — on resume, wrap
     the evaluated value back into a `SharedExpr`), then `evaluated_head =
     SharedExpr::new(owned)`.
   Pass `evaluated_head: SharedExpr` to `resume_call_after_head`.
3. `resume_call_after_head(&mut self, evaluated_head: SharedExpr, args:
   SharedArgs)`: `build_call_dispatch_plan(evaluated_head.as_ref())`; carry the
   `SharedExpr` into the dispatch so the final rebuild can reuse it. The stages
   keep taking `&Expr` — pass `evaluated_head.as_ref()`.
4. Final rebuild (resume_call_with_dispatch_plan, line ~695): use
   `Expr::call_shared(evaluated_head /* SharedExpr */, state.into_final_shared_args())`
   — reuses the head Arc, no `SharedExpr::new`. (You will need to thread the
   `SharedExpr` through `resume_call_with_dispatch_plan`'s `evaluated_head`
   param; it is currently `Expr` — change it to `SharedExpr` and hand
   `.as_ref()` to the stage runner, OR keep an owned `Expr` for the stages and a
   `SharedExpr` for the rebuild, your call, but do not double-allocate.)

**Why this is correct (state it in your report):** the head value is identical;
only its carrier is preserved as a shared `Arc` instead of being unwrapped and
re-wrapped. A stable-symbol head evaluates to itself, so skipping its
`evaluate`/`into_owned` and reusing the Arc yields the same `evaluated_head`. An
unstable/call head still goes through `evaluate` exactly as before. The rebuilt
node is structurally identical.

## Tests

- Ordinary calls unchanged: `If[True,1,2]`→`1`, `f[1,2,3]` stays `f[1, 2, 3]`,
  `fE[1]=2; fE[1]`→`2`, `fP[x_]:=x+1; fP[5]`→`6`.
- A computed (unstable/call) head still evaluates: `(Identity[f])[3]` →
  `f[3]`; `Array[#&, 0]`-style or `Through`/`Composition` head cases in the
  existing suite must stay green.
- Head that needs input resumes correctly (reuse/extend the input-resume test).

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green. Then the counter gate (paste all three):

```
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'Do[Do[If[True,1,2],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fExact[1]=2; Do[Do[fExact[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fPattern[x_Integer]:=x+1; Do[Do[fPattern[1],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
```

Gate: `if-true` `into_owned_clone` ~`1M`→~`0`; the hit shapes drop as described.
Then ratios:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_014_micro
```

Paste rows for `if-true`, `downvalue-exact-hit`, `downvalue-pattern-hit`,
`sameq-integer`, `length-list`, `head-list`. Then the Pair downstream (the real
test):

```
RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance bash \
  scripts/pair-simp-hotspot-probes.sh --mma-hol /Users/fft/Developments/mma-hol-acceptance \
  --only basic-chain-orelsec-split --engine rum --timeout 180 \
  --output-dir /tmp/cdx_brief_014_pair
```

Report the `manual-rule-loop-tryConv` and `full-TRYCONV-REPEATC-depth19` rum
seconds (post-A3 baseline: `1.311s` and `2.273s`). State plainly whether they
move. The mma-hol clone is read-only.

## rum / Rust pitfalls (read twice)

1. Stable-symbol head: do NOT `into_owned`/`evaluate` it; reuse the `SharedExpr`.
   Only unstable/call heads are evaluated.
2. The dispatch stages keep `head: &Expr`; feed them `evaluated_head.as_ref()`.
   Do not change their signatures.
3. Preserve the `InputRequired` suspend/resume contract for an unstable head.
4. Measure the RELEASE binary, profiling OFF for ratios and the Pair probe.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree.
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench + pair-simp scripts, `python3`.
  Remove any `.rum_history` you create.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, OR `if-true` `into_owned_clone`
  does not drop to near zero → STOP and report.
- Minimal diff. Paste real test summary, the three counter lines, the ratio
  rows, and the Pair seconds; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument (head carrier preserved; stable head == itself).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, the three `rum-profile repr` lines, the ratio rows, and the Pair
   seconds (did they move?).
4. Stop-loss / open questions (empty if none).
