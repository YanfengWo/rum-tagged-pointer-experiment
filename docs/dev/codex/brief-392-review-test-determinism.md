# brief-392 — review: test determinism and instruments (4 of 4)

Final part of four independent consistency reviews. **Scope: NONE — review
only, no source change.** Reading, grepping and throwaway probes are in scope;
leave no probe files in the repository.

**Write your findings section first, before running any verification
commands.** Keep each numbered section self-contained.

Parts one to three each returned something worth having, and in two of them the
most useful result was correcting the background I had written. **Do that again
if this background is wrong.**

Part two found a typed wrapper forgeable through an exported decoder; part
three found a public installer that stores an unprepared lhs from outside the
crate. **Both are the same shape — a boundary that holds inside the repository
and not across the published surface.** Keep that shape in mind here: this
part's subject is whether the *tests* establish what they claim, and the
analogous failure is a test that holds under the conditions it happens to run
in rather than by construction.

## Background

Briefs 384-388 closed three long-standing items, each of which turned on a test
asserting something the implementation never promised:

- **The ZeroMQ ordering flake.** A test sent a shutdown on the control channel,
  slept, sent an execute on shell, and asserted the *next* socket event was
  control. The poller's priority is correct but selects among channels already
  pending, and native transport buffers on background workers — so the test
  asserted a timing property where the implementation promises a priority one.
  The repair waits until both channels report pending through the same
  predicate the poller uses, then asserts.
- **Syntax error locations.** Spans left the parser in form-local coordinates
  and were rendered against a document over the whole file, so anything after
  the first form reported the wrong line and column. Spans are now in file
  coordinates at the source.
- **The stability-proof flake.** A 128-bit Bloom takes its bit from a
  process-global interner id, so parallel tests shift which symbols collide.
  The stability predicate is one-sided by design — a true proof is
  authoritative, a false one may be conservative — so the tests were asserting
  cache residency the mechanism never guaranteed. The repair pairs an
  identity-bearing target with regular-lane dependencies so the two Bloom lanes
  cannot intersect for any interner order.

Comparison chains were also changed in this window: chains are now collected in
the parser so parentheses bound them, and mixed chains lower to `Inequality`
with no hold attribute, so every operand evaluates exactly once and nothing
short-circuits.

## What to check

Report each as a finding with `path:line` evidence, or state that you checked
it and found nothing.

1. **Is the ZeroMQ repair deterministic by construction, or by timing?** The
   claim is that waiting for both channels to report pending makes the
   assertion independent of arrival order. Check whether anything between that
   wait and the assertion can consume or reorder an event, and whether the
   helper's own deadline can mask a failure rather than surface it.
2. **Does the Bloom lane argument actually hold?** The claim is that an
   identity-bearing target writes only the generated lane while `System`
   symbols and integer keys write only the regular one, so intersection is
   impossible for any interner order. Verify both halves against
   `symbol_uses_generated_stability_bloom` and confirm nothing in the
   construction can introduce a regular-lane dependency on the target's side.
3. **Do any remaining tests depend on interner order, allocation addresses,
   thread scheduling, or binary layout?** These three were found one at a time
   over months. Sweep for the pattern rather than the instances: assertions on
   pointer identity, on cache residency, on evaluation counts that a
   conservative path could change, or on a "next" event.
4. **Are the new comparison-chain tests complete for the contract?** Both
   parenthesis directions, mixed chains with three or more segments, exactly
   -once counting, and the absence of short-circuit. Name anything the contract
   states that no test covers.
5. **Do the syntax-location tests cover the coordinate systems that coincide?**
   A first-form failure was accidentally correct before the repair. Confirm a
   test would now catch a regression that reverted only the later-form case.

## What this review is not

Do not propose refactors, file splits, or performance changes. If you find
nothing, say so — a clean result is a useful outcome and does not need padding.

## Verification

After the findings are written:

```
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-jupyter zeromq::tests::native
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime frame_root_trace
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-syntax
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
