use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::config::{load_optional_config, resolve_oracle_kernel, resolve_oracle_timeout};
use rum_difftest::lane_b::{self, LaneBOptions, LaneBStepStatus};
use rum_difftest::oracle::OracleKernel;

use super::oracle_cli::print_oracle_header;
use super::oracle_lane_summary_json::{
    DEFAULT_MAX_ISSUE_SAMPLES, OracleLaneIssueSample, OracleLaneIssueSampleKindFilter,
    OracleLaneIssueSamples, OracleLaneSessionBatchTransport, OracleLaneSummaryCounts,
    OracleLaneSummaryJsonContext, parse_issue_sample_kind, parse_issue_sample_source,
    parse_max_issue_samples, write_summary_json,
};

#[path = "lane_b_cli/text_report.rs"]
mod text_report;

pub fn run_lane_b(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut config_path = None;
    let mut oracle_kernel = None;
    let mut fixture = None;
    let mut limit = None;
    let mut include_known_differences = false;
    let mut fail_on_mismatch = false;
    let mut fail_on_project_mismatch = false;
    let mut fail_on_seed_drift = false;
    let mut fail_on_project_seed_drift = false;
    let mut fail_on_resolved_known_difference = false;
    let mut oracle_timeout_seconds = None;
    let mut summary_json_path = None;
    let mut max_issue_samples = DEFAULT_MAX_ISSUE_SAMPLES;
    let mut issue_sample_kind = OracleLaneIssueSampleKindFilter::All;
    let mut issue_sample_source = None;
    let mut source = None;
    let mut category = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => config_path = Some(PathBuf::from(expect_value(&mut iter, "--config")?)),
            "--oracle-kernel" | "--oracle-command" => {
                oracle_kernel = Some(PathBuf::from(expect_value(&mut iter, &arg)?))
            }
            "--fixture" => fixture = Some(PathBuf::from(expect_value(&mut iter, "--fixture")?)),
            "--source" => source = Some(expect_value(&mut iter, "--source")?),
            "--category" => category = Some(expect_value(&mut iter, "--category")?),
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
            "--issue-sample-kind" => {
                let value = expect_value(&mut iter, "--issue-sample-kind")?;
                issue_sample_kind = parse_issue_sample_kind(&value)?;
            }
            "--issue-sample-source" => {
                let value = expect_value(&mut iter, "--issue-sample-source")?;
                issue_sample_source = parse_issue_sample_source(&value)?;
            }
            "--fail-on-mismatch" => fail_on_mismatch = true,
            "--fail-on-project-mismatch" => fail_on_project_mismatch = true,
            "--fail-on-seed-drift" => fail_on_seed_drift = true,
            "--fail-on-project-seed-drift" => fail_on_project_seed_drift = true,
            "--fail-on-resolved-known-difference" => {
                fail_on_resolved_known_difference = true;
            }
            "--help" | "-h" => {
                print_lane_b_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown lane-b flag `{other}`").into()),
        }
    }

    let config = load_optional_config(config_path.as_deref())?;
    let oracle_kernel = resolve_oracle_kernel(oracle_kernel, &config).ok_or(
        "no oracle kernel configured; pass --oracle-kernel/--oracle-command PATH or set oracle_kernel/oracle_command in a config file",
    )?;
    let oracle_timeout = resolve_oracle_timeout(oracle_timeout_seconds, &config)?;

    let mut oracle = OracleKernel::with_timeout(&oracle_kernel, oracle_timeout)?;
    let options = LaneBOptions {
        fixture,
        limit,
        source,
        category,
        include_known_differences,
    };
    let summary = lane_b::run(&mut oracle, &options)?;

    print_oracle_header("lane-b", &oracle);
    let oracle_processes = oracle.session_batch_process_count(summary.oracle_session_batches);
    let oracle_batches = summary.oracle_session_batches;
    let oracle_batch_inputs = summary.oracle_session_steps;
    let project_non_matches = lane_b_project_non_matches(&summary);
    let project_seed_snapshot_drifts = lane_b_project_seed_snapshot_drifts(&summary);
    let session_batch_transport = OracleLaneSessionBatchTransport {
        name: oracle.session_batch_transport_name(),
        process_count: Some(oracle_processes),
    };
    text_report::print_lane_b_report(
        &summary,
        project_non_matches,
        project_seed_snapshot_drifts,
        session_batch_transport,
        oracle_batches,
        oracle_batch_inputs,
    );

    if let Some(path) = &summary_json_path {
        write_summary_json(
            path,
            &OracleLaneSummaryJsonContext {
                lane: "lane-b",
                oracle: &oracle,
                fixture: options.fixture.as_deref(),
                source: options.source.as_deref(),
                category: options.category.as_deref(),
                limit: options.limit,
                include_known_differences: options.include_known_differences,
                fail_on_mismatch,
                fail_on_project_mismatch,
                fail_on_seed_drift,
                fail_on_project_seed_drift,
                fail_on_resolved_known_difference,
                issue_sample_kind,
                issue_sample_source,
                session_batch_transport: Some(session_batch_transport),
                counts: OracleLaneSummaryCounts {
                    total_seen: summary.total_seen,
                    compared: summary.compared,
                    matches: summary.matches,
                    mismatches: summary.mismatches,
                    rum_failures: summary.rum_failures,
                    oracle_failures: summary.oracle_failures,
                    oracle_processes,
                    oracle_batches,
                    oracle_batch_inputs,
                    skipped_known_differences: summary.skipped_known_differences,
                    confirmed_known_differences: summary.confirmed_known_differences,
                    resolved_known_differences: summary.resolved_known_differences,
                    project_non_matches,
                    seed_snapshots_checked: summary.seed_snapshots_checked,
                    seed_snapshots_matched_oracle: summary.seed_snapshots_matched_oracle,
                    seed_snapshots_drifted_from_oracle: summary.seed_snapshots_drifted_from_oracle,
                    project_seed_snapshots_drifted_from_oracle: project_seed_snapshot_drifts,
                    oracle_session_batches: Some(summary.oracle_session_batches),
                    oracle_session_steps: Some(summary.oracle_session_steps),
                },
                issue_samples: lane_b_issue_samples(
                    &summary,
                    max_issue_samples,
                    issue_sample_kind,
                    issue_sample_source,
                ),
            },
        )?;
        println!("summary json: {}", path.display());
    }

    let has_issues =
        summary.mismatches > 0 || summary.rum_failures > 0 || summary.oracle_failures > 0;
    if fail_on_mismatch && has_issues {
        return Ok(ExitCode::FAILURE);
    }
    if fail_on_project_mismatch && project_non_matches > 0 {
        return Ok(ExitCode::FAILURE);
    }
    if fail_on_seed_drift && summary.seed_snapshots_drifted_from_oracle > 0 {
        return Ok(ExitCode::FAILURE);
    }
    if fail_on_project_seed_drift && project_seed_snapshot_drifts > 0 {
        return Ok(ExitCode::FAILURE);
    }
    if fail_on_resolved_known_difference && summary.resolved_known_differences > 0 {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn lane_b_project_non_matches(summary: &lane_b::LaneBSummary) -> usize {
    summary
        .steps
        .iter()
        .filter(|step| {
            step.status != LaneBStepStatus::Match
                && !matches!(lane_b::fixture_source(&step.fixture), "seed-golden")
        })
        .count()
}

fn lane_b_project_seed_snapshot_drifts(summary: &lane_b::LaneBSummary) -> usize {
    summary
        .steps
        .iter()
        .filter(|step| {
            step.seed_matches_oracle == Some(false)
                && !matches!(lane_b::fixture_source(&step.fixture), "seed-golden")
        })
        .count()
}

fn lane_b_issue_samples(
    summary: &lane_b::LaneBSummary,
    max_issue_samples: usize,
    issue_sample_kind: OracleLaneIssueSampleKindFilter,
    issue_sample_source: Option<&str>,
) -> OracleLaneIssueSamples {
    let mut samples = Vec::new();
    for step in &summary.steps {
        if step.status != LaneBStepStatus::Match {
            samples.push(lane_b_sample("non_match", step));
        } else if step.seed_matches_oracle == Some(false) {
            samples.push(lane_b_sample("seed_snapshot_drift", step));
        } else if step.known_difference.is_some() {
            samples.push(lane_b_sample("resolved_known_difference", step));
        }
    }
    OracleLaneIssueSamples::new(
        max_issue_samples,
        issue_sample_kind,
        issue_sample_source,
        samples,
    )
}

fn lane_b_sample(kind: &'static str, step: &lane_b::LaneBStepReport) -> OracleLaneIssueSample {
    let fixture_source = lane_b::fixture_source(&step.fixture);
    OracleLaneIssueSample {
        kind,
        fixture: step.fixture.display().to_string(),
        fixture_source,
        project_owned_fixture: fixture_source != "seed-golden",
        index: Some(step.step_index),
        input: step.input.clone(),
        status: format!("{:?}", step.status),
        known_difference_id: step
            .known_difference
            .as_ref()
            .map(|known_difference| known_difference.id.clone()),
        known_difference_type: step
            .known_difference
            .as_ref()
            .map(|known_difference| known_difference.kind.clone()),
        seed_drift: step.seed_matches_oracle == Some(false),
    }
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
    println!("  --source NAME             Filter default fixtures by source");
    println!("  --category NAME           Filter default fixtures by semantic category");
    println!("  --limit N                 Stop after N compared steps");
    println!("  --include-known-diffs     Include `!!!` steps in the run");
    println!("  --include-known-differences");
    println!("                              Alias for --include-known-diffs");
    println!("  --oracle-timeout-seconds N");
    println!("                              Kill an oracle invocation after N seconds");
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!("  --max-issue-samples N      Bound summary JSON issue samples (default 5)");
    println!(
        "  --issue-sample-kind KIND   Filter samples: all, non-match, seed-drift, resolved-known-difference"
    );
    println!("  --issue-sample-source SRC  Filter samples by fixture source");
    println!("  --fail-on-mismatch        Exit non-zero if any step mismatches or fails");
    println!("  --fail-on-project-mismatch");
    println!(
        "                              Exit non-zero if any non-imported step mismatches or fails"
    );
    println!("  --fail-on-seed-drift      Exit non-zero if any seed snapshot drifts");
    println!("  --fail-on-project-seed-drift");
    println!(
        "                              Exit non-zero if any non-imported seed snapshot drifts"
    );
    println!("  --fail-on-resolved-known-difference");
    println!(
        "                              Exit non-zero if any included known difference now matches"
    );
}
