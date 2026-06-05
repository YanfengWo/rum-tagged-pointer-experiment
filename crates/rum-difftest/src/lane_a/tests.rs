use std::path::Path;

use crate::oracle::{Oracle, OracleError, OracleEvaluation};

use super::*;

struct FakeOracle;

impl Oracle for FakeOracle {
    fn evaluate(&mut self, input: &str) -> Result<OracleEvaluation, OracleError> {
        Ok(OracleEvaluation {
            result: match input {
                "2 + 3" => "5".to_string(),
                _ => "0".to_string(),
            },
            printed_output: String::new(),
            stderr: String::new(),
        })
    }
}

struct StaticOracle {
    output: OracleEvaluation,
}

impl Oracle for StaticOracle {
    fn evaluate(&mut self, _input: &str) -> Result<OracleEvaluation, OracleError> {
        Ok(self.output.clone())
    }
}

#[test]
fn compares_single_case_against_oracle() {
    let mut oracle = FakeOracle;
    let report = run_case(
        &mut oracle,
        Path::new("fixture.txt"),
        "2 + 3".to_string(),
        Some("5".to_string()),
        None,
    );

    assert_eq!(report.status, LaneACaseStatus::Match);
    assert_eq!(report.seed_matches_oracle, Some(true));
}

#[test]
fn compares_input_form_like_list_and_rule_outputs() {
    let mut list_oracle = StaticOracle {
        output: OracleEvaluation {
            result: "{1, 2}".to_string(),
            printed_output: String::new(),
            stderr: String::new(),
        },
    };
    let list_report = run_case(
        &mut list_oracle,
        Path::new("fixture.txt"),
        "{1, 2}".to_string(),
        Some("{1, 2}".to_string()),
        None,
    );
    assert_eq!(list_report.status, LaneACaseStatus::Match);

    let mut rule_list_oracle = StaticOracle {
        output: OracleEvaluation {
            result: "{a -> 1, b -> 2}".to_string(),
            printed_output: String::new(),
            stderr: String::new(),
        },
    };
    let rule_list_report = run_case(
        &mut rule_list_oracle,
        Path::new("fixture.txt"),
        "Normal[<|a -> 1, b -> 2|>]".to_string(),
        Some("{a -> 1, b -> 2}".to_string()),
        None,
    );
    assert_eq!(rule_list_report.status, LaneACaseStatus::Match);
}

#[test]
fn compares_messages_and_printed_output_when_values_match() {
    let mut message_oracle = StaticOracle {
        output: OracleEvaluation {
            result: "2".to_string(),
            printed_output: String::new(),
            stderr: "test::msg: message text".to_string(),
        },
    };
    let message_report = run_case(
        &mut message_oracle,
        Path::new("fixture.txt"),
        "1 + 1".to_string(),
        Some("2".to_string()),
        None,
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

    let mut print_oracle = StaticOracle {
        output: OracleEvaluation {
            result: "2".to_string(),
            printed_output: "extra output".to_string(),
            stderr: String::new(),
        },
    };
    let print_report = run_case(
        &mut print_oracle,
        Path::new("fixture.txt"),
        "1 + 1".to_string(),
        Some("2".to_string()),
        None,
    );
    assert_eq!(print_report.status, LaneACaseStatus::Mismatch);
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
        cases: Vec::new(),
    }
}
