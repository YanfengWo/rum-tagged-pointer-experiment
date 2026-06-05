use std::process::ExitCode;

use rum_difftest::divergences;

pub fn run_lane_c(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_lane_c_usage();
                return Ok(ExitCode::SUCCESS);
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

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn print_lane_c_usage() {
    println!("rum-difftest lane-c");
    println!(
        "  Validate fixture `!!! id=...`, retired seed rows, fixture metadata, and active `tracked_by=path:needle` targets"
    );
}
