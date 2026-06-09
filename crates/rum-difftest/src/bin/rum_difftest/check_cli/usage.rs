pub(super) fn print_check_usage() {
    println!("rum-difftest check");
    println!("  --inventory-baseline PATH  Use an explicit inventory baseline");
    println!("  --include-lane-d-smoke     Also run the local Lane D smoke baseline gate");
    println!("  --lane-d-smoke-baseline PATH");
    println!(
        "                              Use an explicit Lane D smoke baseline and enable that gate"
    );
    println!("  --include-lane-d-readiness");
    println!("                              Also check the Lane D checked-baseline artifact set");
    println!("  --fail-on-open-known-difference");
    println!("                              Fail if any active Lane C ledger row has status=open");
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!("  --max-issue-samples N      Bound aggregate JSON issue samples (default 5)");
    println!("  --issue-sample-check NAME  Filter JSON issue samples by check name, or all");
    println!("  --issue-sample-kind KIND   Filter JSON issue samples by known issue kind, or all");
    println!(
        "  Run the fast non-oracle local gates: inventory baseline, Lane C ledger validation, and artifact hygiene, with optional Lane D gates"
    );
}
