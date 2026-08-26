# brief-356 — `ValueQ` answers proven-stable targets without evaluating

## Goal

Tier 1 of the `ValueQ` work: when the evaluator can already prove the target
would not change, answer `False` immediately — no rollback snapshot, no effect
suppression, no evaluation. Everything else keeps the tier-2 suppressed trial
landed in `b85569c2`.

**No answer may change.** This is purely about reaching the same answer
cheaply for the shapes where a proof already exists.

Today every call pays for the whole session:

```text
ValueQ[qq]   4.6 us with no definitions
             90.6 us at 2 000 definitions
            332.6 us at 8 000
            1.94 ms at 32 000
```

`ValueQ[undefinedSymbol]` is the most common shape there is, and it is exactly
the one the evaluator can already decide.

## The mechanism

The evaluator skips expressions it proves stable
(`eval_core/resolution.rs:321-329` consumes
`cached_expr_is_evaluation_stable`; `eval_core/stability.rs:206-210` handles
plain symbols). brief-346 established that a `true` answer from it is a sound
proof that evaluation returns the expression unchanged — that is precisely
what the evaluator relies on. A `false` answer means only "may change" and
must **not** be turned into `True`; those targets take the existing trial.

So:

```text
if cached_expr_is_evaluation_stable(target):  answer False
else:                                         suppressed trial, as today
```

## Deliverable

- The shortcut, in `eval_value_q`, before the trial is set up.
- **Proof that the shortcut is answer-preserving**: for every shape where the
  stability check says stable, the current implementation also answers
  `False`. Establish this by argument from the code *and* by a differential
  test (below), not by inspection alone.
- Confirm and report whether `cached_expr_is_evaluation_stable` mutates
  anything observable — it is cache-backed, so state plainly what it writes,
  and confirm that a `ValueQ` taking the shortcut leaves no session-visible
  trace it did not leave before.

### Tests

- **A differential test**: over a corpus of targets covering undefined
  symbols, defined own-values, delayed own-values, integers and strings,
  lists, `f[x]` with a matching down-value, with a proven miss, with a
  pattern, with a `Condition` guard, built-in calls (`1 + 1`), nested calls,
  associations, and `Part` forms — assert that the answer equals the answer
  the trial path gives. Drive both paths in the same test so the corpus cannot
  drift out of sync with the implementation.
- Every existing `ValueQ` semantics answer unchanged — the existing suites
  plus `valueq_effect_suppression.rs` are the assertion.
- A counter or equivalent proving the shortcut path captures **no** rollback
  snapshot and performs no suppression.
- The shortcut still answers correctly after an attribute revision bump, since
  stability is cache-backed — assert after `SetAttributes`, per the guard
  line's lesson.

### A/B

`--scale 5`, same-binary noise floor first. Add a stable label for
`ValueQ[undefinedSymbol]` in a session holding a few thousand definitions —
that is the row that should collapse. Also report it at two session sizes to
show the answer is no longer session-scaled.

Controls that must not move:

```
symbol-ownvalue-read   set-ownvalue   downvalue-exact-hit   matcher-single-tiny
```

## Scope

- Files you MAY modify: `messages/builtins_messages_inspect.rs`;
  `eval_core/stability.rs` only if the predicate needs a non-mutating entry
  point; runtime and stdlib tests; `scripts/core-language-microbench.sh`.
- **The general accept/stop criterion**: *any answer change is a stop.*
- MUST NOT touch: the effect suppression landed in `b85569c2`, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **`false` from the stability check is not a proof of anything.** Only the
   `true` branch may shortcut.
2. **Do not extend the stability check** to make more shapes qualify. That is
   a separate design question; this brief takes the predicate as it is.
3. **The predicate is cache-backed.** If consulting it can populate a cache,
   say so and show that this is not observable beyond timing.
4. **No default-path cost** outside `ValueQ`.
5. **Stop-loss is real.** If the differential test finds a divergence, STOP
   and report it — that is a finding about the stability check, not a bug to
   patch around.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file.
2. The answer-preservation argument, and the differential test's corpus and
   result.
3. What the stability predicate writes, and why it is not observable.
4. Verification — each command with verbatim output.
5. The A/B — the collapsed label at two session sizes, the controls named, and
   the same-binary noise floor.
6. Stop-loss / open questions.
