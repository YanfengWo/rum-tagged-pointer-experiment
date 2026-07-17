# WL15/WFR hold-semantics design (wl15/wfr pass, batch 2)

Status: design doc for batch 2 of the wl15/wfr pass (TODO §ACTIVE PLAN
item 5). Probes re-run 2026-07-17 against the current tree (post
unit-9 fixes 215-222, post brief-254); the kernel-b appendix facts
below are re-verified, not trusted from the audit date. Implementation
follows the user rulings recorded in §6.

## 1. Inputs

- `docs/dev/usability-pass/kernel-b-findings.md` §Hold-semantics design
  input (hard-coded-hold trace; matching/replacement facts; the unit-9
  half of the SubValues hold question).
- `docs/dev/wl15-core-survey.md` §1 (SubValuesHoldAll facts).
- `docs/dev/wfr-core-survey.md` §ReplaceAllUnheld (idea-level only).
- `docs/dev/usability-pass.md` close-out: ConformSchema/SchemaErrors
  operator forms DEFERRED (HoldRest position mismatch).
- Standing directives: compat enhancement (WL names keep WL-conformant
  defaults); no default-path cost; A/B gate for call-planning changes.

## 2. Probe results (2026-07-17, current tree)

1. **`HoldAll` Sequence splice — the appendix fact is STALE, resolved
   in our favor.** `SetAttributes[f, HoldAll]; f[Sequence[1,2]]` →
   `f[1, 2]`: ordinary HoldAll now flattens Sequence (unit-9 ruling),
   which is exactly WL15's stated SubValuesHoldAll behavior ("Sequence
   still flattens"). The old A2 divergence question is GONE — the new
   attribute mirrors current HoldAll and both match WL15.
2. **`ReplaceAll` rewrites inside held parts — confirmed current.**
   `Hold[1+1] /. Plus -> Times` → `Hold[Times[1, 1]]`.
   `ReplaceAllUnheld`'s motivation is intact.
3. **`Unevaluated` no-consumer restore works** (unit-9 headline fix):
   `f[Unevaluated[1+1]]` → `f[Unevaluated[Plus[1, 1]]]`.
4. **Attributes confirmed**: ConformSchema/SchemaErrors
   `{HoldRest, Protected}`; Condition `{}`; AppendTo/AssociateTo
   `{HoldFirst, Protected}`.
5. **Subvalue application still evaluates everything** (E9 unchanged):
   `op[a_][x_] := HoldComplete[a, x]` with side-effecting args yields
   `HoldComplete[1, 2]`, both counters fired — the gap the attribute
   addresses is still there.
6. `ReleaseHold` stays one-layer (`ReleaseHold[Hold[Hold[1+1]]]` →
   `Hold[Plus[1, 1]]`).

Probe hygiene note: rum-cli probes MUST feed stdin (`printf '' | …`);
two spurious "hangs" in this pass were the CLI filter mode waiting on
an open stdin pipe, not evaluator bugs (the stdlib-line gate lesson,
re-confirmed).

## 3. Candidate A — `SubValuesHoldAll`

WL15 semantics (survey facts): in `f[…][s…]`, subvalue-position args
stay unevaluated; mirrors HoldAll not HoldAllComplete (Sequence
flattens, `Unevaluated` stripped, upvalues of held args fire,
`Evaluate` re-enables per argument).

rum mechanism today (verified): for `f[a][b]` the outer call plan
looks up attributes ONLY for a literal symbol head; a compound head
gets the empty set (`eval_call_prepare.rs:426-447`); the base symbol
is consulted only for SubValues-stage existence
(`eval_dispatch/mod.rs:15-21`).

Proposed mechanism: when the outer head is a Call, resolve the BASE
symbol (the same walk the SubValues-stage check performs) and consult
its attributes for `SubValuesHoldAll`; if present, plan outer args
with a HoldAll-shaped hold plan (Evaluate pierce on, Unevaluated
strip on, Sequence splice on — all matching current HoldAll).

- **A1 name**: adopt the WL name `SubValuesHoldAll` (it IS WL surface;
  the compat directive keeps WL names WL-conformant). RECOMMEND: yes.
