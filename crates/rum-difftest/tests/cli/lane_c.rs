use std::fs;

use super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};

#[test]
fn lane_c_help_and_unknown_flag_paths_are_stable() {
    let help = run_difftest(&["lane-c", "--help"]);
    assert!(help.status.success());
    assert!(stdout_text(&help).contains("rum-difftest lane-c"));
    assert!(stdout_text(&help).contains("--summary-json"));
    assert!(stdout_text(&help).contains("--fail-on-open-known-difference"));

    let unknown = run_difftest(&["lane-c", "--unknown"]);
    assert!(!unknown.status.success());
    assert!(stderr_text(&unknown).contains("unknown lane-c flag `--unknown`"));
}

#[test]
fn lane_c_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("lane-c-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "lane-c",
        "--fail-on-open-known-difference",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("open known-difference gate: clean open=0"));
    assert!(stdout.contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "lane-c");
    assert!(json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/counts/active_ledger_entries"), 4);
    assert_eq!(
        json_str(
            &json_array(&summary_json, "/counts/active_ledger_status_counts")[0],
            "/status"
        ),
        "deferred"
    );
    assert_eq!(
        json_u64(
            &json_array(&summary_json, "/counts/active_ledger_status_counts")[0],
            "/entries"
        ),
        4
    );
    let type_counts = json_array(&summary_json, "/counts/active_ledger_type_counts");
    assert_eq!(type_counts.len(), 2);
    assert_eq!(json_str(&type_counts[0], "/type"), "intentional");
    assert_eq!(json_u64(&type_counts[0], "/entries"), 1);
    assert_eq!(json_str(&type_counts[1], "/type"), "temporary");
    assert_eq!(json_u64(&type_counts[1], "/entries"), 3);
    assert_eq!(
        json_u64(&summary_json, "/counts/fixture_known_differences"),
        5
    );
    assert!(json_bool(
        &summary_json,
        "/failure_gates/fail_on_open_known_difference"
    ));
    assert_eq!(
        json_u64(&summary_json, "/failure_gates/open_known_differences"),
        0
    );
    assert_eq!(json_u64(&summary_json, "/issues/tracking_target_issues"), 0);
    fs::remove_dir_all(temp_dir).unwrap();
}
