# brief-378 — definition-lhs preparation, repair family B (design only)

Family A landed in `636e01ee` (brief-377). This is the plan for family B.
plan-376 sketched two of its steps under a contract that has since been
settled, and did not cover the two ordinary-`Set` defects at all — those were
outside its scope at the time and are inside this one.

## Goal

Produce the step-by-step plan, plus drafted implementation briefs, for one
attribute-aware preparation of a definition's left-hand side, shared by the
store side and the lookup side.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
with the release `rum-cli` plus throwaway `#[test]` scratch files under `/tmp`
are in scope; leave no probe files in the repository, and feed every
`rum-cli` probe `stdin < /dev/null`.

## The contract — settled, do not re-litigate

Director and user ruling, 2026-08-28:

> A definition's lhs arguments are prepared according to the head's hold
> attributes: non-held positions are fully evaluated, including a delayed
> own-value; held positions keep their raw symbols. **The identity a
> definition is stored under and the identity a lookup constructs must come
> from the same preparation.**

The invariant is **bidirectional**, and that is the point of this brief. rum
violates it in both directions today, so a plan that fixes only one direction
is not a plan for this family — it would close three defects and hide the
fourth, exactly the shape the frame-roots line hit when a `Block` frame's bare
`Symbol` leaked in the opposite direction from everything else in that batch.

Two decisions that follow from it, also settled:

- **`UpSet::nosym` stays rum's diagnostic.** Once `m[ww] ^= 22` with `ww = 5`
  prepares to `m[5]`, no symbol in the lhs can serve as a tag, and rum already
  reports exactly that for `m2[7] ^= 33`. The wider WL family reaches a
  differently-shaped message there; rum keeps the more direct one and stays
  internally consistent. Design input only — no engine comparison enters the
  repository, per the Testing Policy.
- **The "raw rules become reachable after `Clear`" argument is retired.**
  plan-376 recorded it as a reason a repair must diagnose rather than refuse.
  It describes rum's own raw-store behavior, not a semantics worth keeping:
  under the settled contract a rule is stored ready for lookup and is
  reachable immediately. The preserve-raw option and the diagnostic question
  that depended on it are both closed. Do not reopen either.

## The four defects in this family

Director's probes, release CLI at `65c03870`, each with a liveness assertion
and a working control. The first two are from the batch-7 matrix; the last two
were found while probing the contract and are the opposite direction.

**B1 — `AssociateTo`/`KeyDropFrom` store the raw fallback target (silent).**
`ii = 2; af[2] = <|"a" -> 1|>; AssociateTo[af[ii], "k" -> 7]` reads `af[2]`
correctly, computes the update correctly, then installs
`HoldPattern[af[ii]] :> <|"a" -> 1, "k" -> 7|>` as a second definition and
leaves `af[2]` untouched. plan-376 widened this: the fallback also carries
SubValues (`sf[pp][qq]`), evaluated/aliased heads, and delayed own-values
(`iDelayed := 2` behaves the same). The fallback is legitimate and must
survive — only the store identity is wrong. Read `mutation.rs:187` / `:225`
for `AssociateTo` and `:323` / `:364` for `KeyDropFrom`; the raw store is the
single defective one of the three `store_definition_assignment_target_immediate`
call sites (`mutation.rs:412`), the other two constructing symbol targets.

**B2 — `TagSet`/`UpSet` store a raw lhs, and the rule is unreachable
(silent).** `tg = 5; h /: h[tag1, tg] = 7` stores
`HoldPattern[h[tag1, tg]] :> 7`, and neither `h[tag1, 5]` nor `h[tag1, tg]`
matches — the lhs prepares to `h[tag1, 5]` before lookup, so no input reaches
the rule. All six install forms reproduce (`TagSet`, `UpSet`, and both
`*Delayed`, across down-value and up-value ownership). Clean symbols work, and
`m2[7] ^= 33` correctly reports `UpSet::nosym`. Entry points in
`definition_tags.rs`; raw classification/storage at `:137` and `:158`.

**B3 — ordinary `Set` does not complete a delayed own-value (silent).**
`iD := 1 + 1; fD[iD] = 8` stores `HoldPattern[fD[1 + 1]] :> 8`, so `fD[2]`
cannot reach it. Same direction as B1/B2, but on the most-used assignment path
in the language.

