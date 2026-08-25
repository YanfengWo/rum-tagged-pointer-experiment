# brief-337 — charge nested iterator counts as a running product

Follow-up to brief-336 (`a19242d1`), which closed the literal-product case and
left this one open.

## What still gets through

```
n = 10^4; Do[0, {i,1,n}, {j,1,n}]      → completes in 7.6 s
n = 10^4; Table[0, {i,1,n}, {j,1,n}]   → completes in 11.3 s, 10^8 elements
n = 10^4; Sum[1, {i,1,n}, {j,1,n}]     → completes in 17.1 s, returns 10^8
Table[0, {i,1,10^4}, {j,1,i}]          → completes in 5.7 s
```

Refused correctly, for contrast:

```
Table[0, {i,1,10^4}, {j,1,10^4}]       → generated element limit, ~5 ms
n = 10^6; Table[0, {i,1,n}]            → generated element limit, ~5 ms
n = 10^7; Do[0, {i,1,n}]               → iteration work limit, ~5 ms
```

## The seam, and why the stated obstacle does not hold

brief-336 attributed this to needing early evaluation of held iterator
specifications. The last two lines above refute that: the single-iterator path
already refuses a count that came from a variable, so the count is in hand
*after* evaluation, before generation.

`checked_iteration_count`
(`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:271`)
is where each iterator's count arrives, already evaluated. It charges that one
count and nothing else, so N iterators each under the limit pass N times.

An inner iterator that depends on an outer one (`{j,1,i}`) is re-evaluated per
outer iteration, so it arrives here repeatedly — which means a running product
covers that case too, and no up-front product ever could.

## What to build

Charge a **running product** rather than each count in isolation: as each
iterator count is established, multiply it into an accumulated total for the
construct and charge the total.

Three design decisions are yours to make and to state:

1. **Where the accumulator lives.** `checked_iteration_count` currently takes
   only `&Session`. Say what you chose and why.
2. **Its scope.** The intended semantics: one top-level iteration construct and
   everything nested inside it share an accumulator; separate statements do
   not. So `Table[0,{i,1,10^4},{j,1,10^4}]` and
   `Table[Table[0,{j,1,10^4}],{i,1,10^4}]` are both refused — they build the
   same 10^8 elements — while `a = Table[…]; b = Table[…]` charges each
   separately. **The nested-call case matters**: without it, two nested single
   iterators reproduce the whole gap. Confirm this is what you built, or argue
   for different semantics.
3. **Reset points.** When an accumulator starts and ends, including on the
   error and suspension paths. A construct that leaves a stale accumulator
   behind will refuse innocent later work, which is worse than the bug.

## What must not change

- **Anything under the limits.** Ordinary nested `Table`, `Do`, `Sum`,
  `Product` at ordinary sizes: same values, same messages, same speed.
- Single-iterator behavior, which is already correct.
- The limit values and the two existing messages.
- Suspension and resume: these constructs can run user code, so a suspended
  loop must resume with its accumulated total intact, not reset.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/`
- the `Session` field or accessor the accumulator needs, if that is your design
- `crates/rum-stdlib/src/control_flow.rs` — only to remove a literal-only check
  that the running product now subsumes; say so if you do
- test modules under `crates/rum-runtime/src/runtime/tests/` and
  `crates/rum-stdlib/src/tests/`

Out of scope: the limit values, the message text, other builtins' budgets.

Do not add unconditional cost to the default evaluation path — a single
iterator at ordinary size must not pay for machinery it does not use.

## Deliverable

- The running product, covering variable counts, inner-depends-on-outer, and
  nested constructs.
- Tests for each of the four measured bypasses above.
- A nested-construct test (`Table[Table[…]]`) proving the accumulator crosses
  call boundaries.
- A test that separate statements do **not** share an accumulator.
- A suspension test: a nested loop that suspends mid-iteration resumes with its
  total intact.
- An under-limit regression set.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — stdin from `/dev/null`.
**Run each probe as its own script**: the CLI stops at the first error, so a
combined script hides everything after the first refusal.

Probe all four bypasses above (must now refuse, and report elapsed time — a
refusal that takes 7 seconds is not a refusal), the contrast cases (must still
refuse as they do today), and an under-limit set (must be unchanged).

**This one does touch the evaluator's iteration path, so run the microbench
A/B** — `scripts/core-language-microbench.sh` on base and candidate, same
window, back to back. Paste the iteration labels and same-tier neighbors, and
state plainly whether the machine was quiet. If the counters do not move, do
not quote a time.

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
- **Stop-loss is real.** If the accumulator cannot be scoped without changing
  under-limit behavior, stop and report rather than shipping something that
  refuses innocent work.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — where the accumulator lives, its scope, and its reset points,
   with line ranges.
2. The three design decisions, stated.
3. Verification — per-probe with elapsed times, the under-limit set, and the
   A/B rows with neighbors.
4. Stop-loss / open questions.
