use std::path::PathBuf;

use rum_runtime::Session;
use rum_stdlib::install_core;

use crate::lane_a::CaseValue;
use crate::oracle::{Oracle, OracleError, OracleEvaluation};

use super::*;

#[derive(Default)]
struct FakeOracle {
    evaluate_calls: usize,
    session_calls: usize,
}

impl Oracle for FakeOracle {
    fn evaluate(&mut self, _input: &str) -> Result<OracleEvaluation, OracleError> {
        self.evaluate_calls += 1;
        Ok(OracleEvaluation {
            result: "unexpected".to_string(),
            printed_output: String::new(),
            stderr: String::new(),
        })
    }

    fn evaluate_session(
        &mut self,
        inputs: &[String],
    ) -> Result<Vec<OracleEvaluation>, OracleError> {
        self.session_calls += 1;
        assert_eq!(inputs, &["x = 2".to_string(), "x + 3".to_string()]);
        Ok(vec![
            OracleEvaluation {
                result: "2".to_string(),
                printed_output: String::new(),
                stderr: String::new(),
            },
            OracleEvaluation {
                result: "5".to_string(),
                printed_output: String::new(),
                stderr: String::new(),
            },
        ])
    }
}

#[test]
fn compares_steps_against_session_oracle_once() {
    let mut oracle = FakeOracle::default();
    let prepared = vec![
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
            input: "x = 2".to_string(),
            expected_seed: Some("2".to_string()),
        },
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 2,
            input: "x + 3".to_string(),
            expected_seed: Some("5".to_string()),
        },
    ];
    let oracle_results = run_oracle_session(&mut oracle, &prepared);
    let mut session = Session::new();
    install_core(&mut session);

    let first = run_step(&mut session, prepared[0].clone(), oracle_results[0].clone());
    assert_eq!(first.status, LaneBStepStatus::Match);

    let second = run_step(&mut session, prepared[1].clone(), oracle_results[1].clone());
    assert_eq!(second.status, LaneBStepStatus::Match);
    assert_eq!(oracle.session_calls, 1);
    assert_eq!(oracle.evaluate_calls, 0);
}

#[test]
fn compares_messages_and_printed_output_when_values_match() {
    let mut session = Session::new();
    install_core(&mut session);

    let message_mismatch = run_step(
        &mut session,
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
            input: "1 + 1".to_string(),
            expected_seed: Some("2".to_string()),
        },
        CaseValue::Value {
            value: "2".to_string(),
            messages: vec!["! unexpected::tag: text".to_string()],
            printed_output: None,
        },
    );
    assert_eq!(message_mismatch.status, LaneBStepStatus::Mismatch);

    let print_mismatch = run_step(
        &mut session,
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 2,
            input: "Print[2]".to_string(),
            expected_seed: Some("Null".to_string()),
        },
        CaseValue::Value {
            value: "Null".to_string(),
            messages: Vec::new(),
            printed_output: Some("different".to_string()),
        },
    );
    assert_eq!(print_mismatch.status, LaneBStepStatus::Mismatch);
}

#[test]
fn compares_input_form_like_outputs_in_stateful_steps() {
    let mut session = Session::new();
    install_core(&mut session);

    let list_match = run_step(
        &mut session,
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
            input: "{1, 2}".to_string(),
            expected_seed: Some("{1, 2}".to_string()),
        },
        CaseValue::Value {
            value: "{1, 2}".to_string(),
            messages: Vec::new(),
            printed_output: None,
        },
    );
    assert_eq!(list_match.status, LaneBStepStatus::Match);

    let rule_list_match = run_step(
        &mut session,
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 2,
            input: "Normal[<|a -> 1, b -> 2|>]".to_string(),
            expected_seed: Some("{a -> 1, b -> 2}".to_string()),
        },
        CaseValue::Value {
            value: "{a -> 1, b -> 2}".to_string(),
            messages: Vec::new(),
            printed_output: None,
        },
    );
    assert_eq!(rule_list_match.status, LaneBStepStatus::Match);
}