**B4 — ordinary `Set` normalizes a held position that must stay raw
(silent).** With `SetAttributes[fH, HoldFirst]` and `xH = 5`, `fH[xH] = 9`
stores `HoldPattern[fH[5]] :> 9`. The natural `fH[xH]` lookup does not
evaluate its argument (the head is `HoldFirst`), so it looks for `fH[xH]` and
misses; only `fH[5]` reaches the rule. **This is the opposite direction from
B1/B2/B3** and is why the invariant is stated bidirectionally.

For contrast, two things rum already gets right and must not regress:
`fImmediate[iImm] = 7` with `iImm = 2` stores `fImmediate[2]`, and a
`HoldFirst` head under `TagSet` (`heldHead /: heldHead[hv, tagH] = 51`) keeps
its raw symbol and is reachable. plan-376 confirmed the existing `Set`
normalizer (`definitions.rs:498`) is synchronous structural substitution, not
a resumable preparation, and that brief-343 explicitly excluded these defects
(`brief-343:76`) — its pattern-wrapper knowledge is reusable, the function is
not the seam.

## Required first step — re-probe before planning

**Findings age, and probes lie.** This line has already had two recorded
defects turn out to be fixed before they were briefed, one survey finding that
was an artifact of an invalid probe, and a recorded microbench defect that had
been fixed four days earlier. Family A also changed these files.

1. Reproduce B1-B4 on the current tree, at `dfedb317` or later. State the commit.
2. **Widen each.** For B3/B4 in particular: which other hold attributes matter
   (`HoldRest`, `HoldAll`, `HoldAllComplete`, `SequenceHold`), how they
   interact with a *partially* held argument list, what happens with nested
   calls (`f[g[x]]` where only one head holds), and whether `SetDelayed`,
   `TagSetDelayed`, `UpSetDelayed`, and the operator-form assignments behave
   the same as their immediate siblings.
3. **Enumerate what currently encodes the wrong behavior.** Any existing test
   asserting a raw stored lhs, or asserting `fH[5]`-style reachability, has to
   be identified now and its fate stated in the plan — changed, deleted, or
   kept because it was testing something else.
4. Every probe carries a liveness assertion and a working control.
5. If any of B1-B4 does not reproduce, say so loudly and stop planning it.

## What the plan must contain

**The preparation itself.** plan-376's sketch was: pattern-aware,
attribute-aware, resumable; evaluates a call head as ordinary lookup would;
prepares evaluation-active arguments through the existing hold-attribute
machinery; preserves pattern and opaque held regions; stops before applying
the outer definition; carries both the canonical lhs and its classified
storage target; derives `FrameRootTrace`; is required by the immediate
definition-store bridge. Critique that sketch — it is a starting point, not a
specification. In particular, say precisely where "prepares arguments as
ordinary lookup would" differs from "evaluates the expression", because the
lhs must not be applied.

**The lookup side.** The invariant is bidirectional, so the plan must say what
the lookup path does today, whether it already prepares identically, and how a
future divergence between the two sides is prevented — by construction if
possible. A shared function that both sides *happen* to call is weaker than
one neither side can bypass; say which one you can achieve and why.

**Step ordering and independence.** Propose the order and defend it. The
director's prior is: the preparation plus ordinary `Set` (B3/B4) first, since
`Set` is the most basic consumer and the riskiest path, then B1, then B2 with
`TagUnset`. Argue for a different order if the evidence supports one.

**Performance.** Ordinary `Set` runs on every assignment, so this is a hot
path and any per-assignment cost is a real cost. State per step what the
ordinary already-canonical case pays. A fast path that recognizes "nothing to
prepare" without allocating is expected; if it cannot be had, say so
explicitly with the measurement that shows why.

**Suspension.** Preparation can evaluate user code, so it can suspend. Every
new state or frame type derives `FrameRootTrace` and appears in
`runtime/tests/frame_root_trace/derived_inventory.rs`; frames are inserted
only through `resume_or_suspend!`; the frame queue stays private. Say how a
resumed preparation avoids replaying already-prepared arguments.