- **A2 splice rule**: resolved by probe 1 — no decision needed.
- **A3 hot-path cost**: call-planning territory. Mitigation: compound-
  head calls already branch through the SubValues-stage existence
  check; piggyback the attribute consult there so symbol-head calls
  (the hot tier) pay nothing. HARD GATE: microbench A/B, downvalue/
  subvalue-tier labels + same-tier neighbors.
- **A4 attribute plumbing**: new `Attribute` variant → exhaustive-match
  ripple (metadata.rs, parse/print, catalog mirror, SetAttributes
  surface). Grep every exhaustive match BEFORE writing the brief; list
  all as MECHANICAL ONLY in MAY (the brief-232 lesson).

## 4. Candidate B — operator forms for hold-y heads

Two distinct sub-problems; do not conflate:

- **B1 mutation-pipeline operator forms (AppendTo family)** — WL15's
  own motivation. `AppendTo[val][x]`: `x` sits in subvalue position →
  held by candidate A directly. Scope decision: which HoldFirst
  compound-assignment heads get the attribute + registered operator
  form (AppendTo/PrependTo/AssociateTo/KeyDropFrom/…; exact list
  enumerated at brief time from the catalog). RECOMMEND: yes for the
  family — it is the WL15-conformant use.
- **B2 ConformSchema/SchemaErrors operator forms** — candidate A does
  NOT solve these: the schema needing the hold sits at position 1 of
  the CAPTURE call `ConformSchema[schema]` (a direct call), not in a
  subvalue position; attributes cannot be arity-dependent. Options:
  1. Special-dispatch the head so the unary capture stores the schema
     raw (precedent: Rule/RuleDelayed are special); the binary direct
     call's HoldRest semantics are then handler-owned and must be
     re-verified byte-identical.
  2. Ship the operator form with an EVALUATED schema and document the
     divergence — REJECTED once already ("shipping on a guess").
  3. Keep DEFERRED (no operator form).
  RECOMMEND: option 1 if the special-path diff stays small at probe
  time; otherwise 3. Option 2 stays rejected.

## 5. Candidate C — `ReplaceAllUnheld`

Semantics (WFR idea, folk contract to pin in the brief): apply rules
only to parts that WOULD evaluate — respect Hold* attributes of
(evaluated) heads and held subexpressions; `Evaluate` wrappers inside
held args re-enable; `Unevaluated` is masked; HoldAllComplete masks
args and disables re-enabling inside.

- **C1 form**: separate builtin (WFR generic name) vs a `ReplaceAll`
  option. RECOMMEND separate builtin: an option check on every
  ReplaceAll call is default-path cost on a hot label (hard-line
  violation); a sibling name costs nothing and matches the demand
  signal.
- **C2 traversal contract**: head positions always eligible;
  HoldAll/HoldFirst/HoldRest of the (evaluated) head mask the
  corresponding argument subtrees; the mask rules above.
- **C3 shared code**: reuse the rewrite applicator + matcher; the NEW
  traversal lives beside replace_all.rs, never inside its hot loop.
- **C4 out of scope**: the RuleDelayed consumer-divergence defect
  (delayed RHS firing differs between ReplaceAll and ReplacePart/
  ReplaceAt) is a separate pre-existing issue — recorded, not fixed
  here.

## 6. Decision points and rulings

| # | Question | Recommendation | Ruling (user, 2026-07-17) |
|---|---|---|---|
| A1 | Adopt WL name `SubValuesHoldAll`? | yes | **ADOPT the WL name** |
| B1 | AppendTo-family attribute + operator forms? | yes | **FULL FAMILY** (HoldFirst compound-assignment heads; list enumerated at brief time) |
| B2 | ConformSchema/SchemaErrors: special-dispatch capture vs stay deferred | option 1 if small, else 3 | **OPTION 1 with stop-loss**: special-dispatch the unary capture; if the diff bloats, stop-loss back to DEFERRED |
| C1 | `ReplaceAllUnheld` as separate builtin? | yes | **SEPARATE BUILTIN** (never an option on ReplaceAll) |

## 7. Batch split (implementation, post-rulings)

- **2a** mechanism: `SubValuesHoldAll` attribute + planner change +
  mechanical attribute ripple + microbench A/B hard gate.
- **2b** operator forms: AppendTo-family adoption (per B1) +
  ConformSchema/SchemaErrors (per B2). Depends on 2a.
- **2c** `ReplaceAllUnheld`: independent of 2a/2b (read-side
  traversal); briefed separately, dispatched serially.
