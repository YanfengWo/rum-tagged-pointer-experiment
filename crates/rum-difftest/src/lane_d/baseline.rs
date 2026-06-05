use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::fixtures::{crate_root, load_text, parse_metadata_fields};

use super::{
    LaneDCaseReport, LaneDProfile, LaneDSummary, LaneDThresholdProfile, parse_profile_name,
    parse_threshold_profile_name, profile_name, threshold_profile_name,
};

const REQUIRED_BASELINE_FIELDS: [&str; 6] = [
    "profile",
    "suite",
    "input",
    "baseline_elapsed_micros",
    "max_elapsed_micros",
    "max_value_nodes",
];

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

pub fn default_smoke_baseline_path() -> PathBuf {
    crate_root().join("perf/baselines/smoke.txt")
}

pub fn default_checked_baseline_artifacts() -> Vec<LaneDCheckedBaselineArtifact> {
    let baseline_dir = crate_root().join("perf/baselines");
    vec![
        LaneDCheckedBaselineArtifact {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            path: default_smoke_baseline_path(),
        },
        LaneDCheckedBaselineArtifact {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::CiDebug,
            path: baseline_dir.join("smoke-ci-debug.txt"),
        },
        LaneDCheckedBaselineArtifact {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::CiRelease,
            path: baseline_dir.join("smoke-ci-release.txt"),
        },
    ]
}

pub fn load_baseline(path: impl AsRef<Path>) -> Result<LaneDBaseline, LaneDBaselineError> {
    let path = path.as_ref();
    let text = load_text(path).map_err(|source| LaneDBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_baseline(&text)
}

pub fn write_baseline(
    path: impl AsRef<Path>,
    baseline: &LaneDBaseline,
) -> Result<(), LaneDBaselineError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LaneDBaselineError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, format_baseline(baseline)).map_err(|source| LaneDBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn check_checked_baseline_readiness(
    summary: &LaneDSummary,
) -> LaneDCheckedBaselineReadinessReport {
    check_checked_baseline_readiness_for(summary, &default_checked_baseline_artifacts())
}

pub fn check_checked_baseline_readiness_for(
    summary: &LaneDSummary,
    artifacts: &[LaneDCheckedBaselineArtifact],
) -> LaneDCheckedBaselineReadinessReport {
    let entries = artifacts
        .iter()
        .map(|artifact| LaneDCheckedBaselineReadinessEntry {
            profile: artifact.profile,
            threshold_profile: artifact.threshold_profile,
            path: artifact.path.clone(),
            status: check_checked_baseline_artifact(summary, artifact),
        })
        .collect();

    LaneDCheckedBaselineReadinessReport { entries }
}

pub fn capture_baseline(
    summary: &LaneDSummary,
    options: &LaneDBaselineCaptureOptions,
) -> Result<LaneDBaseline, LaneDBaselineError> {
    capture_baseline_from_runs(std::slice::from_ref(summary), options)
}

pub fn capture_baseline_from_runs(
    summaries: &[LaneDSummary],
    options: &LaneDBaselineCaptureOptions,
) -> Result<LaneDBaseline, LaneDBaselineError> {
    let Some(first) = summaries.first() else {
        return Err(LaneDBaselineError::CaptureHasNoRuns);
    };

    let failed = summaries
        .iter()
        .map(|summary| summary.failed)
        .sum::<usize>();
    if failed > 0 {
        return Err(LaneDBaselineError::CaptureHasFailures { count: failed });
    }

    validate_capture_run_shapes(summaries)?;

    let records = first
        .cases
        .iter()
        .enumerate()
        .map(|(case_index, case)| {
            let mut elapsed_samples = summaries
                .iter()
                .map(|summary| summary.cases[case_index].elapsed_micros)
                .collect::<Vec<_>>();
            let baseline_elapsed_micros = median_elapsed_micros(&mut elapsed_samples);
            let max_observed_elapsed_micros = elapsed_samples.iter().copied().max().unwrap_or(0);
            LaneDBaselineRecord {
                profile: options.profile,
                threshold_profile: options.threshold_profile,
                suite: case.suite.clone(),
                category: case.category.clone(),
                input: case.input.clone(),
                baseline_elapsed_micros,
                max_elapsed_micros: checked_elapsed_limit(max_observed_elapsed_micros, options),
                max_value_nodes: max_value_nodes_for_case(summaries, case_index),
                note: (summaries.len() > 1).then(|| format!("capture_runs={}", summaries.len())),
            }
        })
        .collect();

    Ok(LaneDBaseline { records })
}

fn check_checked_baseline_artifact(
    summary: &LaneDSummary,
    artifact: &LaneDCheckedBaselineArtifact,
) -> LaneDCheckedBaselineReadinessStatus {
    if !artifact.path.exists() {
        return LaneDCheckedBaselineReadinessStatus::MissingArtifact;
    }

    let baseline = match load_baseline(&artifact.path) {
        Ok(baseline) => baseline,
        Err(err) => {
            return LaneDCheckedBaselineReadinessStatus::InvalidArtifact {
                message: err.to_string(),
            };
        }
    };
    let report = compare_baseline(
        summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: artifact.profile,
            threshold_profile: artifact.threshold_profile,
            require_all_records: true,
        },
    );

    if report.is_clean() {
        LaneDCheckedBaselineReadinessStatus::Ready {
            records_considered: report.records_considered,
            compared: report.compared,
        }
    } else {
        LaneDCheckedBaselineReadinessStatus::NotClean { report }
    }
}

