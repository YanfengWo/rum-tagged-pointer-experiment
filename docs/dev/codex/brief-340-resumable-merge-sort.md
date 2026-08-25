# brief-340 — put the expression sorts on a resumable merge engine (plan-339 steps 1-3)

From `docs/dev/codex/plan-339-user-callback-ordering.md`. Three parts, in
order; the third is the substance and the first two exist to make it gateable.

## The defect

`Sort`, `SortBy` and `ReverseSort` with a custom comparator build a full
all-pairs table. Measured (director's, release CLI):

```
n=50    4.319 ms      n=100   15.408 ms     9 900 calls
n=200  61.326 ms      n=400  239.230 ms    39 800 calls at n=200
n=800 955.985 ms
```

Per doubling: 3.6 / 4.0 / 3.9 / 4.0 — quadratic. The default sort at n=800 is
0.019 ms. It is also order-insensitive (sorted 946.755 ms, reversed 945.313 ms),
so this is not an insertion sort degrading on bad input; every input is
quadratic.

## Part 1 — measurements

Add microbench labels for the custom-comparator forms of `Sort`, `SortBy` and
`ReverseSort`, plus default-path sentinels for the same builtins so a
regression on the fast path is visible.

**Record callback call counts, not only time.** Counters are load-immune and
are the primary evidence here: the claim is that comparisons drop from
`n(n-1)` to at most `n*ceil(log2 n)`, and that is a count, not a duration.

The benchmark script generates one `.rum` file that both binaries run, so
labels added here work for base and candidate alike.

## Part 2 — the contract

**Director's ruling, already made:** a valid strict weak order determines the
result; the number of comparisons, their order, and their direction are **not**
specified. Document it wherever the ordering builtins' behavior is described.

Without this the change is a breaking one by definition, and no sort algorithm
could ever be improved again. It does not license changing *results*: a valid
comparator must still produce the same ordering, and stability is preserved.

## Part 3 — the engine

An iterative, stable, bottom-up merge, generic over the owned item and a small
completion context, in its own module under
`crates/rum-stdlib/src/structure_ops/`. Its owned state carries the ordering
expression and result policy, the runs for the current pass, the cursor in each
run, the partial merged output, and the next-pass runs.

Rules that matter:

- **Stability is choosing left on a tie.** Compare the right candidate against
  the left; move the right item only when the callback says it strictly
  precedes.
- **Draining a remaining run costs no callback calls.**
- **A loop, not native recursion**, advances runs and passes.
- `SortBy` keeps its existing input-order, exactly-once key decoration before
  entering the driver.
- `ReverseSort` uses the same ascending engine and reverses at completion,
  matching its documented "sort, then reverse" shape.
- **Default branches keep using the native sort, untouched.**

**The suspension pattern is the part most likely to go wrong.** Build the
pending callback expression, call `session.evaluate`, and on `Ok` apply the
result directly — **do not construct or clone a continuation snapshot on the
success path**. Only on `EvalError::InputRequired` move the single owned merge
state into the suspend frame. Propagate any other error unchanged. The eager
whole-state cloning in the Association insertion code is the anti-pattern to
avoid; the existing list-sort loops already pattern-match `InputRequired`
correctly, so keep that part while replacing the algorithm around it.

## Tests that currently pin the defect

Existing tests assert `n(n-1)` comparisons. **Those assertions encode the
quadratic behavior as an expectation and must be replaced, not weakened** —
the new bound is `calls <= n*ceil(log2 n)`, asserted at a non-power-of-two
width so an off-by-one in the pass structure cannot hide.

## What must not change

- **Results for any valid comparator**, including stability of ties.
- Default-path sorting: same algorithm, same speed.
- `SortBy`'s key evaluation: same keys, same order, exactly once.
- Suspension and resume: a comparator that suspends must resume with the same
  partial state and produce the same final value as an uninterrupted run.
- `docs/reference/builtins.md` unless a catalog spec changes; if it does,
  regenerate and say why.

## Scope

Files you MAY modify:

- new module(s) under `crates/rum-stdlib/src/structure_ops/`
- `crates/rum-stdlib/src/structure_ops/mod.rs`, `ordering_sets.rs`,
  `collection_stats.rs`
- `scripts/core-language-microbench.sh`
- test modules under `crates/rum-stdlib/src/tests/`
- the guide page documenting ordering behavior

Out of scope: the Association custom-ordering paths (`KeySort`, `KeySortBy`) —
they are the next brief and depend on this engine. The matcher, the evaluator,
and the default sort implementations.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The three parts above.
- Call-count tests at the new bound, replacing the quadratic assertions.
- Stability tests with tagged ties for all three forms.
- A suspension test: a comparator that suspends on every comparison, resumes
  repeatedly, and yields the same result and the same comparison trace as an
  uninterrupted run.
- A wide input proving no native-frame growth.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — stdin from `/dev/null`,
and **run each probe as its own script** (the CLI stops at the first error).

Report call counts and times for all three forms at n=50/100/200/400/800.
**The call-count drop is the primary evidence**; times support it.

**Microbench A/B required.** Base and candidate back to back in one window.
**Report the load average alongside the numbers; do not wait for a quiet
machine.** Judge against named untouched control labels in the same run — not
the suite median, not an absolute threshold. Whole-run offsets move every label
together, so a lopsided direction split is not by itself evidence; if the
controls move as much as the targets, that is the noise floor.

If a timings file comes back containing only a header, the benchmark failed —
that was a stdin bug fixed in `dd4bdefc`, so investigate rather than re-running
it indefinitely.

Note that iteration constructs now charge a product budget, so a benchmark loop
wrapping a generating construct can hit the generated element limit. Size
probes accordingly; that is expected behavior, not something to work around.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench script, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.**
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- **Every probe needs an assertion that the work happened.** A call-count check
  belongs in the probe itself. A probe measuring a path that silently did
  nothing has already cost this line a full round.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If the merge cannot preserve suspension semantics,
  stop and report rather than shipping a sort that loses state on resume.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the engine's state and loop structure, with line ranges.
2. The call-count table, before and after, for all three forms.
3. Which quadratic assertions you replaced and with what.
4. Verification — probes, the A/B rows with named controls, and the load
   average.
5. Stop-loss / open questions.
