use crate::lane_a::CaseValue;
use crate::oracle::Oracle;

use super::PreparedLaneBFixture;
use super::values::{case_from_oracle_output, render_oracle_error};

pub(super) fn run_oracle_session_batches<O: Oracle>(
    oracle: &mut O,
    fixtures: &[PreparedLaneBFixture],
) -> Vec<Vec<CaseValue>> {
    let input_batches = fixtures
        .iter()
        .map(|fixture| {
            fixture
                .steps
                .iter()
                .map(|step| step.input.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    match oracle.evaluate_session_batches(&input_batches) {
        Ok(outputs) if outputs.len() == fixtures.len() => fixtures
            .iter()
            .zip(outputs)
            .map(|(fixture, outputs)| oracle_fixture_results(fixture, outputs))
            .collect(),
        Ok(outputs) => failure_batches(
            fixtures,
            format!(
                "oracle returned {} session batches for {} fixtures",
                outputs.len(),
                fixtures.len()
            ),
        ),
        Err(err) => failure_batches(fixtures, render_oracle_error(&err)),
    }
}

fn oracle_fixture_results(
    fixture: &PreparedLaneBFixture,
    outputs: Vec<crate::oracle::OracleEvaluation>,
) -> Vec<CaseValue> {
    if outputs.len() == fixture.steps.len() {
        return outputs.into_iter().map(case_from_oracle_output).collect();
    }

    vec![
        CaseValue::Failure {
            message: format!(
                "oracle returned {} session results for {} steps",
                outputs.len(),
                fixture.steps.len()
            ),
        };
        fixture.steps.len()
    ]
}

fn failure_batches(fixtures: &[PreparedLaneBFixture], message: String) -> Vec<Vec<CaseValue>> {
    fixtures
        .iter()
        .map(|fixture| {
            vec![
                CaseValue::Failure {
                    message: message.clone(),
                };
                fixture.steps.len()
            ]
        })
        .collect()
}
