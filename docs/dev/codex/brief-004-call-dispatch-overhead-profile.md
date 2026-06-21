# Brief 004 — Profile the shared call-dispatch overhead via `if-true` (diagnostic)

## Goal

On the corrected baseline, `if-true` (~`7.2x` WK) and `downvalue-pattern-hit`
(~`7.1x`) are the worst remaining primitives. `if-true` is the cleanest
ISOLATOR of pure call-dispatch overhead: `If[True, 1, 2]` hits the existing
literal-`True` fast path in `eval_if`, has no downvalues, and runs no pattern
matcher — so its cost is almost entirely the shared machinery EVERY call pays
(head evaluation, dispatch-plan/stage setup, argument preparation). That shared
constant sits under all the 3-7x ratios, so finding its fattest function is the
highest-leverage next diagnosis. Produce a self-time profile + caller
attribution of the `If[True, 1, 2]` call path and name the fattest shared-
dispatch functions, plus any clearly wasteful step. **No source change.**

## Context pointers

- Read `AGENTS.md` and `docs/performance-stage0-baseline.md` first.
- Reducers: `scripts/sample-self-time.py <sample-file> [N]` and
  `scripts/sample-attribute-callers.py <sample-file> <symbol> [N]`.
- A profiled per-phase view (relative only, absolute ns inflated) already
  exists via `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` printing
  `rum-profile runtime-path-self-timing phase=... self_avg_ns=...`. On
  `if-true` the heavy phases are `prepare_arguments.total`,
  `evaluate_call.dispatch`, `call.stage.*`, and `evaluate_call.head_eval`. Use
  this only to corroborate; the AUTHORITATIVE evidence is the UNPROFILED
  `sample` self-time below.
- Dispatch path is `evaluate -> resume_call_with_dispatch_plan ->
  run_call_dispatch_stage -> (prepare arguments / builtin / down_values
  stages)` in `crates/rum-runtime/src/runtime/eval_core.rs`,
  `eval_dispatch/mod.rs`, `eval_call_prepare.rs`, `control_flow/branching_loops.rs`.
- `eval_if` already fast-paths a literal `True` condition, so the work is NOT
  inside `If`'s branch logic; it is the call overhead to reach it.

## Scope

- Files you MAY modify: NONE (diagnostic). Scratch files only under `/tmp`.
- MUST NOT touch `crates/`, `docs/`, `scripts/`, `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, any `mma-hol` clone, `.git`. MUST NOT add source instrumentation.

## Probe (use EXACTLY — pitfall 2b, no sink Set)

The label is the bare `If[True, 1, 2]`; nested loop to stay under the
single-loop iteration budget. Profiling env OFF for the sample/timing.

`/tmp/brief004-iftrue.wl`:

```wolfram
Print[AbsoluteTiming[Do[Do[If[True, 1, 2], {i, 1, 100000}], {j, 1, 30}]]];
```

Baseline isolator `/tmp/brief004-loopempty.wl` (the same loop with NO call, to
separate loop/harness overhead from call-dispatch overhead):

```wolfram
Print[AbsoluteTiming[Do[Do[Null, {i, 1, 100000}], {j, 1, 30}]]];
```

If 3,000,000 iterations is too long to sample, lower the outer count; do NOT
add a sink, do NOT raise a single inner loop above ~1,000,000.

## Deliverable

1. `sample` capture of `/tmp/brief004-iftrue.wl` (release `rum-cli`, profiling
   OFF) reduced with `scripts/sample-self-time.py` (top ~25 self-time rows),
   and `scripts/sample-attribute-callers.py` for the top 3-4 leaf symbols.
2. The same self-time top for `/tmp/brief004-loopempty.wl`, so the loop/harness
   overhead can be subtracted: the functions that appear in `if-true` but NOT
   (or far less) in `loop-empty` are the call-dispatch cost.
3. A ranked list of the fattest SHARED call-dispatch functions for
   `If[True, 1, 2]` (head evaluation, dispatch-plan build, stage dispatch,
   argument preparation, atom/stability fast paths), each with its self-time
   share and nearest meaningful caller.
4. A one-paragraph finding: where the per-call shared overhead goes, naming
   specific rum functions, and flagging any step that looks wasteful for a
   trivial call (e.g. re-evaluating a self-evaluating head symbol every call, a
   dispatch stage doing setup for a symbol that has no downvalues, redundant
   stability checks, or per-call cloning in argument preparation).

## rum / Rust pitfalls (read twice)

1. Build/sample the RELEASE binary; profiling env OFF for the sample.
2. Probe = the label's exact statement; no sink `Set` (pitfall 2b).
3. `sample` needs a live pid; background the probe, `sample <pid> <secs> -file
   <out>`, then wait.
4. Subtract the `loop-empty` baseline before attributing a function to
   call-dispatch — `resume_do_iterations` and the loop machinery are NOT
   call-dispatch cost.

## Verification (MANDATORY — evidence, no test gate)

Paste VERBATIM: both probe `.wl` contents; the top ~25 self-time rows for BOTH
`if-true` and `loop-empty`; the attribution tables for the top `if-true` leaves;
and the ranked shared-dispatch list with the `loop-empty` baseline subtracted.

If `sample` cannot attach or the probe will not stay alive long enough, STOP and
report what failed.

## Hard rules

- NO source change. NO git. NO edits outside `/tmp`. Repo + `/tmp` only, no
  network. Commands: `cargo build`, release `rum-cli`, `sample`,
  `scripts/sample-*.py`, `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: if attribution is inconclusive after two captures, STOP and report
  what you have plus what is missing.
- Paste real verbatim output; if a step produced no evidence, say so.

## Report format

1. "Diagnostic only, no source change." Both probe `.wl` contents.
2. Verification — verbatim self-time rows (if-true and loop-empty),
   attribution tables, the baseline-subtracted shared-dispatch ranking.
3. The finding — where the per-call shared overhead goes, naming functions, and
   any wasteful step, in one paragraph.
4. Stop-loss / open questions (empty if none).
