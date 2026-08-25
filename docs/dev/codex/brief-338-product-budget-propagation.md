# brief-338 — propagate the iteration product budget only where results accumulate

Fixes a regression introduced by `25f123d3` (brief-337). That commit charged
nested iterator counts as a running product, which was right for the case it
targeted and wrong for a case it did not consider.

## The regression

```
                                        base (before 25f123d3)   HEAD
Do[Table[i, {i,1,1000}], {200}]          accepted                refused
Do[Length[Table[i,{i,1,500}]], {500}]    accepted, 6.65 ms       refused
```

Both are refused with **`generated element limit`**, which is the wrong
resource: a `Do` discards each iteration's value, so at most 1000 elements
exist at once. The running product treated 200 sequential rounds of 1000
elements as though they were 200 000 simultaneous ones.

Still correctly refused, and must stay refused:

```
Table[0, {i,1,10^4}, {j,1,10^4}]                  10^8 elements, all retained
n = 10^4; Table[0, {i,1,n}, {j,1,n}]              same
n = 10^4; Table[Table[0,{j,1,n}], {i,1,n}]        same, through nesting
Table[0, {i,1,10^4}, {j,1,i}]                     inner depends on outer
n = 10^4; Do[0, {i,1,n}, {j,1,n}]                 10^8 iterations of work
n = 10^4; Sum[1, {i,1,n}, {j,1,n}]                same
```

## The distinction to implement

Whether a product propagates into a nested construct depends on whether the
enclosing construct **keeps** what the inner one produces:

- `Table` retains every element it generates, so an inner construct's output
  becomes part of the outer result. The product propagates.
- `Do` discards each value; `Sum` and `Product` fold theirs into a scalar.
  Peak element count is the inner construct's own, not the product. The
  generated-element product does **not** propagate into them.

Within a single construct, multiple iterators still multiply as they do now —
that part of `25f123d3` is correct and must not regress.

Note the two limits are not interchangeable. `generated element limit` bounds
elements alive at once; `iteration work limit` bounds work performed.
`Do[…, {i,1,10^4}, {j,1,10^4}]` genuinely performs 10^8 iterations and is
correctly refused by the work limit today. The bug is only that sequential
repetition is being charged against the *element* limit.

## What must not change

- The six cases listed above as correctly refused.
- Everything under the limits, including the two regressed cases, which must
  return to their base behavior and values.
- Suspension and resume: `with_iteration_product_scope` already restores the
  parent product through a suspension frame; keep that.
- The limit values and both message texts.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/`
- `crates/rum-runtime/src/runtime/model/session.rs` and
  `crates/rum-runtime/src/runtime/session/session_state.rs`, if the accumulator
  needs to distinguish the two limits
- test modules under `crates/rum-runtime/src/runtime/tests/` and
  `crates/rum-stdlib/src/tests/`

Out of scope: the limit values, message text, and other builtins' budgets.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Propagation gated on whether the enclosing consumer retains results.
- A regression test for each of the two cases above, asserting the value, not
  just that it does not error.
- The six must-still-refuse cases as tests.
- A test for `Sum`/`Product` wrapping a `Table`, which has the same shape as
  the `Do` case and the same answer.
- A test that `Table` wrapping `Table` still propagates.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — **stdin from
`/dev/null`**, and **run each probe as its own script** (the CLI stops at the
first error, so a combined script hides everything after the first refusal).

Probe both directions: the two regressed cases must succeed with their base
values, and the six refusal cases must still refuse in milliseconds.

**Microbench A/B required** — this is the evaluator's iteration path.
`scripts/core-language-microbench.sh` on base and candidate, back to back in
one window. Its stdin handling was fixed in `dd4bdefc`; if a run produces a
timings file containing only a header, that is the benchmark failing, not a
slow machine — say so rather than re-running it indefinitely.

**Report the load average alongside the numbers; do not wait for the machine to
become quiet.** Judge by comparing against named untouched control labels in
the same run, not against an absolute threshold and not against the suite
median. Whole-run offsets move every label together, so a lopsided direction
split is not by itself evidence. If the control labels move as much as the
iteration labels, that is the noise floor: report no regression.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench script, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Do not change a limit's value or its message text.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If the distinction cannot be drawn without reopening
  one of the six refusal cases, stop and report — a leak back to the original
  bug is worse than the regression being fixed.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — where propagation is decided, with line ranges.
2. The two regressed cases, with values restored.
3. The six refusal cases, still refusing.
4. Verification — per-probe, plus the A/B rows with named control labels and
   the load average.
5. Stop-loss / open questions.
