use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::divergences::{self, KnownDifferenceLedgerReport};
use rum_difftest::inventory::{
    FixtureInventoryReport, InventoryBaselineReport, collect_fixture_inventory,
    compare_inventory_baseline, default_inventory_baseline_path, display_path,
    load_inventory_baseline,
};

use super::expect_value;

#[path = "check_cli/summary_json.rs"]
mod summary_json;

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
        summary_json::write_summary_json(&path, &report)?;
        println!("summary json: {}", path.display());
    }

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

pub(super) struct LocalCheckReport {
    pub(super) inventory_baseline_path: PathBuf,
    pub(super) inventory: FixtureInventoryReport,
    pub(super) inventory_baseline: InventoryBaselineReport,
    pub(super) known_differences: KnownDifferenceLedgerReport,
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

fn print_check_usage() {
    println!("rum-difftest check");
    println!("  --inventory-baseline PATH  Use an explicit inventory baseline");
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!(
        "  Run the fast non-oracle local gates: inventory baseline and Lane C ledger validation"
    );
}
