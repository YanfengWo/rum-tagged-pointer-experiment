# brief-376 — the assignment target is not normalized before install (design only)

## Goal

Three defects share a shape: an assignment or mutation reads its target
correctly, then installs the result against the *unevaluated* target
expression, or refuses to act because it never evaluated a selector. All three
were found by the batch-7 iterator matrix, were recorded as "not fixed", and
were re-probed by the director on 2026-08-28 against the release CLI at
`65c03870` — all three still reproduce. Two are silent, and one of those
installs a definition that no input can ever reach.

Produce the step-by-step plan to close them, plus drafted follow-up
implementation briefs — **and first, re-probe all three yourself and establish
how wide each one actually is.** The director's probes are in **Established
evidence** below; treat them as a starting point to be widened and checked,
not as a survey.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
with the release `rum-cli` plus throwaway `#[test]` scratch files under `/tmp`
are in scope; leave no probe files in the repository, and feed every
`rum-cli` probe `stdin < /dev/null`.

## The central question this plan must answer

The three defects share a *shape* but reach it by three different mechanisms.
**Whether that is one repair or three is exactly what you must argue, not
assume.** A single normalization seam that all three route through would be
the better outcome if it exists; a forced merge of three unrelated code paths
would be worse than three small repairs.

This project has a scar on precisely this question: brief-208 correctly
observed that `store_held_update` skipped a protection recheck, then
attributed the resulting bypass to all eleven compound assignments when only
`ApplyTo` could actually trigger it. A correct root-cause observation
attributed too widely is a real failure mode here. So: state per defect which
seam it goes through, and let the *evidence* decide the grouping.

**A standing ruling you must apply while planning (user, 2026-08-27):** this
is a research project with no downstream users and no human-labour budget.
**Pick the most correct design; the size of the change is not an argument
against it.** Do not propose a smaller variant because the full one touches
many call sites, and do not propose a stopgap now with the real fix later — a
temporary patch is itself debt here. Arguments that do count: correctness,
semantic loss, whether a design is structurally unavoidable, whether it leaves
an open-ended obligation, enforceability, and measurable cost on paths that
are not assignments. Where a repair can be enforced by the compiler or a
shared seam rather than by remembering to call something, that is the better
repair even if it is larger.

## Established evidence (director, release CLI at `65c03870`)

Every probe below carried a liveness assertion and a working control.

### D1 — `AssociateTo` / `KeyDropFrom` write back to the raw target (silent)

```
ii = 2; af[2] = <|"a" -> 1|>;
AssociateTo[af[ii], "k" -> 7];
```

Observed: `af[2]` is still `<|"a" -> 1|>`, and `DownValues[af]` grew from one
entry to two:

```
{HoldPattern[af[2]] :> <|"a" -> 1|>,
 HoldPattern[af[ii]] :> <|"a" -> 1, "k" -> 7|>}
```

So the read side resolved `af[ii]` to `af[2]` and the update itself was
computed correctly — only the store used the raw expression. No message is
emitted. `KeyDropFrom` behaves identically (`bf[3]` unchanged, second
definition installed).

The mechanism, `crates/rum-stdlib/src/associations/mutation.rs:186-225`: the
fallback path evaluates the target to read the current value —

```rust
let target_result = session.evaluate(target_expr.clone());
```

— and then hands the **unevaluated** `target_expr` to the store:

```rust
store_association_lvalue_update(session, &target_expr, updated.clone());
```

where `store_association_lvalue_update`
(`mutation.rs:411`) forwards straight to
`session.store_definition_assignment_target_immediate(target_expr, updated)`.

Note the two earlier branches in `eval_associate_to` — the
`normalize_symbol_target_expr` branch (`mutation.rs:143`) and the
`normalize_part_mutation_target` branch (`mutation.rs:155`) — both do
normalize. Only the fallback does not. Establish why the fallback exists and
what reaches it.

### D2 — compound assignment heads do not evaluate a `Part` selector (loud)

```
kk = 1; lst = {10, 20, 30}; lst[[kk]] += 5;
```

Observed: `lst` unchanged, and two messages —
`Part::pkspec1: The expression kk cannot be used as a part specification.`
and the same under `Set::pkspec1`.

Two controls both work, which isolates the defect to
compound-assignment × symbol-selector:

- `lst2[[kk]] = 99` → `{99, 20, 30}` (plain `Set`, symbol selector)
- `lst3[[1]] += 5` → `{15, 20, 30}` (compound, literal selector)

Confirmed on nine heads: `AddTo`, `SubtractFrom`, `TimesBy`, `DivideBy`,
`Increment`, `Decrement`, `PreIncrement`, `PreDecrement`, `AppendTo`. The
record says eleven; **enumerate the real list yourself** and probe every one
of them, including `ApplyTo` and `PrependTo`.