fn validate_capture_run_shapes(summaries: &[LaneDSummary]) -> Result<(), LaneDBaselineError> {
    let first = &summaries[0];
    for (run_index, summary) in summaries.iter().enumerate().skip(1) {
        if summary.cases.len() != first.cases.len() {
            return Err(LaneDBaselineError::CaptureRunShapeMismatch {
                run_index: run_index + 1,
                case_index: summary.cases.len().min(first.cases.len()) + 1,
                expected: format!("{} cases", first.cases.len()),
                actual: format!("{} cases", summary.cases.len()),
            });
        }
        for (case_index, (expected, actual)) in first.cases.iter().zip(&summary.cases).enumerate() {
            if expected.suite != actual.suite || expected.input != actual.input {
                return Err(LaneDBaselineError::CaptureRunShapeMismatch {
                    run_index: run_index + 1,
                    case_index: case_index + 1,
                    expected: format!("{}\t{}", expected.suite, expected.input),
                    actual: format!("{}\t{}", actual.suite, actual.input),
                });
            }
        }
    }
    Ok(())
}

fn median_elapsed_micros(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn max_value_nodes_for_case(summaries: &[LaneDSummary], case_index: usize) -> usize {
    summaries
        .iter()
        .filter_map(|summary| summary.cases[case_index].value_nodes)
        .max()
        .unwrap_or(0)
}

pub fn format_baseline(baseline: &LaneDBaseline) -> String {
    let mut output = String::from(
        "# Lane D captured performance baseline.\n\
         # Format:\n\
         # profile=... | threshold_profile=... | suite=... | category=... | input=... | baseline_elapsed_micros=... | max_elapsed_micros=... | max_value_nodes=...\n\
         # `baseline_elapsed_micros` is measurement data from the capture run. `max_elapsed_micros` and `max_value_nodes` are checked guardrails.\n\n",
    );

    for record in &baseline.records {
        output.push_str(&format_baseline_record(record));
        output.push('\n');
    }

    output
}

fn checked_elapsed_limit(elapsed_micros: u128, options: &LaneDBaselineCaptureOptions) -> u128 {
    elapsed_micros
        .saturating_mul(options.elapsed_multiplier)
        .max(options.min_elapsed_micros)
}

fn format_baseline_record(record: &LaneDBaselineRecord) -> String {
    let mut fields = vec![
        format!("profile={}", profile_name(record.profile)),
        format!(
            "threshold_profile={}",
            threshold_profile_name(record.threshold_profile)
        ),
        format!("suite={}", record.suite),
    ];
    if let Some(category) = &record.category {
        fields.push(format!("category={category}"));
    }
    fields.extend([
        format!("input={}", record.input),
        format!("baseline_elapsed_micros={}", record.baseline_elapsed_micros),
        format!("max_elapsed_micros={}", record.max_elapsed_micros),
        format!("max_value_nodes={}", record.max_value_nodes),
    ]);
    if let Some(note) = &record.note {
        fields.push(format!("note={note}"));
    }

    fields.join(" | ")
}

pub fn parse_baseline(text: &str) -> Result<LaneDBaseline, LaneDBaselineError> {
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields = parse_metadata_fields(line, line).map_err(|source| {
            LaneDBaselineError::InvalidLine {
                line: line.to_owned(),
                message: source.to_string(),
            }
        })?;
        for required in REQUIRED_BASELINE_FIELDS {
            if !fields.contains_key(required) {
                return Err(LaneDBaselineError::MissingField {
                    line: line.to_owned(),
                    field: required.to_owned(),
                });
            }
        }

        let profile_text = required_field(&fields, "profile").to_owned();
        let profile = parse_profile_name(&profile_text).ok_or_else(|| {
            LaneDBaselineError::UnknownProfile {
                line: line.to_owned(),
                profile: profile_text.clone(),
            }
        })?;
        let threshold_profile_text = fields
            .get("threshold_profile")
            .map(String::as_str)
            .unwrap_or_else(|| threshold_profile_name(LaneDThresholdProfile::LocalDebug));
        let threshold_profile =
            parse_threshold_profile_name(threshold_profile_text).ok_or_else(|| {
                LaneDBaselineError::UnknownThresholdProfile {
                    line: line.to_owned(),
                    profile: threshold_profile_text.to_owned(),
                }
            })?;
        let suite = required_field(&fields, "suite").to_owned();
        let input = required_field(&fields, "input").to_owned();
        let baseline_elapsed_micros =
            parse_unsigned_field::<u128>(&fields, line, "baseline_elapsed_micros")?;
        let max_elapsed_micros = parse_unsigned_field::<u128>(&fields, line, "max_elapsed_micros")?;
        let max_value_nodes = parse_unsigned_field::<usize>(&fields, line, "max_value_nodes")?;
        let key = LaneDBaselineKey {
            profile,
            threshold_profile,
            suite: suite.clone(),
            input: input.clone(),
        };
        if !seen.insert(key) {
            return Err(LaneDBaselineError::DuplicateRecord {
                profile: profile_name(profile).to_owned(),
                threshold_profile: threshold_profile_name(threshold_profile).to_owned(),
                suite,
                input,
            });
        }

        records.push(LaneDBaselineRecord {
            profile,
            threshold_profile,
            suite,
            category: fields.get("category").cloned(),
            input,
            baseline_elapsed_micros,
            max_elapsed_micros,
            max_value_nodes,
            note: fields.get("note").cloned(),
        });
    }

    Ok(LaneDBaseline { records })
}

