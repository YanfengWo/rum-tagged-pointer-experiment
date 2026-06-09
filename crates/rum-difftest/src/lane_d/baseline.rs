use std::path::PathBuf;

use thiserror::Error;

#[cfg(test)]
use super::LaneDSummary;
use super::{LaneDCaseReport, LaneDProfile, LaneDThresholdProfile};

mod capture;
mod compare;
mod io;
mod readiness;

pub use capture::{capture_baseline, capture_baseline_from_runs};
pub use compare::compare_baseline;
pub use io::{format_baseline, load_baseline, parse_baseline, write_baseline};
pub use readiness::{
    check_checked_baseline_readiness, check_checked_baseline_readiness_for,
    checked_baseline_capture_command, default_checked_baseline_artifacts,
    default_smoke_baseline_path,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDBaseline {
    pub records: Vec<LaneDBaselineRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDBaselineRecord {
    pub profile: LaneDProfile,
    pub threshold_profile: LaneDThresholdProfile,
    pub suite: String,
    pub category: Option<String>,
    pub input: String,
    pub baseline_elapsed_micros: u128,
    pub max_elapsed_micros: u128,
    pub max_value_nodes: usize,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDBaselineCaptureOptions {
    pub profile: LaneDProfile,
    pub threshold_profile: LaneDThresholdProfile,
    pub elapsed_multiplier: u128,
    pub min_elapsed_micros: u128,
}

impl LaneDBaselineCaptureOptions {
    pub fn for_threshold_profile(threshold_profile: LaneDThresholdProfile) -> Self {
        let (elapsed_multiplier, min_elapsed_micros) = match threshold_profile {
            LaneDThresholdProfile::LocalDebug => (10, 250_000),
            LaneDThresholdProfile::CiDebug => (25, 1_000_000),
            LaneDThresholdProfile::CiRelease => (15, 500_000),
        };

        Self {
            profile: LaneDProfile::Smoke,
            threshold_profile,
            elapsed_multiplier,
            min_elapsed_micros,
        }
    }
}

impl Default for LaneDBaselineCaptureOptions {
    fn default() -> Self {
        Self::for_threshold_profile(LaneDThresholdProfile::LocalDebug)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDBaselineCompareOptions {
    pub profile: LaneDProfile,
    pub threshold_profile: LaneDThresholdProfile,
    pub require_all_records: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDBaselineReport {
    pub records_considered: usize,
    pub compared: usize,
    pub missing_records: Vec<LaneDCaseReport>,
    pub unused_records: Vec<LaneDBaselineRecord>,
    pub elapsed_regressions: Vec<LaneDElapsedRegression>,
    pub value_node_regressions: Vec<LaneDValueNodeRegression>,
}

impl LaneDBaselineReport {
    pub fn is_clean(&self) -> bool {
        self.missing_records.is_empty()
            && self.unused_records.is_empty()
            && self.elapsed_regressions.is_empty()
            && self.value_node_regressions.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDCheckedBaselineArtifact {
    pub profile: LaneDProfile,
    pub threshold_profile: LaneDThresholdProfile,
    pub path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDCheckedBaselineReadinessReport {
    pub entries: Vec<LaneDCheckedBaselineReadinessEntry>,
}

impl LaneDCheckedBaselineReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.entries.iter().all(|entry| {
            matches!(
                entry.status,
                LaneDCheckedBaselineReadinessStatus::Ready { .. }
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDCheckedBaselineReadinessEntry {
    pub profile: LaneDProfile,
    pub threshold_profile: LaneDThresholdProfile,
    pub path: PathBuf,
    pub status: LaneDCheckedBaselineReadinessStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneDCheckedBaselineReadinessStatus {
    Ready {
        records_considered: usize,
        compared: usize,
    },
    MissingArtifact,
    InvalidArtifact {
        message: String,
    },
    NotClean {
        report: LaneDBaselineReport,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDElapsedRegression {
    pub case: LaneDCaseReport,
    pub max_elapsed_micros: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDValueNodeRegression {
    pub case: LaneDCaseReport,
    pub max_value_nodes: usize,
}

#[derive(Debug, Error)]
pub enum LaneDBaselineError {
    #[error("failed to read `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid Lane D baseline line `{line}`: {message}")]
    InvalidLine { line: String, message: String },
    #[error("Lane D baseline line is missing `{field}`: {line}")]
    MissingField { line: String, field: String },
    #[error("invalid `{field}` value `{value}` in Lane D baseline line: {line}")]
    InvalidInteger {
        line: String,
        field: String,
        value: String,
    },
    #[error("unknown Lane D baseline profile `{profile}` in line: {line}")]
    UnknownProfile { line: String, profile: String },
    #[error("unknown Lane D baseline threshold profile `{profile}` in line: {line}")]
    UnknownThresholdProfile { line: String, profile: String },
    #[error(
        "duplicate Lane D baseline record for profile `{profile}`, threshold profile `{threshold_profile}`, suite `{suite}`, input `{input}`"
    )]
    DuplicateRecord {
        profile: String,
        threshold_profile: String,
        suite: String,
        input: String,
    },
    #[error("cannot capture Lane D baseline while {count} case(s) failed")]
    CaptureHasFailures { count: usize },
    #[error("cannot capture Lane D baseline without at least one run")]
    CaptureHasNoRuns,
    #[error(
        "Lane D capture run {run_index} case {case_index} does not match the first run: expected `{expected}`, got `{actual}`"
    )]
    CaptureRunShapeMismatch {
        run_index: usize,
        case_index: usize,
        expected: String,
        actual: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LaneDBaselineKey {
    profile: LaneDProfile,
    threshold_profile: LaneDThresholdProfile,
    suite: String,
    input: String,
}

#[cfg(test)]
mod tests;
