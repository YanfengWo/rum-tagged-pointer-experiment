use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::divergences::{self, KnownDifferenceLedgerReport};
use rum_difftest::inventory::{
    FixtureInventoryReport, InventoryBaselineReport, collect_fixture_inventory,
    compare_inventory_baseline, default_inventory_baseline_path, display_path,
    load_inventory_baseline,
};
use rum_difftest::lane_d::{
    self, LaneDBaselineCompareOptions, LaneDBaselineReport, LaneDOptions, LaneDProfile, LaneDSuite,
    LaneDSummary, LaneDThresholdProfile,
};

use super::expect_value;

#[path = "check_cli/summary_json.rs"]
mod summary_json;

pub fn run_check(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut inventory_baseline_path = None;
    let mut lane_d_smoke_baseline = None;
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
            "--include-lane-d-smoke" => {
                if lane_d_smoke_baseline.is_none() {
                    lane_d_smoke_baseline = Some(LaneDSmokeBaseline::Default);
                }
            }
            "--lane-d-smoke-baseline" => {
                lane_d_smoke_baseline = Some(LaneDSmokeBaseline::Path(PathBuf::from(
                    expect_value(&mut iter, "--lane-d-smoke-baseline")?,
                )));
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

    let report = run_local_checks(inventory_baseline_path, lane_d_smoke_baseline)?;
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
    pub(super) lane_d_smoke: Option<LaneDSmokeCheckReport>,
}

pub(super) struct LaneDSmokeCheckReport {
    pub(super) baseline_path: PathBuf,
    pub(super) summary: LaneDSummary,
    pub(super) baseline: LaneDBaselineReport,
}

enum LaneDSmokeBaseline {
    Default,
    Path(PathBuf),
}

impl LocalCheckReport {
    fn is_clean(&self) -> bool {
        self.inventory_baseline.is_clean()
            && self.known_differences.is_clean()
            && self
                .lane_d_smoke
                .as_ref()
                .is_none_or(LaneDSmokeCheckReport::is_clean)
    }

    fn failed_checks(&self) -> usize {
        usize::from(!self.inventory_baseline.is_clean())
            + usize::from(!self.known_differences.is_clean())
            + self
                .lane_d_smoke
                .as_ref()
                .map_or(0, |report| usize::from(!report.is_clean()))
    }

    pub(super) fn total_checks(&self) -> usize {
        2 + usize::from(self.lane_d_smoke.is_some())
    }

    pub(super) fn passed_checks(&self) -> usize {
        self.total_checks() - self.failed_checks()
    }
}

impl LaneDSmokeCheckReport {
    pub(super) fn is_clean(&self) -> bool {
        self.summary.failed == 0 && self.baseline.is_clean()
    }

    fn baseline_issue_count(&self) -> usize {
        self.baseline.missing_records.len()
            + self.baseline.unused_records.len()
            + self.baseline.elapsed_regressions.len()
            + self.baseline.value_node_regressions.len()
    }
}

fn run_local_checks(
    inventory_baseline_path: Option<PathBuf>,
    lane_d_smoke_baseline: Option<LaneDSmokeBaseline>,
) -> Result<LocalCheckReport, Box<dyn std::error::Error>> {
    let inventory_baseline_path =
        inventory_baseline_path.unwrap_or_else(default_inventory_baseline_path);
    let inventory = collect_fixture_inventory()?;
    let baseline = load_inventory_baseline(&inventory_baseline_path)?;
    let inventory_baseline = compare_inventory_baseline(&inventory, &baseline);
    let known_differences = divergences::validate_known_difference_ledger()?;
    let lane_d_smoke = if let Some(baseline) = lane_d_smoke_baseline {
        Some(run_lane_d_smoke_check(baseline)?)
    } else {
        None
    };

    Ok(LocalCheckReport {
        inventory_baseline_path,
        inventory,
        inventory_baseline,
        known_differences,
        lane_d_smoke,
    })
}

fn run_lane_d_smoke_check(
    baseline: LaneDSmokeBaseline,
) -> Result<LaneDSmokeCheckReport, Box<dyn std::error::Error>> {
    let baseline_path = match baseline {
        LaneDSmokeBaseline::Default => lane_d::default_smoke_baseline_path(),
        LaneDSmokeBaseline::Path(path) => path,
    };
    let summary = lane_d::run(&LaneDOptions {
        suite: LaneDSuite::All,
        profile: LaneDProfile::Smoke,
        limit: None,
    })?;
    let baseline = lane_d::load_baseline(&baseline_path)?;
    let baseline = lane_d::compare_baseline(
        &summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            require_all_records: true,
        },
    );

    Ok(LaneDSmokeCheckReport {
        baseline_path,
        summary,
        baseline,
    })
}

fn print_check_report(report: &LocalCheckReport) {
    println!("rum-difftest check");
    print_inventory_baseline_check(report);
    print_lane_c_check(report);
    if let Some(lane_d_smoke) = &report.lane_d_smoke {
        print_lane_d_smoke_check(lane_d_smoke);
    }
    println!("checks passed: {}", report.passed_checks());
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

fn print_lane_d_smoke_check(report: &LaneDSmokeCheckReport) {
    let status = if report.is_clean() { "clean" } else { "dirty" };
    println!("check\tlane-d-smoke\t{status}");
    println!(
        "lane-d smoke baseline: {}",
        display_path(&report.baseline_path)
    );
    println!("lane-d cases seen: {}", report.summary.total_seen);
    println!("lane-d cases executed: {}", report.summary.executed);
    println!("lane-d failures: {}", report.summary.failed);
    println!(
        "lane-d baseline records considered: {}",
        report.baseline.records_considered
    );
    println!(
        "lane-d baseline compared cases: {}",
        report.baseline.compared
    );
    println!(
        "lane-d baseline missing records: {}",
        report.baseline.missing_records.len()
    );
    println!(
        "lane-d baseline unused records: {}",
        report.baseline.unused_records.len()
    );
    println!(
        "lane-d baseline elapsed regressions: {}",
        report.baseline.elapsed_regressions.len()
    );
    println!(
        "lane-d baseline value-node regressions: {}",
        report.baseline.value_node_regressions.len()
    );
    println!("lane-d baseline issues: {}", report.baseline_issue_count());
}

fn print_check_usage() {
    println!("rum-difftest check");
    println!("  --inventory-baseline PATH  Use an explicit inventory baseline");
    println!("  --include-lane-d-smoke     Also run the local Lane D smoke baseline gate");
    println!("  --lane-d-smoke-baseline PATH");
    println!(
        "                              Use an explicit Lane D smoke baseline and enable that gate"
    );
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!(
        "  Run the fast non-oracle local gates: inventory baseline and Lane C ledger validation, with optional Lane D smoke"
    );
}
