use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, InventoryBaselineReport, LaneFixtureInventory,
    collect_fixture_inventory, compare_inventory_baseline, default_inventory_baseline_path,
    display_path, load_inventory_baseline,
};

use super::expect_value;
use super::json_cli::{json_string, write_json_file};

pub fn run_inventory(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut summary_json_path = None;
    let mut baseline_path = None;
    let mut check_baseline = false;
    let mut fail_on_regression = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--baseline" => {
                baseline_path = Some(PathBuf::from(expect_value(&mut iter, "--baseline")?));
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
        write_json_file(
            &path,
            summary_json(
                &report,
                effective_baseline_path.as_deref(),
                baseline_report.as_ref(),
            ),
        )?;
        println!("summary json: {}", path.display());
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
        for fixture in &lane.fixtures {
            println!(
                "{} fixture {}\t{}\tentries={}\tnormal={}\tknown_differences={}",
                lane.lane,
                fixture.source,
                display_path(&fixture.path),
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
            "inventory-baseline-regression\t{}\t{}\t{}",
            regression.record.lane,
            regression.record.source,
            regression.issues.join(" | ")
        );
        if let Some(actual) = &regression.actual {
            println!(
                "inventory-baseline-actual\t{}\t{}\tfixtures={}\tentries={}\tnormal={}\tknown_differences={}",
                regression.record.lane,
                regression.record.source,
                actual.fixtures,
                actual.entries,
                actual.normal_entries,
                actual.known_difference_entries
            );
        }
    }
}

fn summary_json(
    report: &FixtureInventoryReport,
    baseline_path: Option<&Path>,
    baseline_report: Option<&InventoryBaselineReport>,
) -> String {
    let totals = report.totals();
    format!(
        concat!(
            "{{\n",
            "  \"lane\": \"inventory\",\n",
            "  \"totals\": {},\n",
            "  \"baseline\": {},\n",
            "  \"lanes\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        totals_json(&totals, 2),
        baseline_json(baseline_path, baseline_report),
        report
            .lanes
            .iter()
            .map(lane_json)
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn baseline_json(path: Option<&Path>, report: Option<&InventoryBaselineReport>) -> String {
    let Some(report) = report else {
        return "null".to_string();
    };
    let path = path.map(display_path).unwrap_or_default();
    format!(
        concat!(
            "{{\n",
            "    \"path\": \"{}\",\n",
            "    \"clean\": {},\n",
            "    \"records_considered\": {},\n",
            "    \"regressions\": {},\n",
            "    \"regression_details\": [\n",
            "{}\n",
            "    ]\n",
            "  }}"
        ),
        json_string(&path),
        report.is_clean(),
        report.records_considered,
        report.regressions.len(),
        report
            .regressions
            .iter()
            .map(|regression| {
                format!(
                    concat!(
                        "      {{",
                        "\"lane\": \"{}\", ",
                        "\"source\": \"{}\", ",
                        "\"issues\": [{}]",
                        "}}"
                    ),
                    json_string(&regression.record.lane),
                    json_string(&regression.record.source),
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

fn lane_json(lane: &LaneFixtureInventory) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"lane\": \"{}\",\n",
            "      \"totals\": {},\n",
            "      \"sources\": [\n",
            "{}\n",
            "      ],\n",
            "      \"fixtures\": [\n",
            "{}\n",
            "      ]\n",
            "    }}"
        ),
        json_string(&lane.lane),
        totals_json(&lane.totals(), 6),
        lane.totals_by_source()
            .iter()
            .map(|(source, totals)| source_json(source, totals))
            .collect::<Vec<_>>()
            .join(",\n"),
        lane.fixtures
            .iter()
            .map(|fixture| {
                format!(
                    concat!(
                        "        {{",
                        "\"path\": \"{}\", ",
                        "\"source\": \"{}\", ",
                        "\"entries\": {}, ",
                        "\"normal_entries\": {}, ",
                        "\"known_difference_entries\": {}",
                        "}}"
                    ),
                    json_string(&display_path(&fixture.path)),
                    json_string(&fixture.source),
                    fixture.entries,
                    fixture.normal_entries,
                    fixture.known_difference_entries,
                )
            })
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn source_json(source: &str, totals: &FixtureInventoryTotals) -> String {
    format!(
        "        {{\"source\": \"{}\", \"totals\": {}}}",
        json_string(source),
        totals_json(totals, 8)
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

fn print_inventory_usage() {
    println!("rum-difftest inventory");
    println!("  --baseline PATH           Compare fixture inventory against a baseline");
    println!("  --check-baseline          Compare against the checked default baseline");
    println!("  --fail-on-regression      Exit non-zero if baseline comparison is not clean");
    println!("  --summary-json PATH       Write a stable machine-readable fixture inventory");
}
