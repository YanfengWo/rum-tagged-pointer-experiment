use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::{
    assert_json_null, json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest,
    stderr_text, stdout_text, unique_temp_dir,
};

#[test]
fn oracle_lane_help_lists_timeout_and_sample_controls() {
    for lane in ["lane-a", "lane-b"] {
        let output = run_difftest(&[lane, "--help"]);

        assert!(output.status.success());
        let stdout = stdout_text(&output);
        assert!(stdout.contains("--oracle-timeout-seconds"));
        assert!(stdout.contains("--summary-json"));
        assert!(stdout.contains("--max-issue-samples"));
        assert!(stdout.contains("--fail-on-seed-drift"));
    }
}

#[test]
fn oracle_lane_rejects_invalid_numeric_controls() {
    let timeout = run_difftest(&[
        "lane-a",
        "--oracle-command",
        "/bin/true",
        "--oracle-timeout-seconds",
        "0",
        "--limit",
        "1",
    ]);

    assert!(!timeout.status.success());
    assert!(stderr_text(&timeout).contains("expects a positive integer"));

    let samples = run_difftest(&["lane-b", "--max-issue-samples", "0"]);
    assert!(!samples.status.success());
    assert!(stderr_text(&samples).contains("--max-issue-samples expects a positive integer"));
}

