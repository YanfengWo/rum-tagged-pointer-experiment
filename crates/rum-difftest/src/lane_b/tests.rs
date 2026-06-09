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
fn stale_seed_snapshot_does_not_affect_fresh_oracle_step_match() {
    let mut session = Session::new();
    install_core(&mut session);
    let step = run_step(
        &mut session,
        PreparedLaneBStep {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
            input: "2 + 3".to_string(),
            expected_seed: Some("old snapshot".to_string()),
            known_difference: None,
        },
        CaseValue::Value {
            value: "5".to_string(),
            messages: Vec::new(),
            printed_output: None,
        },
    );

    assert_eq!(step.status, LaneBStepStatus::Match);
    assert_eq!(step.seed_matches_oracle, Some(false));

    let mut summary = empty_summary();
    record_step_result(&mut summary, &step);
    assert_eq!(summary.matches, 1);
    assert_eq!(summary.seed_snapshots_checked, 1);
    assert_eq!(summary.seed_snapshots_matched_oracle, 0);
    assert_eq!(summary.seed_snapshots_drifted_from_oracle, 1);
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
            known_difference: None,
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
            known_difference: None,
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
            known_difference: None,
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
            known_difference: None,
        },
        CaseValue::Value {
            value: "{a -> 1, b -> 2}".to_string(),
            messages: Vec::new(),
            printed_output: None,
        },
    );
    assert_eq!(rule_list_match.status, LaneBStepStatus::Match);
}

#[test]
fn summarizes_included_known_difference_status() {
    let mut summary = empty_summary();
    let known_difference = Some(LaneBKnownDifference {
        id: "known-edge".to_string(),
        kind: "fixture-shape".to_string(),
        reason: "temporary difference".to_string(),
    });

    record_step_result(
        &mut summary,
        &LaneBStepReport {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 1,
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
            status: LaneBStepStatus::Mismatch,
        },
    );
    record_step_result(
        &mut summary,
        &LaneBStepReport {
            fixture: PathBuf::from("stateful.txt"),
            step_index: 2,
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
            status: LaneBStepStatus::Match,
        },
    );

    assert_eq!(summary.compared, 2);
    assert_eq!(summary.mismatches, 1);
    assert_eq!(summary.matches, 1);
    assert_eq!(summary.confirmed_known_differences, 1);
    assert_eq!(summary.resolved_known_differences, 1);
}

#[test]
fn tracks_non_empty_oracle_session_batches() {
    let mut summary = empty_summary();
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

    record_oracle_session_batch(&mut summary, &[]);
    record_oracle_session_batch(&mut summary, &prepared);

    assert_eq!(summary.oracle_session_batches, 1);
    assert_eq!(summary.oracle_session_steps, 2);
}

fn empty_summary() -> LaneBSummary {
    LaneBSummary {
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
        oracle_session_batches: 0,
        oracle_session_steps: 0,
        steps: Vec::new(),
    }
}
