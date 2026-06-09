use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};
use super::{check_entry, issue_check_count_entry, issue_kind_count_entry, skipped_check_entry};

#[test]
fn check_can_include_lane_d_smoke_gate() {
    let output = run_difftest(&["check", "--include-lane-d-smoke"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("check\tartifact-hygiene\tclean"));
    assert!(stdout.contains("check\tlane-d-smoke\tclean"));
    assert!(
        stdout.contains("check\tlane-d-readiness\tskipped\topt-in with --include-lane-d-readiness")
    );
    assert!(stdout.contains("lane-d smoke baseline: perf/baselines/smoke.txt"));
    assert!(stdout.contains("lane-d baseline issues: 0"));
    assert!(stdout.contains("checks passed: 4"));
    assert!(stdout.contains("checks failed: 0"));
    assert!(stdout.contains("checks skipped: 1"));
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
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 4);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 0);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 1);
    let smoke = check_entry(&summary_json, "lane-d-smoke");
    assert!(json_bool(smoke, "/clean"));
    assert_eq!(
        json_str(smoke, "/baseline/path"),
        "perf/baselines/smoke.txt"
    );
    assert_eq!(json_str(smoke, "/run_options/suite"), "all");
    assert_eq!(json_str(smoke, "/run_options/profile"), "smoke");
    assert_eq!(
        json_str(smoke, "/run_options/threshold_profile"),
        "local-debug"
    );
    assert_eq!(json_u64(smoke, "/baseline/missing_records"), 0);
    assert_eq!(json_u64(smoke, "/baseline/elapsed_regressions"), 0);
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-readiness"),
            "/reason"
        ),
        "opt-in with --include-lane-d-readiness"
    );
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
    assert!(stdout.contains("check\tartifact-hygiene\tclean"));
    assert!(stdout.contains("check\tlane-d-readiness\tdirty"));
    assert!(stdout.contains(
        "check\tlane-d-smoke\tskipped\topt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    ));
    assert!(stdout.contains("lane-d readiness artifacts: 3"));
    assert!(stdout.contains("lane-d readiness ready artifacts: 1"));
    assert!(stdout.contains("lane-d readiness missing artifacts: 2"));
    assert!(stdout.contains("lane-d-readiness-missing"));
    assert!(stdout.contains("lane-d-readiness-missing\tsmoke\tci-debug\tdebug"));
    assert!(stdout.contains("lane-d-readiness-missing\tsmoke\tci-release\trelease"));
    assert!(stdout.contains("--threshold-profile ci-debug"));
    assert!(stdout.contains("checks passed: 3"));
    assert!(stdout.contains("checks failed: 1"));
    assert!(stdout.contains("checks skipped: 1"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(!json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 3);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 1);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 1);
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-smoke"),
            "/reason"
        ),
        "opt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    );
    let readiness = check_entry(&summary_json, "lane-d-readiness");
    assert!(!json_bool(readiness, "/clean"));
    assert_eq!(json_u64(readiness, "/artifacts/total"), 3);
    assert_eq!(json_u64(readiness, "/artifacts/ready"), 1);
    assert_eq!(json_u64(readiness, "/artifacts/missing"), 2);
    assert_eq!(json_u64(readiness, "/artifacts/invalid"), 0);
    assert_eq!(json_u64(readiness, "/artifacts/not_clean"), 0);
    let artifact_entries = json_array(readiness, "/artifacts/entries");
    assert_eq!(artifact_entries.len(), 3);
    assert!(
        artifact_entries.iter().any(|entry| {
            json_str(entry, "/status") == "missing"
                && json_str(entry, "/threshold_profile") == "ci-release"
                && json_str(entry, "/cargo_profile") == "release"
                && json_str(entry, "/path").contains("smoke-ci-release.txt")
                && json_str(entry, "/suggested_capture_command")
                    .starts_with("cargo run --release -p rum-difftest -- lane-d")
                && json_str(entry, "/suggested_capture_command")
                    .contains("--threshold-profile ci-release")
        }),
        "expected a ci-release missing artifact entry: {artifact_entries:?}"
    );
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "lane-d-readiness"),
            "/count"
        ),
        2
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "lane-d-readiness", "missing_artifact"),
            "/count"
        ),
        2
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert!(samples.iter().any(|sample| {
        json_str(sample, "/check") == "lane-d-readiness"
            && json_str(sample, "/kind") == "missing_artifact"
            && json_str(sample, "/location").contains("smoke-ci-debug.txt")
    }));
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
    assert!(stdout.contains("checks passed: 3"));
    assert!(stdout.contains("checks failed: 1"));
    fs::remove_dir_all(temp_dir).unwrap();
}
