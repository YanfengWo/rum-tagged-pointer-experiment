use std::path::Path;

use crate::oracle::{Oracle, OracleError, OracleEvaluation};

use super::*;

struct BatchOracle {
    batches_seen: Vec<Vec<String>>,
}

struct TimeoutAfterPartialOracle;

#[test]
fn compares_single_case_against_oracle() {
    let report = run_case(prepared_case("2 + 3", Some("5"), None), value("5", "", ""));

    assert_eq!(report.status, LaneACaseStatus::Match);
    assert_eq!(report.seed_matches_oracle, Some(true));
}

#[test]
fn stale_seed_snapshot_does_not_affect_fresh_oracle_match() {
    let report = run_case(
        prepared_case("2 + 3", Some("old snapshot"), None),
        value("5", "", ""),
    );

    assert_eq!(report.status, LaneACaseStatus::Match);
    assert_eq!(report.seed_matches_oracle, Some(false));

    let mut summary = empty_summary();
    record_case_result(&mut summary, &report);
    assert_eq!(summary.matches, 1);
    assert_eq!(summary.seed_snapshots_checked, 1);
    assert_eq!(summary.seed_snapshots_matched_oracle, 0);
    assert_eq!(summary.seed_snapshots_drifted_from_oracle, 1);
}

#[test]
fn compares_input_form_like_list_and_rule_outputs() {
    let list_report = run_case(
        prepared_case("{1, 2}", Some("{1, 2}"), None),
        value("{1, 2}", "", ""),
    );
    assert_eq!(list_report.status, LaneACaseStatus::Match);

    let rule_list_report = run_case(
        prepared_case("Normal[<|a -> 1, b -> 2|>]", Some("{a -> 1, b -> 2}"), None),
        value("{a -> 1, b -> 2}", "", ""),
    );
    assert_eq!(rule_list_report.status, LaneACaseStatus::Match);
}

#[test]
fn compares_messages_and_printed_output_when_values_match() {
    let message_report = run_case(
        prepared_case("1 + 1", Some("2"), None),
        value("2", "test::msg: message text", ""),
    );
    assert_eq!(message_report.status, LaneACaseStatus::Mismatch);
    assert_eq!(
        message_report.oracle,
        CaseValue::Value {
            value: "2".to_string(),
            messages: vec!["! test::msg: message text".to_string()],
            printed_output: None,
        }
    );

    let print_report = run_case(
        prepared_case("1 + 1", Some("2"), None),
        value("2", "", "extra output"),
    );
    assert_eq!(print_report.status, LaneACaseStatus::Mismatch);
}

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

#[test]
fn summarizes_included_known_difference_status() {
    let mut summary = empty_summary();
    let known_difference = Some(LaneAKnownDifference {
        id: "known-edge".to_string(),
        kind: "fixture-shape".to_string(),
        reason: "temporary difference".to_string(),
    });

    record_case_result(
        &mut summary,
        &LaneACaseReport {
            fixture: Path::new("fixture.txt").to_path_buf(),
            input: "edge[]".to_string(),
            expected_seed: None,
            known_difference: known_difference.clone(),
            seed_matches_oracle: None,
            rum: CaseValue::Value {
                value: "old".to_string(),
                messages: Vec::new(),
                printed_output: None,
            },
            oracle: CaseValue::Value {
                value: "new".to_string(),
                messages: Vec::new(),
                printed_output: None,
            },
            status: LaneACaseStatus::Mismatch,
        },
    );
    record_case_result(
        &mut summary,
        &LaneACaseReport {
            fixture: Path::new("fixture.txt").to_path_buf(),
            input: "fixed[]".to_string(),
            expected_seed: None,
            known_difference,
            seed_matches_oracle: None,
            rum: CaseValue::Value {
                value: "fixed".to_string(),
                messages: Vec::new(),
                printed_output: None,
            },
            oracle: CaseValue::Value {
                value: "fixed".to_string(),
                messages: Vec::new(),
                printed_output: None,
            },
            status: LaneACaseStatus::Match,
        },
    );

    assert_eq!(summary.compared, 2);
    assert_eq!(summary.mismatches, 1);
    assert_eq!(summary.matches, 1);
    assert_eq!(summary.confirmed_known_differences, 1);
    assert_eq!(summary.resolved_known_differences, 1);
}

fn empty_summary() -> LaneASummary {
    LaneASummary {
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
    }
}

fn prepared_case(
    input: &str,
    expected_seed: Option<&str>,
    known_difference: Option<LaneAKnownDifference>,
) -> PreparedLaneACase {
    PreparedLaneACase {
        fixture: Path::new("fixture.txt").to_path_buf(),
        input: input.to_string(),
        expected_seed: expected_seed.map(str::to_string),
        known_difference,
    }
}

fn value(result: &str, stderr: &str, printed_output: &str) -> CaseValue {
    case_from_oracle_output(OracleEvaluation {
        result: result.to_string(),
        printed_output: printed_output.to_string(),
        stderr: stderr.to_string(),
    })
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
