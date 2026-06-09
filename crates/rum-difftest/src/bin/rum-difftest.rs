use std::process::ExitCode;

#[path = "rum_difftest/check_cli.rs"]
mod check_cli;
#[path = "rum_difftest/inventory_cli.rs"]
mod inventory_cli;
#[path = "rum_difftest/json_cli.rs"]
mod json_cli;
#[path = "rum_difftest/lane_a_cli.rs"]
mod lane_a_cli;
#[path = "rum_difftest/lane_b_cli.rs"]
mod lane_b_cli;
#[path = "rum_difftest/lane_c_cli.rs"]
mod lane_c_cli;
#[path = "rum_difftest/lane_d_cli.rs"]
mod lane_d_cli;
#[path = "rum_difftest/oracle_cli.rs"]
mod oracle_cli;
#[path = "rum_difftest/oracle_lane_summary_json.rs"]
mod oracle_lane_summary_json;

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
        Some("check") => check_cli::run_check(args.collect()),
        Some("inventory") => inventory_cli::run_inventory(args.collect()),
        Some("lane-a") => lane_a_cli::run_lane_a(args.collect()),
        Some("lane-b") => lane_b_cli::run_lane_b(args.collect()),
        Some("lane-c") => lane_c_cli::run_lane_c(args.collect()),
        Some("lane-d") => lane_d_cli::run_lane_d(args.collect()),
        Some("--help") | Some("-h") | None => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        Some(other) => Err(format!("unknown subcommand `{other}`").into()),
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
        "rum-difftest check [--inventory-baseline PATH] [--include-lane-d-smoke] [--lane-d-smoke-baseline PATH] [--include-lane-d-readiness] [--summary-json PATH] [--max-issue-samples N]"
    );
    println!(
        "rum-difftest lane-a [--oracle-kernel PATH|--oracle-command PATH] [--oracle-timeout-seconds N] [--config PATH] [--fixture PATH] [--limit N] [--summary-json PATH]"
    );
    println!(
        "rum-difftest lane-b [--oracle-kernel PATH|--oracle-command PATH] [--oracle-timeout-seconds N] [--config PATH] [--fixture PATH] [--limit N] [--summary-json PATH]"
    );
    println!("rum-difftest lane-c [--summary-json PATH]");
    println!(
        "rum-difftest lane-d [--suite all|timing|memory] [--profile smoke|full] [--limit N] [--baseline PATH] [--write-baseline PATH] [--check-baseline-readiness] [--summary-json PATH]"
    );
    println!(
        "rum-difftest inventory [--baseline PATH] [--check-baseline] [--fail-on-regression] [--write-baseline PATH] [--summary-json PATH]"
    );
    println!("rum-difftest --help");
}
