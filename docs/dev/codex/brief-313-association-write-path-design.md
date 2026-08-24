# brief-313 — design: the Association single-key write path

## Goal

Decide, with evidence, whether the quadratic in incremental Association writes
should be fixed — and if so, design a fix that cannot reintroduce the ownership
bug that `plan-291` spent three steps removing. Deliver a reviewable plan plus
drafted implementation briefs.

**Scope: NONE — no source change.** Do not edit any source file. Reading,
grepping, and throwaway probes with the release `rum-cli` are in scope; leave no
probe files in the repository. Probes must feed stdin from `/dev/null` — the CLI
waits for input otherwise.

## Why this is a design brief and not an implementation brief

This one has a standing decision against it, and a known trap.

**The standing decision.** `plan-291`'s Step 6 (a bounded key index for
Associations) was deliberately **not done**. `CLAUDE.md` records two reasons:
the linear scan it targeted was not the dominant term at the sizes considered,
and it would reintroduce exactly the extra-strong-holder shape that arc had just
spent three steps removing.

**The trap, in the plan's own words**
(`docs/dev/codex/plan-291-assoc-append-ownership.md:89-93`):

> Step 6 deliberately introduces another strong holder for indexed
> Associations: its identity witness. That holder is bounded and is taken,
> pointer-validated, and dropped before both the stability-cache entry and the
> definition-owned args enter COW mutation; leaving it live would recreate the
> same bug diagnosed here.

An extra strong holder makes `Arc::make_mut` copy instead of mutating in place,
which turns an O(1) write into an O(n) one — the exact failure the arc removed.

## Measured today (release CLI, verified by the director)

`a = <||>; Do[a[i] = i, {i, 1, n}]`:

| n | time |
|---|---|
| 2000 | 0.031 s |
| 4000 | 0.090 s |
| 8000 | 0.356 s |
| 16000 | **1.286 s** |

2.9–4.0× per doubling. So the first of the two reasons — "not the dominant term
at these sizes" — needs re-examination at the sizes people actually reach, not
re-assertion.

## What the survey proposed, and why it might now be safe

A later survey suggested the write path reuse the read side's
`AssociationLookupCache` (`crates/rum-runtime/src/runtime/model/cache.rs:96-120`)
for its lookup. That cache is interesting precisely because of how it avoids the
trap — its own comment:

> A second lookup admits an index. The weak witness prevents pointer reuse from
> producing a false hit **without pinning the Association's COW data.**

It holds `WeakSharedArgs`, not a strong reference. So a design that reuses it may
sidestep what sank Step 6. **May.** That is the central question of this brief,
not its premise.

## Questions the plan must answer, each with `path:line` evidence

1. **Where does the time actually go?** Attribute the quadratic to specific
   code. The survey named `kernel_data/association_lvalue.rs:96-136`
   (`direct_association_rule_plan` scanning all entries even after finding the
   position) and `association_data.rs:705-733` (`set_association_key_path`
   re-parsing, locating and rendering per write). Confirm or correct that, and
   say which term dominates — a fix aimed at the smaller term is not worth its
   risk.
2. **Do `plan-291`'s two reasons still hold?** Answer each separately. For the
   "not dominant" one, use measurements. For the ownership one, say precisely
   what would and would not reintroduce a strong holder.
3. **Does reusing the read-side cache actually avoid the trap?** Show how the
   weak witness interacts with the write path's `Arc::make_mut`. Under what
   circumstances, if any, would the cache cause a copy that does not happen
   today? What proves the answer — a pointer-identity test, an allocation
   counter, or both?
4. **Is there a cheaper fix than an index?** For instance, if
   `direct_association_rule_plan` scans past a found position purely to validate
   something, an early exit might recover most of the win with none of the
   ownership risk. Prefer the smallest change that moves the measured number.
5. **What must not change**: key order, duplicate-key semantics, the
   `Missing` behavior on absent keys, message text, and suspension/resume shape
   in any path that runs user code. Name the tests that pin each.
6. **`AssociateTo`** (`crates/rum-stdlib/src/associations/mutation.rs:223-226`)
   reportedly does a full `retain` per update. Is that the same problem or a
   separate, simpler one? If separate, it may deserve its own step.

## Deliverable

1. The design, answering all six questions with evidence.
2. **A recommendation, including "do not do this"** if the evidence supports it.
   A well-evidenced case for leaving the quadratic in place is a valid outcome —
   the standing decision exists for a reason, and overturning it needs more than
   a scaling table.
3. If you recommend proceeding: a step plan, each step independently gateable,
   with the step that first moves the measured number identified.
4. Drafted implementation briefs in `TEMPLATE.md` shape — raw material for the
   director to rewrite, not to dispatch as-is.
5. Risks and open questions, including anything you want decided before step 1.

## Pitfalls (read twice)

- **Do not design against a property that has to be proved by sampling.** A
  brief in this line died that way. Prefer mechanisms whose correctness follows
  from construction, and say plainly which parts do not.
- **An allocation counter beats a stopwatch here.** Whether a write mutates in
  place or copies is a yes/no question with a load-immune answer; the timing is
  a consequence. `RUM_PROFILE_*`-gated counters exist for this.
- **Findings age.** `plan-291` landed months of changes ago and four more
  optimizations landed today; confirm structures exist as they stand now.
- **Do not over-generalize.** Attribute each finding exactly as far as the
  evidence reaches.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands: file reads, grep/find,
  `python3`, the release `rum-cli`, and the microbench scripts only if you need
  a counter reading you cannot get otherwise.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- Every claim carries `path:line`. A claim you could not confirm is reported as
  unconfirmed, not asserted.

## Report format (your final message)

1. Method — what you read and probed.
2. The design, question by question.
3. The recommendation, stated plainly, with its evidence.
4. The step plan and drafted briefs, if you recommend proceeding.
5. Risks and open questions for the director.
