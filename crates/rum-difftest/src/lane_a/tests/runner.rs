use super::*;

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
