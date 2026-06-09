use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::inventory::{
    capture_inventory_baseline, collect_fixture_inventory, compare_inventory_baseline,
    default_inventory_baseline_path, display_path, load_inventory_baseline,
};

use super::expect_value;

#[path = "inventory_cli/filters.rs"]
mod filters;
#[path = "inventory_cli/summary_json.rs"]
mod summary_json;
#[path = "inventory_cli/text_report.rs"]
mod text_report;

use filters::{
    InventoryFilter, apply_inventory_filter, filter_inventory_baseline,
    write_inventory_baseline_with_filter_context,
};
use text_report::{print_baseline_report, print_inventory, print_inventory_usage};

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
