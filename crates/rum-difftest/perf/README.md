# Performance Baselines

Lane D baseline policy lives here.

These suites are for `rum`'s own regression tracking, not for Wolfram
compatibility oracles.

Current runner:

```bash
cargo run -p rum-difftest -- lane-d
cargo run -p rum-difftest -- lane-d --suite timing --limit 10
cargo run -p rum-difftest -- lane-d --suite memory --fail-on-error
cargo run -p rum-difftest -- lane-d --suite all --profile full
cargo run -p rum-difftest -- lane-d --baseline crates/rum-difftest/perf/baselines/smoke.txt --fail-on-regression
cargo run -p rum-difftest -- lane-d --write-baseline /tmp/rum-lane-d-smoke.txt --fail-on-error
```

The first runner consumes the imported seed suites:

- `seed/ham-import/bench/timing_suite.txt`
- `seed/ham-import/mem/memory_suite.txt`

By default, `lane-d` uses the daily-safe `smoke` profile and skips imported
heavy / very-large categories. `--profile full` is the explicit baseline
collection mode for every imported input.

It reports local evaluation metrics only: elapsed microseconds, result node
count, message count, print count, and any evaluation failure.

Checked baseline artifacts live under [baselines](./baselines/). The first
artifact is [baselines/smoke.txt](./baselines/smoke.txt), covering the default
`smoke` profile. Baseline rows use the same key/value field style as other
`rum-difftest` fixtures:

```text
profile=smoke | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=800 | max_elapsed_micros=250000 | max_value_nodes=1
```

`baseline_elapsed_micros` is historical measurement data. `max_elapsed_micros`
and `max_value_nodes` are the checked guardrails. Keep elapsed thresholds broad
enough to avoid machine-noise failures; tighten them only after repeated
measurements on the intended runner.

`--baseline PATH` enables comparison. `--fail-on-regression` makes missing
baseline rows, stale rows, elapsed regressions, and result-node regressions
exit non-zero. Lane D still does not compare against Wolfram.

`--write-baseline PATH` captures the current run as a baseline artifact. By
default it sets `max_elapsed_micros` to `max(elapsed * 10, 250000)` and records
the observed result node count as `max_value_nodes`. Override those elapsed
guardrails with:

```bash
cargo run -p rum-difftest -- lane-d \
  --write-baseline /tmp/rum-lane-d-smoke.txt \
  --baseline-elapsed-multiplier 20 \
  --baseline-min-elapsed-micros 500000
```

Capture refuses to write a baseline if any executed case fails.
