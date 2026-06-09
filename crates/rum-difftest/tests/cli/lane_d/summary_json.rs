use std::fs;

use super::super::{
    assert_json_null, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
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
fn lane_d_writes_baseline_summary_json() {
    let temp_dir = unique_temp_dir("lane-d-write-baseline-json");
    let summary_path = temp_dir.join("summary.json");
    let baseline_path = temp_dir.join("baseline.txt");
    let output = run_difftest(&[
        "lane-d",
        "--suite",
        "timing",
        "--limit",
        "1",
        "--write-baseline",
        baseline_path.to_str().unwrap(),
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(baseline_path.is_file());
    assert!(stdout_text(&output).contains("baseline written:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(json_bool(&summary_json, "/clean"));
    assert_json_null(&summary_json, "/baseline");
    assert_json_null(&summary_json, "/checked_baseline_readiness");
    assert_eq!(
        json_str(&summary_json, "/written_baseline/path"),
        baseline_path.display().to_string()
    );
    assert_eq!(json_u64(&summary_json, "/written_baseline/records"), 1);
    assert_eq!(json_u64(&summary_json, "/written_baseline/capture_runs"), 1);
    assert_eq!(
        json_str(&summary_json, "/written_baseline/threshold_profile"),
        "local-debug"
    );
    assert_eq!(
        json_u64(&summary_json, "/written_baseline/elapsed_multiplier"),
        10
    );
    assert_eq!(
        json_u64(&summary_json, "/written_baseline/min_elapsed_micros"),
        250000
    );
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn failed_lane_d_baseline_gate_still_writes_dirty_summary_json() {
    let temp_dir = unique_temp_dir("lane-d-dirty-baseline-json");
    let baseline_path = temp_dir.join("baseline.txt");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "profile=smoke | suite=timing | input=1 + 1 | baseline_elapsed_micros=0 | max_elapsed_micros=999999999 | max_value_nodes=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "lane-d",
        "--suite",
        "timing",
        "--limit",
        "1",
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--fail-on-regression",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(stdout_text(&output).contains("baseline value-node regressions: 1"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "lane-d");
    assert!(!json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/counts/cases_executed"), 1);
    assert!(!json_bool(&summary_json, "/baseline/clean"));
    assert_eq!(json_u64(&summary_json, "/baseline/records_considered"), 1);
    assert_eq!(json_u64(&summary_json, "/baseline/compared"), 1);
    assert_eq!(
        json_u64(&summary_json, "/baseline/value_node_regressions"),
        1
    );
    assert_json_null(&summary_json, "/checked_baseline_readiness");
    fs::remove_dir_all(temp_dir).unwrap();
}
