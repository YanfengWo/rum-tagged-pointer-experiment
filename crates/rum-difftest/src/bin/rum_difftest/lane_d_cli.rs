use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::lane_d::{
    self, LaneDBaselineCaptureOptions, LaneDBaselineCompareOptions, LaneDCaseStatus, LaneDOptions,
    LaneDProfile, LaneDSuite,
};

use super::expect_value;

pub fn run_lane_d(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut suite = LaneDSuite::All;
    let mut profile = LaneDProfile::Smoke;
    let mut limit = None;
    let mut baseline_path = None;
    let mut write_baseline_path = None;
    let mut capture_runs = 1usize;
    let mut baseline_elapsed_multiplier = 10;
    let mut baseline_min_elapsed_micros = 250_000;
    let mut fail_on_error = false;
    let mut fail_on_regression = false;

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
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                limit = Some(value.parse::<usize>()?);
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
                capture_runs = value.parse::<usize>()?;
            }
            "--baseline-elapsed-multiplier" => {
                let value = expect_value(&mut iter, "--baseline-elapsed-multiplier")?;
                baseline_elapsed_multiplier = value.parse::<u128>()?;
            }
            "--baseline-min-elapsed-micros" => {
                let value = expect_value(&mut iter, "--baseline-min-elapsed-micros")?;
                baseline_min_elapsed_micros = value.parse::<u128>()?;
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
    if capture_runs == 0 {
        return Err("--capture-runs must be at least 1".into());
    }
    if capture_runs > 1 && write_baseline_path.is_none() {
        return Err("--capture-runs requires --write-baseline PATH".into());
    }

    let summary = lane_d::run(&LaneDOptions {
        suite,
        profile,
        limit,
    })?;

    println!("lane-d suite: {}", lane_d::suite_name(suite));
    println!("lane-d profile: {}", lane_d::profile_name(profile));
    println!("cases seen: {}", summary.total_seen);
    println!("cases skipped by profile: {}", summary.skipped_by_profile);
    println!("cases executed: {}", summary.executed);
    println!("successes: {}", summary.succeeded);
    println!("failures: {}", summary.failed);
    println!("total elapsed micros: {}", summary.total_elapsed_micros);

    for case in &summary.cases {
        let status = match &case.status {
            LaneDCaseStatus::Success => "success",
            LaneDCaseStatus::Failure { .. } => "failure",
        };
        println!(
            "case\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            case.index,
            case.suite,
            case.category.as_deref().unwrap_or("-"),
            status,
            case.elapsed_micros,
            case.value_nodes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            case.input
        );
        if let LaneDCaseStatus::Failure { message } = &case.status {
            println!("failure\t{}\t{}", case.index, message.replace('\n', "\\n"));
        }
    }

    let baseline_report = if let Some(path) = &baseline_path {
        let baseline = lane_d::load_baseline(path)?;
        let report = lane_d::compare_baseline(
            &summary,
            &baseline,
            &LaneDBaselineCompareOptions {
                profile,
                require_all_records: limit.is_none(),
            },
        );
        print_baseline_report(path, &report);
        Some(report)
    } else {
        None
    };

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
            elapsed_multiplier: baseline_elapsed_multiplier,
            min_elapsed_micros: baseline_min_elapsed_micros,
        };
        let baseline = if capture_runs == 1 {
            lane_d::capture_baseline(&summaries[0], &capture_options)?
        } else {
            lane_d::capture_baseline_from_runs(&summaries, &capture_options)?
        };
        lane_d::write_baseline(path, &baseline)?;
        println!("baseline written: {}", path.display());
        println!("baseline written records: {}", baseline.records.len());
        println!("baseline capture runs: {capture_runs}");
        println!("baseline elapsed multiplier: {baseline_elapsed_multiplier}");
        println!("baseline min elapsed micros: {baseline_min_elapsed_micros}");
    }

    if fail_on_error && summary.failed > 0 {
        return Ok(ExitCode::FAILURE);
    }
    if fail_on_regression {
        let Some(report) = baseline_report else {
            return Err("--fail-on-regression requires --baseline PATH".into());
        };
        if !report.is_clean() {
            return Ok(ExitCode::FAILURE);
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn print_baseline_report(path: &PathBuf, report: &lane_d::LaneDBaselineReport) {
    println!("baseline: {}", path.display());
    println!("baseline records considered: {}", report.records_considered);
    println!("baseline compared cases: {}", report.compared);
    println!("baseline missing records: {}", report.missing_records.len());
    println!("baseline unused records: {}", report.unused_records.len());
    println!(
        "baseline elapsed regressions: {}",
        report.elapsed_regressions.len()
    );
    println!(
        "baseline value-node regressions: {}",
        report.value_node_regressions.len()
    );
    for missing in &report.missing_records {
        println!(
            "baseline-missing\t{}\t{}\t{}",
            missing.index, missing.suite, missing.input
        );
    }
    for unused in &report.unused_records {
        println!(
            "baseline-unused\t{}\t{}\t{}",
            lane_d::profile_name(unused.profile),
            unused.suite,
            unused.input
        );
    }
    for regression in &report.elapsed_regressions {
        println!(
            "baseline-regression\telapsed\t{}\t{}\t{}\t{}\t{}",
            regression.case.index,
            regression.case.suite,
            regression.case.elapsed_micros,
            regression.max_elapsed_micros,
            regression.case.input
        );
    }
    for regression in &report.value_node_regressions {
        println!(
            "baseline-regression\tvalue-nodes\t{}\t{}\t{}\t{}\t{}",
            regression.case.index,
            regression.case.suite,
            regression.case.value_nodes.unwrap_or(0),
            regression.max_value_nodes,
            regression.case.input
        );
    }
}

fn print_lane_d_usage() {
    println!("rum-difftest lane-d");
    println!("  --suite all|timing|memory  Select imported performance input suite");
    println!("  --profile smoke|full       Select daily-safe smoke inputs or explicit full inputs");
    println!("  --limit N                  Stop after N executed expressions");
    println!("  --baseline PATH            Compare against a Lane D baseline artifact");
    println!("  --write-baseline PATH      Capture the current run as a Lane D baseline artifact");
    println!("  --capture-runs N           Number of runs to aggregate when writing a baseline");
    println!("  --baseline-elapsed-multiplier N");
    println!("                             Multiplier for captured max elapsed guardrails");
    println!("  --baseline-min-elapsed-micros N");
    println!("                             Minimum captured max elapsed guardrail");
    println!("  --fail-on-error            Exit non-zero if any expression fails");
    println!("  --fail-on-regression       Exit non-zero if baseline comparison is not clean");
}
