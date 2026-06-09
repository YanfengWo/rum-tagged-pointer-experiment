use rum_difftest::divergences::{self, KnownDifferenceLedgerReport};
use rum_difftest::inventory::display_path;

use super::{LaneDSmokeCheckReport, LocalCheckReport};

pub(super) fn print_check_report(report: &LocalCheckReport) {
    println!("rum-difftest check");
    print_inventory_baseline_check(report);
    print_lane_c_check(report);
    if let Some(lane_d_smoke) = &report.lane_d_smoke {
        print_lane_d_smoke_check(lane_d_smoke);
    }
    println!("checks passed: {}", report.passed_checks());
    println!("checks failed: {}", report.failed_checks());
}

fn print_inventory_baseline_check(report: &LocalCheckReport) {
    let status = if report.inventory_baseline.is_clean() {
        "clean"
    } else {
        "dirty"
    };
    println!("check\tinventory-baseline\t{status}");
    println!(
        "inventory baseline: {}",
        display_path(&report.inventory_baseline_path)
    );
    println!(
        "inventory baseline records considered: {}",
        report.inventory_baseline.records_considered
    );
    println!(
        "inventory baseline regressions: {}",
        report.inventory_baseline.regressions.len()
    );
    for regression in &report.inventory_baseline.regressions {
        println!(
            "inventory-baseline-regression\t{}\t{}\t{}\t{}",
            regression.record.lane,
            regression.record.dimension.field_name(),
            regression.record.name,
            regression.issues.join(" | ")
        );
    }
}

fn print_lane_c_check(report: &LocalCheckReport) {
    let status = if report.known_differences.is_clean() {
        "clean"
    } else {
        "dirty"
    };
    println!("check\tlane-c-ledger\t{status}");
    println!(
        "known-difference ledger: {}",
        divergences::known_difference_ledger_path().display()
    );
    println!(
        "retired seed differences: {}",
        divergences::retired_seed_difference_path().display()
    );
    println!(
        "active ledger entries: {}",
        report.known_differences.records.len()
    );
    println!(
        "fixture known differences: {}",
        report.known_differences.fixture_references.len()
    );
    println!(
        "lane-c issues: {}",
        lane_c_issue_count(&report.known_differences)
    );
}

fn lane_c_issue_count(report: &KnownDifferenceLedgerReport) -> usize {
    report.missing_ledger_entries.len()
        + report.unreferenced_ledger_entries.len()
        + report.unreferenced_retired_seed_entries.len()
        + report.fixture_mismatches.len()
        + report.metadata_mismatches.len()
        + report.retired_seed_fixture_mismatches.len()
        + report.tracking_target_issues.len()
}

fn print_lane_d_smoke_check(report: &LaneDSmokeCheckReport) {
    let status = if report.is_clean() { "clean" } else { "dirty" };
    println!("check\tlane-d-smoke\t{status}");
    println!(
        "lane-d smoke baseline: {}",
        display_path(&report.baseline_path)
    );
    println!("lane-d cases seen: {}", report.summary.total_seen);
    println!("lane-d cases executed: {}", report.summary.executed);
    println!("lane-d failures: {}", report.summary.failed);
    println!(
        "lane-d baseline records considered: {}",
        report.baseline.records_considered
    );
    println!(
        "lane-d baseline compared cases: {}",
        report.baseline.compared
    );
    println!(
        "lane-d baseline missing records: {}",
        report.baseline.missing_records.len()
    );
    println!(
        "lane-d baseline unused records: {}",
        report.baseline.unused_records.len()
    );
    println!(
        "lane-d baseline elapsed regressions: {}",
        report.baseline.elapsed_regressions.len()
    );
    println!(
        "lane-d baseline value-node regressions: {}",
        report.baseline.value_node_regressions.len()
    );
    println!("lane-d baseline issues: {}", report.baseline_issue_count());
}