The apparent repair model is a one-call-site difference — the same function,
fed differently:

- `crates/rum-runtime/src/runtime/definitions/part_assignment.rs:228` passes
  `evaluated_selectors` to
  `exact_position_paths_from_part_target_selector_exprs`;
- `crates/rum-runtime/src/runtime/definitions/definition_updates.rs:389`
  passes the raw `selector_exprs` to the same function, and `:404` does the
  same for `selected_expr_from_part_target_selector_exprs`.

**Do not take that at face value.** The held-update path holds its target for
a reason, and the ordering question — when a selector may be evaluated
relative to the target read and to the rhs — is a semantic contract, not a
plumbing detail. The plan must state the evaluation order it intends, per
head, and say what observable behavior that order commits rum to. If the
selector must be evaluated exactly once, say where that is enforced.

### D3 — `TagSet` / `UpSet` install a definition nothing can reach (silent)

Worse than the record, which said only that the lhs is stored unnormalized.

```
tg = 5; h /: h[tag1, tg] = 7;
```

Observed: `DownValues[h]` is `{HoldPattern[h[tag1, tg]] :> 7}`, and **neither**
`h[tag1, 5]` nor `h[tag1, tg]` matches it — the lhs evaluates to `h[tag1, 5]`
before lookup, so the stored rule can never fire from any input. It is dead on
arrival, silently.

The true up-value form and the tagless form do the same:

- `vv = 5; g /: k[g, vv] = 11` → `UpValues[g]` is
  `{HoldPattern[k[g, vv]] :> 11}`, and `k[g, 5]` stays unevaluated.
- `ww = 5; m[ww] ^= 22` → `UpValues[ww]` is `{HoldPattern[m[ww]] :> 22}`, and
  both `m[5]` and `m[ww]` stay unevaluated.

Two controls behave correctly: a clean symbol works (`m3[qq] ^= 44`, then
`m3[qq]` is `44`), and `m2[7] ^= 33` correctly reports `UpSet::nosym`.

Entry points: `crates/rum-runtime/src/runtime/definitions/definition_tags.rs`
(`eval_up_set` at `:7`, `eval_tag_set` at `:44`, and
`store_tagged_definition` / `finish_up_set_assignment`).

**D3 carries a decision that is NOT yours.** Whether a `TagSet`/`UpSet` lhs
*should* normalize its arguments is a standing contract question reserved to
the director and the user. Your job on D3 is to supply the decision input, not
to pick:

- What each option costs to implement, and what each breaks.
- Which existing tests encode the current behavior (if any).
- What the wider WL-family semantics are here — **as design input only.** Do
  not add any comparison against another vendor's engine to the repository, do
  not commit its output, and do not build a differential harness; the Testing
  Policy in `AGENTS.md` is a hard line. Describe the semantics in prose in
  your report.
- The one part that is a defect under *either* answer: installing a rule that
  no input can reach, without a message. Plan that half so it can land
  independently of the contract decision, and say explicitly which of your
  drafted briefs depends on the decision and which does not.

## Required first step — re-probe before planning

**Findings age, and probes lie.** This batch has already had two items turn
out to be fixed before they were briefed, one survey finding that was an
artifact of an invalid probe (`SameTest` passed as an option when rum takes a
positional argument, so the comparison silently compared nothing), and — on
2026-08-28 — a recorded microbench defect that had been fixed four days
earlier and never retired from the record.

So, before any planning:

1. Reproduce D1, D2, D3 yourself on the current tree. State the commit.
2. **Widen each one.** For D2 that means the full head list, not the nine
   above. For D1 that means: which other association mutation heads route
   through the same fallback, and does the same raw-target store exist
   anywhere else (`store_definition_assignment_target_immediate` call sites)?
   For D3 that means all three lhs forms plus `TagSetDelayed` / `UpSetDelayed`.
3. **Every probe carries a liveness assertion** — an explicit check that the
   work actually happened, not just that a number came back. A probe that
   silently measures a failing path is this project's most expensive recorded
   mistake: the `lst[[i]] = v` performance numbers were the cost of
   rebuild-then-reject, and the calibration point was inside the probe but
   never read. Include a working control in each probe, as the director's
   probes above do.
4. If any of the three does **not** reproduce, say so loudly and stop planning
   that one — do not plan a repair for a defect that is not there.

## Deliverable

1. **A re-probe section**: per defect, reproduced or not, with verbatim
   output, the widened surface, and the commit probed.
