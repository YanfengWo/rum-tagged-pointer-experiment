use std::path::PathBuf;

use rum_difftest::lane_d::{
    self, LaneDBaselineCaptureOptions, LaneDProfile, LaneDSuite, LaneDThresholdProfile,
};

use crate::expect_value;
use crate::json_cli::{
    parse_nonnegative_u128_flag, parse_positive_u128_flag, parse_positive_usize_flag,
};

pub(super) enum LaneDCliArgs {
    Help,
    Run(LaneDCliOptions),
}

pub(super) struct LaneDCliOptions {
    pub(super) suite: LaneDSuite,
    pub(super) profile: LaneDProfile,
    pub(super) threshold_profile: LaneDThresholdProfile,
    pub(super) limit: Option<usize>,
    pub(super) baseline_path: Option<PathBuf>,
    pub(super) write_baseline_path: Option<PathBuf>,
    pub(super) capture_runs: usize,
    pub(super) baseline_elapsed_multiplier: u128,
    pub(super) baseline_min_elapsed_micros: u128,
    pub(super) fail_on_error: bool,
    pub(super) fail_on_regression: bool,
    pub(super) check_baseline_readiness: bool,
    pub(super) summary_json_path: Option<PathBuf>,
}

pub(super) fn parse_lane_d_cli_args(
    args: Vec<String>,
) -> Result<LaneDCliArgs, Box<dyn std::error::Error>> {
    let mut options = RawLaneDCliOptions::default();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--suite" => {
                let value = expect_value(&mut iter, "--suite")?;
                options.suite = lane_d::parse_suite_name(&value)
                    .ok_or_else(|| format!("unknown lane-d suite `{value}`"))?;
            }
            "--profile" => {
                let value = expect_value(&mut iter, "--profile")?;
                options.profile = lane_d::parse_profile_name(&value)
                    .ok_or_else(|| format!("unknown lane-d profile `{value}`"))?;
            }
            "--threshold-profile" => {
                let value = expect_value(&mut iter, "--threshold-profile")?;
                options.threshold_profile = lane_d::parse_threshold_profile_name(&value)
                    .ok_or_else(|| format!("unknown lane-d threshold profile `{value}`"))?;
            }
            "--limit" => {
                let value = expect_value(&mut iter, "--limit")?;
                options.limit = Some(parse_positive_usize_flag("--limit", &value)?);
            }
            "--baseline" => {
                options.baseline_path = Some(PathBuf::from(expect_value(&mut iter, "--baseline")?));
            }
            "--write-baseline" => {
                options.write_baseline_path =
                    Some(PathBuf::from(expect_value(&mut iter, "--write-baseline")?));
            }
            "--capture-runs" => {
                let value = expect_value(&mut iter, "--capture-runs")?;
                options.capture_runs_specified = true;
                options.capture_runs = parse_positive_usize_flag("--capture-runs", &value)?;
            }
            "--baseline-elapsed-multiplier" => {
                let value = expect_value(&mut iter, "--baseline-elapsed-multiplier")?;
                options.baseline_elapsed_multiplier_specified = true;
                options.baseline_elapsed_multiplier = Some(parse_positive_u128_flag(
                    "--baseline-elapsed-multiplier",
                    &value,
                )?);
            }
            "--baseline-min-elapsed-micros" => {
                let value = expect_value(&mut iter, "--baseline-min-elapsed-micros")?;
                options.baseline_min_elapsed_micros_specified = true;
                options.baseline_min_elapsed_micros = Some(parse_nonnegative_u128_flag(
                    "--baseline-min-elapsed-micros",
                    &value,
                )?);
            }
            "--check-baseline-readiness" => options.check_baseline_readiness = true,
            "--summary-json" => {
                options.summary_json_path =
                    Some(PathBuf::from(expect_value(&mut iter, "--summary-json")?));
            }
            "--fail-on-error" => options.fail_on_error = true,
            "--fail-on-regression" => options.fail_on_regression = true,
            "--help" | "-h" => return Ok(LaneDCliArgs::Help),
            other => return Err(format!("unknown lane-d flag `{other}`").into()),
        }
    }

    options.finish().map(LaneDCliArgs::Run)
}

pub(super) fn print_lane_d_usage() {
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

struct RawLaneDCliOptions {
    suite: LaneDSuite,
    profile: LaneDProfile,
    threshold_profile: LaneDThresholdProfile,
    limit: Option<usize>,
    baseline_path: Option<PathBuf>,
    write_baseline_path: Option<PathBuf>,
    capture_runs: usize,
    capture_runs_specified: bool,
    baseline_elapsed_multiplier: Option<u128>,
    baseline_elapsed_multiplier_specified: bool,
    baseline_min_elapsed_micros: Option<u128>,
    baseline_min_elapsed_micros_specified: bool,
    fail_on_error: bool,
    fail_on_regression: bool,
    check_baseline_readiness: bool,
    summary_json_path: Option<PathBuf>,
}

impl Default for RawLaneDCliOptions {
    fn default() -> Self {
        Self {
            suite: LaneDSuite::All,
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            limit: None,
            baseline_path: None,
            write_baseline_path: None,
            capture_runs: 1,
            capture_runs_specified: false,
            baseline_elapsed_multiplier: None,
            baseline_elapsed_multiplier_specified: false,
            baseline_min_elapsed_micros: None,
            baseline_min_elapsed_micros_specified: false,
            fail_on_error: false,
            fail_on_regression: false,
            check_baseline_readiness: false,
            summary_json_path: None,
        }
    }
}

impl RawLaneDCliOptions {
    fn finish(self) -> Result<LaneDCliOptions, Box<dyn std::error::Error>> {
        self.validate()?;
        let capture_defaults =
            LaneDBaselineCaptureOptions::for_threshold_profile(self.threshold_profile);
        Ok(LaneDCliOptions {
            suite: self.suite,
            profile: self.profile,
            threshold_profile: self.threshold_profile,
            limit: self.limit,
            baseline_path: self.baseline_path,
            write_baseline_path: self.write_baseline_path,
            capture_runs: self.capture_runs,
            baseline_elapsed_multiplier: self
                .baseline_elapsed_multiplier
                .unwrap_or(capture_defaults.elapsed_multiplier),
            baseline_min_elapsed_micros: self
                .baseline_min_elapsed_micros
                .unwrap_or(capture_defaults.min_elapsed_micros),
            fail_on_error: self.fail_on_error,
            fail_on_regression: self.fail_on_regression,
            check_baseline_readiness: self.check_baseline_readiness,
            summary_json_path: self.summary_json_path,
        })
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.capture_runs_specified && self.write_baseline_path.is_none() {
            return Err("--capture-runs requires --write-baseline PATH".into());
        }
        if self.baseline_elapsed_multiplier_specified && self.write_baseline_path.is_none() {
            return Err("--baseline-elapsed-multiplier requires --write-baseline PATH".into());
        }
        if self.baseline_min_elapsed_micros_specified && self.write_baseline_path.is_none() {
            return Err("--baseline-min-elapsed-micros requires --write-baseline PATH".into());
        }
        if self.check_baseline_readiness
            && (self.suite != LaneDSuite::All
                || self.profile != LaneDProfile::Smoke
                || self.limit.is_some())
        {
            return Err(
                "--check-baseline-readiness expects --suite all --profile smoke without --limit"
                    .into(),
            );
        }
        if self.fail_on_regression && self.baseline_path.is_none() {
            return Err("--fail-on-regression requires --baseline PATH".into());
        }
        Ok(())
    }
}