pub fn compare_baseline(
    summary: &LaneDSummary,
    baseline: &LaneDBaseline,
    options: &LaneDBaselineCompareOptions,
) -> LaneDBaselineReport {
    let suites_run = summary
        .cases
        .iter()
        .map(|case| case.suite.clone())
        .collect::<BTreeSet<_>>();
    let mut records_by_key = baseline
        .records
        .iter()
        .filter(|record| record.profile == options.profile && suites_run.contains(&record.suite))
        .filter(|record| record.threshold_profile == options.threshold_profile)
        .map(|record| {
            (
                LaneDBaselineKey {
                    profile: record.profile,
                    threshold_profile: record.threshold_profile,
                    suite: record.suite.clone(),
                    input: record.input.clone(),
                },
                record,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let records_considered = records_by_key.len();

    let mut compared = 0;
    let mut missing_records = Vec::new();
    let mut elapsed_regressions = Vec::new();
    let mut value_node_regressions = Vec::new();

    for case in &summary.cases {
        let key = LaneDBaselineKey {
            profile: options.profile,
            threshold_profile: options.threshold_profile,
            suite: case.suite.clone(),
            input: case.input.clone(),
        };
        let Some(record) = records_by_key.remove(&key) else {
            missing_records.push(case.clone());
            continue;
        };
        compared += 1;

        if case.elapsed_micros > record.max_elapsed_micros {
            elapsed_regressions.push(LaneDElapsedRegression {
                case: case.clone(),
                max_elapsed_micros: record.max_elapsed_micros,
            });
        }
        if case
            .value_nodes
            .is_some_and(|value_nodes| value_nodes > record.max_value_nodes)
        {
            value_node_regressions.push(LaneDValueNodeRegression {
                case: case.clone(),
                max_value_nodes: record.max_value_nodes,
            });
        }
    }

    let unused_records = if options.require_all_records {
        records_by_key.values().cloned().cloned().collect()
    } else {
        Vec::new()
    };

    LaneDBaselineReport {
        records_considered,
        compared,
        missing_records,
        unused_records,
        elapsed_regressions,
        value_node_regressions,
    }
}

fn required_field<'a>(fields: &'a BTreeMap<String, String>, field: &str) -> &'a str {
    fields
        .get(field)
        .expect("required baseline field was already validated")
}

fn parse_unsigned_field<T>(
    fields: &BTreeMap<String, String>,
    line: &str,
    field: &str,
) -> Result<T, LaneDBaselineError>
where
    T: std::str::FromStr,
{
    let value = required_field(fields, field);
    value
        .parse::<T>()
        .map_err(|_| LaneDBaselineError::InvalidInteger {
            line: line.to_owned(),
            field: field.to_owned(),
            value: value.to_owned(),
        })
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
