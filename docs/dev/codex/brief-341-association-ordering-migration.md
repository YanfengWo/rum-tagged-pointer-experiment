# brief-341 — move Association custom ordering onto the merge engine (plan-339 step 4)

Final step of `docs/dev/codex/plan-339-user-callback-ordering.md`. The engine
landed in `6076867b`; this migrates the Association ordering paths onto it.

## The defect — a crash, not a slowdown

Measured today (release CLI, verified by the director — do not re-derive):

```
KeySort[assoc, comparator]              n=100  ok, 38 ms
                                        n=110  stack overflow, exit -6
                                        n=150  stack overflow
KeySortBy[assoc, keyfn, comparator]     n=50   ok
                                        n=100  stack overflow, exit -6

KeySort[assoc]  (default)               n=2000 ok, 11 ms
```

Comparisons are exactly quadratic: at n=50, `KeySort` with a comparator makes
1225 calls, which is n(n-1)/2.

**An Association of 110 keys cannot be sorted with a custom comparator.** The
cause is recursive insertion with eager whole-state cloning per step — the same
anti-pattern `6076867b` was written to avoid, which is why that engine exists
before this migration rather than after.

## What to build

Replace the recursive `resume_key_sort_with_ordering_*` and custom-ordering
`resume_key_sort_by_ordering_*` insertion functions with adapters to the shared
merge engine.

- **`KeySort`** compares the entry key.
- **Three-argument `KeySortBy`** evaluates one key per entry first, left to
  right, then sorts the decorated entries. Use the same "move owned state only
  on `InputRequired`" pattern the engine uses; do not clone a snapshot on the
  success path.
- **Default `KeySort` and default/key-only `KeySortBy` stay exactly as they
  are.** They are not the defect and must not move.

Also add microbench labels for the Association ordering forms — `6076867b`
added them only for the expression sorts, so there is currently nothing to A/B
these against. Include default sentinels.

## Three things that must survive the migration

1. **Rendering through the entry-form path.** Rule versus delayed-rule heads,
   and Association versus rule-list output, are observable and covered by
   existing behavior tests. Render through
   `render_association_entry_forms_preserving_heads` rather than rebuilding.
2. **The comparator-result interpretation, byte for byte.** `1`, `0`, `-1`,
   `True`, `False` and non-Boolean results all have a current meaning. **This
   step changes which comparisons happen and in what order — not what a
   returned value means.** Keep the decision function as it is.
   (Director's ruling: that policy is preserved as-is, not promoted to a
   documented feature.)
3. **Exactly-once key evaluation in input order.** The schedule contract from
   `6076867b` allows comparison count and order to change; it does **not**
   relax key evaluation, which stays exactly once per entry, left to right.

## What must not change

- Results for any valid comparator, including tie stability.
- Default `KeySort` / `KeySortBy`: same algorithm, same speed.
- Suspension and resume, for both the comparator and the key function.
- `docs/reference/builtins.md` unless a catalog spec changes.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/associations/resumable/key_sort.rs`
- `crates/rum-stdlib/src/associations/transforms.rs` and
  `crates/rum-stdlib/src/associations/resumable.rs` if wiring requires it
- `crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs` — only if the
  engine needs a genuinely general extension; say so and justify it
- `scripts/core-language-microbench.sh`
- test modules under `crates/rum-stdlib/src/tests/`

Out of scope: the expression sorts (done), the default paths, the matcher.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Both custom-ordering paths on the engine, recursion and eager cloning gone.
- Association and rule-list inputs; immediate and delayed rules; ascending,
  descending, ties, transformed ties.
- A key-evaluation trace test: exactly once per entry, input order.
- The full comparator-result interpretation set, unchanged.
- A comparison-count bound test, and a width well above 200 proving both
  overflows are gone.
- Suspension tests for the comparator on every chosen comparison **and** for
  the key function at every key, each asserting identical final value and trace
  against an uninterrupted run.
- Microbench labels plus default sentinels.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — stdin from `/dev/null`,
and **run each probe as its own script** (the CLI stops at the first error).

Re-run the two crash probes above at their measured widths and well beyond;
they must complete. **An exit code of -6 or 134 is a failure of this brief.**
Report comparison counts against the bound at several widths — counts are the
primary evidence, times support them.

**Microbench A/B required.** Base and candidate back to back in one window.
**Report the load average alongside the numbers; do not wait for a quiet
machine.** Judge against named untouched control labels in the same run, not
the suite median. Whole-run offsets move every label together, so a lopsided
direction split is not by itself evidence.

If a timings file comes back containing only a header, the benchmark failed —
a stdin bug fixed in `dd4bdefc` — so investigate rather than re-running it.

Note that iteration constructs charge a product budget, so a benchmark loop
wrapping a generating construct can hit the generated element limit. Size
probes accordingly.

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
- **Every probe needs an assertion that the work happened** — a call count or a
  result length in the probe itself. A probe measuring a path that silently did
  nothing has cost this line two full rounds already.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If the entry-form rendering or the result-policy
  interpretation cannot be preserved through the engine, stop and report rather
  than changing observable output to fit the algorithm.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the adapters and how key decoration is staged, with line
   ranges.
2. The crash probes, now completing, with widths and counts.
3. The comparison-count table against the bound.
4. Verification — probes, the A/B rows with named controls, and the load
   average.
5. Stop-loss / open questions.
