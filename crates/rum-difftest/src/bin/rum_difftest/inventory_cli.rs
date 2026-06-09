use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, InventoryBaseline, InventoryBaselineDimension,
    InventoryBaselineReport, capture_inventory_baseline, collect_fixture_inventory,
    compare_inventory_baseline, default_inventory_baseline_path, display_path,
    format_inventory_baseline, load_inventory_baseline, write_inventory_baseline,
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
    let mut filter = InventoryFilter::default();

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
            "--lane" => filter.lane = Some(expect_value(&mut iter, "--lane")?),
            "--source" => filter.source = Some(expect_value(&mut iter, "--source")?),
            "--category" => filter.category = Some(expect_value(&mut iter, "--category")?),
            "--check-baseline" => check_baseline = true,
            "--fail-on-regression" => fail_on_regression = true,
            "--help" | "-h" => {
                print_inventory_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown inventory flag `{other}`").into()),
        }
    }
    if fail_on_regression && baseline_path.is_none() && !check_baseline {
        return Err("--fail-on-regression requires --baseline PATH or --check-baseline".into());
    }

    let mut report = collect_fixture_inventory()?;
    apply_inventory_filter(&mut report, &filter)?;
    print_inventory(&report, &filter);
    let effective_baseline_path =
        baseline_path.or_else(|| check_baseline.then(default_inventory_baseline_path));
    let baseline_report = if let Some(path) = &effective_baseline_path {
        let baseline = load_inventory_baseline(path)?;
        let baseline = filter_inventory_baseline(&baseline, &filter);
        if filter.is_active() && baseline.records.is_empty() {
            return Err(format!(
                "inventory filters matched no baseline records in {}: {}",
                display_path(path),
                filter.description()
            )
            .into());
        }
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
            &filter,
            effective_baseline_path.as_deref(),
            baseline_report.as_ref(),
        )?;
        println!("summary json: {}", path.display());
    }
    if let Some(path) = write_baseline_path {
        let captured_baseline = capture_inventory_baseline(&report);
        write_inventory_baseline_with_filter_context(&path, &captured_baseline, &filter)?;
        println!("wrote inventory baseline: {}", path.display());
    }
    if fail_on_regression {
        let baseline_report = baseline_report
            .as_ref()
            .expect("inventory baseline report is present after fail-on-regression validation");
        if !baseline_report.is_clean() {
            return Ok(ExitCode::FAILURE);
        }
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct InventoryFilter {
    pub(super) lane: Option<String>,
    pub(super) source: Option<String>,
    pub(super) category: Option<String>,
}

impl InventoryFilter {
    pub(super) fn is_active(&self) -> bool {
        self.lane.is_some() || self.source.is_some() || self.category.is_some()
    }

    fn description(&self) -> String {
        [
            self.lane.as_deref().map(|value| format!("lane={value}")),
            self.source
                .as_deref()
                .map(|value| format!("source={value}")),
            self.category
                .as_deref()
                .map(|value| format!("category={value}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
    }
}

fn apply_inventory_filter(
    report: &mut FixtureInventoryReport,
    filter: &InventoryFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    if !filter.is_active() {
        return Ok(());
    }

    report.lanes.retain(|lane| {
        filter
            .lane
            .as_deref()
            .is_none_or(|value| value == lane.lane)
    });
    for lane in &mut report.lanes {
        lane.fixtures.retain(|fixture| {
            filter
                .source
                .as_deref()
                .is_none_or(|value| value == fixture.source)
                && filter
                    .category
                    .as_deref()
                    .is_none_or(|value| value == fixture.category)
        });
    }
    report.lanes.retain(|lane| !lane.fixtures.is_empty());

    if report.totals().fixtures == 0 {
        return Err(format!(
            "inventory filters matched no fixtures: {}",
            filter.description()
        )
        .into());
    }
    Ok(())
}

fn filter_inventory_baseline(
    baseline: &InventoryBaseline,
    filter: &InventoryFilter,
) -> InventoryBaseline {
    if !filter.is_active() {
        return baseline.clone();
    }

    InventoryBaseline {
        records: baseline
            .records
            .iter()
            .filter(|record| baseline_record_matches_filter(record, filter))
            .cloned()
            .collect(),
    }
}

fn write_inventory_baseline_with_filter_context(
    path: &Path,
    baseline: &InventoryBaseline,
    filter: &InventoryFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    if !filter.is_active() {
        write_inventory_baseline(path, baseline)?;
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut text = format!(
        "# Filtered non-oracle fixture coverage baseline.\n\
         # Filters: {}\n\
         # Use an unfiltered `rum-difftest inventory --write-baseline` for the repository-wide checked baseline.\n",
        filter.description()
    );
    text.push_str(&format_inventory_baseline(baseline));
    fs::write(path, text)?;
    Ok(())
}

fn baseline_record_matches_filter(
    record: &rum_difftest::inventory::InventoryBaselineRecord,
    filter: &InventoryFilter,
) -> bool {
    if filter
        .lane
        .as_deref()
        .is_some_and(|lane| lane != record.lane)
    {
        return false;
    }
    if let Some(category) = filter.category.as_deref() {
        return record.dimension == InventoryBaselineDimension::Category && record.name == category;
    }
    if let Some(source) = filter.source.as_deref() {
        return record.dimension == InventoryBaselineDimension::Source && record.name == source;
    }
    true
}

fn print_inventory(report: &FixtureInventoryReport, filter: &InventoryFilter) {
    let totals = report.totals();
    println!("fixture inventory");
    if filter.is_active() {
        println!("inventory filters: {}", filter.description());
    }
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
    println!("  --lane NAME               Restrict inventory to one lane");
    println!("  --source NAME             Restrict inventory to one fixture source");
    println!("  --category NAME           Restrict inventory to one semantic category");
}
