use rum_difftest::artifact_hygiene;
use rum_difftest::divergences::{self, KnownDifferenceLedgerReport};
use rum_difftest::inventory::display_path;
use rum_difftest::lane_d;

use super::{LaneDSmokeCheckReport, LocalCheckReport};

pub(super) fn print_check_report(report: &LocalCheckReport) {
    println!("rum-difftest check");
    print_inventory_baseline_check(report);
    print_lane_c_check(report);
    print_artifact_hygiene_check(report);
    if let Some(lane_d_smoke) = &report.lane_d_smoke {
        print_lane_d_smoke_check(lane_d_smoke);
    }
    if let Some(lane_d_readiness) = &report.lane_d_readiness {
        print_lane_d_readiness_check(lane_d_readiness);
    }
    for skipped in report.skipped_checks() {
        println!("check\t{}\tskipped\t{}", skipped.name, skipped.reason);
    }
    println!("checks passed: {}", report.passed_checks());
    println!("checks failed: {}", report.failed_checks());
    println!("checks skipped: {}", report.skipped_checks_count());
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
    let status = if report.lane_c_check_clean() {
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
    for (status, entries) in report
        .known_differences
        .active_ledger_field_counts("status")
    {
        println!("active ledger status {status}: {entries}");
    }
    for (kind, entries) in report.known_differences.active_ledger_field_counts("type") {
        println!("active ledger type {kind}: {entries}");
    }
    println!(
        "fixture known differences: {}",
        report.known_differences.fixture_references.len()
    );
    println!(
        "lane-c issues: {}",
        lane_c_issue_count(&report.known_differences)
    );
    if report.fail_on_open_known_difference {
        let status = if report.open_known_differences() == 0 {
            "clean"
        } else {
            "dirty"
        };
        println!(
            "open known-difference gate: {status} open={}",
            report.open_known_differences()
        );
    }
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

fn print_artifact_hygiene_check(report: &LocalCheckReport) {
    let status = if report.artifact_hygiene_issues.is_empty() {
        "clean"
    } else {
        "dirty"
    };
    println!("check\tartifact-hygiene\t{status}");
    println!(
        "artifact hygiene issues: {}",
        report.artifact_hygiene_issues.len()
    );
    let counts = artifact_hygiene::count_artifact_hygiene_issues(&report.artifact_hygiene_issues);
    println!(
        "artifact hygiene oracle artifacts: {}",
        counts.oracle_artifacts
    );
    println!(
        "artifact hygiene stray fixture files: {}",
        counts.stray_fixture_files
    );
    for issue in &report.artifact_hygiene_issues {
        println!(
            "artifact-hygiene-issue\t{}\t{}\t{}",
            issue.kind.as_str(),
            issue.path.display(),
            issue.reason
        );
    }
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

fn print_lane_d_readiness_check(report: &super::LaneDReadinessCheckReport) {
    let status = if report.is_clean() { "clean" } else { "dirty" };
    let counts = report.artifact_counts();
    println!("check\tlane-d-readiness\t{status}");
    println!("lane-d readiness cases seen: {}", report.summary.total_seen);
    println!(
        "lane-d readiness cases executed: {}",
        report.summary.executed
    );
    println!("lane-d readiness failures: {}", report.summary.failed);
    println!("lane-d readiness artifacts: {}", counts.total);
    println!("lane-d readiness ready artifacts: {}", counts.ready);
    println!("lane-d readiness missing artifacts: {}", counts.missing);
    println!("lane-d readiness invalid artifacts: {}", counts.invalid);
    println!("lane-d readiness not-clean artifacts: {}", counts.not_clean);
    for entry in &report.readiness.entries {
        let profile = lane_d::profile_name(entry.profile);
        let threshold_profile = lane_d::threshold_profile_name(entry.threshold_profile);
        match &entry.status {
            lane_d::LaneDCheckedBaselineReadinessStatus::Ready {
                records_considered,
                compared,
            } => println!(
                "lane-d-readiness-ready\t{profile}\t{threshold_profile}\t{}\t{}\t{}",
                display_path(&entry.path),
                records_considered,
                compared
            ),
            lane_d::LaneDCheckedBaselineReadinessStatus::MissingArtifact => println!(
                "lane-d-readiness-missing\t{profile}\t{threshold_profile}\t{}\t{}",
                display_path(&entry.path),
                lane_d::checked_baseline_capture_command(&lane_d::LaneDCheckedBaselineArtifact {
                    profile: entry.profile,
                    threshold_profile: entry.threshold_profile,
                    path: entry.path.clone(),
                })
            ),
            lane_d::LaneDCheckedBaselineReadinessStatus::InvalidArtifact { message } => println!(
                "lane-d-readiness-invalid\t{profile}\t{threshold_profile}\t{}\t{}",
                display_path(&entry.path),
                message.replace('\n', "\\n")
            ),
            lane_d::LaneDCheckedBaselineReadinessStatus::NotClean { report } => println!(
                "lane-d-readiness-not-clean\t{profile}\t{threshold_profile}\t{}\tmissing={}\tunused={}\telapsed={}\tvalue_nodes={}",
                display_path(&entry.path),
                report.missing_records.len(),
                report.unused_records.len(),
                report.elapsed_regressions.len(),
                report.value_node_regressions.len()
            ),
        }
    }
}
