# brief-339 — plan the user-callback ordering paths (DESIGN ONLY)

**Scope: NONE — no source change.** Produce a plan and draft the implementation
briefs. Write no code.

## The measurements (director's, release CLI — do not re-derive)

Time in ms; the last column is the factor per doubling of n, where 4x means
quadratic.

```
                       n=50    n=100   n=200   n=400   n=800   per doubling
Sort with comparator   4.319  15.408  61.326 239.230 955.985  3.6 4.0 3.9 4.0
Sort default           0.003   0.003   0.006   0.008   0.019  1.1 1.8 1.5 2.2
DeleteDuplicates+SameTest
                       0.782   3.071  12.740  49.686 204.394  3.9 4.1 3.9 4.1
DeleteDuplicates default
                       0.014   0.015   0.026   0.063   0.133  1.1 1.8 2.5 2.1
Union with SameTest    1.562   6.223  24.751 103.116 400.266  4.0 4.0 4.2 3.9
```

Two further probes locate the defects precisely:

**`Sort` is order-insensitive**, so it is not an insertion sort that degrades on
bad input — it is quadratic on every input:

```
Sort[Range[800], comparator]           946.755
Sort[Reverse[Range[800]], comparator]  945.313
```

From n=50 to n=800 — 16x — the time grows 221x. Quadratic predicts 256x,
n log n predicts about 27x.

**`SameTest` never exits early.** An all-identical list, where the first
comparison of every element finds its duplicate, costs the same as an
all-distinct list, where every pair must be examined:

```
                          n=200    n=400    n=800
all identical            13.635   51.321  204.896
all distinct             12.691   51.330  203.890
ratio                      1.07     1.00     1.00
```

## A hypothesis to confirm or refute

The default paths are fast and the user-callback paths are not. The default
duplicate scan uses `.any(…)` (`crates/rum-stdlib/src/structure_ops/dedup.rs:24,285`),
which short-circuits. The `SameTest` path is resumable instead
(`crates/rum-stdlib/src/structure_ops/ordering_sets.rs:639` finishes a
"SameTest candidate"), because a user predicate can suspend.

So the hypothesis is that **making these paths resumable cost them their
algorithms**: a short-circuiting `any` became a state machine that visits
everything, and a divide-and-conquer sort became a quadratic one because the
state machine was easier to write that way.

Confirm or refute this from the code. If the real cause is different, say so —
the measurements are solid, the explanation is mine and may be wrong.

## What the plan must decide

1. **Whether a resumable merge sort is worth building.** A comparison sort
   driven by a callback that can suspend needs its state — which run, which
   position in each run, the partial output — to survive a suspension and
   resume exactly. Say whether the existing suspension machinery supports that
   shape, what it would take, and what the risk is. If you conclude the
   quadratic sort should stay, argue it: 956 ms at n=800 against 0.019 ms for
   the default is a large thing to keep, so the argument needs to be strong.

2. **Whether `SameTest` can exit early without changing results.** The
   short-circuit must be observationally identical: the same number of user
   predicate calls in the same order for inputs where the answer does not
   change, and no call that today happens and afterwards does not — unless
   skipping it is provably unobservable. A user predicate can have side
   effects, so "fewer calls" is a semantic change, not only a speed-up. State
   exactly what is guaranteed.

3. **The full inventory of affected builtins.** `Sort`, `DeleteDuplicates` and
   `Union` are what the probes reached. Enumerate every builtin that accepts a
   user comparator or `SameTest`, measure each, and say which share the defect.
   `SortBy`, `Intersection`, `Complement`, `Gather*`, `Count*`, `MemberQ`,
   `Position` and the Association key variants are places to look; the list is
   yours to complete and justify.

4. **Suspension semantics, stated as invariants.** Whatever changes, a
   suspended comparison must resume with the same partial state and produce the
   same result as an uninterrupted run. Say how each proposed step preserves
   that and how a test would demonstrate it.

5. **Step ordering and independent gateability.** If the early-exit work is
   simpler and lower-risk than the sort, the steps should reflect that.

## Evidence standard

Every claim gets `path:line`. Where you assert what a path does at runtime,
cite the code or a probe you ran — the release CLI is available, feed stdin
from `/dev/null`.

## Deliverable

1. Confirmation or refutation of the hypothesis, with evidence.
2. The affected-builtin inventory, measured.
3. A step-by-step plan: files and functions per step, gate tests, ordering.
4. Answers to the five questions; open questions flagged for the director
   rather than guessed.
5. **Draft implementation briefs in `TEMPLATE.md` shape**, one per step, with
   Scope, MAY-modify lists, hard rules, verification commands, report format.

## Hard rules

- **No source changes.** No edits under `crates/`. A new file under
  `docs/dev/codex/` for your drafts is expected; nothing else.
- Do NOT `git commit`, branch, push, or touch git config.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- Probing with the release CLI is encouraged. **Run each probe as its own
  script** — the CLI stops at the first error. Report the load average with any
  timing; do not wait for a quiet machine.
- Note that iteration constructs now charge a product budget, so a benchmark
  loop wrapping a generating construct can hit the generated element limit.
  That is expected behavior, not a defect to work around — size probes
  accordingly.

## Report format (your final message)

1. Hypothesis: confirmed, refuted, or corrected, with evidence.
2. The inventory table.
3. The plan, step by step.
4. Answers to the five questions; open questions separately.
5. Where the draft briefs were written.
