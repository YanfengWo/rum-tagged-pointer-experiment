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
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!("  --max-issue-samples N      Bound aggregate JSON issue samples (default 5)");
    println!(
        "  Run the fast non-oracle local gates: inventory baseline and Lane C ledger validation, with optional Lane D gates"
    );
}