**Ripple.** `TagUnset` removes by raw lhs (`definition_tags.rs:90`,
`definition_clear.rs:132`); if tagged definitions prepare, removal must use the
same preparation. Inventory the rest yourself — every store site, every
lookup-identity construction, `Unset`/`Clear` paths, and anything that renders
a stored lhs.

## Test-shape requirement for every drafted brief

**Assert after the invalidating action, not only before it.** This line lost
six green gates to tests that asserted a guard before the trigger and never
after one. Preparation depends on session state — own-values, attributes — so
each drafted brief's gate must act after that state changes: after
`SetAttributes` adds a hold attribute to a head that already has definitions,
after an own-value used in a stored lhs is reassigned or cleared, after a
`Clear` of the head itself. Say which invalidation each test acts after.

Also: "I added the logic" is not "the logic runs". Brief-348 wrote correct
rebuild code that a fast path returned before reaching, and family A's own
`AppendTo` incremental path is exactly such a fast path in these files. For
each step, name the entry point and show the changed code is on the path an
ordinary call takes; if a fast path can bypass it, the fast path is part of
the gate.

## Deliverable

1. Re-probe section: B1-B4 reproduced or not, verbatim, widened, with the
   commit and the inventory of tests encoding current behavior.
2. The preparation's specification, and how the two sides are kept from
   diverging.
3. Step-by-step plan: files and functions per step, contract, hot-path cost,
   suspension handling, gate tests (with the invalidation each asserts after),
   ripple, risks.
4. Drafted implementation briefs in `TEMPLATE.md` shape, one per step.
5. Open questions, split into resolved-with-evidence and needs-a-ruling.

## Context pointers

- `AGENTS.md` first (operational notes, Testing Policy hard lines).
- `TODO.md`, "Next up — scheduled 2026-08-28": the settled contract, family A's
  close-out, and the A/B offset lesson below.
- plan-376's report for the call-site inventory and the `PreparedDefinitionLhs`
  sketch. Its family-A half is now landed; read `636e01ee` for what changed.
- Key files: `definitions/definitions.rs` (the current `Set` normalizer),
  `definitions/definition_tags.rs`, `definitions/definition_clear.rs`,
  `definitions/definition_analysis_helpers.rs`,
  `crates/rum-stdlib/src/associations/mutation.rs`.

**A/B note for whoever drafts the performance gates.** Family A's run carried
a **+1.81% whole-run offset** measured on the 102 untouched control labels,
against +1.02% median on the ten affected ones; the raw split was 86-of-112
labels slower and reads as a regression until the controls are consulted.
Name the control labels in the brief, judge against them, never against the
suite median, and remember two runs do not estimate a noise floor.

Watch: `association_data.rs` is 32.2k and `definition_updates.rs` 26.3k, both
under the 35k architecture-debt line but rising, and this family touches both.
Do not plan a split into these steps — note it for after the family closes.

## Scope

- Files you MAY modify: **NONE — design only.** No source change, no doc
  change. Your report is the deliverable.
- Throwaway probe files go under `/tmp`, never in the repository.
- Do NOT `git commit`, branch, push, or touch git config.

## Hard rules

- **Testing Policy is a hard line** (`AGENTS.md`): no differential testing
  against another vendor's engine, no oracle harness, no compatibility matrix,
  no third-party output committed. WL semantics may inform the design and be
  described in prose; they never enter the repo as a test oracle.
- **No new `unsafe`** in any drafted plan.
- **No unconditional cost on the ordinary evaluation path** beyond what the
  contract genuinely requires; instrumentation goes behind the env-gated
  cold-load `AtomicU8` pattern.
- **Stop-loss**: if this brief is unsatisfiable as written — a prohibition
  blocking a legitimate necessary case, an instruction contradicting what you
  find — STOP and report the specific contradiction rather than working around
  it. Recent stop-losses on this line were correct calls on the director's
  omissions.
- Paste real verbatim probe output. If a probe did not run or did not
  reproduce, say so explicitly.

## Report format

1. Re-probe results — per defect, verbatim, widened, commit, plus the
   existing-test inventory.
2. The preparation's specification and the anti-divergence argument.
3. The plan — steps with files, contract, hot-path cost, suspension, gates,
   ripple, risks.
4. Drafted implementation briefs.
5. Open questions — resolved-with-evidence vs. needs-a-ruling.
