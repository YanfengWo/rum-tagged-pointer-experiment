use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::fixtures::{GoldenEntry, lane_a_default_fixture_paths, load_text, parse_golden_file};
use crate::oracle::{Oracle, OracleError};
use crate::value_render::render_input_form_like;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneAOptions {
    pub fixture: Option<PathBuf>,
    pub limit: Option<usize>,
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
    let fixtures = collect_fixture_paths(options.fixture.as_deref())?;
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

fn run_oracle_case_batches<O: Oracle>(
    oracle: &mut O,
    cases: &[PreparedLaneACase],
) -> Vec<CaseValue> {
    let input_batches = cases
        .iter()
        .map(|case| vec![case.input.clone()])
        .collect::<Vec<_>>();

    match oracle.evaluate_session_batches(&input_batches) {
        Ok(outputs) if outputs.len() == cases.len() => outputs
            .into_iter()
            .map(|batch| match batch.as_slice() {
                [output] => case_from_oracle_output(output.clone()),
                _ => CaseValue::Failure {
                    message: format!(
                        "oracle returned {} session results for 1 Lane A case",
                        batch.len()
                    ),
                },
            })
            .collect(),
        Ok(outputs) => vec![
            CaseValue::Failure {
                message: format!(
                    "oracle returned {} session batches for {} Lane A cases",
                    outputs.len(),
                    cases.len()
                ),
            };
            cases.len()
        ],
        Err(err) => vec![
            CaseValue::Failure {
                message: render_oracle_error(&err),
            };
            cases.len()
        ],
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

fn run_rum_case(input: &str) -> CaseValue {
    let mut session = Session::new();
    install_core(&mut session);

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

fn case_from_oracle_output(output: crate::oracle::OracleEvaluation) -> CaseValue {
    CaseValue::Value {
        value: output.result,
        messages: normalize_oracle_messages(&output.stderr),
        printed_output: (!output.printed_output.is_empty()).then_some(output.printed_output),
    }
}

fn case_values_match(rum: &CaseValue, oracle: &CaseValue) -> bool {
    match (rum, oracle) {
        (
            CaseValue::Value {
                value: rum_value,
                messages: rum_messages,
                printed_output: rum_printed_output,
            },
            CaseValue::Value {
                value: oracle_value,
                messages: oracle_messages,
                printed_output: oracle_printed_output,
            },
        ) => {
            rum_value == oracle_value
                && rum_messages == oracle_messages
                && rum_printed_output == oracle_printed_output
        }
        _ => false,
    }
}

fn normalize_oracle_messages(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| format!("! {line}"))
        .collect()
}

fn render_oracle_error(error: &OracleError) -> String {
    error.to_string()
}

fn collect_fixture_paths(single_fixture: Option<&Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    if let Some(path) = single_fixture {
        return Ok(vec![path.to_path_buf()]);
    }

    lane_a_default_fixture_paths()
}

#[cfg(test)]
mod tests;
