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
```

The first runner consumes the imported seed suites:

- `seed/ham-import/bench/timing_suite.txt`
- `seed/ham-import/mem/memory_suite.txt`

By default, `lane-d` uses the daily-safe `smoke` profile and skips imported
heavy / very-large categories. `--profile full` is the explicit baseline
collection mode for every imported input.

It reports local evaluation metrics only: elapsed microseconds, result node
count, message count, print count, and any evaluation failure. It does not yet
store historical baselines, enforce thresholds, or compare against Wolfram.

Future Lane D work should add a checked baseline artifact format and explicit
regression policy before using these numbers as pass/fail performance gates.
