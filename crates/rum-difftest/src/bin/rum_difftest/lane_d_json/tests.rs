use std::path::PathBuf;

use super::*;

#[test]
fn baseline_report_json_renders_counts_and_path() {
    let json = baseline_report_json(
        "perf/baselines/smoke.txt",
        &clean_baseline_report(),
        "  ",
        "",
    );

    assert!(json.starts_with("{\n  \"path\": \"perf/baselines/smoke.txt\""));
    assert!(json.contains("  \"clean\": true"));
    assert!(json.contains("  \"records_considered\": 0"));
    assert!(json.ends_with("\n}"));
}

#[test]
fn run_counts_json_renders_summary_counts() {
    let json = run_counts_json(&summary(), "  ", "");

    assert!(json.starts_with("{\n  \"cases_seen\": 5"));
    assert!(json.contains("  \"cases_skipped_by_profile\": 1"));
    assert!(json.contains("  \"cases_executed\": 4"));
    assert!(json.contains("  \"successes\": 3"));
    assert!(json.contains("  \"failures\": 1"));
    assert!(json.contains("  \"total_elapsed_micros\": 99"));
    assert!(json.ends_with("\n}"));
}

#[test]
fn readiness_artifact_counts_distinguish_statuses() {
    let report = LaneDCheckedBaselineReadinessReport {
        entries: vec![
            entry(LaneDCheckedBaselineReadinessStatus::Ready {
                records_considered: 3,
                compared: 3,
            }),
            entry(LaneDCheckedBaselineReadinessStatus::MissingArtifact),
            entry(LaneDCheckedBaselineReadinessStatus::InvalidArtifact {
                message: "bad artifact".to_string(),
            }),
            entry(LaneDCheckedBaselineReadinessStatus::NotClean {
                report: clean_baseline_report(),
            }),
        ],
    };

    assert_eq!(
        readiness_artifact_counts(&report),
        LaneDReadinessArtifactCounts {
            total: 4,
            ready: 1,
            missing: 1,
            invalid: 1,
            not_clean: 1,
        }
    );
}

#[test]
fn readiness_entries_json_renders_detail_fields() {
    let report = LaneDCheckedBaselineReadinessReport {
        entries: vec![
            entry(LaneDCheckedBaselineReadinessStatus::Ready {
                records_considered: 7,
                compared: 6,
            }),
            release_entry(LaneDCheckedBaselineReadinessStatus::InvalidArtifact {
                message: "bad \"json\"".to_string(),
            }),
        ],
    };

    let json = readiness_entries_json(&report, "  ", "    ", |path| {
        format!("displayed/{}", path.display())
    });

    assert!(json.contains("  {"));
    assert!(json.contains("    \"status\": \"ready\""));
    assert!(json.contains("    \"records_considered\": 7"));
    assert!(json.contains("    \"compared\": 6"));
    assert!(json.contains("    \"path\": \"displayed/baseline.txt\""));
    assert!(json.contains("    \"cargo_profile\": \"debug\""));
    assert!(json.contains("    \"status\": \"invalid\""));
    assert!(json.contains("    \"cargo_profile\": \"release\""));
    assert!(json.contains("    \"message\": \"bad \\\"json\\\"\""));
}

fn entry(
    status: LaneDCheckedBaselineReadinessStatus,
) -> lane_d::LaneDCheckedBaselineReadinessEntry {
    entry_with_threshold(lane_d::LaneDThresholdProfile::LocalDebug, status)
}

fn release_entry(
    status: LaneDCheckedBaselineReadinessStatus,
) -> lane_d::LaneDCheckedBaselineReadinessEntry {
    entry_with_threshold(lane_d::LaneDThresholdProfile::CiRelease, status)
}

fn entry_with_threshold(
    threshold_profile: lane_d::LaneDThresholdProfile,
    status: LaneDCheckedBaselineReadinessStatus,
) -> lane_d::LaneDCheckedBaselineReadinessEntry {
    lane_d::LaneDCheckedBaselineReadinessEntry {
        profile: lane_d::LaneDProfile::Smoke,
        threshold_profile,
        path: PathBuf::from("baseline.txt"),
        status,
    }
}

fn clean_baseline_report() -> lane_d::LaneDBaselineReport {
    lane_d::LaneDBaselineReport {
        records_considered: 0,
        compared: 0,
        missing_records: Vec::new(),
        unused_records: Vec::new(),
        elapsed_regressions: Vec::new(),
        value_node_regressions: Vec::new(),
    }
}

fn summary() -> LaneDSummary {
    LaneDSummary {
        total_seen: 5,
        executed: 4,
        skipped_by_profile: 1,
        succeeded: 3,
        failed: 1,
        total_elapsed_micros: 99,
        cases: Vec::new(),
    }
}
