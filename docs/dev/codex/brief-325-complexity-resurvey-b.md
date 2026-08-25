# brief-325 — complexity re-survey after the sweep (independent second pass)

## Goal

Two sweeps have closed 70 performance and crash defects. Find what they missed.
This is a **diagnostic survey** producing a ranked candidate list with
`path:line` evidence and measurements, plus drafted implementation briefs — not
a code change.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes with
the release `rum-cli` are in scope; leave no probe files in the repository.
Probes must feed stdin from `/dev/null` — the CLI waits for input otherwise.

Another agent has already surveyed this same question, and its report is
deliberately withheld from you. Derive your candidates from the code, not from
what you imagine it found — two independent derivations are worth comparing;
one confirmed twice is worth little. Where you reach the same place it did,
that agreement is evidence; where you reach somewhere else, that is the point
of running this twice.

## What "missed" means here

The previous surveys were not sloppy, and repeating their method will repeat
their blind spots. Three kinds of miss are known to have happened, and are worth
hunting deliberately:

1. **A defect outside the question asked.** The sweeps looked for O(n^2); they
   found, incidentally, that thirteen builtins aborted the process on wide input
   and that association writes fell off a 40x cliff at a cache threshold. Neither
   is a complexity class. **Look for cost shapes that are not curves**: cliffs at
   thresholds, regressions that only appear after some other operation has run,
   costs that depend on session history rather than input size.
2. **A measurement that stopped short.** One brief's table ended at 16 000 keys;
   the cliff began at 16 384. **Push probes past the sizes that look sufficient**,
   and past the sizes the code's own thresholds suggest.
3. **A structural reading that was never probed.** `DownValue` dispatch was
   named alongside `UpValue` on code shape alone and measured fine; a linear
   `StringCases` was cited as a model when the measurement had used a different
   code path. **Every claim needs a probe, and the probe must reach the code you
   are claiming about.**

## Ground already covered — do not re-report

70 fixes landed. Do not propose anything whose mechanism one of these already
addresses; if you believe one is incomplete, that is a different claim and needs
evidence that the remaining cost is still superlinear.

```
fix(runtime): iterate the remaining wide-input  fix(runtime): iterate uninitialized Module bin
fix(runtime): recover the Association write pr  fix(runtime): walk wide children iteratively
fix(syntax): bound recursive operator parsing   fix(syntax): split the parse-depth guard and b
perf: index wide codec formatting and dispatch  perf: linearize resumable collection traversal
perf(runtime): apply ReplacePart rules in one   perf(runtime): cursor ordered matcher argument
perf(runtime): discover UpValue candidates at   perf(runtime): index Reap's tag buckets
perf(runtime): index repeated association read  perf(runtime): index the Association write pat
perf(runtime): index wide arithmetic groups     perf(runtime): index wide exact ReplaceAll rul
perf(runtime): index wide ground DownValues     perf(runtime): index wide UpValue symbols
perf(runtime): let script forms carry their pa  perf(runtime): linearize batched association k
perf(runtime): linearize Module escape closure  perf(runtime): linearize wide DownValue constr
perf(runtime): linearize wide tagged-value con  perf(runtime): prepare wide association lookup
perf(runtime): resolve replace part conflicts   perf(runtime): share one compiled rule plan ac
perf(runtime): share ReplaceAll rule tables     perf(runtime): skip impossible ReplaceAll cand
perf(runtime): stop copying ReplaceAll argumen  perf(runtime): stop copying traversal suffixes
perf(stdlib): avoid eager iteration continuati  perf(stdlib): batch monomial factor products
perf(stdlib): batch order-preserving polynomia  perf(stdlib): batch wide sequence edits
perf(stdlib): compare polynomial terms sparsel  perf(stdlib): convert RumTokens offsets with o
perf(stdlib): encode TOML in linear time        perf(stdlib): finalize polynomial term batches
perf(stdlib): index collection grouping and ag  perf(stdlib): index the key-selection builtins
perf(stdlib): index wide association grouping   perf(stdlib): index wide commonest counts
perf(stdlib): index wide polynomial factor mul  perf(stdlib): index wide polynomial variable b
perf(stdlib): index wide schema association ke  perf(stdlib): index wide string match states
perf(stdlib): index wide subset map paths       perf(stdlib): index wide TOML tables while dec
perf(stdlib): iterate custom equivalence tests  perf(stdlib): iterate custom sort comparisons
perf(stdlib): iterate GroupBy output phases     perf(stdlib): linearize bounded Nearest select
perf(stdlib): linearize custom same-test state  perf(stdlib): linearize custom set same-test s
perf(stdlib): linearize delayed association ag  perf(stdlib): linearize keysortby decoration
perf(stdlib): linearize resumable fold familie  perf(stdlib): linearize resumable tensor callb
perf(stdlib): linearize reversible suffix matc  perf(stdlib): linearize string Except filterin
perf(stdlib): linearize targeted map updates    perf(stdlib): linearize wide Drop filtering
perf(stdlib): linearize wide literal string se  perf(stdlib): make resumable predicate loops i
perf(stdlib): merge Together addends pairwise   perf(stdlib): stop rescanning unterminated HTM
perf(stdlib): streamline common factors and di  perf(stdlib): use augmenting paths for exact c
perf(stdlib): walk string lists iteratively     perf(syntax): split scripts with a boundary-aw
```

