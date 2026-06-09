use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stdout_text,
    unique_temp_dir,
};

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
                && json_str(entry, "/cargo_profile") == "debug"
                && json_str(entry, "/path").contains("smoke-ci-debug.txt")
                && json_str(entry, "/suggested_capture_command")
                    .contains("--threshold-profile ci-debug")
        }),
        "expected a ci-debug missing artifact entry: {entries:?}"
    );
    assert!(
        entries.iter().any(|entry| {
            json_str(entry, "/status") == "missing"
                && json_str(entry, "/threshold_profile") == "ci-release"
                && json_str(entry, "/cargo_profile") == "release"
                && json_str(entry, "/path").contains("smoke-ci-release.txt")
                && json_str(entry, "/suggested_capture_command")
                    .starts_with("cargo run --release -p rum-difftest -- lane-d")
                && json_str(entry, "/suggested_capture_command")
                    .contains("--threshold-profile ci-release")
        }),
        "expected a ci-release missing artifact entry with release capture command: {entries:?}"
    );
    fs::remove_dir_all(temp_dir).unwrap();
}
