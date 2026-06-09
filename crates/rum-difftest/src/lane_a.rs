use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::fixtures::{
    GoldenEntry, curated_wolfram_core_dir, lane_a_default_fixture_paths, load_text,
    parse_golden_file, seed_root,
};
use crate::oracle::Oracle;

#[path = "lane_a/oracle_batches.rs"]
mod oracle_batches;
#[path = "lane_a/values.rs"]
mod values;

use oracle_batches::run_oracle_case_batches;
use values::{case_from_oracle_output, case_values_match, run_rum_case};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneAOptions {
    pub fixture: Option<PathBuf>,
    pub limit: Option<usize>,
    pub source: Option<String>,
    pub category: Option<String>,
    pub include_known_differences: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneASummary {
    pub total_seen: usize,
    pub compared: usize,
    pub matches: usize,
    pub mismatches: usize,
    pub rum_failures: usize,
    pub oracle_failures: usize,
    pub skipped_known_differences: usize,
    pub confirmed_known_differences: usize,
    pub resolved_known_differences: usize,
    pub seed_snapshots_checked: usize,
    pub seed_snapshots_matched_oracle: usize,
    pub seed_snapshots_drifted_from_oracle: usize,
    pub cases: Vec<LaneACaseReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneACaseReport {
    pub fixture: PathBuf,
    pub input: String,
    pub expected_seed: Option<String>,
    pub known_difference: Option<LaneAKnownDifference>,
    pub seed_matches_oracle: Option<bool>,
    pub rum: CaseValue,
    pub oracle: CaseValue,
    pub status: LaneACaseStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneAKnownDifference {
    pub id: String,
    pub kind: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseValue {
    Value {
        value: String,
        messages: Vec<String>,
        printed_output: Option<String>,
    },
    Failure {
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneACaseStatus {
    Match,
    Mismatch,
    RumFailure,
    OracleFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreparedLaneACase {
    fixture: PathBuf,
    input: String,
    expected_seed: Option<String>,
    known_difference: Option<LaneAKnownDifference>,
}

pub fn run<O: Oracle>(
    oracle: &mut O,
    options: &LaneAOptions,
) -> Result<LaneASummary, std::io::Error> {
    let mut summary = LaneASummary {
        total_seen: 0,
        compared: 0,
        matches: 0,
        mismatches: 0,
        rum_failures: 0,
        oracle_failures: 0,
        skipped_known_differences: 0,
        confirmed_known_differences: 0,
        resolved_known_differences: 0,
        seed_snapshots_checked: 0,
        seed_snapshots_matched_oracle: 0,
        seed_snapshots_drifted_from_oracle: 0,
        cases: Vec::new(),
    };
    let prepared_cases = prepare_cases(options, &mut summary)?;

    let oracle_results = run_oracle_case_batches(oracle, &prepared_cases);
    for (prepared, oracle_result) in prepared_cases.into_iter().zip(oracle_results) {
        let case = run_case(prepared, oracle_result);
        record_case_result(&mut summary, &case);
        summary.cases.push(case);
    }

    Ok(summary)
}

fn prepare_cases(
    options: &LaneAOptions,
    summary: &mut LaneASummary,
) -> Result<Vec<PreparedLaneACase>, std::io::Error> {
    let fixtures = collect_fixture_paths(options)?;
    let mut prepared_cases = Vec::new();

    'outer: for fixture in fixtures {
        let text = load_text(&fixture)?;
        let entries = parse_golden_file(&text).map_err(std::io::Error::other)?;

        for entry in entries {
            summary.total_seen += 1;

            let (input, expected_seed, known_difference) = match entry {
                GoldenEntry::Normal { input, expected } => (input, Some(expected), None),
                GoldenEntry::KnownDifference { input, metadata } => {
                    if !options.include_known_differences {
                        summary.skipped_known_differences += 1;
                        continue;
                    }
                    (
                        input,
                        None,
                        Some(LaneAKnownDifference::from_metadata(&metadata)),
                    )
                }
            };

            prepared_cases.push(PreparedLaneACase {
                fixture: fixture.clone(),
                input,
                expected_seed,
                known_difference,
            });

            if options
                .limit
                .is_some_and(|limit| prepared_cases.len() >= limit)
            {
                break 'outer;
            }
        }
    }

    Ok(prepared_cases)
}

impl LaneAKnownDifference {
    fn from_metadata(metadata: &BTreeMap<String, String>) -> Self {
        Self {
            id: metadata["id"].clone(),
            kind: metadata["type"].clone(),
            reason: metadata["reason"].clone(),
        }
    }
}

fn record_case_result(summary: &mut LaneASummary, case: &LaneACaseReport) {
    summary.compared += 1;
    match case.status {
        LaneACaseStatus::Match => summary.matches += 1,
        LaneACaseStatus::Mismatch => summary.mismatches += 1,
        LaneACaseStatus::RumFailure => summary.rum_failures += 1,
        LaneACaseStatus::OracleFailure => summary.oracle_failures += 1,
    }

    if case.known_difference.is_some() {
        match case.status {
            LaneACaseStatus::Match => summary.resolved_known_differences += 1,
            LaneACaseStatus::Mismatch
            | LaneACaseStatus::RumFailure
            | LaneACaseStatus::OracleFailure => summary.confirmed_known_differences += 1,
        }
    }

    match case.seed_matches_oracle {
        Some(true) => {
            summary.seed_snapshots_checked += 1;
            summary.seed_snapshots_matched_oracle += 1;
        }
        Some(false) => {
            summary.seed_snapshots_checked += 1;
            summary.seed_snapshots_drifted_from_oracle += 1;
        }
        None => {}
    }
}

fn run_case(prepared: PreparedLaneACase, oracle_result: CaseValue) -> LaneACaseReport {
    let rum = run_rum_case(&prepared.input);
    let status = match (&rum, &oracle_result) {
        (CaseValue::Value { .. }, CaseValue::Value { .. }) => {
            if case_values_match(&rum, &oracle_result) {
                LaneACaseStatus::Match
            } else {
                LaneACaseStatus::Mismatch
            }
        }
        (CaseValue::Failure { .. }, _) => LaneACaseStatus::RumFailure,
        (_, CaseValue::Failure { .. }) => LaneACaseStatus::OracleFailure,
    };
    let seed_matches_oracle = match (&prepared.expected_seed, &oracle_result) {
        (Some(expected), CaseValue::Value { value, .. }) => Some(expected == value),
        _ => None,
    };

    LaneACaseReport {
        fixture: prepared.fixture,
        input: prepared.input,
        expected_seed: prepared.expected_seed,
        known_difference: prepared.known_difference,
        seed_matches_oracle,
        rum,
        oracle: oracle_result,
        status,
    }
}

pub fn fixture_source(path: &Path) -> &'static str {
    let seed_root = seed_root().canonicalize().unwrap_or_else(|_| seed_root());
    let curated_root = curated_wolfram_core_dir()
        .canonicalize()
        .unwrap_or_else(|_| curated_wolfram_core_dir());
    if path_is_under(path, &seed_root) {
        "seed-golden"
    } else if path_is_under(path, &curated_root) {
        "curated-core"
    } else {
        "lane-a-custom"
    }
}

pub fn fixture_category(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace('_', "-"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn path_is_under(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
        || path
            .canonicalize()
            .is_ok_and(|canonical| canonical.starts_with(root))
}

fn collect_fixture_paths(options: &LaneAOptions) -> Result<Vec<PathBuf>, std::io::Error> {
    let fixtures = if let Some(path) = options.fixture.as_deref() {
        vec![path.to_path_buf()]
    } else {
        lane_a_default_fixture_paths()?
    };
    let filtered = fixtures
        .into_iter()
        .filter(|path| fixture_matches_filters(path, options))
        .collect::<Vec<_>>();

    if filtered.is_empty() && (options.source.is_some() || options.category.is_some()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "lane-a filters matched no fixtures: {}",
                describe_filters(options)
            ),
        ));
    }

    Ok(filtered)
}

fn fixture_matches_filters(path: &Path, options: &LaneAOptions) -> bool {
    options
        .source
        .as_deref()
        .is_none_or(|source| fixture_source(path) == source)
        && options
            .category
            .as_deref()
            .is_none_or(|category| fixture_category(path) == category)
}

fn describe_filters(options: &LaneAOptions) -> String {
    let mut parts = Vec::new();
    if let Some(source) = &options.source {
        parts.push(format!("source={source}"));
    }
    if let Some(category) = &options.category {
        parts.push(format!("category={category}"));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests;
