use std::path::Path;

use crate::oracle::{Oracle, OracleError, OracleEvaluation};

use super::*;

struct BatchOracle {
    batches_seen: Vec<Vec<String>>,
}

struct TimeoutAfterPartialOracle;

#[test]
fn submits_lane_a_cases_as_isolated_oracle_batches() {
    let mut oracle = BatchOracle {
        batches_seen: Vec::new(),
    };
    let cases = vec![
        prepared_case("first", Some("1"), None),
        prepared_case("second", Some("2"), None),
    ];

    let outputs = run_oracle_case_batches(&mut oracle, &cases);

    assert_eq!(
        oracle.batches_seen,
        vec![vec!["first".to_string()], vec!["second".to_string()]]
    );
    assert_eq!(outputs, vec![value("1", "", ""), value("2", "", "")]);
}

#[test]
fn preserves_completed_lane_a_batches_when_batch_oracle_times_out() {
    let mut oracle = TimeoutAfterPartialOracle;
    let cases = vec![
        prepared_case("first", Some("1"), None),
        prepared_case("second", Some("2"), None),
        prepared_case("third", Some("3"), None),
    ];

    let outputs = run_oracle_case_batches(&mut oracle, &cases);

    assert_eq!(
        outputs[0],
        CaseValue::Value {
            value: "1".to_string(),
            messages: Vec::new(),
            printed_output: None,
        }
    );
    assert_eq!(
        outputs[1],
        CaseValue::Value {
            value: "2".to_string(),
            messages: Vec::new(),
            printed_output: None,
        }
    );
    let CaseValue::Failure { message } = &outputs[2] else {
        panic!("unfinished batch should be reported as a failure");
    };
    assert!(message.contains("timed out after 30000 ms after completing 2 of 3 Lane A batches"));
    assert!(message.contains("first incomplete batch 3: third"));
    assert!(message.contains("this case is batch 3"));
    assert!(!message.contains("__RUM_DIFFTEST_SESSION_BATCH_RESULT__"));
}

impl Oracle for BatchOracle {
    fn evaluate(&mut self, _input: &str) -> Result<OracleEvaluation, OracleError> {
        panic!("Lane A should use evaluate_session_batches")
    }

    fn evaluate_session_batches(
        &mut self,
        batches: &[Vec<String>],
    ) -> Result<Vec<Vec<OracleEvaluation>>, OracleError> {
        self.batches_seen = batches.to_vec();
        Ok(batches
            .iter()
            .map(|batch| {
                batch
                    .iter()
                    .map(|input| OracleEvaluation {
                        result: match input.as_str() {
                            "first" => "1".to_string(),
                            "second" => "2".to_string(),
                            _ => "0".to_string(),
                        },
                        printed_output: String::new(),
                        stderr: String::new(),
                    })
                    .collect()
            })
            .collect())
    }
}

impl Oracle for TimeoutAfterPartialOracle {
    fn evaluate(&mut self, _input: &str) -> Result<OracleEvaluation, OracleError> {
        panic!("Lane A should use evaluate_session_batches")
    }

    fn evaluate_session_batches(
        &mut self,
        _batches: &[Vec<String>],
    ) -> Result<Vec<Vec<OracleEvaluation>>, OracleError> {
        Err(OracleError::TimedOut {
            executable: Path::new("/oracle").to_path_buf(),
            timeout_millis: 30_000,
            stdout: [
                "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=1\t1\t1",
                "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=2\t1\t2",
            ]
            .join("\n"),
            stderr: String::new(),
        })
    }
}
