use std::fs;

use super::super::{
    json_array, json_bool, json_u64, parse_summary_json, run_difftest, stderr_text, stdout_text,
    unique_temp_dir,
};
use super::check_entry;

#[test]
fn check_can_fail_on_open_known_differences_when_requested() {
    let temp_dir = unique_temp_dir("check-open-known-diffs-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "check",
        "--fail-on-open-known-difference",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("open known-difference gate: clean open=0"));
    assert!(stdout.contains("checks failed: 0"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(json_bool(&summary_json, "/clean"));
    let lane_c = check_entry(&summary_json, "lane-c-ledger");
    assert!(json_bool(lane_c, "/clean"));
    assert!(json_bool(
        lane_c,
        "/failure_gates/fail_on_open_known_difference"
    ));
    assert_eq!(json_u64(lane_c, "/failure_gates/open_known_differences"), 0);
    assert!(json_array(&summary_json, "/issue_samples").is_empty());
    fs::remove_dir_all(temp_dir).unwrap();
}
