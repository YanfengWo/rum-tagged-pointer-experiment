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
engines through `scripts/run-bounded.py`.

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

## Pair/Simp Baseline Status

The older `scripts/perf-baseline-stage0.sh` Pair/Simp suite remains useful as a
downstream acceptance suite and as historical evidence. It is not the active
Stage 0 entry point. Do not use it to select the next optimization while core
language labels still show unexplained `4-5x` gaps.
