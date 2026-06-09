use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::config::{load_optional_config, resolve_oracle_kernel, resolve_oracle_timeout};
use rum_difftest::lane_a::{self, LaneACaseStatus, LaneAOptions};
use rum_difftest::oracle::OracleKernel;

use super::oracle_cli::print_oracle_header;
use super::oracle_lane_summary_json::{
    DEFAULT_MAX_ISSUE_SAMPLES, OracleLaneIssueSample, OracleLaneIssueSamples,
    OracleLaneSessionBatchTransport, OracleLaneSummaryCounts, OracleLaneSummaryJsonContext,
    parse_max_issue_samples, write_summary_json,
};

#[path = "lane_a_cli/text_report.rs"]
mod text_report;

pub fn run_lane_a(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut config_path = None;
    let mut oracle_kernel = None;
    let mut fixture = None;
    let mut limit = None;
    let mut include_known_differences = false;
    let mut fail_on_mismatch = false;
    let mut oracle_timeout_seconds = None;
    let mut summary_json_path = None;
    let mut max_issue_samples = DEFAULT_MAX_ISSUE_SAMPLES;

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
            "--oracle-timeout-seconds" => {
                let value = expect_value(&mut iter, "--oracle-timeout-seconds")?;
                oracle_timeout_seconds = Some(value.parse::<u64>()?);
            }
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--max-issue-samples" => {
                let value = expect_value(&mut iter, "--max-issue-samples")?;
                max_issue_samples = parse_max_issue_samples(&value)?;
            }
            "--fail-on-mismatch" => fail_on_mismatch = true,
            "--help" | "-h" => {
                print_lane_a_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown lane-a flag `{other}`").into()),
        }
    }

    let config = load_optional_config(config_path.as_deref())?;
    let oracle_kernel = resolve_oracle_kernel(oracle_kernel, &config).ok_or(
        "no oracle kernel configured; pass --oracle-kernel/--oracle-command PATH or set oracle_kernel/oracle_command in a config file",
    )?;
    let oracle_timeout = resolve_oracle_timeout(oracle_timeout_seconds, &config)?;

    let mut oracle = OracleKernel::with_timeout(&oracle_kernel, oracle_timeout)?;
    let options = LaneAOptions {
        fixture,
        limit,
        include_known_differences,
    };
    let summary = lane_a::run(&mut oracle, &options)?;

    print_oracle_header("lane-a", &oracle);
    let oracle_processes = oracle.evaluation_process_count(summary.compared);
    let session_batch_transport = OracleLaneSessionBatchTransport {
        name: oracle.session_batch_transport_name(),
        process_count: Some(oracle_processes),
    };
    text_report::print_lane_a_report(&summary, oracle_processes);

    if let Some(path) = &summary_json_path {
        write_summary_json(
            path,
            &OracleLaneSummaryJsonContext {
                lane: "lane-a",
                oracle: &oracle,
                fixture: options.fixture.as_deref(),
                limit: options.limit,
                include_known_differences: options.include_known_differences,
                session_batch_transport: Some(session_batch_transport),
                counts: OracleLaneSummaryCounts {
                    total_seen: summary.total_seen,
                    compared: summary.compared,
                    matches: summary.matches,
                    mismatches: summary.mismatches,
                    rum_failures: summary.rum_failures,
                    oracle_failures: summary.oracle_failures,
                    oracle_processes,
                    skipped_known_differences: summary.skipped_known_differences,
                    confirmed_known_differences: summary.confirmed_known_differences,
                    resolved_known_differences: summary.resolved_known_differences,
                    seed_snapshots_checked: summary.seed_snapshots_checked,
                    seed_snapshots_matched_oracle: summary.seed_snapshots_matched_oracle,
                    seed_snapshots_drifted_from_oracle: summary.seed_snapshots_drifted_from_oracle,
                    oracle_session_batches: None,
                    oracle_session_steps: None,
                },
                issue_samples: lane_a_issue_samples(&summary, max_issue_samples),
            },
        )?;
        println!("summary json: {}", path.display());
    }

    let has_issues =
        summary.mismatches > 0 || summary.rum_failures > 0 || summary.oracle_failures > 0;
    if fail_on_mismatch && has_issues {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn lane_a_issue_samples(
    summary: &lane_a::LaneASummary,
    max_issue_samples: usize,
) -> OracleLaneIssueSamples {
    let mut samples = Vec::new();
    for (index, case) in summary.cases.iter().enumerate() {
        if case.status != LaneACaseStatus::Match {
            samples.push(lane_a_sample("non_match", index + 1, case));
        } else if case.seed_matches_oracle == Some(false) {
            samples.push(lane_a_sample("seed_snapshot_drift", index + 1, case));
        } else if case.known_difference.is_some() {
            samples.push(lane_a_sample("resolved_known_difference", index + 1, case));
        }
    }
    OracleLaneIssueSamples::new(max_issue_samples, samples)
}

fn lane_a_sample(
    kind: &'static str,
    index: usize,
    case: &lane_a::LaneACaseReport,
) -> OracleLaneIssueSample {
    OracleLaneIssueSample {
        kind,
        fixture: case.fixture.display().to_string(),
        index: Some(index),
        input: case.input.clone(),
        status: format!("{:?}", case.status),
        known_difference_id: case
            .known_difference
            .as_ref()
            .map(|known_difference| known_difference.id.clone()),
        known_difference_type: case
            .known_difference
            .as_ref()
            .map(|known_difference| known_difference.kind.clone()),
        seed_drift: case.seed_matches_oracle == Some(false),
    }
}

fn expect_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next()
        .ok_or_else(|| format!("{flag} expects a value").into())
}

fn print_lane_a_usage() {
    println!("rum-difftest lane-a [--oracle-kernel PATH]");
    println!("rum-difftest lane-a [--oracle-command PATH]");
    println!("rum-difftest lane-a [--config PATH]");
    println!("  --fixture PATH            Run a single imported golden file");
    println!("  --limit N                 Stop after N compared cases");
    println!("  --include-known-diffs     Include `!!!` cases in the run");
    println!("  --include-known-differences");
    println!("                              Alias for --include-known-diffs");
    println!("  --oracle-timeout-seconds N");
    println!("                              Kill an oracle invocation after N seconds");
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!("  --max-issue-samples N      Bound summary JSON issue samples (default 5)");
    println!("  --fail-on-mismatch        Exit non-zero if any case mismatches or fails");
}
