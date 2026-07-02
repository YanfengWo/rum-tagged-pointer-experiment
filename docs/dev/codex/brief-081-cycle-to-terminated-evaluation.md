# brief-081 — genuine evaluation cycle → TerminatedEvaluation[IterationLimit] (conformance)

## Goal

A genuine evaluation cycle (`EvalError::EvaluationCycleDetected`) currently surfaces ONLY
as a rum-specific `General::cycle` message and propagates as a hard error (it can abort a
loader mid-file — the class of bug brief-051 hit). WK has no "cycle" message: the same
non-terminating input hits WK's iteration limit. rum already represents iteration-limit
exhaustion (`EvalError::EvaluationPassLimitExceeded`) as the result expression
`TerminatedEvaluation[IterationLimit]` (see `terminated_evaluation_reason_for_error` in
`crates/rum-runtime/src/runtime/session_helpers.rs:205`). Make a genuine cycle produce the
SAME outcome — `TerminatedEvaluation[IterationLimit]` as its result — instead of the
anomalous cycle-message-and-abort. This unifies non-termination handling and is more robust
(a cycle in one expression returns a `TerminatedEvaluation` result rather than aborting the
whole evaluation/loader). KEEP the early cycle DETECTION (a safety/perf feature); only change
what a detected cycle PRODUCES.

## Method (calibrate, then route through the existing path)

1. **Calibrate WK** (real `wolframscript`; save raw output to a file, then read) for a few
   genuine cycles so you can confirm WK's class + document the mapping in the report, e.g.:
   - `ClearAll[a, b]; a := b; b := a; a`
   - `ClearAll[c]; c := c + 0; c` (if it cycles rather than fixed-points)
   - any other shape rum's detector catches.
   WK will hit `$IterationLimit::itlim` (iteration limit) — confirm that is the class (NOT a
   distinct WK "cycle" concept). rum's chosen representation of that class is
   `TerminatedEvaluation[IterationLimit]` (already used for the pass limit); we align cycle
   to it.
2. **Route `EvaluationCycleDetected` through the iteration-limit outcome:** add it to
   `terminated_evaluation_reason_for_error` returning `Some("IterationLimit")` (same as
   `EvaluationPassLimitExceeded`), so a detected cycle yields the
   `TerminatedEvaluation[IterationLimit]` RESULT and no longer propagates as a hard
   error/abort. Align the emitted message with the iteration-limit class rather than the
   WK-nonexistent `General::cycle` (prefer reusing the pass-limit message path for
   uniformity); confirm the exact message keeps the mma-hol suite byte-identical (below).
3. **Preserve brief-051's benign self-fixed-point:** `a := a` / ownvalue `a :> a` must still
   RETURN `a` (NOT trip the cycle path). Add/keep a test for it. Confirm the arith_tests
   corpus (the brief-051 regression) still passes.
4. Trace the actual flow — `EvaluationCycleDetected` is raised in `resolution.rs` and may be
   caught/propagated differently than the pass limit; make sure it reaches the
   TerminatedEvaluation result path (not an earlier hard-error return). Explain the flow in
   the report.

## No-regression gate (ABSOLUTE — this changes an eval-error surface)

```
cargo fmt --check
cargo test -p rum-runtime            # green; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # cold-lane byte-identical
```
Plus a representative mma-hol run byte-identical to before (a cycle surface change must not
alter the acceptance suite — run the established real-tower/heavy-prover lane the repo uses
and confirm PASS byte-identical). Paste evidence.

Add rum-runtime tests (calibrated where WK applies):
- a genuine cycle (e.g. `ClearAll[a,b]; a:=b; b:=a; a`) → result is
  `TerminatedEvaluation["IterationLimit"]` (assert the FullForm/result), no abort;
- the benign self-fixed-point (`a:=a; a` and ownvalue `a:>a`) → returns `a` (brief-051
  preserved);
- (if any) a loader-style multi-expression case: a cycle in one expression does NOT abort
  the surrounding evaluation.

## Hard rules

- No `git commit`/branch/push. Never edit any `mma-hol` file. No new `unsafe`. No
  `Expr`/serde shape change.
- **Absolute no-regression:** workspace + mma-hol byte-identical. If aligning the message
  regresses the mma-hol suite, keep the message minimal-change but STILL produce the
  `TerminatedEvaluation[IterationLimit]` result, and report the tradeoff.
- Keep early cycle DETECTION; only change the produced outcome. Preserve brief-051.
- WK is the oracle (real `wolframscript`); save raw output to a file then parse.
- Paste REAL verbatim output; never fake a green.

## Report format

1. WK calibration — the genuine-cycle inputs + WK's exact message/result (verbatim), and the
   mapping to rum's `TerminatedEvaluation[IterationLimit]`.
2. What changed — the code edit (session_helpers + wherever the flow needed it), and how a
   detected cycle now reaches the TerminatedEvaluation result instead of aborting.
3. Gate — `cargo test -p rum-runtime` + `--workspace` + mma-hol byte-identical evidence,
   verbatim; the new tests.
4. Open questions — anything about the message alignment or a cycle shape that didn't map
   cleanly.
