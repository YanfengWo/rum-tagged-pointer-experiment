use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::divergences;

use super::expect_value;
use super::json_cli::{SUMMARY_SCHEMA_VERSION, json_string, write_json_file};

pub fn run_lane_c(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut summary_json_path = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_lane_c_usage();
                return Ok(ExitCode::SUCCESS);
            }
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            other => return Err(format!("unknown lane-c flag `{other}`").into()),
        }
    }

    let report = divergences::validate_known_difference_ledger()?;

    println!(
        "known-difference ledger: {}",
        divergences::known_difference_ledger_path().display()
    );
    println!(
        "retired seed differences: {}",
        divergences::retired_seed_difference_path().display()
    );
    println!("active ledger entries: {}", report.records.len());
    for (status, entries) in report.active_ledger_field_counts("status") {
        println!("active ledger status {status}: {entries}");
    }
    for (kind, entries) in report.active_ledger_field_counts("type") {
        println!("active ledger type {kind}: {entries}");
    }
    println!(
        "retired seed entries: {}",
        report.retired_seed_records.len()
    );
    println!(
        "fixture known differences: {}",
        report.fixture_references.len()
    );
    println!(
        "missing ledger entries: {}",
        report.missing_ledger_entries.len()
    );
    println!(
        "unreferenced ledger entries: {}",
        report.unreferenced_ledger_entries.len()
    );
    println!(
        "unreferenced retired seed entries: {}",
        report.unreferenced_retired_seed_entries.len()
    );
    println!("fixture mismatches: {}", report.fixture_mismatches.len());
    println!("metadata mismatches: {}", report.metadata_mismatches.len());
    println!(
        "retired seed fixture mismatches: {}",
        report.retired_seed_fixture_mismatches.len()
    );
    println!(
        "tracking target issues: {}",
        report.tracking_target_issues.len()
    );

    for reference in &report.missing_ledger_entries {
        println!(
            "missing: id={} fixture={}",
            reference.id,
            reference.fixture.display()
        );
    }
    for record in &report.unreferenced_ledger_entries {
        let fixture = record
            .fields
            .get("fixture")
            .map(String::as_str)
            .unwrap_or("<missing fixture>");
        println!("unreferenced: id={} fixture={fixture}", record.id);
    }
    for record in &report.unreferenced_retired_seed_entries {
        let fixture = record
            .fields
            .get("seed_fixture")
            .map(String::as_str)
            .unwrap_or("<missing seed_fixture>");
        println!(
            "unreferenced-retired-seed: id={} fixture={fixture}",
            record.id
        );
    }
    for mismatch in &report.fixture_mismatches {
        println!(
            "fixture-mismatch: id={} ledger={} actual={}",
            mismatch.id,
            mismatch.ledger_fixture.display(),
            mismatch.actual_fixture.display()
        );
    }
    for mismatch in &report.metadata_mismatches {
        println!(
            "metadata-mismatch: id={} field={} ledger={} fixture={}",
            mismatch.id, mismatch.field, mismatch.ledger_value, mismatch.fixture_value
        );
    }
    for mismatch in &report.retired_seed_fixture_mismatches {
        println!(
            "retired-seed-fixture-mismatch: id={} retired={} actual={}",
            mismatch.id,
            mismatch.ledger_fixture.display(),
            mismatch.actual_fixture.display()
        );
    }
    for issue in &report.tracking_target_issues {
        println!(
            "tracking-target-issue: id={} target={} message={}",
            issue.id, issue.target, issue.message
        );
    }

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

fn summary_json(report: &divergences::KnownDifferenceLedgerReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"lane\": \"lane-c\",\n",
            "  \"schema_version\": {},\n",
            "  \"clean\": {},\n",
            "  \"known_difference_ledger\": \"{}\",\n",
            "  \"retired_seed_differences\": \"{}\",\n",
            "  \"counts\": {{\n",
            "    \"active_ledger_entries\": {},\n",
            "    \"active_ledger_status_counts\": [\n",
            "{}\n",
            "    ],\n",
            "    \"active_ledger_type_counts\": [\n",
            "{}\n",
            "    ],\n",
            "    \"retired_seed_entries\": {},\n",
            "    \"fixture_known_differences\": {}\n",
            "  }},\n",
            "  \"issues\": {{\n",
            "    \"missing_ledger_entries\": {},\n",
            "    \"unreferenced_ledger_entries\": {},\n",
            "    \"unreferenced_retired_seed_entries\": {},\n",
            "    \"fixture_mismatches\": {},\n",
            "    \"metadata_mismatches\": {},\n",
            "    \"retired_seed_fixture_mismatches\": {},\n",
            "    \"tracking_target_issues\": {}\n",
            "  }}\n",
            "}}\n"
        ),
        SUMMARY_SCHEMA_VERSION,
        report.is_clean(),
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
        report.records.len(),
        active_ledger_field_counts_json(report, "status", "status", 6),
        active_ledger_field_counts_json(report, "type", "type", 6),
        report.retired_seed_records.len(),
        report.fixture_references.len(),
        report.missing_ledger_entries.len(),
        report.unreferenced_ledger_entries.len(),
        report.unreferenced_retired_seed_entries.len(),
        report.fixture_mismatches.len(),
        report.metadata_mismatches.len(),
        report.retired_seed_fixture_mismatches.len(),
        report.tracking_target_issues.len(),
    )
}

pub(super) fn active_ledger_field_counts_json(
    report: &divergences::KnownDifferenceLedgerReport,
    field: &str,
    json_key: &str,
    indent: usize,
) -> String {
    let spaces = " ".repeat(indent);
    report
        .active_ledger_field_counts(field)
        .into_iter()
        .map(|(value, entries)| {
            format!(
                "{spaces}{{\"{}\": \"{}\", \"entries\": {}}}",
                json_string(json_key),
                json_string(&value),
                entries
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn print_lane_c_usage() {
    println!("rum-difftest lane-c");
    println!("  --summary-json PATH       Write a stable machine-readable summary");
    println!(
        "  Validate fixture `!!! id=...`, retired seed rows, fixture/type metadata, and active `tracked_by=path:needle` targets"
    );
}
