# Performance Stage 0 Baseline

Stage 0 is the bottom-up performance baseline for the kernel rewrite. It is no
longer a Pair/Simp-first suite. Pair/Simp remains valuable downstream evidence,
but the active Stage 0 surface is the smallest shared Wolfram-language
statements that both rum and WolframKernel can execute.

## Runner

Use:

```bash
bash scripts/core-language-microbench.sh \
  --engine both \
  --output-dir /tmp/rum-core-microbench
```

The runner writes:

- `core-language-microbench.wls`: the generated benchmark source.
- `runs.tsv`: one row per engine execution.
- `measurements.tsv`: parsed `RUM_CORE_MICRO` rows with label, iterations, and
  seconds.
- `ratios.tsv`: joined rum/WolframKernel timings by label.
- `raw/`: stdout and stderr logs.

For a quick rum-only smoke while editing the harness:

```bash
bash scripts/core-language-microbench.sh --engine rum --scale 1 --timeout 120
```

The script disables rum profiling environment variables by default and runs
engines through `scripts/run-bounded.py`. Each label is timed `--reps` times
(default `7`) and the MINIMUM is reported, so sub-millisecond labels are not
dominated by scheduling jitter; raise `--reps` if a label is still unstable.

For a rum-local regression gate over the same primitive families, use Lane D:

```bash
cargo run -p rum-difftest -- lane-d \
  --suite core-language \
  --summary-json /tmp/rum-core-language-lane-d.json
```

This does not replace the cross-engine script. It keeps the current
time-efficiency work inside `rum-difftest`'s baseline capture, comparison, and
summary tooling so local optimizations can gain a checked rum baseline before
or after a WolframKernel ratio run.

## Initial Label Set

The first label set intentionally starts with ordinary language mechanics
rather than HOL-specific proof code:

| tier | labels |
| --- | --- |
| harness | `loop-empty`, `do-count-null`, `do-iterator-null`, `table-count-null`, `table-iterator-symbol` |
| atoms and symbols | `literal-integer`, `literal-string`, `symbol-ownvalue-read`, `set-ownvalue` |
| structure | `list-literal-small`, `head-list`, `length-list`, `part-list`, `sameq-integer`, `sameq-list` |
| control and frames | `if-true`, `module-local`, `block-local`, `with-local`, `table-small`, `do-small` |
| definitions | `downvalue-exact-hit`, `downvalue-pattern-hit`, `downvalue-pattern-miss` |
| matching and rewrite | `matchq-integer-success`, `matchq-integer-fail`, `replaceall-simple` |
| small builtins | `string-repeat-small`, `character-range-small` |

Labels may be added as soon as a real downstream statement reduces to a
missing primitive. Do not add a larger composite benchmark until the smaller
tier beneath it has been measured.

## Acceptance Use

For each performance cut:

- run the core microbench before the change or cite the latest compatible
  baseline;
- select the lowest-tier label with a clear ratio problem;
- optimize only the runtime mechanism needed by that label;
- rerun the same label and neighboring labels after the change;
- run focused semantic tests for the changed runtime area.

Treat a `4-5x` ratio on a primitive label as the next mainline target unless a
lower-tier label is already worse or the measurement is below timing noise.
Use larger `mma-hol` and Pair/Simp probes only after the primitive result has
been handled.

## Harness Faithfulness (corrected 2026-06-21)

The Stage 0 harness was re-verified after a contaminated probe surfaced in
Codex's brief-001. Two foundation issues were found and both are now fixed.

1. **Per-iteration `Set` contamination (FIXED).** `rumMicroBench` timed
   `Do[result = expr, {n}]` — an ownvalue `Set` to a local every iteration.
   A three-way control (pure `Do[expr]` vs local-`Set` vs global-`Set`, 1M
   iters) showed the `Set` form costs `6.6s` vs the pure `4.2s` and injects
   ~1M `symbol_invalidations` vs `1`. Because rum's Set triggers cache
   invalidation while a WK local write is cheap, this inflated rum more than
   WK and **deflated the rum/WK ratio on the cheapest primitives** (the `Set`
   was a larger share of WK's tiny per-iteration time). The timed loop now
   runs the bare `expr`.

2. **Measurement noise on sub-millisecond labels (FIXED via min-of-N).**
   Run-to-run jitter was <8% on slow targets but 20-78% on the fast and
   `*-repeat` labels. The harness now times each label `rumMicroReps` times
   (default `7`, `--reps`) and reports the MINIMUM, which rejects OS scheduling
   spikes. Residual run-to-run jitter is now <8% on every decision-grade label
   (1/38 labels, the `loop-empty` overhead baseline, sits at ~11%). The
   `*-repeat` labels — the evidence cited for the recent cache-stable-miss
   optimizations — went from ~71-78% jitter to <4%, so those optimizations can
   now be re-credited against a stable baseline.

Corrected, denoised baseline (harness fix + min-of-7, both engines at their
cleanest), worst-first decision-grade targets:

| label | clean ratio | was (contaminated) |
| --- | ---: | ---: |
| `downvalue-exact-hit` | `7.76` | `4.63` |
| `if-true` | `7.20` | `4.39` |
| `downvalue-pattern-hit` | `7.07` | `5.18` |
| `sameq-integer` | `6.33` | `4.17` |
| `block-local` / `with-local` | `~5.3` | `~4.4` |

The contaminated baseline both **mis-ranked** the targets (it put
`downvalue-pattern-hit` worst; the true worst are `downvalue-exact-hit` and
`if-true`) and **understated the gap** by roughly half on the cheapest
primitives. Re-select the next optimization target from this corrected table.

## Pair/Simp Baseline Status

The older `scripts/perf-baseline-stage0.sh` Pair/Simp suite remains useful as a
downstream acceptance suite and as historical evidence. It is not the active
Stage 0 entry point. Do not use it to select the next optimization while core
language labels still show unexplained `4-5x` gaps.
