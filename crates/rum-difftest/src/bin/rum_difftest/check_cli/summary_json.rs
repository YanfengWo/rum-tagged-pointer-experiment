use std::path::Path;

use rum_difftest::divergences;
use rum_difftest::inventory::{FixtureInventoryTotals, display_path};

use crate::json_cli::{SUMMARY_SCHEMA_VERSION, json_string, write_json_file};

use super::{LocalCheckReport, SkippedCheck};

#[path = "summary_json/issue_samples.rs"]
mod issue_samples;
#[path = "summary_json/lane_d.rs"]
mod lane_d_json;

pub(super) const DEFAULT_MAX_ISSUE_SAMPLES: usize = 5;

pub(super) fn write_summary_json(
    path: &Path,
    report: &LocalCheckReport,
    max_issue_samples: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_file(path, summary_json(report, max_issue_samples))
}

fn summary_json(report: &LocalCheckReport, max_issue_samples: usize) -> String {
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
    let issue_samples = issue_samples::issue_samples_json(report, max_issue_samples);
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
            "  \"issue_samples_limit\": {},\n",
            "  \"issue_samples_emitted\": {},\n",
            "  \"issue_samples_total\": {},\n",
            "  \"issue_samples_truncated\": {},\n",
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
        max_issue_samples,
        issue_samples.emitted,
        issue_samples.total,
        issue_samples.truncated(),
        issue_kind_counts,
        issue_samples.samples,
    )
}

fn skipped_checks_json(checks: &[SkippedCheck]) -> String {
    checks
        .iter()
        .map(|check| {
            format!(
                "    {{\"name\": \"{}\", \"reason\": \"{}\"}}",
                json_string(check.name),
                json_string(check.reason),
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn inventory_baseline_check_json(report: &LocalCheckReport) -> String {
    let totals = report.inventory.totals();
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"inventory-baseline\",\n",
            "      \"clean\": {},\n",
            "      \"path\": \"{}\",\n",
            "      \"fixture_totals\": {},\n",
            "      \"records_considered\": {},\n",
            "      \"regressions\": {},\n",
            "      \"regression_details\": [\n",
            "{}\n",
            "      ]\n",
            "    }}"
        ),
        report.inventory_baseline.is_clean(),
        json_string(&display_path(&report.inventory_baseline_path)),
        totals_json(&totals, 6),
        report.inventory_baseline.records_considered,
        report.inventory_baseline.regressions.len(),
        report
            .inventory_baseline
            .regressions
            .iter()
            .map(|regression| {
                format!(
                    concat!(
                        "        {{",
                        "\"lane\": \"{}\", ",
                        "\"dimension\": \"{}\", ",
                        "\"name\": \"{}\", ",
                        "\"issues\": [{}]",
                        "}}"
                    ),
                    json_string(&regression.record.lane),
                    json_string(regression.record.dimension.field_name()),
                    json_string(&regression.record.name),
                    regression
                        .issues
                        .iter()
                        .map(|issue| format!("\"{}\"", json_string(issue)))
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            })
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn artifact_hygiene_check_json(report: &LocalCheckReport) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"artifact-hygiene\",\n",
            "      \"clean\": {},\n",
            "      \"issues\": {},\n",
            "      \"issue_details\": [\n",
            "{}\n",
            "      ]\n",
            "    }}"
        ),
        report.artifact_hygiene_issues.is_empty(),
        report.artifact_hygiene_issues.len(),
        report
            .artifact_hygiene_issues
            .iter()
            .map(|issue| {
                format!(
                    "        {{\"path\": \"{}\", \"reason\": \"{}\"}}",
                    json_string(&issue.path.display().to_string()),
                    json_string(&issue.reason),
                )
            })
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn lane_c_check_json(report: &LocalCheckReport) -> String {
    let ledger = &report.known_differences;
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"lane-c-ledger\",\n",
            "      \"clean\": {},\n",
            "      \"known_difference_ledger\": \"{}\",\n",
            "      \"retired_seed_differences\": \"{}\",\n",
            "      \"counts\": {{\n",
            "        \"active_ledger_entries\": {},\n",
            "        \"retired_seed_entries\": {},\n",
            "        \"fixture_known_differences\": {}\n",
            "      }},\n",
            "      \"issues\": {{\n",
            "        \"missing_ledger_entries\": {},\n",
            "        \"unreferenced_ledger_entries\": {},\n",
            "        \"unreferenced_retired_seed_entries\": {},\n",
            "        \"fixture_mismatches\": {},\n",
            "        \"metadata_mismatches\": {},\n",
            "        \"retired_seed_fixture_mismatches\": {},\n",
            "        \"tracking_target_issues\": {}\n",
            "      }}\n",
            "    }}"
        ),
        ledger.is_clean(),
        json_string(
            &divergences::known_difference_ledger_path()
                .display()
                .to_string()
        ),
        json_string(
            &divergences::retired_seed_difference_path()
                .display()
                .to_string()
        ),
        ledger.records.len(),
        ledger.retired_seed_records.len(),
        ledger.fixture_references.len(),
        ledger.missing_ledger_entries.len(),
        ledger.unreferenced_ledger_entries.len(),
        ledger.unreferenced_retired_seed_entries.len(),
        ledger.fixture_mismatches.len(),
        ledger.metadata_mismatches.len(),
        ledger.retired_seed_fixture_mismatches.len(),
        ledger.tracking_target_issues.len(),
    )
}

fn totals_json(totals: &FixtureInventoryTotals, indent: usize) -> String {
    let spaces = " ".repeat(indent);
    format!(
        concat!(
            "{{\n",
            "{0}  \"fixtures\": {1},\n",
            "{0}  \"entries\": {2},\n",
            "{0}  \"normal_entries\": {3},\n",
            "{0}  \"known_difference_entries\": {4}\n",
            "{0}}}"
        ),
        spaces,
        totals.fixtures,
        totals.entries,
        totals.normal_entries,
        totals.known_difference_entries,
    )
}
