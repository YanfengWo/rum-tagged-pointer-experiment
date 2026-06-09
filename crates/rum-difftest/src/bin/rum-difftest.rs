use std::process::ExitCode;

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
        "rum-difftest lane-a [--oracle-kernel PATH|--oracle-command PATH] [--oracle-timeout-seconds N] [--config PATH] [--fixture PATH] [--limit N]"
    );
    println!(
        "rum-difftest lane-b [--oracle-kernel PATH|--oracle-command PATH] [--oracle-timeout-seconds N] [--config PATH] [--fixture PATH] [--limit N]"
    );
    println!("rum-difftest lane-c");
    println!(
        "rum-difftest lane-d [--suite all|timing|memory] [--profile smoke|full] [--limit N] [--baseline PATH] [--write-baseline PATH] [--check-baseline-readiness]"
    );
    println!("rum-difftest --help");
}
