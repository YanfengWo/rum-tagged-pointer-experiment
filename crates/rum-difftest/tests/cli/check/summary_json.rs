use std::fs;

use super::super::{
    assert_json_null, json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest,
    stderr_text, stdout_text, unique_temp_dir,
};
use super::{check_entry, skipped_check_entry};

#[test]
fn check_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("check-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&["check", "--summary-json", summary_path.to_str().unwrap()]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/command"), "check");
    assert!(json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 3);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 0);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 2);
    assert_json_null(&summary_json, "/issue_sample_check");
    assert_json_null(&summary_json, "/issue_sample_kind");

    let inventory = check_entry(&summary_json, "inventory-baseline");
    assert!(json_bool(inventory, "/clean"));
    assert_eq!(json_u64(inventory, "/records_considered"), 50);
    let lane_c = check_entry(&summary_json, "lane-c-ledger");
    assert!(json_bool(lane_c, "/clean"));
    assert_eq!(json_u64(lane_c, "/counts/active_ledger_entries"), 4);
    assert!(!json_bool(
        lane_c,
        "/failure_gates/fail_on_open_known_difference"
    ));
    assert_eq!(json_u64(lane_c, "/failure_gates/open_known_differences"), 0);
    assert_eq!(
        json_str(
            &json_array(lane_c, "/counts/active_ledger_status_counts")[0],
            "/status"
        ),
        "deferred"
    );
    assert_eq!(
        json_u64(
            &json_array(lane_c, "/counts/active_ledger_status_counts")[0],
            "/entries"
        ),
        4
    );
    let type_counts = json_array(lane_c, "/counts/active_ledger_type_counts");
    assert_eq!(type_counts.len(), 2);
    assert_eq!(json_str(&type_counts[0], "/type"), "intentional");
    assert_eq!(json_u64(&type_counts[0], "/entries"), 1);
    assert_eq!(json_str(&type_counts[1], "/type"), "temporary");
    assert_eq!(json_u64(&type_counts[1], "/entries"), 3);
    assert_eq!(json_u64(lane_c, "/issues/tracking_target_issues"), 0);
    let artifact_hygiene = check_entry(&summary_json, "artifact-hygiene");
    assert!(json_bool(artifact_hygiene, "/clean"));
    assert_eq!(json_str(artifact_hygiene, "/root"), ".");
    assert_eq!(json_u64(artifact_hygiene, "/issues"), 0);
    assert_eq!(
        json_u64(artifact_hygiene, "/issue_kind_counts/oracle_artifact"),
        0
    );
    assert_eq!(
        json_u64(artifact_hygiene, "/issue_kind_counts/stray_fixture_file"),
        0
    );
    assert!(json_array(artifact_hygiene, "/issue_details").is_empty());

    assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 5);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 0);
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        0
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 0);
    assert!(!json_bool(&summary_json, "/issue_samples_truncated"));
    assert!(json_array(&summary_json, "/issue_check_counts").is_empty());
    assert!(json_array(&summary_json, "/issue_kind_counts").is_empty());
    assert!(json_array(&summary_json, "/issue_samples").is_empty());
    assert_eq!(json_array(&summary_json, "/skipped_checks").len(), 2);
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-smoke"),
            "/reason"
        ),
        "opt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    );
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-readiness"),
            "/reason"
        ),
        "opt-in with --include-lane-d-readiness"
    );
    fs::remove_dir_all(temp_dir).unwrap();
}
