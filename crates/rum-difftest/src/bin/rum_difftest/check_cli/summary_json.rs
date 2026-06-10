use std::path::Path;

use crate::json_cli::{SUMMARY_SCHEMA_VERSION, json_string_option, write_json_file};

use super::LocalCheckReport;

#[path = "summary_json/checks.rs"]
mod checks;
#[path = "summary_json/issue_samples.rs"]
mod issue_samples;
#[path = "summary_json/lane_d.rs"]
mod lane_d_json;

use checks::{
    artifact_hygiene_check_json, inventory_baseline_check_json, lane_c_check_json,
    skipped_checks_json,
};

pub(super) const DEFAULT_MAX_ISSUE_SAMPLES: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CheckIssueSampleFilter {
    pub(crate) check: Option<String>,
    pub(crate) kind: Option<String>,
}

pub(super) fn issue_sample_check_values() -> &'static [&'static str] {
    issue_samples::ISSUE_SAMPLE_CHECK_VALUES
}

pub(super) fn issue_sample_kind_values() -> &'static [&'static str] {
    issue_samples::ISSUE_SAMPLE_KIND_VALUES
}

pub(super) fn write_summary_json(
    path: &Path,
    report: &LocalCheckReport,
    max_issue_samples: usize,
    issue_sample_filter: &CheckIssueSampleFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_file(
        path,
        summary_json(report, max_issue_samples, issue_sample_filter),
    )
}

fn summary_json(
    report: &LocalCheckReport,
    max_issue_samples: usize,
    issue_sample_filter: &CheckIssueSampleFilter,
) -> String {
    let mut checks = vec![
        inventory_baseline_check_json(report),
        lane_c_check_json(report),
        artifact_hygiene_check_json(report),
    ];
    if let Some(lane_d_smoke) = &report.lane_d_smoke {
        checks.push(lane_d_json::lane_d_smoke_check_json(lane_d_smoke));
    }
    if let Some(lane_d_readiness) = &report.lane_d_readiness {
        checks.push(lane_d_json::lane_d_readiness_check_json(lane_d_readiness));
    }
    let skipped_checks = report.skipped_checks();
    let issue_samples =
        issue_samples::issue_samples_json(report, max_issue_samples, issue_sample_filter);
    let issue_check_counts = issue_samples::issue_check_counts_json(report);
    let issue_kind_counts = issue_samples::issue_kind_counts_json(report);

    format!(
        concat!(
            "{{\n",
            "  \"command\": \"check\",\n",
            "  \"schema_version\": {},\n",
            "  \"clean\": {},\n",
            "  \"checks_passed\": {},\n",
            "  \"checks_failed\": {},\n",
            "  \"checks_skipped\": {},\n",
            "  \"checks\": [\n",
            "{}\n",
            "  ],\n",
            "  \"skipped_checks\": [\n",
            "{}\n",
            "  ],\n",
            "  \"issue_sample_check\": {},\n",
            "  \"issue_sample_kind\": {},\n",
            "  \"issue_samples_limit\": {},\n",
            "  \"issue_samples_emitted\": {},\n",
            "  \"issue_samples_unfiltered_total\": {},\n",
            "  \"issue_samples_total\": {},\n",
            "  \"issue_samples_truncated\": {},\n",
            "  \"issue_check_counts\": [\n",
            "{}\n",
            "  ],\n",
            "  \"issue_kind_counts\": [\n",
            "{}\n",
            "  ],\n",
            "  \"issue_samples\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        SUMMARY_SCHEMA_VERSION,
        report.is_clean(),
        report.passed_checks(),
        report.failed_checks(),
        skipped_checks.len(),
        checks.join(",\n"),
        skipped_checks_json(&skipped_checks),
        json_string_option(issue_sample_filter.check.as_deref()),
        json_string_option(issue_sample_filter.kind.as_deref()),
        max_issue_samples,
        issue_samples.emitted,
        issue_samples.unfiltered_total,
        issue_samples.total,
        issue_samples.truncated(),
        issue_check_counts,
        issue_kind_counts,
        issue_samples.samples,
    )
}

#[cfg(test)]
#[path = "summary_json/tests.rs"]
mod tests;
