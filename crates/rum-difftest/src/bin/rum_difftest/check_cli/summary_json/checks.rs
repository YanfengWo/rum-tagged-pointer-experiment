use rum_difftest::artifact_hygiene::{self, ArtifactHygieneIssue};
use rum_difftest::divergences;
use rum_difftest::inventory::display_path;

use crate::inventory_json::{inventory_regression_details_json, inventory_totals_json};
use crate::json_cli::json_string;
use crate::lane_c_json::active_ledger_field_counts_json;

use super::super::{LocalCheckReport, SkippedCheck, display_check_path};

pub(super) fn skipped_checks_json(checks: &[SkippedCheck]) -> String {
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

pub(super) fn inventory_baseline_check_json(report: &LocalCheckReport) -> String {
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
        inventory_totals_json(&totals, 6),
        report.inventory_baseline.records_considered,
        report.inventory_baseline.regressions.len(),
        inventory_regression_details_json(&report.inventory_baseline.regressions, 8),
    )
}

pub(super) fn artifact_hygiene_check_json(report: &LocalCheckReport) -> String {
    let counts = artifact_hygiene::count_artifact_hygiene_issues(&report.artifact_hygiene_issues);
    format!(
        concat!(
            "    {{\n",
            "      \"name\": \"artifact-hygiene\",\n",
            "      \"clean\": {},\n",
            "      \"root\": \"{}\",\n",
            "      \"issues\": {},\n",
            "      \"issue_kind_counts\": {{\n",
            "        \"oracle_artifact\": {},\n",
            "        \"stray_fixture_file\": {}\n",
            "      }},\n",
            "      \"issue_details\": [\n",
            "{}\n",
            "      ]\n",
            "    }}"
        ),
        report.artifact_hygiene_issues.is_empty(),
        json_string(&display_check_path(&report.artifact_hygiene_root)),
        report.artifact_hygiene_issues.len(),
        counts.oracle_artifacts,
        counts.stray_fixture_files,
        artifact_hygiene_issue_details_json(&report.artifact_hygiene_issues),
    )
}

pub(super) fn artifact_hygiene_issue_details_json(issues: &[ArtifactHygieneIssue]) -> String {
    issues
        .iter()
        .map(|issue| {
            format!(
                "        {{\"path\": \"{}\", \"kind\": \"{}\", \"reason\": \"{}\"}}",
                json_string(&issue.path.display().to_string()),
                json_string(issue.kind.as_str()),
                json_string(&issue.reason),
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

pub(super) fn lane_c_check_json(report: &LocalCheckReport) -> String {
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
            "        \"active_ledger_status_counts\": [\n",
            "{}\n",
            "        ],\n",
            "        \"active_ledger_type_counts\": [\n",
            "{}\n",
            "        ],\n",
            "        \"retired_seed_entries\": {},\n",
            "        \"fixture_known_differences\": {}\n",
            "      }},\n",
            "      \"failure_gates\": {{\n",
            "        \"fail_on_open_known_difference\": {},\n",
            "        \"open_known_differences\": {}\n",
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
        report.lane_c_check_clean(),
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
        active_ledger_field_counts_json(ledger, "status", "status", 10),
        active_ledger_field_counts_json(ledger, "type", "type", 10),
        ledger.retired_seed_records.len(),
        ledger.fixture_references.len(),
        report.fail_on_open_known_difference,
        report.open_known_differences(),
        ledger.missing_ledger_entries.len(),
        ledger.unreferenced_ledger_entries.len(),
        ledger.unreferenced_retired_seed_entries.len(),
        ledger.fixture_mismatches.len(),
        ledger.metadata_mismatches.len(),
        ledger.retired_seed_fixture_mismatches.len(),
        ledger.tracking_target_issues.len(),
    )
}
