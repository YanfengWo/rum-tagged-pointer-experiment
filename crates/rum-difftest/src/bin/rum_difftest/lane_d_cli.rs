use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::lane_d::{
    self, LaneDBaselineCaptureOptions, LaneDBaselineCompareOptions, LaneDOptions, LaneDProfile,
    LaneDSuite, LaneDThresholdProfile,
};

use super::expect_value;
use super::json_cli::{
    parse_nonnegative_u128_flag, parse_positive_u128_flag, parse_positive_usize_flag,
};

#[path = "lane_d_cli/summary_json.rs"]
mod summary_json;
#[path = "lane_d_cli/text_report.rs"]
mod text_report;

pub fn run_lane_d(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut suite = LaneDSuite::All;
    let mut profile = LaneDProfile::Smoke;
    let mut threshold_profile = LaneDThresholdProfile::LocalDebug;
    let mut limit = None;
    let mut baseline_path = None;
    let mut write_baseline_path = None;
    let mut capture_runs = 1usize;
    let mut capture_runs_specified = false;
    let mut baseline_elapsed_multiplier = None;
    let mut baseline_elapsed_multiplier_specified = false;
    let mut baseline_min_elapsed_micros = None;
    let mut baseline_min_elapsed_micros_specified = false;
    let mut fail_on_error = false;
    let mut fail_on_regression = false;
    let mut check_baseline_readiness = false;
    let mut summary_json_path = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--suite" => {
                let value = expect_value(&mut iter, "--suite")?;
                suite = lane_d::parse_suite_name(&value)
                    .ok_or_else(|| format!("unknown lane-d suite `{value}`"))?;
            }
            "--profile" => {
                let value = expect_value(&mut iter, "--profile")?;
                profile = lane_d::parse_profile_name(&value)
                    .ok_or_else(|| format!("unknown lane-d profile `{value}`"))?;
            }
            "--threshold-profile" => {
                let value = expect_value(&mut iter, "--threshold-profile")?;
                threshold_profile = lane_d::parse_threshold_profile_name(&value)
                    .ok_or_else(|| format!("unknown lane-d threshold profile `{value}`"))?;
            }
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                limit = Some(parse_positive_usize_flag("--limit", &value)?);
            }
            "--baseline" => {
                baseline_path = Some(PathBuf::from(expect_value(&mut iter, "--baseline")?));
            }
            "--write-baseline" => {
                write_baseline_path =
                    Some(PathBuf::from(expect_value(&mut iter, "--write-baseline")?));
            }
            "--capture-runs" => {
                let value = expect_value(&mut iter, "--capture-runs")?;
                capture_runs_specified = true;
                capture_runs = parse_positive_usize_flag("--capture-runs", &value)?;
            }
            "--baseline-elapsed-multiplier" => {
                let value = expect_value(&mut iter, "--baseline-elapsed-multiplier")?;
                baseline_elapsed_multiplier_specified = true;
                baseline_elapsed_multiplier = Some(parse_positive_u128_flag(
                    "--baseline-elapsed-multiplier",
                    &value,
                )?);
            }
            "--baseline-min-elapsed-micros" => {
                let value = expect_value(&mut iter, "--baseline-min-elapsed-micros")?;
                baseline_min_elapsed_micros_specified = true;
                baseline_min_elapsed_micros = Some(parse_nonnegative_u128_flag(
                    "--baseline-min-elapsed-micros",
                    &value,
                )?);
            }
            "--check-baseline-readiness" => check_baseline_readiness = true,
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--fail-on-error" => fail_on_error = true,
            "--fail-on-regression" => fail_on_regression = true,
            "--help" | "-h" => {
                print_lane_d_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown lane-d flag `{other}`").into()),
        }
    }
    if capture_runs_specified && write_baseline_path.is_none() {
        return Err("--capture-runs requires --write-baseline PATH".into());
    }
    if baseline_elapsed_multiplier_specified && write_baseline_path.is_none() {
        return Err("--baseline-elapsed-multiplier requires --write-baseline PATH".into());
    }
    if baseline_min_elapsed_micros_specified && write_baseline_path.is_none() {
        return Err("--baseline-min-elapsed-micros requires --write-baseline PATH".into());
    }
    if check_baseline_readiness
        && (suite != LaneDSuite::All || profile != LaneDProfile::Smoke || limit.is_some())
    {
        return Err(
            "--check-baseline-readiness expects --suite all --profile smoke without --limit".into(),
        );
    }
    if fail_on_regression && baseline_path.is_none() {
        return Err("--fail-on-regression requires --baseline PATH".into());
    }
    let capture_defaults = LaneDBaselineCaptureOptions::for_threshold_profile(threshold_profile);
    let baseline_elapsed_multiplier =
        baseline_elapsed_multiplier.unwrap_or(capture_defaults.elapsed_multiplier);
    let baseline_min_elapsed_micros =
        baseline_min_elapsed_micros.unwrap_or(capture_defaults.min_elapsed_micros);

    let summary = lane_d::run(&LaneDOptions {
        suite,
        profile,
        limit,
    })?;
    text_report::print_run_summary(suite, profile, threshold_profile, &summary);

    let baseline_report = if let Some(path) = &baseline_path {
        let baseline = lane_d::load_baseline(path)?;
        let report = lane_d::compare_baseline(
            &summary,
            &baseline,
            &LaneDBaselineCompareOptions {
                profile,
                threshold_profile,
                require_all_records: limit.is_none(),
            },
        );
        text_report::print_baseline_report(path, &report);
        Some(report)
    } else {
        None
    };
    let readiness_report = if check_baseline_readiness {
        let report = lane_d::check_checked_baseline_readiness(&summary);
        text_report::print_checked_baseline_readiness_report(&report);
        Some(report)
    } else {
        None
    };

    let mut written_baseline = None;
    if let Some(path) = &write_baseline_path {
        let run_options = LaneDOptions {
            suite,
            profile,
            limit,
        };
        let mut summaries = vec![summary.clone()];
        for _ in 1..capture_runs {
            summaries.push(lane_d::run(&run_options)?);
        }
        let capture_options = LaneDBaselineCaptureOptions {
            profile,
            threshold_profile,
            elapsed_multiplier: baseline_elapsed_multiplier,
            min_elapsed_micros: baseline_min_elapsed_micros,
        };
        let baseline = if capture_runs == 1 {
            lane_d::capture_baseline(&summaries[0], &capture_options)?
        } else {
            lane_d::capture_baseline_from_runs(&summaries, &capture_options)?
        };
        lane_d::write_baseline(path, &baseline)?;
        text_report::print_written_baseline_report(
            path,
            baseline.records.len(),
            capture_runs,
            threshold_profile,
            baseline_elapsed_multiplier,
            baseline_min_elapsed_micros,
        );
        written_baseline = Some(summary_json::WrittenBaselineSummary {
            path: path.clone(),
            records: baseline.records.len(),
            capture_runs,
            threshold_profile,
            elapsed_multiplier: baseline_elapsed_multiplier,
            min_elapsed_micros: baseline_min_elapsed_micros,
        });
    }

    if let Some(path) = &summary_json_path {
        summary_json::write_summary_json(
            path,
            &summary_json::LaneDSummaryJsonContext {
                suite,
                profile,
                threshold_profile,
                summary: &summary,
                baseline_path: baseline_path.as_deref(),
                baseline_report: baseline_report.as_ref(),
                readiness_report: readiness_report.as_ref(),
                written_baseline: written_baseline.as_ref(),
            },
        )?;
        println!("summary json: {}", path.display());
    }

    if fail_on_error && summary.failed > 0 {
        return Ok(ExitCode::FAILURE);
    }
    if fail_on_regression {
        let report = baseline_report
            .as_ref()
            .expect("baseline report is present after fail-on-regression validation");
        if !report.is_clean() {
            return Ok(ExitCode::FAILURE);
        }
    }
    if readiness_report.is_some_and(|report| !report.is_ready()) {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

fn print_lane_d_usage() {
    println!("rum-difftest lane-d");
    println!("  --suite all|timing|memory  Select imported performance input suite");
    println!("  --profile smoke|full       Select daily-safe smoke inputs or explicit full inputs");
    println!("  --threshold-profile local-debug|ci-debug|ci-release");
    println!("                             Select baseline threshold records and capture defaults");
    println!("  --limit N                  Stop after N executed expressions");
    println!("  --baseline PATH            Compare against a Lane D baseline artifact");
    println!("  --write-baseline PATH      Capture the current run as a Lane D baseline artifact");
    println!("  --capture-runs N           Number of runs to aggregate when writing a baseline");
    println!("  --baseline-elapsed-multiplier N");
    println!("                             Multiplier for captured max elapsed guardrails");
    println!("  --baseline-min-elapsed-micros N");
    println!("                             Minimum captured max elapsed guardrail");
    println!("  --check-baseline-readiness");
    println!(
        "                             Check default checked smoke baselines for all threshold profiles"
    );
    println!("  --summary-json PATH        Write a stable machine-readable summary");
    println!("  --fail-on-error            Exit non-zero if any expression fails");
    println!("  --fail-on-regression       Exit non-zero if baseline comparison is not clean");
}
