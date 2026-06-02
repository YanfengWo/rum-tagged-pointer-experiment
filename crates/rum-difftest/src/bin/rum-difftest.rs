use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::config::{load_optional_config, resolve_oracle_kernel};
use rum_difftest::divergences;
use rum_difftest::lane_a::{self, CaseValue, LaneACaseStatus, LaneAOptions};
use rum_difftest::lane_b::{self, LaneBOptions, LaneBStepStatus};
use rum_difftest::oracle::OracleKernel;

fn main() -> ExitCode {
    match run() {
        Ok(exit_code) => exit_code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("lane-a") => run_lane_a(args.collect()),
        Some("lane-b") => run_lane_b(args.collect()),
        Some("lane-c") => run_lane_c(args.collect()),
        Some("--help") | Some("-h") | None => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        Some(other) => Err(format!("unknown subcommand `{other}`").into()),
    }
}

fn run_lane_a(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut config_path = None;
    let mut oracle_kernel = None;
    let mut fixture = None;
    let mut limit = None;
    let mut include_known_differences = false;
    let mut fail_on_mismatch = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => config_path = Some(PathBuf::from(expect_value(&mut iter, "--config")?)),
            "--oracle-kernel" => {
                oracle_kernel = Some(PathBuf::from(expect_value(&mut iter, "--oracle-kernel")?))
            }
            "--fixture" => fixture = Some(PathBuf::from(expect_value(&mut iter, "--fixture")?)),
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                limit = Some(value.parse::<usize>()?);
            }
            "--include-known-diffs" => include_known_differences = true,
            "--fail-on-mismatch" => fail_on_mismatch = true,
            "--help" | "-h" => {
                print_lane_a_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown lane-a flag `{other}`").into()),
        }
    }

    let config = load_optional_config(config_path.as_deref())?;
    let oracle_kernel = resolve_oracle_kernel(oracle_kernel, &config).ok_or_else(|| {
        "no oracle kernel configured; pass --oracle-kernel PATH or set oracle_kernel in a config file"
    })?;

    let mut oracle = OracleKernel::new(&oracle_kernel)?;
    let summary = lane_a::run(
        &mut oracle,
        &LaneAOptions {
            fixture,
            limit,
            include_known_differences,
        },
    )?;

    println!("lane-a oracle: {}", oracle.executable().display());
    println!("cases seen: {}", summary.total_seen);
    println!("cases compared: {}", summary.compared);
    println!("matches: {}", summary.matches);
    println!("mismatches: {}", summary.mismatches);
    println!("rum failures: {}", summary.rum_failures);
    println!("oracle failures: {}", summary.oracle_failures);
    println!(
        "skipped known differences: {}",
        summary.skipped_known_differences
    );

    for case in summary
        .cases
        .iter()
        .filter(|case| case.status != LaneACaseStatus::Match)
        .take(5)
    {
        println!("---");
        println!("fixture: {}", case.fixture.display());
        println!("input: {}", case.input);
        println!("status: {:?}", case.status);
        if let Some(expected_seed) = &case.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        match &case.rum {
            CaseValue::Value {
                value, messages, ..
            } => {
                println!("rum value: {value}");
                if !messages.is_empty() {
                    println!("rum messages: {}", messages.join(" | "));
                }
            }
            CaseValue::Failure { message } => println!("rum failure: {message}"),
        }
        match &case.oracle {
            CaseValue::Value {
                value,
                messages,
                printed_output,
            } => {
                println!("oracle value: {value}");
                if !messages.is_empty() {
                    println!("oracle messages: {}", messages.join(" | "));
                }
                if let Some(printed_output) = printed_output {
                    println!("oracle printed: {printed_output}");
                }
            }
            CaseValue::Failure { message } => println!("oracle failure: {message}"),
        }
    }

    let has_issues =
        summary.mismatches > 0 || summary.rum_failures > 0 || summary.oracle_failures > 0;
    if fail_on_mismatch && has_issues {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn run_lane_b(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut config_path = None;
    let mut oracle_kernel = None;
    let mut fixture = None;
    let mut limit = None;
    let mut include_known_differences = false;
    let mut fail_on_mismatch = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => config_path = Some(PathBuf::from(expect_value(&mut iter, "--config")?)),
            "--oracle-kernel" => {
                oracle_kernel = Some(PathBuf::from(expect_value(&mut iter, "--oracle-kernel")?))
            }
            "--fixture" => fixture = Some(PathBuf::from(expect_value(&mut iter, "--fixture")?)),
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                limit = Some(value.parse::<usize>()?);
            }
            "--include-known-diffs" => include_known_differences = true,
            "--fail-on-mismatch" => fail_on_mismatch = true,
            "--help" | "-h" => {
                print_lane_b_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown lane-b flag `{other}`").into()),
        }
    }

    let config = load_optional_config(config_path.as_deref())?;
    let oracle_kernel = resolve_oracle_kernel(oracle_kernel, &config).ok_or_else(|| {
        "no oracle kernel configured; pass --oracle-kernel PATH or set oracle_kernel in a config file"
    })?;

    let mut oracle = OracleKernel::new(&oracle_kernel)?;
    let summary = lane_b::run(
        &mut oracle,
        &LaneBOptions {
            fixture,
            limit,
            include_known_differences,
        },
    )?;

    println!("lane-b oracle: {}", oracle.executable().display());
    println!("steps seen: {}", summary.total_seen);
    println!("steps compared: {}", summary.compared);
    println!("matches: {}", summary.matches);
    println!("mismatches: {}", summary.mismatches);
    println!("rum failures: {}", summary.rum_failures);
    println!("oracle failures: {}", summary.oracle_failures);
    println!(
        "skipped known differences: {}",
        summary.skipped_known_differences
    );

    for step in summary
        .steps
        .iter()
        .filter(|step| step.status != LaneBStepStatus::Match)
        .take(5)
    {
        println!("---");
        println!("fixture: {}", step.fixture.display());
        println!("step: {}", step.step_index);
        println!("input: {}", step.input);
        println!("status: {:?}", step.status);
        if let Some(expected_seed) = &step.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        print_case_value("rum", &step.rum);
        print_case_value("oracle", &step.oracle);
    }

    let has_issues =
        summary.mismatches > 0 || summary.rum_failures > 0 || summary.oracle_failures > 0;
    if fail_on_mismatch && has_issues {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn run_lane_c(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
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
    println!("ledger entries: {}", report.records.len());
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

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn expect_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next()
        .ok_or_else(|| format!("{flag} expects a value").into())
}

fn print_usage() {
    println!(
        "rum-difftest lane-a [--oracle-kernel PATH] [--config PATH] [--fixture PATH] [--limit N]"
    );
    println!(
        "rum-difftest lane-b [--oracle-kernel PATH] [--config PATH] [--fixture PATH] [--limit N]"
    );
    println!("rum-difftest lane-c");
    println!("rum-difftest --help");
}

fn print_lane_a_usage() {
    println!("rum-difftest lane-a [--oracle-kernel PATH]");
    println!("rum-difftest lane-a [--config PATH]");
    println!("  --fixture PATH            Run a single imported golden file");
    println!("  --limit N                 Stop after N compared cases");
    println!("  --include-known-diffs     Include `!!!` cases in the run");
    println!("  --fail-on-mismatch        Exit non-zero if any case mismatches or fails");
}

fn print_lane_b_usage() {
    println!("rum-difftest lane-b [--oracle-kernel PATH]");
    println!("rum-difftest lane-b [--config PATH]");
    println!("  --fixture PATH            Run a single stateful fixture file");
    println!("  --limit N                 Stop after N compared steps");
    println!("  --include-known-diffs     Include `!!!` steps in the run");
    println!("  --fail-on-mismatch        Exit non-zero if any step mismatches or fails");
}

fn print_lane_c_usage() {
    println!("rum-difftest lane-c");
    println!("  Validate that every fixture `!!! id=...` is present in the Lane C ledger");
}

fn print_case_value(label: &str, value: &CaseValue) {
    match value {
        CaseValue::Value {
            value,
            messages,
            printed_output,
        } => {
            println!("{label} value: {value}");
            if !messages.is_empty() {
                println!("{label} messages: {}", messages.join(" | "));
            }
            if let Some(printed_output) = printed_output {
                println!("{label} printed: {printed_output}");
            }
        }
        CaseValue::Failure { message } => println!("{label} failure: {message}"),
    }
}