Also settled by standing decision (`TODO.md`): custom `Sort`/`KeySort`
comparator reordering, full polynomial factorization, formatter binary-chain
planning, and iterative `Drop` for deep expression trees (three designs
regressed the hot path 20-50%).

Also **already recorded** in `TODO.md` as known and deferred — report them only
if you find the assessment wrong:

- Error spans are computed per form but rendered against the whole file.
- A trailing-operator form (`1 +\n2 +\n3`) still re-parses per line.
- Pattern-driven `ReplacePart` rules stay sequential.

## Where to look

Cover the workspace, weighted by where wide user data flows: `rum-runtime` and
`rum-stdlib` first, then `rum-syntax`, then host/CLI/jupyter. Structural shapes
worth grepping, beyond the obvious nested loops:

- a loop body that scans or rebuilds the whole collection;
- a container cloned per element, especially before knowing whether a child
  suspends;
- re-parsing, re-formatting or re-deriving something already computed;
- recursion passing the remaining slice by value;
- a cache or index that is **discarded and rebuilt** rather than maintained;
- work whose cost depends on how many definitions or cache entries a session has
  accumulated, not on the current call;
- a threshold constant (`16`, `32`, `64`, `256`, `1024`, `16384`) — check what
  happens **on both sides of it**.

## What each candidate must carry

1. **Location** — `path:line`, the function, and what calls it.
2. **The cost shape** — the class, or the cliff, in terms of a named `n`. Say
   what `n` is.
3. **A measurement** — a probe and its numbers at three or more sizes. A
   structural reading without a probe is a lead, and must be labelled one.
4. **The triggering input** — a concrete expression or script, and the size at
   which the cost becomes visible.
5. **Reachability** — does an ordinary user path reach this, and is `n`
   realistically large there?
6. **Risk** — order sensitivity, observable side-effect order, suspension shape,
   message behavior, small-input allocation.

Rank by real impact times certainty.

**Report the negative cases too.** Places that look quadratic and are bounded in
practice, with one line of why. That list is a deliverable, not filler — it is
what stops the next survey re-walking the same ground.

## Deliverable

1. The ranked candidate table.
2. The negative list.
3. A batching plan: coherent, independently gateable batches, ordered, with the
   batch that first delivers a measured win identified.
4. Drafted implementation briefs in `TEMPLATE.md` shape — raw material for the
   director to rewrite, not to dispatch.
5. Open questions.

## Pitfalls (read twice)

- **Counters beat stopwatches** for yes/no questions like "did this rebuild" or
  "did this copy". The `RUM_PROFILE_*` counters exist for this.
- **A cost that does not scale with `n` is not a finding** unless it is a cliff.
- **Do not over-generalize.** Attribute each finding exactly as far as the
  evidence reaches.
- **Findings age.** 103 commits landed since the last survey; confirm structures
  exist as they stand now.
- **If the survey is too large to finish well, prefer depth over coverage** —
  fewer fully evidenced candidates plus an explicit list of what you did not
  reach beats a padded list.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands: file reads, grep/find,
  `python3`, the release `rum-cli`, and `cargo` only if a counter needs a build.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Every claim carries `path:line` or a measurement. A claim you could not
  confirm is reported as unconfirmed, not asserted.

## Report format (your final message)

1. Method — what you scanned, how, and what you deliberately skipped.
2. The ranked candidate table.
3. The negative list.
4. The batching plan and drafted briefs.
5. Open questions for the director.
