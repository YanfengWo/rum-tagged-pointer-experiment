use std::path::Path;

use rum_difftest::lane_d::{
    self, LaneDBaselineReport, LaneDCheckedBaselineReadinessReport,
    LaneDCheckedBaselineReadinessStatus, LaneDSummary,
};

use crate::json_cli::json_string;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LaneDReadinessArtifactCounts {
    pub total: usize,
    pub ready: usize,
    pub missing: usize,
    pub invalid: usize,
    pub not_clean: usize,
}

pub fn readiness_artifact_counts(
    report: &LaneDCheckedBaselineReadinessReport,
) -> LaneDReadinessArtifactCounts {
    let mut counts = LaneDReadinessArtifactCounts::default();
    for entry in &report.entries {
        counts.total += 1;
        match &entry.status {
            LaneDCheckedBaselineReadinessStatus::Ready { .. } => counts.ready += 1,
            LaneDCheckedBaselineReadinessStatus::MissingArtifact => counts.missing += 1,
            LaneDCheckedBaselineReadinessStatus::InvalidArtifact { .. } => counts.invalid += 1,
            LaneDCheckedBaselineReadinessStatus::NotClean { .. } => counts.not_clean += 1,
        }
    }
    counts
}

pub fn baseline_report_json(
    path: &str,
    report: &LaneDBaselineReport,
    field_indent: &str,
    close_indent: &str,
) -> String {
    format!(
        concat!(
            "{{\n",
            "{}\"path\": \"{}\",\n",
            "{}\"clean\": {},\n",
            "{}\"records_considered\": {},\n",
            "{}\"compared\": {},\n",
            "{}\"missing_records\": {},\n",
            "{}\"unused_records\": {},\n",
            "{}\"elapsed_regressions\": {},\n",
            "{}\"value_node_regressions\": {}\n",
            "{}}}"
        ),
        field_indent,
        json_string(path),
        field_indent,
        report.is_clean(),
        field_indent,
        report.records_considered,
        field_indent,
        report.compared,
        field_indent,
        report.missing_records.len(),
        field_indent,
        report.unused_records.len(),
        field_indent,
        report.elapsed_regressions.len(),
        field_indent,
        report.value_node_regressions.len(),
        close_indent,
    )
}

pub fn run_counts_json(summary: &LaneDSummary, field_indent: &str, close_indent: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "{}\"cases_seen\": {},\n",
            "{}\"cases_skipped_by_profile\": {},\n",
            "{}\"cases_executed\": {},\n",
            "{}\"successes\": {},\n",
            "{}\"failures\": {},\n",
            "{}\"total_elapsed_micros\": {}\n",
            "{}}}"
        ),
        field_indent,
        summary.total_seen,
        field_indent,
        summary.skipped_by_profile,
        field_indent,
        summary.executed,
        field_indent,
        summary.succeeded,
        field_indent,
        summary.failed,
        field_indent,
        summary.total_elapsed_micros,
        close_indent,
    )
}

pub fn readiness_entries_json(
    report: &LaneDCheckedBaselineReadinessReport,
    entry_indent: &str,
    field_indent: &str,
    path_display: impl Fn(&Path) -> String,
) -> String {
    report
        .entries
        .iter()
        .map(|entry| readiness_entry_json(entry, entry_indent, field_indent, &path_display))
        .collect::<Vec<_>>()
        .join(",\n")
}

pub fn readiness_status_name(status: &LaneDCheckedBaselineReadinessStatus) -> &'static str {
    match status {
        LaneDCheckedBaselineReadinessStatus::Ready { .. } => "ready",
        LaneDCheckedBaselineReadinessStatus::MissingArtifact => "missing",
        LaneDCheckedBaselineReadinessStatus::InvalidArtifact { .. } => "invalid",
        LaneDCheckedBaselineReadinessStatus::NotClean { .. } => "not_clean",
    }
}

fn readiness_entry_json(
    entry: &lane_d::LaneDCheckedBaselineReadinessEntry,
    entry_indent: &str,
    field_indent: &str,
    path_display: &impl Fn(&Path) -> String,
) -> String {
    let capture_command =
        lane_d::checked_baseline_capture_command(&lane_d::LaneDCheckedBaselineArtifact {
            profile: entry.profile,
            threshold_profile: entry.threshold_profile,
            path: entry.path.clone(),
        });
    let detail_fields = match &entry.status {
        LaneDCheckedBaselineReadinessStatus::Ready {
            records_considered,
            compared,
        } => format!(
            concat!(
                ",\n",
                "{}\"records_considered\": {},\n",
                "{}\"compared\": {}"
            ),
            field_indent, records_considered, field_indent, compared
        ),
        LaneDCheckedBaselineReadinessStatus::MissingArtifact => String::new(),
        LaneDCheckedBaselineReadinessStatus::InvalidArtifact { message } => {
            format!(
                concat!(",\n", "{}\"message\": \"{}\""),
                field_indent,
                json_string(message)
            )
        }
        LaneDCheckedBaselineReadinessStatus::NotClean { report } => format!(
            concat!(
                ",\n",
                "{}\"missing_records\": {},\n",
                "{}\"unused_records\": {},\n",
                "{}\"elapsed_regressions\": {},\n",
                "{}\"value_node_regressions\": {}"
            ),
            field_indent,
            report.missing_records.len(),
            field_indent,
            report.unused_records.len(),
            field_indent,
            report.elapsed_regressions.len(),
            field_indent,
            report.value_node_regressions.len(),
        ),
    };
    format!(
        concat!(
            "{}{{\n",
            "{}\"profile\": \"{}\",\n",
            "{}\"threshold_profile\": \"{}\",\n",
            "{}\"path\": \"{}\",\n",
            "{}\"status\": \"{}\",\n",
            "{}\"suggested_capture_command\": \"{}\"",
            "{}\n",
            "{}}}"
        ),
        entry_indent,
        field_indent,
        json_string(lane_d::profile_name(entry.profile)),
        field_indent,
        json_string(lane_d::threshold_profile_name(entry.threshold_profile)),
        field_indent,
        json_string(&path_display(&entry.path)),
        field_indent,
        readiness_status_name(&entry.status),
        field_indent,
        json_string(&capture_command),
        detail_fields,
        entry_indent,
    )
}

#[cfg(test)]
mod tests {
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
                entry(LaneDCheckedBaselineReadinessStatus::InvalidArtifact {
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
        assert!(json.contains("    \"status\": \"invalid\""));
        assert!(json.contains("    \"message\": \"bad \\\"json\\\"\""));
    }

    fn entry(
        status: LaneDCheckedBaselineReadinessStatus,
    ) -> lane_d::LaneDCheckedBaselineReadinessEntry {
        lane_d::LaneDCheckedBaselineReadinessEntry {
            profile: lane_d::LaneDProfile::Smoke,
            threshold_profile: lane_d::LaneDThresholdProfile::LocalDebug,
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
}
