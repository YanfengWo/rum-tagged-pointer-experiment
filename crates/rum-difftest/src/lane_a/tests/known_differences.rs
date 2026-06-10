use std::path::Path;

use super::*;

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
