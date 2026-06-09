use std::path::{Path, PathBuf};

use rum_difftest::lane_d::{
    self, LaneDBaselineReport, LaneDCheckedBaselineReadinessReport, LaneDProfile, LaneDSuite,
    LaneDThresholdProfile,
};

use crate::json_cli::{SUMMARY_SCHEMA_VERSION, json_string, write_json_file};
use crate::lane_d_json::{baseline_report_json, readiness_artifact_counts, readiness_entries_json};

pub(super) struct WrittenBaselineSummary {
    pub(super) path: PathBuf,
    pub(super) records: usize,
    pub(super) capture_runs: usize,
    pub(super) threshold_profile: LaneDThresholdProfile,
    pub(super) elapsed_multiplier: u128,
    pub(super) min_elapsed_micros: u128,
}

pub(super) struct LaneDSummaryJsonContext<'a> {
    pub(super) suite: LaneDSuite,
    pub(super) profile: LaneDProfile,
    pub(super) threshold_profile: LaneDThresholdProfile,
    pub(super) summary: &'a lane_d::LaneDSummary,
    pub(super) baseline_path: Option<&'a Path>,
    pub(super) baseline_report: Option<&'a LaneDBaselineReport>,
    pub(super) readiness_report: Option<&'a LaneDCheckedBaselineReadinessReport>,
    pub(super) written_baseline: Option<&'a WrittenBaselineSummary>,
}

pub(super) fn write_summary_json(
    path: &Path,
    context: &LaneDSummaryJsonContext<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_file(path, summary_json(context))
}

fn summary_json(context: &LaneDSummaryJsonContext<'_>) -> String {
    let clean = context.summary.failed == 0
        && context
            .baseline_report
            .is_none_or(LaneDBaselineReport::is_clean)
        && context
            .readiness_report
            .is_none_or(LaneDCheckedBaselineReadinessReport::is_ready);

    format!(
        concat!(
            "{{\n",
            "  \"lane\": \"lane-d\",\n",
            "  \"schema_version\": {},\n",
            "  \"suite\": \"{}\",\n",
            "  \"profile\": \"{}\",\n",
            "  \"threshold_profile\": \"{}\",\n",
            "  \"clean\": {},\n",
            "  \"counts\": {{\n",
            "    \"cases_seen\": {},\n",
            "    \"cases_skipped_by_profile\": {},\n",
            "    \"cases_executed\": {},\n",
            "    \"successes\": {},\n",
            "    \"failures\": {},\n",
            "    \"total_elapsed_micros\": {}\n",
            "  }},\n",
            "  \"baseline\": {},\n",
            "  \"checked_baseline_readiness\": {},\n",
            "  \"written_baseline\": {}\n",
            "}}\n"
        ),
        SUMMARY_SCHEMA_VERSION,
        json_string(lane_d::suite_name(context.suite)),
        json_string(lane_d::profile_name(context.profile)),
        json_string(lane_d::threshold_profile_name(context.threshold_profile)),
        clean,
        context.summary.total_seen,
        context.summary.skipped_by_profile,
        context.summary.executed,
        context.summary.succeeded,
        context.summary.failed,
        context.summary.total_elapsed_micros,
        baseline_summary_json(context.baseline_path, context.baseline_report),
        readiness_summary_json(context.readiness_report),
        written_baseline_summary_json(context.written_baseline),
    )
}

fn baseline_summary_json(path: Option<&Path>, report: Option<&LaneDBaselineReport>) -> String {
    let Some(report) = report else {
        return "null".to_string();
    };
    let path = path
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    baseline_report_json(&path, report, "    ", "  ")
}

fn readiness_summary_json(report: Option<&LaneDCheckedBaselineReadinessReport>) -> String {
    let Some(report) = report else {
        return "null".to_string();
    };
    let counts = readiness_artifact_counts(report);
    format!(
        concat!(
            "{{\n",
            "    \"ready\": {},\n",
            "    \"artifacts\": {},\n",
            "    \"ready_artifacts\": {},\n",
            "    \"missing_artifacts\": {},\n",
            "    \"invalid_artifacts\": {},\n",
            "    \"not_clean_artifacts\": {},\n",
            "    \"entries\": [\n",
            "{}\n",
            "    ]\n",
            "  }}"
        ),
        report.is_ready(),
        counts.total,
        counts.ready,
        counts.missing,
        counts.invalid,
        counts.not_clean,
        readiness_entries_json(report, "      ", "        ", |path| path
            .display()
            .to_string()),
    )
}

fn written_baseline_summary_json(summary: Option<&WrittenBaselineSummary>) -> String {
    let Some(summary) = summary else {
        return "null".to_string();
    };
    format!(
        concat!(
            "{{\n",
            "    \"path\": \"{}\",\n",
            "    \"records\": {},\n",
            "    \"capture_runs\": {},\n",
            "    \"threshold_profile\": \"{}\",\n",
            "    \"elapsed_multiplier\": {},\n",
            "    \"min_elapsed_micros\": {}\n",
            "  }}"
        ),
        json_string(&summary.path.display().to_string()),
        summary.records,
        summary.capture_runs,
        json_string(lane_d::threshold_profile_name(summary.threshold_profile)),
        summary.elapsed_multiplier,
        summary.min_elapsed_micros,
    )
}
