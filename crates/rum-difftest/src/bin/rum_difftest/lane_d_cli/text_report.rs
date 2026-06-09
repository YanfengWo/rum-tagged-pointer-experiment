use std::path::Path;

use rum_difftest::lane_d::{
    self, LaneDCaseStatus, LaneDCheckedBaselineReadinessEntry, LaneDCheckedBaselineReadinessReport,
    LaneDCheckedBaselineReadinessStatus, LaneDProfile, LaneDSuite, LaneDSummary,
    LaneDThresholdProfile,
};

pub(super) fn print_run_summary(
    suite: LaneDSuite,
    profile: LaneDProfile,
    threshold_profile: LaneDThresholdProfile,
    summary: &LaneDSummary,
) {
    println!("lane-d suite: {}", lane_d::suite_name(suite));
    println!("lane-d profile: {}", lane_d::profile_name(profile));
    println!(
        "lane-d threshold profile: {}",
        lane_d::threshold_profile_name(threshold_profile)
    );
    println!("cases seen: {}", summary.total_seen);
    println!("cases skipped by profile: {}", summary.skipped_by_profile);
    println!("cases executed: {}", summary.executed);
    println!("successes: {}", summary.succeeded);
    println!("failures: {}", summary.failed);
    println!("total elapsed micros: {}", summary.total_elapsed_micros);

    for case in &summary.cases {
        let status = match &case.status {
            LaneDCaseStatus::Success => "success",
            LaneDCaseStatus::Failure { .. } => "failure",
        };
        println!(
            "case\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            case.index,
            case.suite,
            case.category.as_deref().unwrap_or("-"),
            status,
            case.elapsed_micros,
            case.value_nodes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            case.input
        );
        if let LaneDCaseStatus::Failure { message } = &case.status {
            println!("failure\t{}\t{}", case.index, message.replace('\n', "\\n"));
        }
    }
}

pub(super) fn print_baseline_report(path: &Path, report: &lane_d::LaneDBaselineReport) {
    println!("baseline: {}", path.display());
    println!("baseline records considered: {}", report.records_considered);
    println!("baseline compared cases: {}", report.compared);
    println!("baseline missing records: {}", report.missing_records.len());
    println!("baseline unused records: {}", report.unused_records.len());
    println!(
        "baseline elapsed regressions: {}",
        report.elapsed_regressions.len()
    );
    println!(
        "baseline value-node regressions: {}",
        report.value_node_regressions.len()
    );
    for missing in &report.missing_records {
        println!(
            "baseline-missing\t{}\t{}\t{}",
            missing.index, missing.suite, missing.input
        );
    }
    for unused in &report.unused_records {
        println!(
            "baseline-unused\t{}\t{}\t{}\t{}",
            lane_d::profile_name(unused.profile),
            lane_d::threshold_profile_name(unused.threshold_profile),
            unused.suite,
            unused.input
        );
    }
    for regression in &report.elapsed_regressions {
        println!(
            "baseline-regression\telapsed\t{}\t{}\t{}\t{}\t{}",
            regression.case.index,
            regression.case.suite,
            regression.case.elapsed_micros,
            regression.max_elapsed_micros,
            regression.case.input
        );
    }
    for regression in &report.value_node_regressions {
        println!(
            "baseline-regression\tvalue-nodes\t{}\t{}\t{}\t{}\t{}",
            regression.case.index,
            regression.case.suite,
            regression.case.value_nodes.unwrap_or(0),
            regression.max_value_nodes,
            regression.case.input
        );
    }
}

pub(super) fn print_checked_baseline_readiness_report(
    report: &LaneDCheckedBaselineReadinessReport,
) {
    println!("checked baseline readiness: {}", report.is_ready());
    println!("checked baseline artifacts: {}", report.entries.len());
    for entry in &report.entries {
        let profile = lane_d::profile_name(entry.profile);
        let threshold_profile = lane_d::threshold_profile_name(entry.threshold_profile);
        match &entry.status {
            LaneDCheckedBaselineReadinessStatus::Ready {
                records_considered,
                compared,
            } => println!(
                "checked-baseline-ready\t{profile}\t{threshold_profile}\t{}\t{}\t{}",
                entry.path.display(),
                records_considered,
                compared
            ),
            LaneDCheckedBaselineReadinessStatus::MissingArtifact => {
                println!(
                    "checked-baseline-missing\t{profile}\t{threshold_profile}\t{}\t{}",
                    entry.path.display(),
                    capture_command_for_entry(entry)
                )
            }
            LaneDCheckedBaselineReadinessStatus::InvalidArtifact { message } => println!(
                "checked-baseline-invalid\t{profile}\t{threshold_profile}\t{}\t{}",
                entry.path.display(),
                message.replace('\n', "\\n")
            ),
            LaneDCheckedBaselineReadinessStatus::NotClean { report } => println!(
                "checked-baseline-not-clean\t{profile}\t{threshold_profile}\t{}\tmissing={}\tunused={}\telapsed={}\tvalue_nodes={}",
                entry.path.display(),
                report.missing_records.len(),
                report.unused_records.len(),
                report.elapsed_regressions.len(),
                report.value_node_regressions.len()
            ),
        }
    }
}

fn capture_command_for_entry(entry: &LaneDCheckedBaselineReadinessEntry) -> String {
    lane_d::checked_baseline_capture_command(&lane_d::LaneDCheckedBaselineArtifact {
        profile: entry.profile,
        threshold_profile: entry.threshold_profile,
        path: entry.path.clone(),
    })
}

pub(super) fn print_written_baseline_report(
    path: &Path,
    records: usize,
    capture_runs: usize,
    threshold_profile: LaneDThresholdProfile,
    elapsed_multiplier: u128,
    min_elapsed_micros: u128,
) {
    println!("baseline written: {}", path.display());
    println!("baseline written records: {records}");
    println!("baseline capture runs: {capture_runs}");
    println!(
        "baseline threshold profile: {}",
        lane_d::threshold_profile_name(threshold_profile)
    );
    println!("baseline elapsed multiplier: {elapsed_multiplier}");
    println!("baseline min elapsed micros: {min_elapsed_micros}");
}
