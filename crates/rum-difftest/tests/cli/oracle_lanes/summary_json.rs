use std::fs;

use super::super::{
    assert_json_null, json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest,
    stderr_text, stdout_text, unique_temp_dir,
};

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
        assert_json_null(&summary_json, "/options/source");
        assert_json_null(&summary_json, "/options/category");
        assert_eq!(json_str(&summary_json, "/options/issue_sample_kind"), "all");
        assert_json_null(&summary_json, "/options/issue_sample_source");
        assert!(!json_bool(&summary_json, "/options/fail_on_mismatch"));
        assert!(!json_bool(
            &summary_json,
            "/options/fail_on_project_mismatch"
        ));
        assert!(!json_bool(&summary_json, "/options/fail_on_seed_drift"));
        assert!(!json_bool(
            &summary_json,
            "/options/fail_on_project_seed_drift"
        ));
        assert!(!json_bool(
            &summary_json,
            "/options/fail_on_resolved_known_difference"
        ));
        assert_eq!(json_u64(&summary_json, "/counts/items_compared"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_failures"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/project_non_matches"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_processes"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_batches"), 1);
        assert_eq!(json_u64(&summary_json, "/counts/oracle_batch_inputs"), 1);
        assert_eq!(
            json_u64(
                &summary_json,
                "/counts/project_seed_snapshots_drifted_from_oracle"
            ),
            0
        );
        assert!(stdout.contains("oracle batches: 1"));
        assert!(stdout.contains("oracle batch inputs: 1"));
        assert!(stdout.contains("project non-matches: 1"));
        assert_eq!(json_u64(&summary_json, "/issue_counts/total"), 1);
        assert_eq!(json_u64(&summary_json, "/issue_counts/non_match"), 1);
        assert_eq!(
            json_u64(&summary_json, "/issue_counts/seed_snapshot_drift"),
            0
        );
        assert_eq!(
            json_u64(&summary_json, "/issue_counts/resolved_known_difference"),
            0
        );
        let source_counts = json_array(&summary_json, "/issue_source_counts");
        assert_eq!(source_counts.len(), 1);
        if lane == "lane-a" {
            assert_eq!(
                json_str(&source_counts[0], "/fixture_source"),
                "curated-core"
            );
        } else {
            assert_eq!(json_str(&source_counts[0], "/fixture_source"), "stateful");
        }
        assert!(json_bool(&source_counts[0], "/project_owned_fixture"));
        assert_eq!(json_u64(&source_counts[0], "/total"), 1);
        assert_eq!(json_u64(&source_counts[0], "/non_match"), 1);
        assert_eq!(json_u64(&source_counts[0], "/seed_snapshot_drift"), 0);
        assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 5);
        assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
        assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 1);
        assert!(!json_bool(&summary_json, "/issue_samples_truncated"));
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(samples.len(), 1);
        assert_eq!(json_str(&samples[0], "/kind"), "non_match");
        if lane == "lane-a" {
            assert_eq!(json_str(&samples[0], "/fixture_source"), "curated-core");
        } else {
            assert_eq!(json_str(&samples[0], "/fixture_source"), "stateful");
        }
        assert!(json_bool(&samples[0], "/project_owned_fixture"));
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
