use rum_runtime::Session;
use rum_stdlib::install_core;
use rum_terminal::{render_request_failure, render_runtime_message};

use crate::oracle::OracleEvaluation;
use crate::value_render::render_input_form_like;

use super::CaseValue;

pub(super) fn run_rum_case(input: &str) -> CaseValue {
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

pub(super) fn case_from_oracle_output(output: OracleEvaluation) -> CaseValue {
    CaseValue::Value {
        value: output.result,
        messages: normalize_oracle_messages(&output.stderr),
        printed_output: (!output.printed_output.is_empty()).then_some(output.printed_output),
    }
}

pub(super) fn case_values_match(rum: &CaseValue, oracle: &CaseValue) -> bool {
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
