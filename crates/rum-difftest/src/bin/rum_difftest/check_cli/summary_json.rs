use std::path::Path;

use rum_difftest::divergences;
use rum_difftest::inventory::{FixtureInventoryTotals, display_path};

use crate::json_cli::{json_string, write_json_file};

use super::LocalCheckReport;

pub(super) fn write_summary_json(
    path: &Path,
    report: &LocalCheckReport,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_file(path, summary_json(report))
}

fn summary_json(report: &LocalCheckReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"command\": \"check\",\n",
            "  \"clean\": {},\n",
            "  \"checks_passed\": {},\n",
            "  \"checks_failed\": {},\n",
            "  \"checks\": [\n",
            "{},\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        report.is_clean(),
        2 - report.failed_checks(),
        report.failed_checks(),
        inventory_baseline_check_json(report),
        lane_c_check_json(report),
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
