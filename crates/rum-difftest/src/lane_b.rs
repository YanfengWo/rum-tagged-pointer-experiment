use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::fixtures::{GoldenEntry, lane_b_default_fixture_paths, load_text, parse_golden_file};
use crate::lane_a::CaseValue;
use crate::oracle::Oracle;
use crate::value_render::render_input_form_like;

#[path = "lane_b/values.rs"]
mod values;
use values::{case_from_oracle_output, case_values_match, render_oracle_error};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneBOptions {
    pub fixture: Option<PathBuf>,
    pub limit: Option<usize>,
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

pub fn run<O: Oracle>(
    oracle: &mut O,
    options: &LaneBOptions,
) -> Result<LaneBSummary, std::io::Error> {
    let fixtures = collect_fixture_paths(options.fixture.as_deref())?;
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

    'outer: for fixture in fixtures {
        let text = load_text(&fixture)?;
        let entries = parse_golden_file(&text).map_err(std::io::Error::other)?;
        let mut prepared_steps = Vec::new();
        let mut session = Session::new();
        install_core(&mut session);
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

            if options
                .limit
                .is_some_and(|limit| summary.compared + prepared_steps.len() >= limit)
            {
                break;
            }
        }

        record_oracle_session_batch(&mut summary, &prepared_steps);
        let oracle_results = run_oracle_session(oracle, &prepared_steps);
        for (prepared, oracle_result) in prepared_steps.into_iter().zip(oracle_results) {
            let step = run_step(&mut session, prepared, oracle_result);
            record_step_result(&mut summary, &step);
            summary.steps.push(step);

            if options.limit.is_some_and(|limit| summary.compared >= limit) {
                break 'outer;
            }
        }
    }

    Ok(summary)
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

fn record_oracle_session_batch(summary: &mut LaneBSummary, steps: &[PreparedLaneBStep]) {
    if steps.is_empty() {
        return;
    }
    summary.oracle_session_batches += 1;
    summary.oracle_session_steps += steps.len();
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

fn run_oracle_session<O: Oracle>(oracle: &mut O, steps: &[PreparedLaneBStep]) -> Vec<CaseValue> {
    let inputs = steps
        .iter()
        .map(|step| step.input.clone())
        .collect::<Vec<_>>();
    match oracle.evaluate_session(&inputs) {
        Ok(outputs) if outputs.len() == steps.len() => {
            outputs.into_iter().map(case_from_oracle_output).collect()
        }
        Ok(outputs) => vec![
            CaseValue::Failure {
                message: format!(
                    "oracle returned {} session results for {} steps",
                    outputs.len(),
                    steps.len()
                ),
            };
            steps.len()
        ],
        Err(err) => vec![
            CaseValue::Failure {
                message: render_oracle_error(&err),
            };
            steps.len()
        ],
    }
}

fn collect_fixture_paths(single_fixture: Option<&Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    if let Some(path) = single_fixture {
        return Ok(vec![path.to_path_buf()]);
    }

    lane_b_default_fixture_paths()
}

#[cfg(test)]
mod tests;
