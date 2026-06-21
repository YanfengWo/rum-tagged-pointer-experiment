# Brief 002 — Profile the `downvalue-exact-hit` primitive (diagnostic only)

## Goal

On the corrected, denoised Stage 0 baseline, `downvalue-exact-hit` is the worst
primitive at ~`7.76x` WolframKernel (rum `0.002601s` vs WK `0.000335s` over
2000 iterations, min-of-7). It is the *simplest* definition dispatch: a literal
(non-pattern) downvalue `fExact[1] = 2` looked up by the exact call `fExact[1]`
— no pattern matching, no binding. Yet it is far worse than the corresponding
MISS (`downvalue-exact-miss`, `fExact[2]`, ~`2.81x`). Produce a self-time
profile + caller attribution + counters that explain WHERE rum spends the
per-hit time, and specifically WHAT the hit does that the miss does not.
**No source change.** The deliverable is evidence Claude will use to write the
implementation brief.

## Context pointers

- Read `AGENTS.md` (the unmodified-`mma-hol` hard line) and
  `docs/performance-stage0-baseline.md` (the corrected baseline + the
  harness-faithfulness note) first.
- `docs/proptaut-performance-audit.md` describes the profiling method; reducers
  are `scripts/sample-self-time.py <sample-file> [N]` and
  `scripts/sample-attribute-callers.py <sample-file> <symbol> [N]`.
- Counters: `RUM_PROFILE_EVALUATION_CACHE=1` prints power-of-two checkpoints and
  a final summary for `evaluate_entries`, `down_value_scans`,
  `down_value_applications`, `stability_checks`, `builtin_applications`, and the
  call-stability hit/miss reasons.
- The label is defined in `scripts/core-language-microbench.sh` as
  `fExact[1] = 2` (an exact, literal downvalue — NOT `fExact[x_] := ...`) timed
  as repeated `fExact[1]`.

## Scope

- Files you MAY modify: NONE. This is diagnostic only.
- You MAY create scratch files only under `/tmp`.
- MUST NOT touch any file under `crates/`, `docs/`, `scripts/`, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, any `mma-hol` clone, `.git`, or git config.
- MUST NOT add counters/instrumentation to source; the existing env-gated
  counters are enough. If they are not, STOP and report what you would need.

## Probe (use these EXACTLY — do not modify the shape)

The label statement is the bare `fExact[1]`; the timed loop must run it with NO
per-iteration assignment (see pitfall 2b — a `sink = fExact[1]` injects an
ownvalue `Set` per iteration that is NOT part of this label and contaminated
brief-001). Use a nested loop to stay under rum's single-loop iteration budget.

Timing/sample probe (`/tmp/brief002-hit.wl`), profiling env OFF:

```wolfram
fExact[1] = 2;
Print[AbsoluteTiming[Do[Do[fExact[1], {i, 1, 100000}], {j, 1, 30}]]];
```

Miss control (`/tmp/brief002-miss.wl`), same shape, for the hit-vs-miss delta:

```wolfram
fExact[1] = 2;
Print[AbsoluteTiming[Do[Do[fExact[2], {i, 1, 100000}], {j, 1, 30}]]];
```

If 3,000,000 total iterations runs too long to sample, lower the outer count;
do NOT add a sink to "use" the result, and do NOT raise a single inner loop
above ~1,000,000 (rum's iteration-work limit terminates it).

## Deliverable

1. A `sample` capture of `/tmp/brief002-hit.wl` (release `rum-cli`, profiling
   OFF), reduced with `scripts/sample-self-time.py` (top ~25 self-time rows)
   and `scripts/sample-attribute-callers.py` for the top 2-3 leaf symbols.
2. A counter run of BOTH `/tmp/brief002-hit.wl` and `/tmp/brief002-miss.wl`
   with `RUM_PROFILE_EVALUATION_CACHE=1` (counts only, not timing). Report for
   each: `evaluate_entries`, `down_value_scans`, `down_value_applications`,
   `stability_checks`, `builtin_applications`, call-stability hit/miss reasons,
   and `symbol_invalidations` (this MUST be ~1, not ~per-iteration — if it is
   per-iteration your probe has a stray assignment; fix the probe).
3. The hit-vs-miss comparison: which counters and which hot functions differ
   between hit and miss, expressed per application. The hit finds and evaluates
   the stored RHS; the miss does not. Name what the hit does extra.
4. A one-paragraph mechanism hypothesis: where the per-hit ~`7.76x` goes
   (downvalue lookup/scan, RHS retrieval/clone, re-evaluation of the RHS,
   stability checks on the result, owned-tree movement), naming the specific
   rum functions from the attribution.

## rum / Rust pitfalls (read twice)

1. Build release first: `cargo build --release -p rum-cli`. Sample the release
   binary, never debug.
2. Profiling env OFF for the `sample` capture and any timing; ON only for the
   counter runs. Never mix.
3. **Probe = the label's exact statement. No sink `Set`, no `Print` inside the
   loop, nothing added** (TEMPLATE pitfall 2b). Verify by checking
   `symbol_invalidations` is ~1 in the counter run.
4. `sample` needs a live pid: launch the probe in the background, `sample
   <pid> <seconds> -file <out>`, then wait; the probe must stay alive through
   the window.

## Verification (MANDATORY — produce evidence; no test gate)

Paste VERBATIM: the two probe `.wl` contents; the top ~25 self-time rows; the
attribution tables; both counter summaries (hit and miss) with the per-
application ratios; and the `symbol_invalidations` value for each (proving the
probe is clean).

If `sample` cannot attach or the probe will not stay alive long enough, STOP
and report exactly what failed.

## Hard rules

- NO source change. NO git. NO edits outside `/tmp`.
- Full machine access, but touch nothing outside this repo + `/tmp`, no
  network. Commands: `cargo build`, release `rum-cli`, `sample`,
  `scripts/sample-*.py`, `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: if attribution is inconclusive after two captures, STOP and report
  what you have plus what is missing — do not start changing code.
- Paste real verbatim output; if a step did not produce evidence, say so.

## Report format (your final message)

1. "Diagnostic only, no source change." The two probe `.wl` contents.
2. Verification — verbatim self-time rows, attribution tables, both counter
   summaries + per-application ratios + `symbol_invalidations`.
3. The finding — the hit-vs-miss delta and where the per-hit time goes, naming
   specific rum functions, in one paragraph.
4. Stop-loss / open questions (empty if none).
