use std::fs;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::fixtures::{GoldenEntry, golden_dir, load_text, parse_golden_file};
use crate::oracle::{Oracle, OracleError};
use crate::value_render::render_input_form_like;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneAOptions {
    pub fixture: Option<PathBuf>,
    pub limit: Option<usize>,
    pub include_known_differences: bool,
}

impl Default for LaneAOptions {
    fn default() -> Self {
        Self {
            fixture: None,
            limit: None,
            include_known_differences: false,
        }
    }
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
    pub cases: Vec<LaneACaseReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneACaseReport {
    pub fixture: PathBuf,
    pub input: String,
    pub expected_seed: Option<String>,
    pub seed_matches_oracle: Option<bool>,
    pub rum: CaseValue,
    pub oracle: CaseValue,
    pub status: LaneACaseStatus,
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

pub fn run<O: Oracle>(
    oracle: &mut O,
    options: &LaneAOptions,
) -> Result<LaneASummary, std::io::Error> {
    let fixtures = collect_fixture_paths(options.fixture.as_deref())?;
    let mut summary = LaneASummary {
        total_seen: 0,
        compared: 0,
        matches: 0,
        mismatches: 0,
        rum_failures: 0,
        oracle_failures: 0,
        skipped_known_differences: 0,
        cases: Vec::new(),
    };

    'outer: for fixture in fixtures {
        let text = load_text(&fixture)?;
        let entries = parse_golden_file(&text).map_err(std::io::Error::other)?;

        for entry in entries {
            summary.total_seen += 1;

            let (input, expected_seed) = match entry {
                GoldenEntry::Normal { input, expected } => (input, Some(expected)),
                GoldenEntry::KnownDifference { input, .. } => {
                    if !options.include_known_differences {
                        summary.skipped_known_differences += 1;
                        continue;
                    }
                    (input, None)
                }
            };

            let case = run_case(oracle, &fixture, input, expected_seed);
            summary.compared += 1;
            match case.status {
                LaneACaseStatus::Match => summary.matches += 1,
                LaneACaseStatus::Mismatch => summary.mismatches += 1,
                LaneACaseStatus::RumFailure => summary.rum_failures += 1,
                LaneACaseStatus::OracleFailure => summary.oracle_failures += 1,
            }
            summary.cases.push(case);

            if options.limit.is_some_and(|limit| summary.compared >= limit) {
                break 'outer;
            }
        }
    }

    Ok(summary)
}

fn run_case<O: Oracle>(
    oracle: &mut O,
    fixture: &Path,
    input: String,
    expected_seed: Option<String>,
) -> LaneACaseReport {
    let rum = run_rum_case(&input);
    let oracle_result = run_oracle_case(oracle, &input);
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
    let seed_matches_oracle = match (&expected_seed, &oracle_result) {
        (Some(expected), CaseValue::Value { value, .. }) => Some(expected == value),
        _ => None,
    };

    LaneACaseReport {
        fixture: fixture.to_path_buf(),
        input,
        expected_seed,
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

fn run_oracle_case<O: Oracle>(oracle: &mut O, input: &str) -> CaseValue {
    match oracle.evaluate(input) {
        Ok(output) => CaseValue::Value {
            value: output.result,
            messages: normalize_oracle_messages(&output.stderr),
            printed_output: (!output.printed_output.is_empty()).then_some(output.printed_output),
        },
        Err(err) => CaseValue::Failure {
            message: render_oracle_error(&err),
        },
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

    let mut files = fs::read_dir(golden_dir())?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests;
