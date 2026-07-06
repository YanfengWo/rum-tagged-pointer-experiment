# Brief 149 — Coverage baseline: host lifecycle and limit paths (DIAGNOSTIC)

DIAGNOSTIC brief — **no source changes**. Opens
`docs/dev/hardening-coverage-plan.md`. The deliverable is a measured,
verified hole list that brief-150 will implement against.

## Goal

Three evidence artifacts, verbatim in the report:

1. **Workspace coverage summary** (region + line) from `cargo llvm-cov`
   (0.6.21, installed; llvm-tools present; NO network).
2. **Focused per-file tables** for:
   - `crates/rum-host/src/**` (every file);
   - the rum-runtime lifecycle/limit surface: session_api / session_helpers
     / session_state / session_eval / session_requests, suspension/*,
     eval_core (the poll/limit paths), file_io, random, capabilities,
     and wherever the resource-limit enforcement lives (find via
     `rg -l 'TerminatedEvaluation|EvalError' crates/rum-runtime/src`).
3. **The hole list**: uncovered or thin (<50% region) FUNCTIONS in those
   files, each verified by opening the source (function name, file:line,
   one line on what it does, and your judgment: real hole / test-only
   plumbing / unreachable-by-design). Plus a `TerminatedEvaluation` tag
   audit: for each reason tag the runtime can produce (recursion,
   iteration/pass, generated-element, integer-work, iteration-work,
   replace-repeated, match-depth, stack, cancel, time-constraint, plus any
   others you find), name the test that exercises it or mark UNTESTED.

## Method

```
cargo llvm-cov --workspace --summary-only 2>&1 | tail -5
cargo llvm-cov --workspace 2>&1 | <filter the per-file table to the focus lists>
# For function-level holes on the focus files:
cargo llvm-cov --workspace --json --output-path /tmp/rum-149-cov.json
# then parse with python3 (persist intermediate tables under /tmp/rum-149-*)
```

- Persist raw outputs to files FIRST, then parse (standing measurement
  hygiene). Budget: instrumented rebuild + one workspace test run —
  bounded, single pass; use `scripts/run-bounded.py --timeout 1800`.
- The tag audit cross-references tests via `rg` over the test trees for
  each tag string; a tag asserted only through an unrelated snapshot does
  not count as exercised — judge and say so.

## Deliverable / Report format

1. The three artifacts above, with the hole list RANKED (highest-value
   first) into a proposed brief-150 checklist: for each hole, the test
   shape you would write (one line each).
2. Anything that looked like dead code or a real bug while reading —
   report-only list.
3. Stop-loss / open questions.

## Scope

- Files you MAY modify: NONE (diagnostic; /tmp artifacts only).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- No source, test, doc, or manifest changes of any kind.
- Real verbatim outputs only; every hole claim verified against the source.
- Stop-loss: if llvm-cov fails to run twice for the same reason, STOP and
  report the error verbatim.
