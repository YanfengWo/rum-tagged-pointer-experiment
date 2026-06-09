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
fn lane_d_readiness_rejects_limited_runs() {
    let output = run_difftest(&["lane-d", "--check-baseline-readiness", "--limit", "1"]);

    assert!(!output.status.success());
    assert!(stderr_text(&output).contains(
        "--check-baseline-readiness expects --suite all --profile smoke without --limit"
    ));
}

#[test]
fn lane_d_rejects_invalid_baseline_gate_options_before_running() {
    let temp_dir = unique_temp_dir("lane-d-invalid-baseline-gates");
    let baseline_path = temp_dir.join("baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();

    for args in [
        &["lane-d", "--capture-runs", "1"][..],
        &["lane-d", "--capture-runs", "2"][..],
    ] {
        let missing_write_baseline = run_difftest(args);
        assert!(!missing_write_baseline.status.success());
        assert!(
            stderr_text(&missing_write_baseline)
                .contains("--capture-runs requires --write-baseline PATH")
        );
        assert!(stdout_text(&missing_write_baseline).is_empty());
    }

    let missing_multiplier_baseline =
        run_difftest(&["lane-d", "--baseline-elapsed-multiplier", "2"]);
    assert!(!missing_multiplier_baseline.status.success());
    assert!(
        stderr_text(&missing_multiplier_baseline)
            .contains("--baseline-elapsed-multiplier requires --write-baseline PATH")
    );
    assert!(stdout_text(&missing_multiplier_baseline).is_empty());

    let missing_min_elapsed_baseline =
        run_difftest(&["lane-d", "--baseline-min-elapsed-micros", "0"]);
    assert!(!missing_min_elapsed_baseline.status.success());
    assert!(
        stderr_text(&missing_min_elapsed_baseline)
            .contains("--baseline-min-elapsed-micros requires --write-baseline PATH")
    );
    assert!(stdout_text(&missing_min_elapsed_baseline).is_empty());

    let zero_capture_runs = run_difftest(&[
        "lane-d",
        "--capture-runs",
        "0",
        "--write-baseline",
        baseline_path.to_str().unwrap(),
    ]);
    assert!(!zero_capture_runs.status.success());
    assert!(
        stderr_text(&zero_capture_runs)
            .contains("--capture-runs expects a positive integer, got `0`")
    );
    assert!(stdout_text(&zero_capture_runs).is_empty());
    assert!(!baseline_path.exists());

    let missing_baseline = run_difftest(&["lane-d", "--fail-on-regression"]);
    assert!(!missing_baseline.status.success());
    assert!(
        stderr_text(&missing_baseline).contains("--fail-on-regression requires --baseline PATH")
    );
    assert!(stdout_text(&missing_baseline).is_empty());

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn lane_d_rejects_invalid_numeric_controls_before_running() {
    let temp_dir = unique_temp_dir("lane-d-invalid-numeric-controls");
    let baseline_path = temp_dir.join("baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();

    let cases: &[(&[&str], &str)] = &[
        (
            &["lane-d", "--limit", "0"],
            "--limit expects a positive integer, got `0`",
        ),
        (
            &["lane-d", "--limit", "many"],
            "--limit expects a positive integer, got `many`",
        ),
        (
            &[
                "lane-d",
                "--capture-runs",
                "many",
                "--write-baseline",
                baseline_path.to_str().unwrap(),
            ],
            "--capture-runs expects a positive integer, got `many`",
        ),
        (
            &[
                "lane-d",
                "--baseline-elapsed-multiplier",
                "0",
                "--write-baseline",
                baseline_path.to_str().unwrap(),
            ],
            "--baseline-elapsed-multiplier expects a positive integer, got `0`",
        ),
        (
            &[
                "lane-d",
                "--baseline-elapsed-multiplier",
                "many",
                "--write-baseline",
                baseline_path.to_str().unwrap(),
            ],
            "--baseline-elapsed-multiplier expects a positive integer, got `many`",
        ),
        (
            &[
                "lane-d",
                "--baseline-min-elapsed-micros",
                "-1",
                "--write-baseline",
                baseline_path.to_str().unwrap(),
            ],
            "--baseline-min-elapsed-micros expects a non-negative integer, got `-1`",
        ),
        (
            &[
                "lane-d",
                "--baseline-min-elapsed-micros",
                "soon",
                "--write-baseline",
                baseline_path.to_str().unwrap(),
            ],
            "--baseline-min-elapsed-micros expects a non-negative integer, got `soon`",
        ),
    ];

    for (args, expected_error) in cases {
        let output = run_difftest(args);
        assert!(
            !output.status.success(),
            "expected failure for {args:?}, stdout: {}",
            stdout_text(&output)
        );
        assert!(
            stderr_text(&output).contains(expected_error),
            "expected `{expected_error}` for {args:?}, stderr: {}",
            stderr_text(&output)
        );
        assert!(
            stdout_text(&output).is_empty(),
            "expected no workload output for {args:?}"
        );
        assert!(
            !baseline_path.exists(),
            "invalid numeric options should not write a baseline for {args:?}"
        );
    }

    fs::remove_dir_all(temp_dir).unwrap();
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
