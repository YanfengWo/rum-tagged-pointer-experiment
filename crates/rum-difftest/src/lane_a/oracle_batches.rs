use crate::oracle::{Oracle, OracleError};

use super::{CaseValue, PreparedLaneACase, case_from_oracle_output};

const SESSION_BATCH_RESULT_MARKER: &str = "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=";

pub(super) fn run_oracle_case_batches<O: Oracle>(
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
        Err(err) => partial_timeout_case_values(&err, cases)
            .unwrap_or_else(|| failure_case_values(cases.len(), render_oracle_error(&err))),
    }
}

fn partial_timeout_case_values(
    error: &OracleError,
    cases: &[PreparedLaneACase],
) -> Option<Vec<CaseValue>> {
    let OracleError::TimedOut {
        executable,
        timeout_millis,
        stdout,
        stderr,
    } = error
    else {
        return None;
    };

    let mut values = vec![None; cases.len()];
    for line in stdout.lines() {
        let Some(payload) = line.strip_prefix(SESSION_BATCH_RESULT_MARKER) else {
            continue;
        };
        let Some((batch_text, rest)) = payload.split_once('\t') else {
            continue;
        };
        let Some((step_text, value)) = rest.split_once('\t') else {
            continue;
        };
        let Ok(batch_index) = batch_text.parse::<usize>() else {
            continue;
        };
        let Ok(step_index) = step_text.parse::<usize>() else {
            continue;
        };
        if batch_index == 0 || batch_index > values.len() || step_index != 1 {
            continue;
        }
        values[batch_index - 1] = Some(value.trim().to_owned());
    }

    let completed = values.iter().filter(|value| value.is_some()).count();
    if completed == 0 {
        return None;
    }

    let first_incomplete = values
        .iter()
        .position(Option::is_none)
        .map(|index| (index + 1, cases[index].input.as_str()));
    let timeout_context = PartialTimeoutContext {
        executable: executable.display().to_string(),
        timeout_millis: *timeout_millis,
        completed,
        total: cases.len(),
        first_incomplete,
        stdout_len: stdout.len(),
        stderr_len: stderr.len(),
    };
    Some(
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| match value {
                Some(value) => CaseValue::Value {
                    value,
                    messages: Vec::new(),
                    printed_output: None,
                },
                None => CaseValue::Failure {
                    message: timeout_context.message(index + 1),
                },
            })
            .collect(),
    )
}

fn failure_case_values(count: usize, message: String) -> Vec<CaseValue> {
    vec![CaseValue::Failure { message }; count]
}

fn render_oracle_error(error: &OracleError) -> String {
    match error {
        OracleError::TimedOut {
            executable,
            timeout_millis,
            stdout,
            stderr,
        } => format!(
            "oracle executable `{}` timed out after {timeout_millis} ms (captured stdout bytes: {}, stderr bytes: {})",
            executable.display(),
            stdout.len(),
            stderr.len()
        ),
        _ => error.to_string(),
    }
}

struct PartialTimeoutContext<'a> {
    executable: String,
    timeout_millis: u128,
    completed: usize,
    total: usize,
    first_incomplete: Option<(usize, &'a str)>,
    stdout_len: usize,
    stderr_len: usize,
}

impl PartialTimeoutContext<'_> {
    fn message(&self, batch_index: usize) -> String {
        let first_incomplete = self
            .first_incomplete
            .map(|(index, input)| format!("; first incomplete batch {index}: {input}"))
            .unwrap_or_default();
        format!(
            "oracle executable `{}` timed out after {} ms after completing {} of {} Lane A batches{first_incomplete}; this case is batch {batch_index}; captured stdout bytes: {}, stderr bytes: {}",
            self.executable,
            self.timeout_millis,
            self.completed,
            self.total,
            self.stdout_len,
            self.stderr_len
        )
    }
}
