use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::config::{load_optional_config, resolve_oracle_kernel};
use rum_difftest::lane_a::CaseValue;
use rum_difftest::lane_b::{self, LaneBOptions, LaneBStepStatus};
use rum_difftest::oracle::OracleKernel;

use super::oracle_cli::print_oracle_header;

pub fn run_lane_b(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
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
            "--oracle-kernel" | "--oracle-command" => {
                oracle_kernel = Some(PathBuf::from(expect_value(&mut iter, &arg)?))
            }
            "--fixture" => fixture = Some(PathBuf::from(expect_value(&mut iter, "--fixture")?)),
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                limit = Some(value.parse::<usize>()?);
            }
            "--include-known-diffs" | "--include-known-differences" => {
                include_known_differences = true
            }
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
        "no oracle kernel configured; pass --oracle-kernel/--oracle-command PATH or set oracle_kernel/oracle_command in a config file"
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

    print_oracle_header("lane-b", &oracle);
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
    println!(
        "confirmed known differences: {}",
        summary.confirmed_known_differences
    );
    println!(
        "resolved known differences: {}",
        summary.resolved_known_differences
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
        if let Some(known_difference) = &step.known_difference {
            println!(
                "known difference: id={} type={}",
                known_difference.id, known_difference.kind
            );
            println!("known reason: {}", known_difference.reason);
        }
        if let Some(expected_seed) = &step.expected_seed {
            println!("seed expected: {expected_seed}");
        }
        print_case_value("rum", &step.rum);
        print_case_value("oracle", &step.oracle);
    }
    for step in summary
        .steps
        .iter()
        .filter(|step| step.status == LaneBStepStatus::Match && step.known_difference.is_some())
        .take(5)
    {
        if let Some(known_difference) = &step.known_difference {
            println!("---");
            println!(
                "resolved known difference: id={} type={}",
                known_difference.id, known_difference.kind
            );
            println!("fixture: {}", step.fixture.display());
            println!("step: {}", step.step_index);
            println!("input: {}", step.input);
        }
    }

    let has_issues =
        summary.mismatches > 0 || summary.rum_failures > 0 || summary.oracle_failures > 0;
    if fail_on_mismatch && has_issues {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn expect_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next()
        .ok_or_else(|| format!("{flag} expects a value").into())
}

fn print_lane_b_usage() {
    println!("rum-difftest lane-b [--oracle-kernel PATH]");
    println!("rum-difftest lane-b [--oracle-command PATH]");
    println!("rum-difftest lane-b [--config PATH]");
    println!("  --fixture PATH            Run a single stateful fixture file");
    println!("  --limit N                 Stop after N compared steps");
    println!("  --include-known-diffs     Include `!!!` steps in the run");
    println!("  --include-known-differences");
    println!("                              Alias for --include-known-diffs");
    println!("  --fail-on-mismatch        Exit non-zero if any step mismatches or fails");
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
