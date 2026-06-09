use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, LaneFixtureInventory,
    collect_fixture_inventory, display_path,
};

use super::expect_value;
use super::json_cli::{json_string, write_json_file};

pub fn run_inventory(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut summary_json_path = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--help" | "-h" => {
                print_inventory_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown inventory flag `{other}`").into()),
        }
    }

    let report = collect_fixture_inventory()?;
    print_inventory(&report);
    if let Some(path) = summary_json_path {
        write_json_file(&path, summary_json(&report))?;
        println!("summary json: {}", path.display());
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

fn summary_json(report: &FixtureInventoryReport) -> String {
    let totals = report.totals();
    format!(
        concat!(
            "{{\n",
            "  \"lane\": \"inventory\",\n",
            "  \"totals\": {},\n",
            "  \"lanes\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        totals_json(&totals, 2),
        report
            .lanes
            .iter()
            .map(lane_json)
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
    println!("  --summary-json PATH       Write a stable machine-readable fixture inventory");
}
