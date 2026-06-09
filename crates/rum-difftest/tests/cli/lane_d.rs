use std::fs;

use super::{
    assert_json_null, json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest,
    stderr_text, stdout_text, unique_temp_dir,
};

#[test]
fn lane_d_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("lane-d-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "lane-d",
        "--suite",
        "timing",
        "--limit",
        "2",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "lane-d");
    assert_eq!(json_str(&summary_json, "/suite"), "timing");
    assert_eq!(json_str(&summary_json, "/profile"), "smoke");
    assert_eq!(json_str(&summary_json, "/threshold_profile"), "local-debug");
    assert!(json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/counts/cases_executed"), 2);
    assert_eq!(json_u64(&summary_json, "/counts/failures"), 0);
    assert_json_null(&summary_json, "/baseline");
    assert_json_null(&summary_json, "/checked_baseline_readiness");
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn lane_d_readiness_rejects_limited_runs() {
    let output = run_difftest(&["lane-d", "--check-baseline-readiness", "--limit", "1"]);

    assert!(!output.status.success());
    assert!(stderr_text(&output).contains(
        "--check-baseline-readiness expects --suite all --profile smoke without --limit"
    ));
}

#[test]
fn lane_d_readiness_summary_lists_artifacts_and_capture_commands() {
    let temp_dir = unique_temp_dir("lane-d-readiness-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "lane-d",
        "--check-baseline-readiness",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("checked-baseline-missing"));
    assert!(stdout.contains("--threshold-profile ci-debug"));
    assert!(stdout.contains("--write-baseline"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(!json_bool(&summary_json, "/clean"));
    let readiness = summary_json
        .pointer("/checked_baseline_readiness")
        .expect("lane-d readiness summary should be present");
    assert!(!json_bool(readiness, "/ready"));
    assert_eq!(json_u64(readiness, "/artifacts"), 3);
    assert_eq!(json_u64(readiness, "/missing_artifacts"), 2);
    let entries = json_array(readiness, "/entries");
    assert_eq!(entries.len(), 3);
    assert!(
        entries.iter().any(|entry| {
            json_str(entry, "/status") == "missing"
                && json_str(entry, "/threshold_profile") == "ci-debug"
                && json_str(entry, "/path").contains("smoke-ci-debug.txt")
                && json_str(entry, "/suggested_capture_command")
                    .contains("--threshold-profile ci-debug")
        }),
        "expected a ci-debug missing artifact entry: {entries:?}"
    );
    fs::remove_dir_all(temp_dir).unwrap();
}
