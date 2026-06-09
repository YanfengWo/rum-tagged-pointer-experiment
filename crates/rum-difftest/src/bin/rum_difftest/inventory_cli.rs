use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, InventoryBaselineReport,
    capture_inventory_baseline, collect_fixture_inventory, compare_inventory_baseline,
    default_inventory_baseline_path, display_path, load_inventory_baseline,
    write_inventory_baseline,
};

use super::expect_value;

#[path = "inventory_cli/summary_json.rs"]
mod summary_json;

pub fn run_inventory(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut summary_json_path = None;
    let mut baseline_path = None;
    let mut check_baseline = false;
    let mut fail_on_regression = false;
    let mut write_baseline_path = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--baseline" => {
                baseline_path = Some(PathBuf::from(expect_value(&mut iter, "--baseline")?));
            }
            "--write-baseline" => {
                write_baseline_path =
                    Some(PathBuf::from(expect_value(&mut iter, "--write-baseline")?));
            }
            "--check-baseline" => check_baseline = true,
            "--fail-on-regression" => fail_on_regression = true,
            "--help" | "-h" => {
                print_inventory_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown inventory flag `{other}`").into()),
        }
    }

    let report = collect_fixture_inventory()?;
    print_inventory(&report);
    let effective_baseline_path =
        baseline_path.or_else(|| check_baseline.then(default_inventory_baseline_path));
    let baseline_report = if let Some(path) = &effective_baseline_path {
        let baseline = load_inventory_baseline(path)?;
        let baseline_report = compare_inventory_baseline(&report, &baseline);
        print_baseline_report(path, &baseline_report);
        Some(baseline_report)
    } else {
        None
    };
    if let Some(path) = summary_json_path {
        summary_json::write_summary_json(
            &path,
            &report,
            effective_baseline_path.as_deref(),
            baseline_report.as_ref(),
        )?;
        println!("summary json: {}", path.display());
    }
    if let Some(path) = write_baseline_path {
        let captured_baseline = capture_inventory_baseline(&report);
        write_inventory_baseline(&path, &captured_baseline)?;
        println!("wrote inventory baseline: {}", path.display());
    }
    if fail_on_regression {
        let Some(baseline_report) = baseline_report else {
            return Err("--fail-on-regression requires --baseline PATH or --check-baseline".into());
        };
        if !baseline_report.is_clean() {
            return Ok(ExitCode::FAILURE);
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn print_inventory(report: &FixtureInventoryReport) {
    let totals = report.totals();
    println!("fixture inventory");
    print_totals("total", &totals);
    for lane in &report.lanes {
        println!("---");
        print_totals(&lane.lane, &lane.totals());
        for (source, totals) in lane.totals_by_source() {
            println!(
                "{} source {}: fixtures={} entries={} normal={} known_differences={}",
                lane.lane,
                source,
                totals.fixtures,
                totals.entries,
                totals.normal_entries,
                totals.known_difference_entries
            );
        }
        for (category, totals) in lane.totals_by_category() {
            println!(
                "{} category {}: fixtures={} entries={} normal={} known_differences={}",
                lane.lane,
                category,
                totals.fixtures,
                totals.entries,
                totals.normal_entries,
                totals.known_difference_entries
            );
        }
        for fixture in &lane.fixtures {
            println!(
                "{} fixture {}\t{}\tcategory={}\tentries={}\tnormal={}\tknown_differences={}",
                lane.lane,
                fixture.source,
                display_path(&fixture.path),
                fixture.category,
                fixture.entries,
                fixture.normal_entries,
                fixture.known_difference_entries
            );
        }
    }
}

fn print_totals(label: &str, totals: &FixtureInventoryTotals) {
    println!("{label} fixtures: {}", totals.fixtures);
    println!("{label} entries: {}", totals.entries);
    println!("{label} normal entries: {}", totals.normal_entries);
    println!(
        "{label} known differences: {}",
        totals.known_difference_entries
    );
}

fn print_baseline_report(path: &Path, report: &InventoryBaselineReport) {
    println!("inventory baseline: {}", display_path(path));
    println!(
        "inventory baseline records considered: {}",
        report.records_considered
    );
    println!(
        "inventory baseline regressions: {}",
        report.regressions.len()
    );
    for regression in &report.regressions {
        println!(
            "inventory-baseline-regression\t{}\t{}\t{}\t{}",
            regression.record.lane,
            regression.record.dimension.field_name(),
            regression.record.name,
            regression.issues.join(" | ")
        );
        if let Some(actual) = &regression.actual {
            println!(
                "inventory-baseline-actual\t{}\t{}\t{}\tfixtures={}\tentries={}\tnormal={}\tknown_differences={}",
                regression.record.lane,
                regression.record.dimension.field_name(),
                regression.record.name,
                actual.fixtures,
                actual.entries,
                actual.normal_entries,
                actual.known_difference_entries
            );
        }
    }
}

fn print_inventory_usage() {
    println!("rum-difftest inventory");
    println!("  --baseline PATH           Compare fixture inventory against a baseline");
    println!("  --check-baseline          Compare against the checked default baseline");
    println!("  --fail-on-regression      Exit non-zero if baseline comparison is not clean");
    println!("  --write-baseline PATH     Capture the current inventory as a baseline");
    println!("  --summary-json PATH       Write a stable machine-readable fixture inventory");
}