2. **The grouping verdict**: one repair, three, or two — argued from the seams
   each defect actually goes through, with `path:line` evidence. Include the
   call-site / ripple inventory that backs it: every head affected, every
   store call site, and anything that would have to change mechanically.
3. **A step-by-step plan**, one step per dispatchable unit: files and
   functions per step, the evaluation-order contract each step commits to, how
   suspension safety is preserved (these are `eval_*` paths with resume
   frames — see the `SuspensionFrame: FrameRootTrace` requirement that landed
   in briefs 364-375), the gate tests per step, and the risks.
4. **Drafted follow-up implementation briefs** in `TEMPLATE.md` shape, one per
   step — real briefs, not step descriptions. The director rewrites them
   before dispatch; they are raw material.
5. **Open questions**, separated into ones you resolved with evidence and ones
   that need a director or user ruling (D3's contract is at least one).

## Test-shape requirement for every drafted brief

A gate that asserts only *before* the triggering action cannot see a defect
that the action introduces. This line lost six green gates to exactly that: a
guarded `DownValue` stopped enforcing its guard the moment anything bumped the
attribute revision, and every gate — plus the whole-line regression test —
asserted the guard before the trigger and never after one.

So each drafted brief's test gate must assert **after** the state change it
depends on, not only before: after the target has been reassigned, after a
selector symbol has been rebound, after an intervening definition has been
installed. Where a repair caches or compiles anything derived from a
definition, say which invalidation the test acts after.

Related trap to avoid in your own reasoning: "I added the logic" is not "the
logic runs". Brief-348 wrote correct rebuild code that was unreachable,
because a fast path returned first. For each step, name the entry point and
show that the changed code is on the path an ordinary call takes.

## Context pointers

- Read `AGENTS.md` (operational notes, the Testing Policy hard lines) first.
- `TODO.md`, section "Next up — scheduled 2026-08-28" and the three defects
  under "the same matrix exposed three defects" — the director's re-probe
  notes, already updated with the evidence above.
- Key files: `crates/rum-stdlib/src/associations/mutation.rs` (D1),
  `crates/rum-runtime/src/runtime/definitions/definition_updates.rs` and
  `part_assignment.rs` (D2), `crates/rum-runtime/src/runtime/definitions/
  definition_tags.rs` (D3),
  `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` (the
  shared selector helpers both D2 paths call).
- Prior art worth reading before you plan: briefs 342-343 fixed the sibling
  defect in this same family — iterator variables invisible on an assignment
  lhs — by making the held-lhs normalizer take a third resolution step.
  **Establish why that repair did not cover D1/D2/D3**, and whether its seam
  is the right one to extend. If it is, say so; if extending it would
  over-generalize, say that instead.

## Scope

- Files you MAY modify: **NONE — design only.** No source change, no doc
  change. Your report is the deliverable.
- Throwaway probe files go under `/tmp`, never in the repository.
- Do NOT `git commit`, branch, push, or touch git config.

## Hard rules

- **Testing Policy is a hard line** (`AGENTS.md`): no black-box comparison
  against other vendors' engines, no oracle harness, no compatibility matrix,
  no third-party engine output committed. WL semantics may inform the design
  and may be described in prose in your report; they never enter the repo as a
  test oracle.
- **No new `unsafe`** in any drafted plan. `rum-core` and `rum-runtime` are
  zero-`unsafe`.
- **No default-path cost.** Any plan step that would add an allocation, a
  clone, or a branch to the ordinary (non-assignment) evaluation path must say
  so explicitly and justify it; instrumentation goes behind the env-gated
  cold-load `AtomicU8` pattern.
- **Suspension safety**: D1 and D3 both run through `eval_*` paths with resume
  frames. Any plan that changes what a frame holds must respect the frame-root
  traversal contract from briefs 364-375 — the derived `FrameRootTrace` walk,
  and the private frame queue as the single insertion route.
- **Stop-loss is real.** If the brief is unsatisfiable as written — a
  prohibition that blocks a legitimate necessary case, an instruction that
  contradicts the evidence you find — STOP and report that, with the specific
  contradiction. Do not work around it silently. Two of this line's recent
  stop-losses were correct calls on the director's own omissions.
- You MUST paste real verbatim probe output. If a probe did not run or did not
  reproduce, say so explicitly — never fake it.

## Report format (your final message)

1. Re-probe results — per defect, verbatim output, widened surface, commit.
2. Grouping verdict — one/two/three repairs, argued, with the call-site
   inventory.
3. The plan — steps, each with files, functions, contract, gate tests, risks.
4. Drafted implementation briefs — in `TEMPLATE.md` shape.
5. Open questions — resolved-with-evidence vs. needs-a-ruling (D3's contract
   belongs in the second group).
