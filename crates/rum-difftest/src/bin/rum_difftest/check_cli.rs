use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::divergences::{self, KnownDifferenceLedgerReport};
use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, InventoryBaselineReport,
    collect_fixture_inventory, compare_inventory_baseline, default_inventory_baseline_path,
    display_path, load_inventory_baseline,
};

use super::expect_value;
use super::json_cli::{json_string, write_json_file};

pub fn run_check(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut inventory_baseline_path = None;
    let mut summary_json_path = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--inventory-baseline" => {
                inventory_baseline_path = Some(PathBuf::from(expect_value(
                    &mut iter,
                    "--inventory-baseline",
                )?));
            }
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--help" | "-h" => {
                print_check_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown check flag `{other}`").into()),
        }
    }

    let report = run_local_checks(inventory_baseline_path)?;
    print_check_report(&report);

    if let Some(path) = summary_json_path {
        write_json_file(&path, summary_json(&report))?;
        println!("summary json: {}", path.display());
    }

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

struct LocalCheckReport {
    inventory_baseline_path: PathBuf,
    inventory: FixtureInventoryReport,
    inventory_baseline: InventoryBaselineReport,
    known_differences: KnownDifferenceLedgerReport,
}

impl LocalCheckReport {
    fn is_clean(&self) -> bool {
        self.inventory_baseline.is_clean() && self.known_differences.is_clean()
    }

    fn failed_checks(&self) -> usize {
        usize::from(!self.inventory_baseline.is_clean())
            + usize::from(!self.known_differences.is_clean())
    }
}

fn run_local_checks(
    inventory_baseline_path: Option<PathBuf>,
) -> Result<LocalCheckReport, Box<dyn std::error::Error>> {
    let inventory_baseline_path =
        inventory_baseline_path.unwrap_or_else(default_inventory_baseline_path);
    let inventory = collect_fixture_inventory()?;
    let baseline = load_inventory_baseline(&inventory_baseline_path)?;
    let inventory_baseline = compare_inventory_baseline(&inventory, &baseline);
    let known_differences = divergences::validate_known_difference_ledger()?;

    Ok(LocalCheckReport {
        inventory_baseline_path,
        inventory,
        inventory_baseline,
        known_differences,
    })
}

fn print_check_report(report: &LocalCheckReport) {
    println!("rum-difftest check");
    print_inventory_baseline_check(report);
    print_lane_c_check(report);
    println!("checks passed: {}", 2 - report.failed_checks());
    println!("checks failed: {}", report.failed_checks());
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
    let status = if report.known_differences.is_clean() {
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
    println!(
        "fixture known differences: {}",
        report.known_differences.fixture_references.len()
    );
    println!(
        "lane-c issues: {}",
        lane_c_issue_count(&report.known_differences)
    );
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

fn print_check_usage() {
    println!("rum-difftest check");
    println!("  --inventory-baseline PATH  Use an explicit inventory baseline");
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!(
        "  Run the fast non-oracle local gates: inventory baseline and Lane C ledger validation"
    );
}
