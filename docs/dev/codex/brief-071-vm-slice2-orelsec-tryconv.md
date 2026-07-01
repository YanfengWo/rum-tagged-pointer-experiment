# brief-071 — VM slice 2: compiled ORELSEC/TRYCONV Module-combinator bodies

## Goal

Slice 2 of the compiled-body VM (`docs/bytecode-vm-design.md`): compile the two
hottest `HOL`Drule`` conversion combinators — `ORELSEC` and `TRYCONV` — whose
down-value RHS bodies are the exact shape

```
Module[{res}, res = <evalExpr>; If[SameQ[res, <constExpr>], <altExpr>, res]]
```

into the VM (extending slice 1's `vm_rhs_template.rs`). These bodies drive the
1.467x combinator amplification and are inside the 22% `module.resume_initializers`
Pair row (brief-069). The KEY correctness+perf insight: in this shape the Module
local `res` is a PURE TEMPORARY — `Set` once, read as a bare symbol, never escapes
(the `alt` branch does not reference it) — so the VM can treat `res` as a REGISTER
and NOT allocate a Module gensym at all. That is byte-identical (res never escapes)
AND eliminates the Module-frame setup cost (the prize). Behind the existing
`RUM_VM_RHS_TEMPLATE` flag; zero-cost off; byte-identical on. This is the first
slice with a real expected Pair win.

## Context pointers

- Read `docs/bytecode-vm-design.md` (THE spec — read the "Grounding correction"
  section: hot combinators are Module-wrapped, suspension is `InputRequired`-only,
  reuse `suspend_with_frame_lazy` around `Op::Eval`) and brief-070's Outcome (slice
  1 machinery you extend). PROGRESS `## 2026-07-01 brief-069/070`.
- The exact target bodies (in the acceptance clone `Drule.wl`, for reference — do
  NOT edit it):
  - `ORELSEC[c1_,c2_][t_] := Module[{res}, res = tryConv[c1[t]]; If[res===$convFailed, c2[t], res]]`
  - `TRYCONV[c_][t_]      := Module[{res}, res = tryConv[c[t]];   If[res===$convFailed, REFL[t], res]]`
  - `REPEATC` (has `Return` + self-recursion) — **out of scope, must fall back.**
- Key files:
  - `crates/rum-runtime/src/runtime/vm_rhs_template.rs` — slice 1's IR/compiler/
    executor/cache; EXTEND it (add `Op::Eval`, the branch, native `SameQ`, the
    Module-body compile path). Keep slice 1's plain-substitution path working.
  - `crates/rum-runtime/src/runtime/control_flow/branching_loops.rs` `eval_if`
    (`:` truthiness handling, `suspend_with_frame_lazy` usage — MIRROR this
    suspension pattern) and how `If` truthiness / `SameQ` (`===`) resolve
    (`truthiness_symbol`, the SameQ builtin) — REUSE, do not reinvent.
  - `crates/rum-runtime/src/runtime/suspension/core.rs` `suspend_with_frame_lazy` /
    `drive_evaluated_value` — the primitives to wrap each `Op::Eval` so an
    `InputRequired` suspension resumes into the rest of the VM ops (the design's
    approach; no bespoke resume loop).
  - `crates/rum-runtime/src/runtime/lexical.rs` `eval_module` and the
    `module.allocate_fresh_symbols` path — for reference on Module semantics you must
    match; but for this shape you REGISTER-IZE `res` and skip gensym allocation.
  - `crates/rum-runtime/src/runtime/definition_application.rs` — the hook is already
    routed through `substitute_rule_bindings_for_rhs_with_template` (slice 1). But
    slice 1 only SUBSTITUTES; slice 2 needs to also EVALUATE (the body does Set/If/
    Eval). Decide the integration point: the compiled Module body must run at
    application time where the RHS would be substituted THEN evaluated — you may need
    a second entry that both substitutes and evaluates for the compiled-Module case,
    while the plain slice-1 path stays construct-only. Keep the tree-walk fallback
    for every non-matching shape and preserve the suspend/resume contract.
- Acceptance/source clone (READ-ONLY): `/Users/fft/Developments/mma-hol-acceptance`.

## Scope

