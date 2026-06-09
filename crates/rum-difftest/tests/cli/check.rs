use std::fs;

use super::{run_difftest, stderr_text, stdout_text, unique_temp_dir};

#[test]
fn check_runs_fast_non_oracle_gates() {
    let output = run_difftest(&["check"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("rum-difftest check"));
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("inventory baseline records considered: 31"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("lane-c issues: 0"));
    assert!(stdout.contains("checks passed: 2"));
    assert!(stdout.contains("checks failed: 0"));
    assert!(!stdout.contains("check\tlane-d-smoke"));
}

#[test]
fn check_help_and_unknown_flag_paths_are_stable() {
    let help = run_difftest(&["check", "--help"]);
    assert!(help.status.success());
    assert!(stdout_text(&help).contains("rum-difftest check"));
    assert!(stdout_text(&help).contains("--inventory-baseline"));
    assert!(stdout_text(&help).contains("--include-lane-d-smoke"));
    assert!(stdout_text(&help).contains("--lane-d-smoke-baseline"));
    assert!(stdout_text(&help).contains("--include-lane-d-readiness"));
    assert!(stdout_text(&help).contains("--summary-json"));
    assert!(stdout_text(&help).contains("--max-issue-samples"));

    let unknown = run_difftest(&["check", "--unknown"]);
    assert!(!unknown.status.success());
    assert!(stderr_text(&unknown).contains("unknown check flag `--unknown`"));

    let invalid_max = run_difftest(&["check", "--max-issue-samples", "0"]);
    assert!(!invalid_max.status.success());
    assert!(stderr_text(&invalid_max).contains("--max-issue-samples expects a positive integer"));
}

#[test]
fn check_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("check-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&["check", "--summary-json", summary_path.to_str().unwrap()]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    assert!(summary.contains("\"command\": \"check\""));
    assert!(summary.contains("\"clean\": true"));
    assert!(summary.contains("\"checks_passed\": 2"));
    assert!(summary.contains("\"checks_failed\": 0"));
    assert!(summary.contains("\"name\": \"inventory-baseline\""));
    assert!(summary.contains("\"records_considered\": 31"));
    assert!(summary.contains("\"name\": \"lane-c-ledger\""));
    assert!(summary.contains("\"active_ledger_entries\": 2"));
    assert!(summary.contains("\"tracking_target_issues\": 0"));
    assert!(summary.contains("\"issue_samples\": ["));
    assert!(!summary.contains("\"name\": \"lane-d-smoke\""));
    assert!(!summary.contains("\"name\": \"lane-d-readiness\""));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_can_include_lane_d_smoke_gate() {
    let output = run_difftest(&["check", "--include-lane-d-smoke"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("check\tlane-d-smoke\tclean"));
    assert!(stdout.contains("lane-d smoke baseline: perf/baselines/smoke.txt"));
    assert!(stdout.contains("lane-d baseline issues: 0"));
    assert!(stdout.contains("checks passed: 3"));
    assert!(stdout.contains("checks failed: 0"));
}

#[test]
fn check_writes_lane_d_smoke_summary_when_included() {
    let temp_dir = unique_temp_dir("check-lane-d-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "check",
        "--include-lane-d-smoke",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let summary = fs::read_to_string(&summary_path).unwrap();
    assert!(summary.contains("\"checks_passed\": 3"));
    assert!(summary.contains("\"checks_failed\": 0"));
    assert!(summary.contains("\"name\": \"lane-d-smoke\""));
    assert!(summary.contains("\"path\": \"perf/baselines/smoke.txt\""));
    assert!(summary.contains("\"suite\": \"all\""));
    assert!(summary.contains("\"profile\": \"smoke\""));
    assert!(summary.contains("\"threshold_profile\": \"local-debug\""));
    assert!(summary.contains("\"missing_records\": 0"));
    assert!(summary.contains("\"elapsed_regressions\": 0"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_can_include_lane_d_readiness_gate() {
    let temp_dir = unique_temp_dir("check-lane-d-readiness-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "check",
        "--include-lane-d-readiness",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("check\tlane-d-readiness\tdirty"));
    assert!(stdout.contains("lane-d readiness artifacts: 3"));
    assert!(stdout.contains("lane-d readiness ready artifacts: 1"));
    assert!(stdout.contains("lane-d readiness missing artifacts: 2"));
    assert!(stdout.contains("checks passed: 2"));
    assert!(stdout.contains("checks failed: 1"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    assert!(summary.contains("\"clean\": false"));
    assert!(summary.contains("\"checks_passed\": 2"));
    assert!(summary.contains("\"checks_failed\": 1"));
    assert!(summary.contains("\"name\": \"lane-d-readiness\""));
    assert!(summary.contains("\"total\": 3"));
    assert!(summary.contains("\"ready\": 1"));
    assert!(summary.contains("\"missing\": 2"));
    assert!(summary.contains("\"invalid\": 0"));
    assert!(summary.contains("\"not_clean\": 0"));
    assert!(summary.contains("\"check\": \"lane-d-readiness\""));
    assert!(summary.contains("\"kind\": \"missing_artifact\""));
    assert!(summary.contains("smoke-ci-debug.txt"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_lane_d_smoke_baseline_regression_exits_nonzero() {
    let temp_dir = unique_temp_dir("check-lane-d-baseline");
    let baseline_path = temp_dir.join("lane-d-baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "profile=smoke | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=0 | max_elapsed_micros=0 | max_value_nodes=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "check",
        "--lane-d-smoke-baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tlane-d-smoke\tdirty"));
    assert!(stdout.contains("lane-d baseline missing records:"));
    assert!(stdout.contains("lane-d baseline issues:"));
    assert!(stdout.contains("checks passed: 2"));
    assert!(stdout.contains("checks failed: 1"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_exits_nonzero_for_inventory_baseline_regression() {
    let temp_dir = unique_temp_dir("check-inventory-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "check",
        "--inventory-baseline",
        baseline_path.to_str().unwrap(),
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--max-issue-samples",
        "1",
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tdirty"));
    assert!(stdout.contains("inventory baseline regressions: 1"));
    assert!(stdout.contains("checks passed: 1"));
    assert!(stdout.contains("checks failed: 1"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    assert!(summary.contains("\"check\": \"inventory-baseline\""));
    assert!(summary.contains("\"kind\": \"regression\""));
    assert!(summary.contains("\"location\": \"lane-a/source/curated-core\""));
    assert!(summary.contains("\"message\": \"fixtures "));
    assert_eq!(
        summary.matches("\"check\": \"inventory-baseline\"").count(),
        1
    );
    fs::remove_dir_all(temp_dir).unwrap();
}
