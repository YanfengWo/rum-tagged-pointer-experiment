pub(crate) fn print_lane_d_usage() {
    println!("rum-difftest lane-d");
    println!("  --suite all|timing|memory|core-language");
    println!("                             Select performance input suite");
    println!("  --profile smoke|full       Select daily-safe smoke inputs or explicit full inputs");
    println!("  --threshold-profile local-debug|ci-debug|ci-release");
    println!("                             Select baseline threshold records and capture defaults");
    println!("  --limit N                  Stop after N executed expressions");
    println!("  --baseline PATH            Compare against a Lane D baseline artifact");
    println!("  --write-baseline PATH      Capture the current run as a Lane D baseline artifact");
    println!("  --capture-runs N           Number of runs to aggregate when writing a baseline");
    println!("  --baseline-elapsed-multiplier N");
    println!("                             Multiplier for captured max elapsed guardrails");
    println!("  --baseline-min-elapsed-micros N");
    println!("                             Minimum captured max elapsed guardrail");
    println!("  --check-baseline-readiness");
    println!(
        "                             Check default checked smoke baselines for all threshold profiles"
    );
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!("  --fail-on-error            Exit non-zero if any expression fails");
    println!("  --fail-on-regression       Exit non-zero if baseline comparison is not clean");
}
