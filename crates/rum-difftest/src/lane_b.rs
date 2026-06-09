use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::fixtures::{
    GoldenEntry, lane_b_default_fixture_paths, load_text, parse_golden_file, seed_root,
    stateful_dir,
};
use crate::lane_a::CaseValue;
use crate::oracle::Oracle;
use crate::value_render::render_input_form_like;

#[path = "lane_b/oracle_batches.rs"]
mod oracle_batches;
#[path = "lane_b/values.rs"]
mod values;
use oracle_batches::run_oracle_session_batches;
use values::case_values_match;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneBOptions {
    pub fixture: Option<PathBuf>,
    pub limit: Option<usize>,
    pub source: Option<String>,
    pub category: Option<String>,
    pub include_known_differences: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneBSummary {
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
    pub oracle_session_batches: usize,
    pub oracle_session_steps: usize,
    pub steps: Vec<LaneBStepReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneBStepReport {
    pub fixture: PathBuf,
    pub step_index: usize,
    pub input: String,
    pub expected_seed: Option<String>,
    pub known_difference: Option<LaneBKnownDifference>,
    pub seed_matches_oracle: Option<bool>,
    pub rum: CaseValue,
    pub oracle: CaseValue,
    pub status: LaneBStepStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneBKnownDifference {
    pub id: String,
    pub kind: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneBStepStatus {
    Match,
    Mismatch,
    RumFailure,
    OracleFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreparedLaneBStep {
    fixture: PathBuf,
    step_index: usize,
    input: String,
    expected_seed: Option<String>,
    known_difference: Option<LaneBKnownDifference>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreparedLaneBFixture {
    steps: Vec<PreparedLaneBStep>,
}

pub fn run<O: Oracle>(
    oracle: &mut O,
    options: &LaneBOptions,
) -> Result<LaneBSummary, std::io::Error> {
    let mut summary = LaneBSummary {
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
        oracle_session_batches: 0,
        oracle_session_steps: 0,
        steps: Vec::new(),
    };
    let prepared_fixtures = prepare_fixture_batches(options, &mut summary)?;

    record_oracle_session_batches(&mut summary, &prepared_fixtures);
    let oracle_batches = run_oracle_session_batches(oracle, &prepared_fixtures);
    for (prepared_fixture, oracle_results) in prepared_fixtures.into_iter().zip(oracle_batches) {
        let mut session = Session::new();
        install_core(&mut session);
        for (prepared, oracle_result) in prepared_fixture.steps.into_iter().zip(oracle_results) {
            let step = run_step(&mut session, prepared, oracle_result);
            record_step_result(&mut summary, &step);
            summary.steps.push(step);
        }
    }

    Ok(summary)
}

fn prepare_fixture_batches(
    options: &LaneBOptions,
    summary: &mut LaneBSummary,
) -> Result<Vec<PreparedLaneBFixture>, std::io::Error> {
    let fixtures = collect_fixture_paths(options)?;
    let mut prepared_fixtures = Vec::new();
    let mut prepared_count = 0usize;

    'outer: for fixture in fixtures {
        let text = load_text(&fixture)?;
        let entries = parse_golden_file(&text).map_err(std::io::Error::other)?;
        let mut prepared_steps = Vec::new();
        let mut step_index = 0usize;

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
                        Some(LaneBKnownDifference::from_metadata(&metadata)),
                    )
                }
            };

            step_index += 1;
            prepared_steps.push(PreparedLaneBStep {
                fixture: fixture.clone(),
                step_index,
                input,
                expected_seed,
                known_difference,
            });
            prepared_count += 1;

            if options.limit.is_some_and(|limit| prepared_count >= limit) {
                break;
            }
        }

        if !prepared_steps.is_empty() {
            prepared_fixtures.push(PreparedLaneBFixture {
                steps: prepared_steps,
            });
        }
        if options.limit.is_some_and(|limit| prepared_count >= limit) {
            break 'outer;
        }
    }

    Ok(prepared_fixtures)
}

impl LaneBKnownDifference {
    fn from_metadata(metadata: &BTreeMap<String, String>) -> Self {
        Self {
            id: metadata["id"].clone(),
            kind: metadata["type"].clone(),
            reason: metadata["reason"].clone(),
        }
    }
}

fn record_step_result(summary: &mut LaneBSummary, step: &LaneBStepReport) {
    summary.compared += 1;
    match step.status {
        LaneBStepStatus::Match => summary.matches += 1,
        LaneBStepStatus::Mismatch => summary.mismatches += 1,
        LaneBStepStatus::RumFailure => summary.rum_failures += 1,
        LaneBStepStatus::OracleFailure => summary.oracle_failures += 1,
    }

    if step.known_difference.is_some() {
        match step.status {
            LaneBStepStatus::Match => summary.resolved_known_differences += 1,
            LaneBStepStatus::Mismatch
            | LaneBStepStatus::RumFailure
            | LaneBStepStatus::OracleFailure => summary.confirmed_known_differences += 1,
        }
    }

    match step.seed_matches_oracle {
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

fn record_oracle_session_batches(summary: &mut LaneBSummary, fixtures: &[PreparedLaneBFixture]) {
    for fixture in fixtures {
        if fixture.steps.is_empty() {
            continue;
        }
        summary.oracle_session_batches += 1;
        summary.oracle_session_steps += fixture.steps.len();
    }
}

fn run_step(
    session: &mut Session,
    prepared: PreparedLaneBStep,
    oracle_result: CaseValue,
) -> LaneBStepReport {
    let PreparedLaneBStep {
        fixture,
        step_index,
        input,
        expected_seed,
        known_difference,
    } = prepared;
    let rum = run_rum_step(session, &input);
    let status = match (&rum, &oracle_result) {
        (CaseValue::Value { .. }, CaseValue::Value { .. }) => {
            if case_values_match(&rum, &oracle_result) {
                LaneBStepStatus::Match
            } else {
                LaneBStepStatus::Mismatch
            }
        }
        (CaseValue::Failure { .. }, _) => LaneBStepStatus::RumFailure,
        (_, CaseValue::Failure { .. }) => LaneBStepStatus::OracleFailure,
    };
    let seed_matches_oracle = match (&expected_seed, &oracle_result) {
        (Some(expected), CaseValue::Value { value, .. }) => Some(expected == value),
        _ => None,
    };

    LaneBStepReport {
        fixture,
        step_index,
        input,
        expected_seed,
        known_difference,
        seed_matches_oracle,
        rum,
        oracle: oracle_result,
        status,
    }
}

fn run_rum_step(session: &mut Session, input: &str) -> CaseValue {
    match session.eval_request(input) {
        Ok(response) => CaseValue::Value {
            value: render_input_form_like(&response.value),
            messages: response
                .messages
                .iter()
                .map(render_runtime_message)
                .collect(),
            printed_output: (!response.prints.is_empty()).then(|| response.prints.join("\n")),
        },
        Err(failure) => CaseValue::Failure {
            message: render_request_failure(&failure),
        },
    }
}

pub fn fixture_source(path: &Path) -> &'static str {
    let seed_root = seed_root().canonicalize().unwrap_or_else(|_| seed_root());
    let stateful_root = stateful_dir()
        .canonicalize()
        .unwrap_or_else(|_| stateful_dir());
    if path_is_under(path, &seed_root) {
        "seed-golden"
    } else if path_is_under(path, &stateful_root) {
        "stateful"
    } else {
        "lane-b-custom"
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

fn collect_fixture_paths(options: &LaneBOptions) -> Result<Vec<PathBuf>, std::io::Error> {
    let fixtures = if let Some(path) = options.fixture.as_deref() {
        vec![path.to_path_buf()]
    } else {
        lane_b_default_fixture_paths()?
    };
    let filtered = fixtures
        .into_iter()
        .filter(|path| fixture_matches_filters(path, options))
        .collect::<Vec<_>>();

    if filtered.is_empty() && (options.source.is_some() || options.category.is_some()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "lane-b filters matched no fixtures: {}",
                describe_filters(options)
            ),
        ));
    }

    Ok(filtered)
}

fn fixture_matches_filters(path: &Path, options: &LaneBOptions) -> bool {
    options
        .source
        .as_deref()
        .is_none_or(|source| fixture_source(path) == source)
        && options
            .category
            .as_deref()
            .is_none_or(|category| fixture_category(path) == category)
}

fn describe_filters(options: &LaneBOptions) -> String {
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
