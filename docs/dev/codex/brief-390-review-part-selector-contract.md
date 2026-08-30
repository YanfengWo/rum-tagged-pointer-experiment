# brief-390 — review: `Part` selector evaluation contract (2 of 4)

Part two of four independent consistency reviews. **Scope: NONE — review only,
no source change.** Reading, grepping and throwaway probes are in scope; leave
no probe files in the repository.

**Write your findings section first, before running any verification
commands.** Keep each numbered section self-contained.

Part one (brief-389) reviewed the frame-root traversal and found the invariant
holds, with one accurate correction to how widely the guard reaches. It also
corrected two overstated claims in the background I had written. **Do the same
here: if the background below is wrong, say so — that is a finding, not a
digression.**

## Background

brief-377 changed how assignment-style `Part` mutations obtain their selectors.
Previously the mutation paths handed raw expressions to selector parsers, so
`kk = 1; lst[[kk]] += 5` reported `Part::pkspec1` and changed nothing while
`lst[[kk]] = 99` worked. The repair introduced a typed `EvaluatedPartSelectors`
that the assignment-target parsers accept, obtainable only two ways: by
exactly-once evaluation, or by a constructor that proves the selectors are
stable positive literals (which keeps `AppendTo`'s incremental path free of
callbacks).

Thirteen heads route through it: eleven compound assignments (`AddTo`,
`SubtractFrom`, `TimesBy`, `DivideBy`, `ApplyTo`, `AppendTo`, `PrependTo`,
`Increment`, `Decrement`, `PreIncrement`, `PreDecrement`) plus the `Part`
branches of `AssociateTo` and `KeyDropFrom`. Plain `Set` and association `Set`
were routed through the same driver.

The documented order is in `docs/guide/evaluation.md`: for the binary heads the
second argument is already evaluated once by `HoldFirst` preparation; selectors
then evaluate exactly once, left to right; only after the last selector is the
owner's removal, protection and value state read; the update evaluates once;
each head keeps its return convention.

## What to check

Report each as a finding with `path:line` evidence, or state that you checked
it and found nothing.

1. **Can a raw selector still reach a parser?** The claim is that the type makes
   this impossible. Find the weakest point — a constructor that is more
   permissive than described, a path that rebuilds the typed value from parts,
   a test-only helper with wider visibility — and say whether it holds.
2. **Is the stable-literal constructor's proof actually sufficient?** It exists
   to skip evaluation. If it accepts anything that could evaluate to something
   different — a symbol, a nested call, a negative or non-integer literal — the
   fast path would silently diverge from the evaluated path.
3. **Exactly-once, on every head.** Trace each of the thirteen to the shared
   driver and confirm no head evaluates a selector twice, or zero times, on any
   branch — including its error and early-return branches.
4. **Does a fast path bypass the seam?** `AppendTo`'s incremental path and the
   symbol/`Part` fast paths in association mutation return before the general
   route. Confirm each either does not need selectors or obtains them through
   the typed value.
5. **Is the documented order the implemented order?** Compare
   `docs/guide/evaluation.md` against the code for at least the owner re-read
   after the last selector, and the return conventions (`Increment` and
   `Decrement` return the previous selection; the others the updated one).

## What this review is not

Do not propose refactors, file splits, or performance changes. If you find
nothing, say so — a clean result is a useful outcome and does not need padding.

## Verification

After the findings are written:

```
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-stdlib incidental_definitions_updates
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime part_assignment
```

Paste the results. If a finding needs a probe, write it under `/tmp`, run it
with `stdin < /dev/null`, paste the output, and delete it.

## Hard rules

- Do NOT modify any file in the repository. Do NOT `git commit`, branch, push,
  or touch git config. The worktree must be clean when you finish.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`.
- Report only what you verified. If you suspect something but could not confirm
  it, say which and what would settle it.

## Report format

1. Findings — numbered, each with `path:line` evidence and a one-line statement
   of what would go wrong. Write this section first.
2. Checks that came back clean, listed briefly.
3. Verification output.
4. Anything you could not settle, including any error in this brief's
   background.