- Files you MAY modify: `vm_rhs_template.rs` (extend), `definition_application.rs`
  (the compiled-Module integration point, minimal), possibly `mod.rs`. NOTHING else.
- MUST NOT: change `Expr`/`Symbol`/serde; edit any `mma-hol` file; touch
  `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your own report; `.git`.
- **Zero-cost off**: unchanged — the existing `rhs_template_enabled()` gate stays the
  single relaxed load guarding everything.

## Compilable shape (match EXACTLY; fall back otherwise)

Compile a down-value RHS iff it is precisely:
`Module[{L}, L = RHS0; If[SameQ[L, C], ALT, L]]` (or `L===C` sugar) where:
- exactly ONE Module local `L`;
- the body is exactly two statements: `Set[L, RHS0]` then a final
  `If[SameQ[L, C], ALT, L]` (3-arg If; the else-branch is the bare local `L`);
- `L` occurs ONLY as the `Set` LHS, the first `SameQ` arg, and the `If` else-branch
  (bare reads) — i.e., `L` does NOT appear in `RHS0`, `C`, or `ALT` (proves it does
  not escape → register-izable). If `L` appears anywhere else, FALL BACK.
Anything else (REPEATC's `Return`/recursion, THENC's multi-Set, any deviation) →
fall back to the existing tree-walk. Be conservative; a missed compile is safe, a
wrong compile is a bug.

## Execution semantics (byte-identical to Module+Set+If)

With bindings applied (`c1`,`c2`/`c`,`t` → their bound values):
1. `v = Op::Eval(RHS0[bindings])` — via `evaluate`; suspension handled by
   `suspend_with_frame_lazy` resuming at step 2.
2. `same = SameQ(v, C[bindings])` — native structural SameQ (byte-identical to
   `===`; `C` is a constant like `$convFailed`).
3. if `same`: `result = Op::Eval(ALT[bindings])` (only NOW is ALT constructed +
   evaluated — the win: the untaken branch is never built when not taken); else:
   `result = v`.
4. return `result`.
No gensym for `L` is created (it never escapes). This must equal what
`Module[{res}, res=RHS0; If[res===C, ALT, res]]` produces via the tree-walk.

## rum / Rust pitfalls (read twice)

1. **Suspension via the existing primitives.** Each `Op::Eval` may return
   `Err(InputRequired)` (only trigger; never in the hot descent, but the workspace
   gate ON exercises interactive tests). Wrap each `Op::Eval` with
   `suspend_with_frame_lazy` so the continuation (steps 2-4) resumes correctly — do
   NOT invent a new resume loop, and do NOT bail-after-eval (side effects already
   ran). Mirror `eval_if`.
2. **`res` register-ization is only valid because `L` does not escape** — the
   compile-time occurrence check above is the correctness guard. Do not relax it.
3. **Byte-identical or it's a bug** (not a perf tradeoff): SameQ semantics, If
   truthiness, evaluation ORDER (RHS0 before the branch; ALT only when taken), and
   `Nothing`/`Sequence` handling must match the tree-walk exactly.
4. **Zero-cost off**; **no new `unsafe`**; **minimal diff**.
5. **Stop-loss**: if byte-identical can't be met, NARROW the shape (fall back more)
   until green, or STOP and report. Do not ship a divergence.

## Verification (MANDATORY — probe budget applies)

```
cargo fmt --check
cargo test --workspace                          # OFF baseline green
RUM_VM_RHS_TEMPLATE=1 cargo test --workspace    # ON must be EQUALLY green (byte-identical proof)
```

Add NEW unit tests that define ORELSEC/TRYCONV-shape down-values and assert the VM
result equals the tree-walk (both branches taken/not-taken; the debug differential
also fires). Then a TARGETED combinator differential + the Pair delta (PROBE
BUDGET: best-of-2, ≤6 total Pair runs, full env up front):

```
# drule_tests.wl directly exercises ORELSEC/TRYCONV — run OFF vs ON, assert identical pass/fail:
RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance <run tests/drule_tests.wl OFF>
RUM_VM_RHS_TEMPLATE=1 RUM_MMA_HOL_ROOT=... <run tests/drule_tests.wl ON>   # identical results
# Pair descent OFF vs ON best-of-2 (result must stay is_thm=True changed=False):
<runtime-profile-basic-only-pair4 OFF x2 and ON x2>
```

Paste: both `cargo test` summaries; the new tests; the drule_tests OFF-vs-ON identical
result lines; the Pair OFF-vs-ON best-of-2 (correct `simpFixpointConv` `seconds=`
metric — NOT the `preheat-basic` line) + ratio + is_thm lines; and the coverage
counter (ORELSEC/TRYCONV bodies compiled vs fallback).

## Hard rules

- No `git commit`/branch/push. Leave changes in the tree for Claude to gate.
- No files outside Scope; never edit `mma-hol`; no `Expr`/serde change; no new `unsafe`.
- OFF path zero-cost (unchanged gate). Probe budget: best-of-2, ≤6 Pair runs, state
  the count, full env up front; if a number looks off, REPORT — do not silently
  re-measure everything.
- Stop-loss: byte-identical unmet → narrow/fall back or STOP; same failure twice → STOP.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — per file/line, the new ops + the Module-combinator compile path +
   the exact compilable-shape boundary; confirm no `Expr`/serde change, no gensym
   allocated for `res`.
2. Verification — both `cargo test --workspace` summaries, the new tests, drule_tests
   OFF-vs-ON identical, Pair OFF-vs-ON best-of-2 + ratio + is_thm, coverage counter.
3. Finding — the Pair delta (kept or not), how many ORELSEC/TRYCONV bodies compiled,
   whether the Eval+branch+suspension machinery is proven for slice 3 (REPEATC).
4. Stop-loss / open questions — what fell back, anything unsure (esp. any suspension
   or evaluation-order edge you were unsure about).

---

## Outcome (Claude gate, 2026-07-01) — DONE + KEEP. ~2x Pair, byte-identical, flag OFF-default.

Committed flag-OFF-default. Codex compiled the exact `Module[{res}, res=eval;
If[SameQ[res,C],alt,res]]` shape (ORELSEC/TRYCONV), register-izing the non-escaping
`res` (no gensym), `Op::Eval` + native SameQ + branch, suspension via
`suspend_with_frame_lazy`. Scope clean (`vm_rhs_template.rs` + `definition_application.rs`).

- **Perf (Claude repro, best-of-2, clean release):** Pair `simpFixpointConv` OFF
  55.67/58.89s, ON 27.44/29.35s → **ON/OFF best = 0.493 (~2.03x faster)**, non-overlapping.
  Bigger than the ~modest expectation because ORELSEC/TRYCONV wrap ~every conversion
  in the descent, so eliminating their Module frame + per-call Set/SameQ/If dispatches
  compounds over the deep descent.
- **Correctness (strong; one complete full-tower pass-count deferred):**
  `cargo test --workspace` 719/0 OFF and ON (debug `debug_assert_eq!` template-vs-tree
  on EVERY application); `drule_tests` OFF vs ON identical 50/0 (the actual
  ORELSEC/TRYCONV mma-hol tests, complete, on-target); multiple run_fast attempts
  byte-identical over their common prefix (arith/set/… hundreds of assertions), no
  divergence/panic, coverage confirms the 2 combinators compiled; Pair
  `is_thm=True changed=False`; +7 new unit tests. `res` never escapes and the shape
  NEVER matches `defineKernel`'s kernel-state Module (trust boundary untouched).
- **Could NOT obtain a single COMPLETE full-tower OFF==ON pass-count** via run_fast:
  rum test files are slow (`set`+`setProve` is heavy automated proving, not "light"),
  rum-cli does not forward run_fast's subset args, and OFF is ~2x slower so it times
  out sooner — 4 mis-bounded timeout runs (a process failure, see the TEMPLATE
  timeout rule added this session). Deferred to the pre-default-flip bucket, to be
  done via the snapshot-chain method (build cp00→cp42 with the new HEAD, then restore
  a checkpoint + load one file — the reliable targeted path), NOT run_fast/run_all.

**Verdict: KEEP, flag-OFF-default.** Banks the Eval+control+suspension machinery
(foundation for slice 3 = REPEATC) and a ~2x activatable Pair win. Before flipping
the default ON (with slice 1's deferred items): the complete full-tower OFF==ON
(ideally an ON-vs-OFF snapshot byte-diff) + a Strict-mode check + remove the
eprintln coverage counter + per-Session cache.
