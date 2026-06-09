use std::path::PathBuf;

use rum_runtime::Session;
use rum_stdlib::install_core;

use crate::lane_a::CaseValue;
use crate::oracle::{Oracle, OracleError, OracleEvaluation};

use super::super::*;

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

#[derive(Default)]
struct FakeBatchOracle {
    batch_calls: usize,
    evaluate_calls: usize,
    session_calls: usize,
}

impl Oracle for FakeBatchOracle {
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
        _inputs: &[String],
    ) -> Result<Vec<OracleEvaluation>, OracleError> {
        self.session_calls += 1;
        Ok(Vec::new())
    }

    fn evaluate_session_batches(
        &mut self,
        batches: &[Vec<String>],
    ) -> Result<Vec<Vec<OracleEvaluation>>, OracleError> {
        self.batch_calls += 1;
        assert_eq!(
            batches,
            &[
                vec!["x = 2".to_string(), "x + 3".to_string()],
                vec!["y = 4".to_string()],
            ]
        );
        Ok(vec![
            vec![
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
            ],
            vec![OracleEvaluation {
                result: "4".to_string(),
                printed_output: String::new(),
                stderr: String::new(),
            }],
        ])
    }
}

#[test]
fn compares_steps_against_default_session_batch_oracle() {
    let mut oracle = FakeOracle::default();
    let prepared = vec![
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
            input: "x = 2".to_string(),
            expected_seed: Some("2".to_string()),
            known_difference: None,
        },
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 2,
            input: "x + 3".to_string(),
            expected_seed: Some("5".to_string()),
            known_difference: None,
        },
    ];
    let oracle_results = run_oracle_session_batches(
        &mut oracle,
        &[PreparedLaneBFixture {
            steps: prepared.clone(),
        }],
    )
    .remove(0);
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
fn submits_multiple_fixtures_through_session_batch_interface() {
    let mut oracle = FakeBatchOracle::default();
    let fixtures = vec![
        PreparedLaneBFixture {
            steps: vec![
                PreparedLaneBStep {
                    fixture: PathBuf::from("first.txt"),
                    step_index: 1,
                    input: "x = 2".to_string(),
                    expected_seed: Some("2".to_string()),
                    known_difference: None,
                },
                PreparedLaneBStep {
                    fixture: PathBuf::from("first.txt"),
                    step_index: 2,
                    input: "x + 3".to_string(),
                    expected_seed: Some("5".to_string()),
                    known_difference: None,
                },
            ],
        },
        PreparedLaneBFixture {
            steps: vec![PreparedLaneBStep {
                fixture: PathBuf::from("second.txt"),
                step_index: 1,
                input: "y = 4".to_string(),
                expected_seed: Some("4".to_string()),
                known_difference: None,
            }],
        },
    ];

    let oracle_results = run_oracle_session_batches(&mut oracle, &fixtures);

    assert_eq!(oracle_results.len(), 2);
    assert_eq!(
        oracle_results[0][0],
        CaseValue::Value {
            value: "2".to_string(),
            messages: Vec::new(),
            printed_output: None,
        }
    );
    assert_eq!(
        oracle_results[1][0],
        CaseValue::Value {
            value: "4".to_string(),
            messages: Vec::new(),
            printed_output: None,
        }
    );
    assert_eq!(oracle.batch_calls, 1);
    assert_eq!(oracle.session_calls, 0);
    assert_eq!(oracle.evaluate_calls, 0);
}

#[test]
fn tracks_non_empty_oracle_session_batches() {
    let mut summary = super::empty_summary();
    let prepared = vec![
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
            input: "x = 2".to_string(),
            expected_seed: Some("2".to_string()),
            known_difference: None,
        },
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 2,
            input: "x + 3".to_string(),
            expected_seed: Some("5".to_string()),
            known_difference: None,
        },
    ];

    record_oracle_session_batches(
        &mut summary,
        &[
            PreparedLaneBFixture { steps: Vec::new() },
            PreparedLaneBFixture { steps: prepared },
        ],
    );

    assert_eq!(summary.oracle_session_batches, 1);
    assert_eq!(summary.oracle_session_steps, 2);
}
