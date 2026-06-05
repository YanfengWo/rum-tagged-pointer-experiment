use std::fs;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::fixtures::{GoldenEntry, load_text, parse_golden_file, stateful_dir};
use crate::lane_a::CaseValue;
use crate::oracle::{Oracle, OracleError};
use crate::value_render::render_input_form_like;

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreparedLaneBStep {
    fixture: PathBuf,
    step_index: usize,
    input: String,
    expected_seed: Option<String>,
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
        let mut prepared_steps = Vec::new();
        let mut session = Session::new();
        install_core(&mut session);
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
            prepared_steps.push(PreparedLaneBStep {
                fixture: fixture.clone(),
                step_index,
                input,
                expected_seed,
            });

            if options
                .limit
                .is_some_and(|limit| summary.compared + prepared_steps.len() >= limit)
            {
                break;
            }
        }

        let oracle_results = run_oracle_session(oracle, &prepared_steps);
        for (prepared, oracle_result) in prepared_steps.into_iter().zip(oracle_results) {
            let step = run_step(&mut session, prepared, oracle_result);
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
mod tests;
