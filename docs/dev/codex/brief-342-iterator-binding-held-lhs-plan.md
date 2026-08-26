# brief-342 — iterator bindings invisible inside a held assignment lhs (design only)

## Goal

`Do[f[i] = i, {i, 1, 3}]` does not define `f[1]`, `f[2]`, `f[3]`. It installs a
single definition `HoldPattern[f[i]] :> 3` — the iterator variable is
substituted on the rhs but stays symbolic on the lhs — and emits no message.
The same shape fails for `SetDelayed`, `Sum`, and `Table`, while the same
assignment through an ordinary global symbol, a `Module`/`With` local, a `For`
loop variable, a `Part` position, or an association key all work correctly.

**Scope: NONE — no source change.** This brief asks for a diagnosis and a
step-by-step repair plan with drafted follow-up implementation briefs, every
claim carrying `path:line` evidence. Do not change source. Throwaway probes
with the release `rum-cli` are in scope; leave no probe files in the
repository, and feed every probe `stdin < /dev/null` (the CLI waits for input
otherwise).

This is a correctness defect first and a design question second:
`Do[memo[i] = expensive[i], {i, 1, n}]` is the standard Wolfram-language table
fill and memoization idiom, and rum currently accepts it silently while doing
something else.

## Measured evidence (director's probes, 2026-08-26, release build at `c18b01f9`)

Run verbatim through `./target/release/rum-cli <file> < /dev/null`. Every line
below is real output, not an expectation.

Iterator binding *is* a real binding — it is visible to ordinary evaluation:

```
Do[Print[{"j=", j, " ValueQ[j]=", ValueQ[j], " OwnValues=", OwnValues[j]}], {j, 1, 2}]
{"j=", 1, " ValueQ[j]=", True, " OwnValues=", {HoldPattern[j] :> 1}}
{"j=", 2, " ValueQ[j]=", True, " OwnValues=", {HoldPattern[j] :> 2}}
```

Reads through the iterator variable are correct:

```
d[1] = "one"; d[2] = "two"
Do[Print[{"read d[j]=", d[j]}], {j, 1, 2}]
{"read d[j]=", "one"}
{"read d[j]=", "two"}
```

The broken cell — the iterator variable inside a held assignment lhs:

```
Do[a1[j] = j, {j, 1, 3}];        DownValues[a1] -> {HoldPattern[a1[j]] :> 3}
Do[a2[j] := j, {j, 1, 3}];       DownValues[a2] -> {HoldPattern[a2[j]] :> j}
Sum[b1[j] = j, {j, 1, 3}];       DownValues[b1] -> {HoldPattern[b1[j]] :> 3}
Table[b2[j] = j, {j, 1, 3}];     DownValues[b2] -> {HoldPattern[b2[j]] :> 3}
Table[h[m] = m*100, {m, 1, 3}];  DownValues[h]  -> {HoldPattern[h[m]] :> 300}
```

The compound form fails loudly instead of silently — the same root cause with
a different surface:

```
a3[1] = 0; a3[2] = 0
Do[a3[j] += 5, {j, 1, 2}]
! AddTo::rvalue: a3[j] is not a variable with a value, so its value cannot be changed.
! AddTo::rvalue: a3[j] is not a variable with a value, so its value cannot be changed.
DownValues[a3] -> {HoldPattern[a3[1]] :> 0, HoldPattern[a3[2]] :> 0}
```

Everything adjacent is correct and MUST STAY correct — these are the controls
for any repair:

```
i = 3; f[i] = 99                 DownValues[f] -> {HoldPattern[f[3]] :> 99}   (plain global)
w = 4; Do[r[w] = 1, {1}]         DownValues[r] -> {HoldPattern[r[4]] :> 1}    (global inside Do)
For[n = 1, n <= 3, n = n + 1, p[n] = n*7]
                                 DownValues[p] -> {p[1] :> 7, p[2] :> 14, p[3] :> 21}
Module[{u}, u = 5; s2[u] = 1]    DownValues[s2] -> {HoldPattern[s2[5]] :> 1}
With[{y = 6}, t2[y] = 1]         DownValues[t2] -> {HoldPattern[t2[6]] :> 1}
q = {1, 2, 3}; Do[q[[j]] = j*11, {j, 1, 3}]      q -> {11, 22, 33}
as = <||>;    Do[as[j] = j, {j, 1, 3}]           as -> <|1 -> 1, 2 -> 2, 3 -> 3|>
Do[b3 = j, {j, 1, 3}]                            b3 -> 3
```

So the failing cell is narrow: **an iterator-bound variable in the argument
position of a `DownValue` assignment lhs**. Part assignment, association-key
assignment, own-value assignment, and every other binder are fine. This is the
same oddly-shaped gap as `lst[[i]] = v` (briefs 297-300): the neighbours all
work, so nothing caught it.

## Entry points (starting evidence only — verify and extend)

- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:8`
  — `eval_do`; `:21` `eval_table`; the iteration spec and fast path live beside
  it in `spec.rs` and `table_fast_path.rs`.
- `crates/rum-runtime/src/runtime/definitions/definitions.rs:14` — `eval_set`
  clones the lhs unevaluated and hands it to `resume_set_with_rhs`; find where
  the lhs arguments *do* get resolved (the plain-global case proves such a step
  exists) and why the iterator binding is not visible there.
- `crates/rum-runtime/src/runtime/scoping/` — `Module`/`With`/`Block` binding,
  which works in this position; the difference between that mechanism and the
  iterator mechanism is the likely root cause.

## What the plan must contain

1. **The mechanism, proved.** Where the iterator value lives, which lookup the
   lhs-argument resolution uses, and why one sees the binding and the other
   does not — with `path:line` evidence and a probe that reaches the code you
   name. State plainly whether the current behavior is deliberate (a guard,
   a hold rule, a performance shortcut) or an omission.

2. **The full matrix, enumerated, not sampled.** Rows = every iterating or
   binding construct that introduces a variable (`Do`, `Table`, `Sum`,
   `Product`, `NestWhile*`/`Fold*` style callbacks if they bind, `For`,
   `Module`, `With`, `Block`, and any lazy `LazyTable`/`LazyMap` analogue).
   Columns = every assignment head whose lhs is held (`Set`, `SetDelayed`,
   `TagSet`, `TagSetDelayed`, `UpSet`, `UpSetDelayed`, all eleven compound
   assignments, `AppendTo`/`PrependTo`, `AssociateTo`, `Part` assignment,
   association-key assignment). Report the observed result for every cell, and
   mark which cells are correct today. A repair that fixes one row and leaves
   its siblings broken is not acceptable, and a repair that changes a
   currently-correct cell is a regression. (Director's standing lesson: this
   class of change is where an outer x inner matrix has to be enumerated.)

3. **The intended semantics, argued.** What each broken cell should do,
   grounded in rum's own documented behavior (`docs/reference/builtins.md`,
   `docs/guide/`) and in general Wolfram-language scoping semantics — not in
   any other engine's observed output (Testing Policy). Call out anything that
   is a genuine semantic decision for the director rather than an obvious fix,
   including whether `Do[a2[j] := j, ...]` should capture the iterator value or
   stay delayed on the symbol.

4. **A step-by-step repair plan**, smallest safe steps first, each step naming
   files/functions, the invariant it must preserve, its gate tests, and its
   ripple/call-site inventory. Identify explicitly whether the fix belongs at
   the binder (iterator installs a real scoped value) or at the assignment
   (lhs-argument resolution consults the binding), and why the other layer is
   the wrong place — fixing at the wrong layer is a known failure mode here.

5. **Cost analysis.** `Do`/`Table` bodies and `Set` are hot evaluator paths.
   Say what each proposed step costs on the default path, and which steps need
   the microbench A/B gate. A repair that adds a per-iteration allocation or an
   extra lookup per assignment is rejected on those grounds alone.

6. **Existing-test survey.** Which current tests assert the present behavior
   (they may encode the defect), which would catch the repair, and which new
   tests are needed. Quote the assertions you find.

7. **Drafted follow-up implementation briefs**, in `TEMPLATE.md` shape, one per
   step — usable drafts, not step summaries. They are raw material for the
   director's own briefs.

8. **Open questions for the director**, each with your recommendation and the
   evidence behind it.

## Scope

- Files you MAY modify: **NONE**. This is design-only. Do not create files in
  the repository; put probes in `/tmp` and delete nothing that is not yours.
- Files/areas you MUST NOT touch: everything, including `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`. Your report is the
  deliverable.

## rum / Rust pitfalls (read twice)

1. **Every claim needs a probe that reaches the code you are claiming about.**
   Two of this sweep's survey findings were artifacts of probes that never
   reached the named path.
2. **Every probe needs a liveness assertion** — something in the output that
   proves the work actually happened (a printed value, a `DownValues` listing,
   a count), not just a time or an absence of error. A probe that measures a
   failing path looks perfectly well-behaved.
3. **Iterator variables are not the only binder** — check whether the same lhs
   resolution is used by pattern-bound variables in a rule rhs
   (`f[x_] := (g[x] = x)`), which is the same shape from a different binder.
4. **Do not propose relaxing hold semantics wholesale.** The lhs of `Set` is
   held for good reasons; the target is that the iterator binding is visible to
   whatever resolution already handles the plain-global case, not that the lhs
   becomes evaluated.
5. **Suspension safety.** `Do`/`Table` bodies and assignment paths are
   suspension-capable (`suspend_with_frame*`); any proposed change must
   preserve the continuation shape.
6. **Stop-loss is real.** If the mechanism does not resolve after two serious
   attempts, report what you established and where the trail ends.

## Verification (MANDATORY)

No source change, so no test gate. The evidence IS the deliverable:

- Paste the full matrix as a table, with the verbatim `rum-cli` output lines
  backing every cell you call broken.
- Paste the verbatim source lines (with `path:line`) for the mechanism you
  identify.
- Paste the verbatim test assertions you found in the existing-test survey.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config, and do NOT modify any
  file in the repository.
- Touch nothing outside this repo except `/tmp`; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No black-box comparison against another vendor's engine, no
  oracle harness, no compatibility matrix, no third-party engine output in the
  report. Argue intended semantics from rum's docs and from general language
  semantics, not from another implementation's observed output.
- No new `unsafe` may appear in any drafted plan for `rum-core`/`rum-runtime`.
- Bounded probes: use `scripts/run-bounded.py --timeout SECONDS -- <command>`
  for anything that could run long; state the bound you chose and why.

## Report format (your final message)

1. Diagnostic only, no source change — state it.
2. The mechanism, with `path:line` evidence and the probe that reaches it.
3. The full construct x assignment-head matrix, with verbatim backing lines.
4. The repair plan, step by step, with per-step gates, ripple inventory, and
   cost analysis.
5. The existing-test survey.
6. The drafted follow-up implementation briefs.
7. Open questions for the director, each with your recommendation.
8. Stop-loss — where you stopped and why (empty if none).
