use std::path::PathBuf;

use rum_runtime::Session;
use rum_stdlib::install_core;

use crate::lane_a::CaseValue;

use super::*;

#[path = "tests/oracle_batches.rs"]
mod oracle_batches;

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
fn filters_default_fixtures_by_source_and_category() {
    let options = LaneBOptions {
        source: Some("stateful".to_string()),
        category: Some("mutation-updates".to_string()),
        ..LaneBOptions::default()
    };

    let paths = collect_fixture_paths(&options).unwrap();

    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("cases/stateful/mutation_updates.txt"));
    assert_eq!(fixture_source(&paths[0]), "stateful");
    assert_eq!(fixture_category(&paths[0]), "mutation-updates");
}

#[test]
fn filtered_lane_b_fixture_collection_fails_when_empty() {
    let options = LaneBOptions {
        source: Some("stateful".to_string()),
        category: Some("not-a-category".to_string()),
        ..LaneBOptions::default()
    };

    let err = collect_fixture_paths(&options).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    assert!(
        err.to_string().contains(
            "lane-b filters matched no fixtures: source=stateful category=not-a-category"
        )
    );
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
