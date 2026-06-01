use std::fs;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::fixtures::{GoldenEntry, load_text, parse_golden_file, stateful_dir};
use crate::lane_a::CaseValue;
use crate::oracle::{Oracle, OracleError};

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
    pub steps: Vec<LaneBStepReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneBStepReport {
    pub fixture: PathBuf,
    pub step_index: usize,
    pub input: String,
    pub expected_seed: Option<String>,
    pub seed_matches_oracle: Option<bool>,
    pub rum: CaseValue,
    pub oracle: CaseValue,
    pub status: LaneBStepStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneBStepStatus {
    Match,
    Mismatch,
    RumFailure,
    OracleFailure,
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
        steps: Vec::new(),
    };

    'outer: for fixture in fixtures {
        let text = load_text(&fixture)?;
        let entries = parse_golden_file(&text).map_err(std::io::Error::other)?;
        let mut session = Session::new();
        install_core(&mut session);
        let mut prior_oracle_inputs = Vec::new();
        let mut step_index = 0usize;

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

            step_index += 1;
            let step = run_step(
                oracle,
                &mut session,
                &fixture,
                step_index,
                input,
                expected_seed,
                &prior_oracle_inputs,
            );
            prior_oracle_inputs.push(step.input.clone());
            summary.compared += 1;
            match step.status {
                LaneBStepStatus::Match => summary.matches += 1,
                LaneBStepStatus::Mismatch => summary.mismatches += 1,
                LaneBStepStatus::RumFailure => summary.rum_failures += 1,
                LaneBStepStatus::OracleFailure => summary.oracle_failures += 1,
            }
            summary.steps.push(step);

            if options.limit.is_some_and(|limit| summary.compared >= limit) {
                break 'outer;
            }
        }
    }

    Ok(summary)
}

fn run_step<O: Oracle>(
    oracle: &mut O,
    session: &mut Session,
    fixture: &Path,
    step_index: usize,
    input: String,
    expected_seed: Option<String>,
    prior_oracle_inputs: &[String],
) -> LaneBStepReport {
    let rum = run_rum_step(session, &input);
    let oracle_input = render_oracle_stateful_input(prior_oracle_inputs, &input);
    let oracle_result = run_oracle_step(oracle, &oracle_input);
    let status = match (&rum, &oracle_result) {
        (
            CaseValue::Value {
                value: rum_value, ..
            },
            CaseValue::Value {
                value: oracle_value,
                ..
            },
        ) => {
            if rum_value == oracle_value {
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
        fixture: fixture.to_path_buf(),
        step_index,
        input,
        expected_seed,
        seed_matches_oracle,
        rum,
        oracle: oracle_result,
        status,
    }
}

fn run_rum_step(session: &mut Session, input: &str) -> CaseValue {
    match session.eval_request(input) {
        Ok(response) => CaseValue::Value {
            value: response.value.to_string(),
            messages: response
                .messages
                .iter()
                .map(render_runtime_message)
                .collect(),
            printed_output: None,
        },
        Err(failure) => CaseValue::Failure {
            message: render_request_failure(&failure),
        },
    }
}

fn run_oracle_step<O: Oracle>(oracle: &mut O, input: &str) -> CaseValue {
    match oracle.evaluate(input) {
        Ok(output) => CaseValue::Value {
            value: output.result,
            messages: if output.stderr.is_empty() {
                Vec::new()
            } else {
                output.stderr.lines().map(|line| line.to_owned()).collect()
            },
            printed_output: (!output.printed_output.is_empty()).then_some(output.printed_output),
        },
        Err(err) => CaseValue::Failure {
            message: render_oracle_error(&err),
        },
    }
}

fn render_oracle_stateful_input(prior_inputs: &[String], current: &str) -> String {
    if prior_inputs.is_empty() {
        return current.to_owned();
    }

    format!("{}; {current}", prior_inputs.join("; "))
}

fn render_oracle_error(error: &OracleError) -> String {
    error.to_string()
}

fn collect_fixture_paths(single_fixture: Option<&Path>) -> Result<Vec<PathBuf>, std::io::Error> {
    if let Some(path) = single_fixture {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut files = fs::read_dir(stateful_dir())?
        .map(|entry| entry.map(|entry| entry.path()))
        .filter(|entry| {
            entry.as_ref().map_or(true, |path| {
                path.extension().and_then(|ext| ext.to_str()) == Some("txt")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::{Oracle, OracleError, OracleEvaluation};

    struct FakeOracle;

    impl Oracle for FakeOracle {
        fn evaluate(&mut self, input: &str) -> Result<OracleEvaluation, OracleError> {
            Ok(OracleEvaluation {
                result: match input {
                    "x = 2" => "2".to_string(),
                    "x = 2; x + 3" => "5".to_string(),
                    _ => "0".to_string(),
                },
                printed_output: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn compares_steps_against_accumulated_oracle_input() {
        let mut oracle = FakeOracle;
        let mut session = Session::new();
        install_core(&mut session);
        let mut prior = Vec::new();

        let first = run_step(
            &mut oracle,
            &mut session,
            Path::new("stateful.txt"),
            1,
            "x = 2".to_string(),
            Some("2".to_string()),
            &prior,
        );
        assert_eq!(first.status, LaneBStepStatus::Match);
        prior.push(first.input);

        let second = run_step(
            &mut oracle,
            &mut session,
            Path::new("stateful.txt"),
            2,
            "x + 3".to_string(),
            Some("5".to_string()),
            &prior,
        );
        assert_eq!(second.status, LaneBStepStatus::Match);
    }
}
