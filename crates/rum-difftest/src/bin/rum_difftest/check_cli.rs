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

use super::expect_value;
use super::json_cli::parse_max_issue_samples;

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
    let mut inventory_baseline_path = None;
    let mut lane_d_smoke_baseline = None;
    let mut include_lane_d_readiness = false;
    let mut summary_json_path = None;
    let mut max_issue_samples = summary_json::DEFAULT_MAX_ISSUE_SAMPLES;
    let mut max_issue_samples_specified = false;
    let mut issue_sample_filter = summary_json::CheckIssueSampleFilter::default();
    let mut issue_sample_check_specified = false;
    let mut issue_sample_kind_specified = false;
    let mut fail_on_open_known_difference = false;
    let mut artifact_hygiene_root = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--inventory-baseline" => {
                inventory_baseline_path = Some(PathBuf::from(expect_value(
                    &mut iter,
                    "--inventory-baseline",
                )?));
            }
            "--include-lane-d-smoke" => {
                if lane_d_smoke_baseline.is_none() {
                    lane_d_smoke_baseline = Some(LaneDSmokeBaseline::Default);
                }
            }
            "--lane-d-smoke-baseline" => {
                lane_d_smoke_baseline = Some(LaneDSmokeBaseline::Path(PathBuf::from(
                    expect_value(&mut iter, "--lane-d-smoke-baseline")?,
                )));
            }
            "--include-lane-d-readiness" => {
                include_lane_d_readiness = true;
            }
            "--fail-on-open-known-difference" => {
                fail_on_open_known_difference = true;
            }
            "--artifact-hygiene-root" => {
                artifact_hygiene_root = Some(PathBuf::from(expect_value(
                    &mut iter,
                    "--artifact-hygiene-root",
                )?));
            }
            "--summary-json" => {
                summary_json_path = Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--max-issue-samples" => {
                let value = expect_value(&mut iter, "--max-issue-samples")?;
                max_issue_samples_specified = true;
                max_issue_samples = parse_max_issue_samples(&value)?;
            }
            "--issue-sample-check" => {
                let value = expect_value(&mut iter, "--issue-sample-check")?;
                issue_sample_check_specified = true;
                issue_sample_filter.check = parse_issue_sample_check(&value)?;
            }
            "--issue-sample-kind" => {
                let value = expect_value(&mut iter, "--issue-sample-kind")?;
                issue_sample_kind_specified = true;
                issue_sample_filter.kind = parse_issue_sample_kind(&value)?;
            }
            "--help" | "-h" => {
                usage::print_check_usage();
                return Ok(ExitCode::SUCCESS);
            }
            other => return Err(format!("unknown check flag `{other}`").into()),
        }
    }
    if summary_json_path.is_none() {
        if max_issue_samples_specified {
            return Err("--max-issue-samples requires --summary-json PATH".into());
        }
        if issue_sample_check_specified {
            return Err("--issue-sample-check requires --summary-json PATH".into());
        }
        if issue_sample_kind_specified {
            return Err("--issue-sample-kind requires --summary-json PATH".into());
        }
    }

    let report = run_local_checks(
        inventory_baseline_path,
        lane_d_smoke_baseline,
        include_lane_d_readiness,
        fail_on_open_known_difference,
        artifact_hygiene_root,
    )?;
    text_report::print_check_report(&report);

    if let Some(path) = summary_json_path {
        summary_json::write_summary_json(&path, &report, max_issue_samples, &issue_sample_filter)?;
        println!("summary json: {}", path.display());
    }

    if report.is_clean() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

enum LaneDSmokeBaseline {
    Default,
    Path(PathBuf),
}

fn run_local_checks(
    inventory_baseline_path: Option<PathBuf>,
    lane_d_smoke_baseline: Option<LaneDSmokeBaseline>,
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
    baseline: LaneDSmokeBaseline,
    summary: &LaneDSummary,
) -> Result<LaneDSmokeCheckReport, Box<dyn std::error::Error>> {
    let baseline_path = match baseline {
        LaneDSmokeBaseline::Default => lane_d::default_smoke_baseline_path(),
        LaneDSmokeBaseline::Path(path) => path,
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

fn parse_issue_sample_check(value: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if value == "all" {
        return Ok(None);
    }
    let values = summary_json::issue_sample_check_values();
    if values.contains(&value) {
        return Ok(Some(value.to_string()));
    }
    Err(format!(
        "--issue-sample-check expects one of all, {}; got `{value}`",
        values.join(", ")
    )
    .into())
}

fn parse_issue_sample_kind(value: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if value == "all" {
        return Ok(None);
    }
    let values = summary_json::issue_sample_kind_values();
    if values.contains(&value) {
        return Ok(Some(value.to_string()));
    }
    Err(format!(
        "--issue-sample-kind expects one of all, {}; got `{value}`",
        values.join(", ")
    )
    .into())
}
