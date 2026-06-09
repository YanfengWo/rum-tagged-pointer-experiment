use std::path::Path;

use rum_difftest::inventory::{FixtureInventoryTotals, display_path};
use rum_difftest::{divergences, lane_d};

use crate::json_cli::{json_string, write_json_file};

use super::LocalCheckReport;

#[path = "summary_json/issue_samples.rs"]
mod issue_samples;

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
    ];
    if let Some(lane_d_smoke) = &report.lane_d_smoke {
        checks.push(lane_d_smoke_check_json(lane_d_smoke));
    }
    if let Some(lane_d_readiness) = &report.lane_d_readiness {
        checks.push(lane_d_readiness_check_json(lane_d_readiness));
    }

    format!(
        concat!(
            "{{\n",
            "  \"command\": \"check\",\n",
            "  \"clean\": {},\n",
            "  \"checks_passed\": {},\n",
            "  \"checks_failed\": {},\n",
            "  \"checks\": [\n",
            "{}\n",
            "  ],\n",
            "  \"issue_samples\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        report.is_clean(),
        report.passed_checks(),
        report.failed_checks(),
        checks.join(",\n"),
        issue_samples::issue_samples_json(report, max_issue_samples),
    )
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

fn lane_d_smoke_check_json(report: &super::LaneDSmokeCheckReport) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"lane-d-smoke\",\n",
            "      \"clean\": {},\n",
            "      \"baseline\": {{\n",
            "        \"path\": \"{}\",\n",
            "        \"clean\": {},\n",
            "        \"records_considered\": {},\n",
            "        \"compared\": {},\n",
            "        \"missing_records\": {},\n",
            "        \"unused_records\": {},\n",
            "        \"elapsed_regressions\": {},\n",
            "        \"value_node_regressions\": {}\n",
            "      }},\n",
            "      \"counts\": {{\n",
            "        \"cases_seen\": {},\n",
            "        \"cases_skipped_by_profile\": {},\n",
            "        \"cases_executed\": {},\n",
            "        \"successes\": {},\n",
            "        \"failures\": {},\n",
            "        \"total_elapsed_micros\": {}\n",
            "      }},\n",
            "      \"run_options\": {{\n",
            "        \"suite\": \"{}\",\n",
            "        \"profile\": \"{}\",\n",
            "        \"threshold_profile\": \"{}\"\n",
            "      }}\n",
            "    }}"
        ),
        report.is_clean(),
        json_string(&display_path(&report.baseline_path)),
        report.baseline.is_clean(),
        report.baseline.records_considered,
        report.baseline.compared,
        report.baseline.missing_records.len(),
        report.baseline.unused_records.len(),
        report.baseline.elapsed_regressions.len(),
        report.baseline.value_node_regressions.len(),
        report.summary.total_seen,
        report.summary.skipped_by_profile,
        report.summary.executed,
        report.summary.succeeded,
        report.summary.failed,
        report.summary.total_elapsed_micros,
        json_string(lane_d::suite_name(lane_d::LaneDSuite::All)),
        json_string(lane_d::profile_name(lane_d::LaneDProfile::Smoke)),
        json_string(lane_d::threshold_profile_name(
            lane_d::LaneDThresholdProfile::LocalDebug
        )),
    )
}

fn lane_d_readiness_check_json(report: &super::LaneDReadinessCheckReport) -> String {
    let counts = report.artifact_counts();
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"lane-d-readiness\",\n",
            "      \"clean\": {},\n",
            "      \"artifacts\": {{\n",
            "        \"total\": {},\n",
            "        \"ready\": {},\n",
            "        \"missing\": {},\n",
            "        \"invalid\": {},\n",
            "        \"not_clean\": {}\n",
            "      }},\n",
            "      \"counts\": {{\n",
            "        \"cases_seen\": {},\n",
            "        \"cases_skipped_by_profile\": {},\n",
            "        \"cases_executed\": {},\n",
            "        \"successes\": {},\n",
            "        \"failures\": {},\n",
            "        \"total_elapsed_micros\": {}\n",
            "      }},\n",
            "      \"run_options\": {{\n",
            "        \"suite\": \"{}\",\n",
            "        \"profile\": \"{}\"\n",
            "      }}\n",
            "    }}"
        ),
        report.is_clean(),
        counts.total,
        counts.ready,
        counts.missing,
        counts.invalid,
        counts.not_clean,
        report.summary.total_seen,
        report.summary.skipped_by_profile,
        report.summary.executed,
        report.summary.succeeded,
        report.summary.failed,
        report.summary.total_elapsed_micros,
        json_string(lane_d::suite_name(lane_d::LaneDSuite::All)),
        json_string(lane_d::profile_name(lane_d::LaneDProfile::Smoke)),
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
