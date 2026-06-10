use std::path::PathBuf;

use super::summary_json::{self, CheckIssueSampleFilter};
use crate::expect_value;
use crate::json_cli::parse_max_issue_samples;

pub(super) enum CheckCliArgs {
    Help,
    Run(CheckCliOptions),
}

pub(super) struct CheckCliOptions {
    pub(super) inventory_baseline_path: Option<PathBuf>,
    pub(super) lane_d_smoke_baseline: Option<LaneDSmokeBaseline>,
    pub(super) include_lane_d_readiness: bool,
    pub(super) summary_json_path: Option<PathBuf>,
    pub(super) max_issue_samples: usize,
    pub(super) issue_sample_filter: CheckIssueSampleFilter,
    pub(super) fail_on_open_known_difference: bool,
    pub(super) artifact_hygiene_root: Option<PathBuf>,
}

pub(super) enum LaneDSmokeBaseline {
    Default,
    Path(PathBuf),
}

pub(super) fn parse_check_cli_args(
    args: Vec<String>,
) -> Result<CheckCliArgs, Box<dyn std::error::Error>> {
    let mut options = RawCheckCliOptions::default();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if matches!(arg.as_str(), "--help" | "-h") {
            return Ok(CheckCliArgs::Help);
        }
        parse_check_flag(&mut options, &mut iter, &arg)?;
    }

    options.finish().map(CheckCliArgs::Run)
}

fn parse_check_flag(
    options: &mut RawCheckCliOptions,
    iter: &mut impl Iterator<Item = String>,
    arg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match arg {
        "--inventory-baseline" => {
            options.inventory_baseline_path =
                Some(PathBuf::from(expect_value(iter, "--inventory-baseline")?));
        }
        "--include-lane-d-smoke" => {
            if options.lane_d_smoke_baseline.is_none() {
                options.lane_d_smoke_baseline = Some(LaneDSmokeBaseline::Default);
            }
        }
        "--lane-d-smoke-baseline" => {
            options.lane_d_smoke_baseline = Some(LaneDSmokeBaseline::Path(PathBuf::from(
                expect_value(iter, "--lane-d-smoke-baseline")?,
            )));
        }
        "--include-lane-d-readiness" => {
            options.include_lane_d_readiness = true;
        }
        "--fail-on-open-known-difference" => {
            options.fail_on_open_known_difference = true;
        }
        "--artifact-hygiene-root" => {
            options.artifact_hygiene_root = Some(PathBuf::from(expect_value(
                iter,
                "--artifact-hygiene-root",
            )?));
        }
        "--summary-json" => {
            options.summary_json_path = Some(PathBuf::from(expect_value(iter, "--summary-json")?));
        }
        "--max-issue-samples" => {
            let value = expect_value(iter, "--max-issue-samples")?;
            options.max_issue_samples_specified = true;
            options.max_issue_samples = parse_max_issue_samples(&value)?;
        }
        "--issue-sample-check" => {
            let value = expect_value(iter, "--issue-sample-check")?;
            options.issue_sample_check_specified = true;
            options.issue_sample_filter.check = parse_issue_sample_check(&value)?;
        }
        "--issue-sample-kind" => {
            let value = expect_value(iter, "--issue-sample-kind")?;
            options.issue_sample_kind_specified = true;
            options.issue_sample_filter.kind = parse_issue_sample_kind(&value)?;
        }
        other => return Err(format!("unknown check flag `{other}`").into()),
    }
    Ok(())
}

struct RawCheckCliOptions {
    inventory_baseline_path: Option<PathBuf>,
    lane_d_smoke_baseline: Option<LaneDSmokeBaseline>,
    include_lane_d_readiness: bool,
    summary_json_path: Option<PathBuf>,
    max_issue_samples: usize,
    max_issue_samples_specified: bool,
    issue_sample_filter: CheckIssueSampleFilter,
    issue_sample_check_specified: bool,
    issue_sample_kind_specified: bool,
    fail_on_open_known_difference: bool,
    artifact_hygiene_root: Option<PathBuf>,
}

impl Default for RawCheckCliOptions {
    fn default() -> Self {
        Self {
            inventory_baseline_path: None,
            lane_d_smoke_baseline: None,
            include_lane_d_readiness: false,
            summary_json_path: None,
            max_issue_samples: summary_json::DEFAULT_MAX_ISSUE_SAMPLES,
            max_issue_samples_specified: false,
            issue_sample_filter: CheckIssueSampleFilter::default(),
            issue_sample_check_specified: false,
            issue_sample_kind_specified: false,
            fail_on_open_known_difference: false,
            artifact_hygiene_root: None,
        }
    }
}

impl RawCheckCliOptions {
    fn finish(self) -> Result<CheckCliOptions, Box<dyn std::error::Error>> {
        self.validate()?;
        Ok(CheckCliOptions {
            inventory_baseline_path: self.inventory_baseline_path,
            lane_d_smoke_baseline: self.lane_d_smoke_baseline,
            include_lane_d_readiness: self.include_lane_d_readiness,
            summary_json_path: self.summary_json_path,
            max_issue_samples: self.max_issue_samples,
            issue_sample_filter: self.issue_sample_filter,
            fail_on_open_known_difference: self.fail_on_open_known_difference,
            artifact_hygiene_root: self.artifact_hygiene_root,
        })
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.summary_json_path.is_some() {
            return Ok(());
        }
        if self.max_issue_samples_specified {
            return Err("--max-issue-samples requires --summary-json PATH".into());
        }
        if self.issue_sample_check_specified {
            return Err("--issue-sample-check requires --summary-json PATH".into());
        }
        if self.issue_sample_kind_specified {
            return Err("--issue-sample-kind requires --summary-json PATH".into());
        }
        Ok(())
    }
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

#[cfg(test)]
#[path = "args/tests.rs"]
mod tests;
