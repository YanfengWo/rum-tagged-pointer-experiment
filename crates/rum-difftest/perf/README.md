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
cargo run -p rum-difftest -- lane-d --capture-runs 5 --write-baseline /tmp/rum-lane-d-smoke.txt --fail-on-error
cargo run -p rum-difftest -- lane-d --threshold-profile ci-debug --capture-runs 5 --write-baseline /tmp/rum-lane-d-ci-debug.txt --fail-on-error
cargo run -p rum-difftest -- lane-d --check-baseline-readiness
```

The first runner consumes the imported seed suites:

- `seed/ham-import/bench/timing_suite.txt`
- `seed/ham-import/mem/memory_suite.txt`

By default, `lane-d` uses the daily-safe `smoke` profile and skips imported
heavy / very-large categories. `--profile full` is the explicit baseline
collection mode for every imported input.

`--threshold-profile` selects which baseline threshold records are compared and
which capture defaults are used:

- `local-debug`: local developer debug runs, using `observed * 10` and a
  `250000` microsecond minimum guardrail.
- `ci-debug`: debug builds on shared CI machines, using `observed * 25` and a
  `1000000` microsecond minimum guardrail.
- `ci-release`: release builds on CI/release machines, using `observed * 15`
  and a `500000` microsecond minimum guardrail.

It reports local evaluation metrics only: elapsed microseconds, result node
count, message count, print count, and any evaluation failure.

Checked baseline artifacts live under [baselines](./baselines/). The first
artifact is [baselines/smoke.txt](./baselines/smoke.txt), covering the default
`smoke` profile. Baseline rows use the same key/value field style as other
`rum-difftest` fixtures:

```text
profile=smoke | threshold_profile=local-debug | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=800 | max_elapsed_micros=250000 | max_value_nodes=1
```

`baseline_elapsed_micros` is historical measurement data. `max_elapsed_micros`
and `max_value_nodes` are the checked guardrails. Keep elapsed thresholds broad
enough to avoid machine-noise failures; tighten them only after repeated
measurements on the intended runner. Baseline rows without `threshold_profile`
are treated as `local-debug` for compatibility with older artifacts.

`--check-baseline-readiness` checks the default checked smoke baseline artifact
set and exits non-zero if any expected file is missing, invalid, or no longer
covers the current smoke lane. Text output and `--summary-json PATH` include
one readiness entry per expected artifact plus a suggested `lane-d
--write-baseline` command for each artifact, so CI runners can surface the
exact capture command for missing `ci-debug` / `ci-release` baselines. The
`ci-release` suggestion uses `cargo run --release` to avoid capturing a release
threshold profile from a debug build, and JSON entries expose `cargo_profile`
as `debug` or `release` for CI dashboards. The current default artifact paths
are:

- [baselines/smoke.txt](./baselines/smoke.txt): `local-debug`
- `baselines/smoke-ci-debug.txt`: `ci-debug`, to be captured on the intended
  debug CI runner
- `baselines/smoke-ci-release.txt`: `ci-release`, to be captured on the
  intended CI/release runner

Do not synthesize CI artifacts from a local developer machine just to satisfy
the readiness gate; missing CI paths are an accurate signal that runner-specific
capture is still pending.

`--baseline PATH` enables comparison. `--fail-on-regression` makes missing
baseline rows, stale rows, elapsed regressions, and result-node regressions
exit non-zero. Lane D still does not compare against Wolfram.

`--write-baseline PATH` captures the current run as a baseline artifact.
`--capture-runs N` repeats the same Lane D run `N` times before writing the
artifact; captured `baseline_elapsed_micros` uses the median elapsed sample for
each case, while `max_elapsed_micros` uses the slowest observed sample times
the configured multiplier. The capture path validates that every run produced
the same case sequence. Capture-only guardrail controls such as
`--capture-runs`, `--baseline-elapsed-multiplier`, and
`--baseline-min-elapsed-micros` require `--write-baseline PATH` so calibration
settings cannot be silently ignored.

By default capture sets `max_elapsed_micros` to `max(observed * 10, 250000)`
for `local-debug` and records the largest observed result node count as
`max_value_nodes`. Select CI/release guardrail defaults with
`--threshold-profile`, or override elapsed guardrails explicitly with:

```bash
cargo run -p rum-difftest -- lane-d \
  --write-baseline /tmp/rum-lane-d-smoke.txt \
  --capture-runs 5 \
  --baseline-elapsed-multiplier 20 \
  --baseline-min-elapsed-micros 500000
```

Capture refuses to write a baseline if any executed case fails.
