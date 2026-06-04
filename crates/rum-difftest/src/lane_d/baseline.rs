use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::fixtures::{crate_root, load_text, parse_metadata_fields};

use super::{LaneDCaseReport, LaneDProfile, LaneDSummary, parse_profile_name, profile_name};

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
    pub suite: String,
    pub category: Option<String>,
    pub input: String,
    pub baseline_elapsed_micros: u128,
    pub max_elapsed_micros: u128,
    pub max_value_nodes: usize,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDBaselineCompareOptions {
    pub profile: LaneDProfile,
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
    #[error(
        "duplicate Lane D baseline record for profile `{profile}`, suite `{suite}`, input `{input}`"
    )]
    DuplicateRecord {
        profile: String,
        suite: String,
        input: String,
    },
}

pub fn default_smoke_baseline_path() -> PathBuf {
    crate_root().join("perf/baselines/smoke.txt")
}

pub fn load_baseline(path: impl AsRef<Path>) -> Result<LaneDBaseline, LaneDBaselineError> {
    let path = path.as_ref();
    let text = load_text(path).map_err(|source| LaneDBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_baseline(&text)
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
        let suite = required_field(&fields, "suite").to_owned();
        let input = required_field(&fields, "input").to_owned();
        let baseline_elapsed_micros =
            parse_unsigned_field::<u128>(&fields, line, "baseline_elapsed_micros")?;
        let max_elapsed_micros = parse_unsigned_field::<u128>(&fields, line, "max_elapsed_micros")?;
        let max_value_nodes = parse_unsigned_field::<usize>(&fields, line, "max_value_nodes")?;
        let key = LaneDBaselineKey {
            profile,
            suite: suite.clone(),
            input: input.clone(),
        };
        if !seen.insert(key) {
            return Err(LaneDBaselineError::DuplicateRecord {
                profile: profile_name(profile).to_owned(),
                suite,
                input,
            });
        }

        records.push(LaneDBaselineRecord {
            profile,
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
        .map(|record| {
            (
                LaneDBaselineKey {
                    profile: record.profile,
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
    suite: String,
    input: String,
}

#[cfg(test)]
mod tests {
    use super::super::{LaneDCaseStatus, LaneDOptions, LaneDSuite, run};
    use super::*;

    #[test]
    fn parses_lane_d_baseline_records() {
        let baseline = parse_baseline(
            "profile=smoke | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=1",
        )
        .unwrap();

        assert_eq!(baseline.records.len(), 1);
        assert_eq!(baseline.records[0].profile, LaneDProfile::Smoke);
        assert_eq!(baseline.records[0].suite, "timing");
        assert_eq!(baseline.records[0].category.as_deref(), Some("trivial"));
        assert_eq!(baseline.records[0].baseline_elapsed_micros, 10);
        assert_eq!(baseline.records[0].max_elapsed_micros, 100);
        assert_eq!(baseline.records[0].max_value_nodes, 1);
    }

    #[test]
    fn baseline_comparison_reports_missing_and_regressions() {
        let baseline = parse_baseline(
            r#"
            profile=smoke | suite=timing | input=fast[] | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=2
            profile=smoke | suite=timing | input=stale[] | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=2
            "#,
        )
        .unwrap();
        let summary = LaneDSummary {
            total_seen: 2,
            executed: 2,
            skipped_by_profile: 0,
            succeeded: 2,
            failed: 0,
            total_elapsed_micros: 501,
            cases: vec![
                test_case(1, "timing", "fast[]", 500, Some(3)),
                test_case(2, "timing", "missing[]", 1, Some(1)),
            ],
        };

        let report = compare_baseline(
            &summary,
            &baseline,
            &LaneDBaselineCompareOptions {
                profile: LaneDProfile::Smoke,
                require_all_records: true,
            },
        );

        assert!(!report.is_clean());
        assert_eq!(report.compared, 1);
        assert_eq!(report.missing_records.len(), 1);
        assert_eq!(report.unused_records.len(), 1);
        assert_eq!(report.elapsed_regressions.len(), 1);
        assert_eq!(report.value_node_regressions.len(), 1);
    }

    #[test]
    fn default_smoke_baseline_covers_current_smoke_lane() {
        let summary = run(&LaneDOptions {
            suite: LaneDSuite::All,
            profile: LaneDProfile::Smoke,
            limit: None,
        })
        .unwrap();
        let baseline = load_baseline(default_smoke_baseline_path()).unwrap();
        let report = compare_baseline(
            &summary,
            &baseline,
            &LaneDBaselineCompareOptions {
                profile: LaneDProfile::Smoke,
                require_all_records: true,
            },
        );

        assert!(
            report.is_clean(),
            "baseline comparison was not clean: {:?}",
            report
        );
    }

    fn test_case(
        index: usize,
        suite: &str,
        input: &str,
        elapsed_micros: u128,
        value_nodes: Option<usize>,
    ) -> LaneDCaseReport {
        LaneDCaseReport {
            suite: suite.to_owned(),
            source: PathBuf::from("test"),
            index,
            category: None,
            threshold: None,
            input: input.to_owned(),
            status: LaneDCaseStatus::Success,
            elapsed_micros,
            value_nodes,
            message_count: 0,
            print_count: 0,
        }
    }
}
