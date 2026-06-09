use std::process::ExitCode;

use rum_difftest::lane_d::{
    self, LaneDBaselineCaptureOptions, LaneDBaselineCompareOptions, LaneDOptions,
};

#[path = "lane_d_cli/args.rs"]
mod args;
#[path = "lane_d_cli/summary_json.rs"]
mod summary_json;
#[path = "lane_d_cli/text_report.rs"]
mod text_report;

pub fn run_lane_d(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let cli = match args::parse_lane_d_cli_args(args)? {
        args::LaneDCliArgs::Help => {
            args::print_lane_d_usage();
            return Ok(ExitCode::SUCCESS);
        }
        args::LaneDCliArgs::Run(cli) => cli,
    };
    let suite = cli.suite;
    let profile = cli.profile;
    let threshold_profile = cli.threshold_profile;
    let limit = cli.limit;
    let baseline_path = cli.baseline_path;
    let write_baseline_path = cli.write_baseline_path;
    let capture_runs = cli.capture_runs;
    let baseline_elapsed_multiplier = cli.baseline_elapsed_multiplier;
    let baseline_min_elapsed_micros = cli.baseline_min_elapsed_micros;
    let fail_on_error = cli.fail_on_error;
    let fail_on_regression = cli.fail_on_regression;
    let check_baseline_readiness = cli.check_baseline_readiness;
    let summary_json_path = cli.summary_json_path;

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
