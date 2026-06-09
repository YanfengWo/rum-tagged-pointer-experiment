use std::fs;

use super::{parse_summary_json, run_difftest, stderr_text, stdout_text, unique_temp_dir};

#[test]
fn oracle_lane_help_lists_timeout_and_sample_controls() {
    for lane in ["lane-a", "lane-b"] {
        let output = run_difftest(&[lane, "--help"]);

        assert!(output.status.success());
        let stdout = stdout_text(&output);
        assert!(stdout.contains("--oracle-timeout-seconds"));
        assert!(stdout.contains("--summary-json"));
        assert!(stdout.contains("--max-issue-samples"));
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
        parse_summary_json(&summary);
        assert!(summary.contains(&format!("\"lane\": \"{lane}\"")));
        assert!(summary.contains("\"schema_version\": 1"));
        assert!(summary.contains("\"clean\": false"));
        assert!(summary.contains("\"executable\": \"/bin/true\""));
        assert!(summary.contains("\"version_policy\": \"user-provided/unpinned\""));
        assert!(summary.contains("\"limit\": 1"));
        assert!(summary.contains("\"items_compared\": 1"));
        assert!(summary.contains("\"oracle_failures\": 1"));
        assert!(summary.contains("\"issue_samples_limit\": 5"));
        assert!(summary.contains("\"issue_samples_emitted\": 1"));
        assert!(summary.contains("\"issue_samples_total\": 1"));
        assert!(summary.contains("\"issue_samples_truncated\": false"));
        assert!(summary.contains("\"issue_samples\": ["));
        assert!(summary.contains("\"kind\": \"non_match\""));
        assert!(summary.contains("\"index\": 1"));
        assert!(summary.contains("\"status\": \"OracleFailure\""));
        assert!(summary.contains("\"seed_drift\": false"));
        if lane == "lane-b" {
            assert!(stdout.contains("oracle session batches: 1"));
            assert!(stdout.contains("oracle session steps: 1"));
            assert!(stdout.contains("oracle session transport: kernel-single-process-batches"));
            assert!(stdout.contains("oracle session processes: 1"));
            assert!(
                summary.contains("\"session_batch_transport\": \"kernel-single-process-batches\"")
            );
            assert!(summary.contains("\"session_batch_processes\": 1"));
            assert!(summary.contains("\"oracle_session_batches\": 1"));
            assert!(summary.contains("\"oracle_session_steps\": 1"));
        } else {
            assert!(summary.contains("\"session_batch_transport\": null"));
            assert!(summary.contains("\"session_batch_processes\": null"));
            assert!(summary.contains("\"oracle_session_batches\": null"));
            assert!(summary.contains("\"oracle_session_steps\": null"));
        }
        fs::remove_dir_all(temp_dir).unwrap();
    }
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
        parse_summary_json(&summary);
        assert!(summary.contains("\"issue_samples_limit\": 1"));
        assert!(summary.contains("\"issue_samples_emitted\": 1"));
        assert!(summary.contains("\"issue_samples_total\": 2"));
        assert!(summary.contains("\"issue_samples_truncated\": true"));
        assert_eq!(summary.matches("\"kind\": \"non_match\"").count(), 1);
        if lane == "lane-b" {
            assert!(summary.contains("\"oracle_session_batches\": 1"));
            assert!(summary.contains("\"oracle_session_steps\": 2"));
        }
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