#[test]
fn oracle_lanes_write_machine_readable_summary_json() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-json"));
        let summary_path = temp_dir.join("summary.json");
        let output = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--limit",
            "1",
            "--summary-json",
            summary_path.to_str().unwrap(),
        ]);

        assert!(output.status.success(), "stderr: {}", stderr_text(&output));
        let stdout = stdout_text(&output);
        assert!(stdout.contains("summary json:"));
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert_eq!(json_str(&summary_json, "/lane"), lane);
        assert!(!json_bool(&summary_json, "/clean"));
        assert_eq!(json_str(&summary_json, "/oracle/executable"), "/bin/true");
        assert_eq!(
            json_str(&summary_json, "/oracle/version_policy"),
            "user-provided/unpinned"
        );
        assert_eq!(json_u64(&summary_json, "/options/limit"), 1);
        assert!(!json_bool(&summary_json, "/options/fail_on_mismatch"));
        assert!(!json_bool(&summary_json, "/options/fail_on_seed_drift"));
        assert_eq!(json_u64(&summary_json, "/counts/items_compared"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_failures"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_processes"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_batches"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_batch_inputs"), 1);
        assert!(stdout.contains("oracle batches: 1"));
        assert!(stdout.contains("oracle batch inputs: 1"));
        assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 5);
        assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
        assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 1);
        assert!(!json_bool(&summary_json, "/issue_samples_truncated"));
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(samples.len(), 1);
        assert_eq!(json_str(&samples[0], "/kind"), "non_match");
        assert_eq!(json_u64(&samples[0], "/index"), 1);
        assert_eq!(json_str(&samples[0], "/status"), "OracleFailure");
        assert!(!json_bool(&samples[0], "/seed_drift"));
        if lane == "lane-b" {
            assert!(stdout.contains("oracle session batches: 1"));
            assert!(stdout.contains("oracle session steps: 1"));
            assert!(stdout.contains("oracle session transport: kernel-single-process-batches"));
            assert!(stdout.contains("oracle session processes: 1"));
            assert_eq!(
                json_str(&summary_json, "/oracle/session_batch_transport"),
                "kernel-single-process-batches"
            );
            assert_eq!(
                json_u64(&summary_json, "/oracle/session_batch_processes"),
                1
            );
            assert_eq!(json_u64(&summary_json, "/counts/oracle_session_batches"), 1);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_session_steps"), 1);
        } else {
            assert!(stdout.contains("oracle evaluation processes: 1"));
            assert_eq!(
                json_str(&summary_json, "/oracle/session_batch_transport"),
                "kernel-single-process-batches"
            );
            assert_eq!(
                json_u64(&summary_json, "/oracle/session_batch_processes"),
                1
            );
            assert_json_null(&summary_json, "/counts/oracle_session_batches");
            assert_json_null(&summary_json, "/counts/oracle_session_steps");
        }
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn oracle_lanes_can_fail_on_seed_snapshot_drift() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-seed-drift"));
        let oracle_path = temp_dir.join("fake-oracle.sh");
        let fixture_path = temp_dir.join("seed-drift.txt");
        let summary_path = temp_dir.join("summary.json");
        let failure_summary_path = temp_dir.join("failure-summary.json");
        fs::create_dir_all(&temp_dir).unwrap();
        write_fixed_value_batch_oracle(&oracle_path);
        fs::write(&fixture_path, "1 + 1 ||| stale-seed\n").unwrap();

        let output = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            summary_path.to_str().unwrap(),
        ]);

        assert!(output.status.success(), "stderr: {}", stderr_text(&output));
        let stdout = stdout_text(&output);
        assert!(stdout.contains("matches: 1"));
        assert!(stdout.contains("seed snapshots drifted from oracle: 1"));
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert!(json_bool(&summary_json, "/clean"));
        assert_eq!(
            json_u64(&summary_json, "/counts/seed_snapshots_drifted_from_oracle"),
            1
        );
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(json_str(&samples[0], "/kind"), "seed_snapshot_drift");
        assert!(json_bool(&samples[0], "/seed_drift"));

        let failure = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            failure_summary_path.to_str().unwrap(),
            "--fail-on-seed-drift",
        ]);

        assert!(
            !failure.status.success(),
            "seed drift should be fatal when requested"
        );
        assert!(stdout_text(&failure).contains("seed snapshots drifted from oracle: 1"));
        let failure_summary = fs::read_to_string(&failure_summary_path).unwrap();
        let failure_summary_json = parse_summary_json(&failure_summary);
        assert!(json_bool(
            &failure_summary_json,
            "/options/fail_on_seed_drift"
        ));
        assert!(!json_bool(
            &failure_summary_json,
            "/options/fail_on_mismatch"
        ));
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[cfg(unix)]
fn write_fixed_value_batch_oracle(path: &std::path::Path) {
    fs::write(
        path,
        r#"#!/bin/sh
wrapper=$3
manifest=$4
case "$wrapper" in
  *oracle_version*)
    printf '%s\n' '__RUM_DIFFTEST_VERSION__=fake Wolfram oracle'
    exit 0
    ;;
esac

batch_index=1
while IFS= read -r batch_manifest || [ -n "$batch_manifest" ]; do
  [ -z "$batch_manifest" ] && continue
  session_dir=$(dirname "$batch_manifest")
  step_index=1
  while IFS= read -r input_path || [ -n "$input_path" ]; do
    [ -z "$input_path" ] && continue
    suffix=$(printf '%04d-%04d' "$batch_index" "$step_index")
    : > "$session_dir/messages-$suffix.txt"
    : > "$session_dir/prints-$suffix.txt"
    printf '__RUM_DIFFTEST_SESSION_BATCH_RESULT__=%d\t%d\t2\n' "$batch_index" "$step_index"
    step_index=$((step_index + 1))
  done < "$batch_manifest"
  batch_index=$((batch_index + 1))
done < "$manifest"
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[test]
fn oracle_lanes_bound_summary_json_issue_samples() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-json-samples"));
        let summary_path = temp_dir.join("summary.json");
        let output = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--limit",
            "2",
            "--summary-json",
            summary_path.to_str().unwrap(),
            "--max-issue-samples",
            "1",
        ]);

        assert!(output.status.success(), "stderr: {}", stderr_text(&output));
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 1);
        assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
        assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 2);
        assert!(json_bool(&summary_json, "/issue_samples_truncated"));
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(samples.len(), 1);
        assert_eq!(json_str(&samples[0], "/kind"), "non_match");
        if lane == "lane-b" {
            assert_eq!(json_u64(&summary_json, "/counts/oracle_processes"), 1);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_batches"), 1);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_batch_inputs"), 2);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_session_batches"), 1);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_session_steps"), 2);
        } else {
            assert_eq!(json_u64(&summary_json, "/counts/oracle_processes"), 1);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_batches"), 2);
            assert_eq!(json_u64(&summary_json, "/counts/oracle_batch_inputs"), 2);
        }
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
