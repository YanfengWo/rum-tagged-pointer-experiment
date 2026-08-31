# brief-396 — the review's remaining four items

The last of the brief-389 to 392 findings. Three of the four are test
robustness; one is a documentation correction. They are unrelated to each other
and are batched only because each is small — **treat them as four independent
items and report them separately.** If one turns out to be larger than it
looks, stop on that one and report rather than compressing it.

## Item 1 — cancellation tests synchronize with sleeps

Top-level evaluation clears a pending interrupt
(`session/session_helpers.rs:30`), and four tests start a canceller that sleeps
and then cancels, before calling evaluation:
`rum-host/tests/lifecycle_contract.rs:118`,
`rum-host/src/tests/eval_contract.rs:210`, `rum-host/tests/stress.rs:52`, and
`runtime/tests/session_requests/api.rs:502`.

If the main thread is delayed past the worker's sleep, the cancellation lands
before evaluation starts and is then erased, and the test fails. Not yet
observed failing; it is the same shape as the flakes already closed.

Make the canceller synchronize on evaluation having started rather than on a
duration. If a test genuinely needs to cancel before evaluation begins, that is
a different test and should say so.

## Item 2 — native IOPub tests rest on a fixed propagation delay

Four tests sleep 50 ms for PUB/SUB propagation:
`zeromq/tests/native/signed.rs:24`,
`zeromq/tests/native/frontend/execute.rs:24`,
`zeromq/tests/native/frontend/stdin.rs:39`, and
`zeromq/tests/native/frontend/session_flow.rs:48`. The pinned zeromq 0.5.0
queues subscriptions without publisher acknowledgement and PUB sends only to
subscriptions already processed, so a delayed subscription drops early frames
and the two-second receive loops time out.

This was reached by reading the dependency, not by reproduction — the native
tests skip when loopback binding is unavailable, which was the case in the
reviewing environment. **Establish first whether they run in yours**; if they
skip, say so and reason from the dependency as the review did.

Replace the duration with something that establishes the subscription is live —
an acknowledged round trip before the assertions begin, or an equivalent. A
longer sleep is not a repair.

## Item 3 — two stated parts of the comparison-chain contract have no test

The contract says a uniform chain keeps its n-ary head and every operand
evaluates exactly once. No test lowers a surface uniform-`Less` chain
(`a < b < c` written as source, not constructed), and none counts the leftmost
operand. A uniform-relational lowering regression or a leftmost double
evaluation would escape the suite. Add both.

## Item 4 — the selector order contract is stated unconditionally

`docs/guide/evaluation.md:168` says selectors evaluate exactly once. brief-390
found `AssociateTo` returning before selector handling when its update
specification is invalid (`associations/mutation.rs:166`), so they evaluate
zero times on that branch.

**The code is right and the document is wrong.** The specification is an
ordinary evaluated argument; a call that fails validation should not produce
partial selector side effects. Reword the contract to say the selectors
evaluate once when the mutation reaches target selection, and check whether any
neighbouring sentence in that section has the same unconditional shape.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

Per item:

- **Item 1**: show the synchronization is on evaluation state, not elapsed
  time. If you can construct the interleaving that fails today — delay the main
  thread past the worker's sleep — demonstrate it before and after.
- **Item 2**: state whether the native tests run or skip in your environment.
  If they run, repeat the four tests enough times to say something about
  stability, and report the count. If they skip, say so plainly; a green run of
  skipped tests is not evidence.
- **Item 3**: the two new tests, with the leftmost operand's counter.
- **Item 4**: the reworded contract, and any neighbouring sentence you also
  corrected.

## Scope

Files you MAY modify: the four cancellation tests, the four native IOPub tests
and their support module, the comparison-chain test module, and
`docs/guide/evaluation.md`.

**No production behavior may change.** All four items are test or documentation
work. If any of them cannot be done without touching production code, STOP and
report which and why.

**The MAY list is a floor, not a ceiling** for purely mechanical ripple — a
test-support signature, a helper's visibility. List each under "mechanical
ripple". Anything else unlisted: STOP and report.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`.
- **A longer sleep, a bigger timeout, or a retry is not a repair** for items 1
  or 2. If an item cannot be made deterministic, report that rather than
  lowering its failure rate.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

One numbered section per item, each self-contained: what you found, what you
changed, and its verification. Then `fmt` and workspace counts, mechanical
ripple, and stop-loss or open questions.
