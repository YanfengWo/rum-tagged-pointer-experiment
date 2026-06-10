use std::path::PathBuf;

use rum_difftest::lane_d::{
    LaneDBaselineCaptureOptions, LaneDProfile, LaneDSuite, LaneDThresholdProfile,
};

pub(crate) enum LaneDCliArgs {
    Help,
    Run(LaneDCliOptions),
}

pub(crate) struct LaneDCliOptions {
    pub(crate) suite: LaneDSuite,
    pub(crate) profile: LaneDProfile,
    pub(crate) threshold_profile: LaneDThresholdProfile,
    pub(crate) limit: Option<usize>,
    pub(crate) baseline_path: Option<PathBuf>,
    pub(crate) write_baseline_path: Option<PathBuf>,
    pub(crate) capture_runs: usize,
    pub(crate) baseline_elapsed_multiplier: u128,
    pub(crate) baseline_min_elapsed_micros: u128,
    pub(crate) fail_on_error: bool,
    pub(crate) fail_on_regression: bool,
    pub(crate) check_baseline_readiness: bool,
    pub(crate) summary_json_path: Option<PathBuf>,
}

pub(crate) struct RawLaneDCliOptions {
    pub(crate) suite: LaneDSuite,
    pub(crate) profile: LaneDProfile,
    pub(crate) threshold_profile: LaneDThresholdProfile,
    pub(crate) limit: Option<usize>,
    pub(crate) baseline_path: Option<PathBuf>,
    pub(crate) write_baseline_path: Option<PathBuf>,
    pub(crate) capture_runs: usize,
    pub(crate) capture_runs_specified: bool,
    pub(crate) baseline_elapsed_multiplier: Option<u128>,
    pub(crate) baseline_elapsed_multiplier_specified: bool,
    pub(crate) baseline_min_elapsed_micros: Option<u128>,
    pub(crate) baseline_min_elapsed_micros_specified: bool,
    pub(crate) fail_on_error: bool,
    pub(crate) fail_on_regression: bool,
    pub(crate) check_baseline_readiness: bool,
    pub(crate) summary_json_path: Option<PathBuf>,
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
    pub(crate) fn finish(self) -> Result<LaneDCliOptions, Box<dyn std::error::Error>> {
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
