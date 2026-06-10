use std::path::PathBuf;
use std::process::ExitCode;

use rum_difftest::artifact_hygiene;
use rum_difftest::divergences;
use rum_difftest::fixtures;
use rum_difftest::inventory::{
    collect_fixture_inventory, compare_inventory_baseline, default_inventory_baseline_path,
    load_inventory_baseline,
};
use rum_difftest::lane_d::{
    self, LaneDBaselineCompareOptions, LaneDOptions, LaneDProfile, LaneDSuite, LaneDSummary,
    LaneDThresholdProfile,
};

#[path = "check_cli/args.rs"]
mod args;
#[path = "check_cli/report.rs"]
mod report;
#[path = "check_cli/summary_json.rs"]
mod summary_json;
#[path = "check_cli/text_report.rs"]
mod text_report;
#[path = "check_cli/usage.rs"]
mod usage;

use report::{
    LaneDReadinessCheckReport, LaneDSmokeCheckReport, LocalCheckReport, SkippedCheck,
    display_check_path,
};

pub fn run_check(args: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let cli = match args::parse_check_cli_args(args)? {
        args::CheckCliArgs::Help => {
            usage::print_check_usage();
            return Ok(ExitCode::SUCCESS);
        }
        args::CheckCliArgs::Run(cli) => cli,
    };

    let report = run_local_checks(
        cli.inventory_baseline_path,
        cli.lane_d_smoke_baseline,
        cli.include_lane_d_readiness,
        cli.fail_on_open_known_difference,
        cli.artifact_hygiene_root,
    )?;
    text_report::print_check_report(&report);

    if let Some(path) = cli.summary_json_path {
        summary_json::write_summary_json(
            &path,
            &report,
            cli.max_issue_samples,
            &cli.issue_sample_filter,
        )?;
        println!("summary json: {}", path.display());
    }

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn run_local_checks(
    inventory_baseline_path: Option<PathBuf>,
    lane_d_smoke_baseline: Option<args::LaneDSmokeBaseline>,
    include_lane_d_readiness: bool,
    fail_on_open_known_difference: bool,
    artifact_hygiene_root: Option<PathBuf>,
) -> Result<LocalCheckReport, Box<dyn std::error::Error>> {
    let inventory_baseline_path =
        inventory_baseline_path.unwrap_or_else(default_inventory_baseline_path);
    let inventory = collect_fixture_inventory()?;
    let baseline = load_inventory_baseline(&inventory_baseline_path)?;
    let inventory_baseline = compare_inventory_baseline(&inventory, &baseline);
    let known_differences = divergences::validate_known_difference_ledger()?;
    let artifact_hygiene_root = artifact_hygiene_root.unwrap_or_else(fixtures::crate_root);
    let artifact_hygiene_issues =
        artifact_hygiene::validate_artifact_hygiene_in(&artifact_hygiene_root)?;
    let lane_d_summary = if lane_d_smoke_baseline.is_some() || include_lane_d_readiness {
        Some(run_lane_d_smoke_summary()?)
    } else {
        None
    };
    let lane_d_smoke = if let Some(baseline) = lane_d_smoke_baseline {
        Some(run_lane_d_smoke_check(
            baseline,
            lane_d_summary
                .as_ref()
                .expect("Lane D summary is captured when smoke is requested"),
        )?)
    } else {
        None
    };
    let lane_d_readiness = if include_lane_d_readiness {
        Some(run_lane_d_readiness_check(lane_d_summary.as_ref().expect(
            "Lane D summary is captured when readiness is requested",
        )))
    } else {
        None
    };

    Ok(LocalCheckReport {
        inventory_baseline_path,
        inventory,
        inventory_baseline,
        known_differences,
        fail_on_open_known_difference,
        artifact_hygiene_root,
        artifact_hygiene_issues,
        lane_d_smoke,
        lane_d_readiness,
    })
}

fn run_lane_d_smoke_check(
    baseline: args::LaneDSmokeBaseline,
    summary: &LaneDSummary,
) -> Result<LaneDSmokeCheckReport, Box<dyn std::error::Error>> {
    let baseline_path = match baseline {
        args::LaneDSmokeBaseline::Default => lane_d::default_smoke_baseline_path(),
        args::LaneDSmokeBaseline::Path(path) => path,
    };
    let baseline = lane_d::load_baseline(&baseline_path)?;
    let baseline = lane_d::compare_baseline(
        summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            require_all_records: true,
        },
    );

    Ok(LaneDSmokeCheckReport {
        baseline_path,
        summary: summary.clone(),
        baseline,
    })
}

fn run_lane_d_readiness_check(summary: &LaneDSummary) -> LaneDReadinessCheckReport {
    LaneDReadinessCheckReport {
        summary: summary.clone(),
        readiness: lane_d::check_checked_baseline_readiness(summary),
    }
}

fn run_lane_d_smoke_summary() -> Result<LaneDSummary, std::io::Error> {
    lane_d::run(&LaneDOptions {
        suite: LaneDSuite::All,
        profile: LaneDProfile::Smoke,
        limit: None,
    })
}
