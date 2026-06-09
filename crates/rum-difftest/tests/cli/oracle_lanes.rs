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
        assert!(stdout.contains("--issue-sample-kind"));
        assert!(stdout.contains("--issue-sample-source"));
        assert!(stdout.contains("--fail-on-seed-drift"));
        assert!(stdout.contains("--fail-on-project-mismatch"));
        assert!(stdout.contains("--fail-on-project-seed-drift"));
        assert!(stdout.contains("--fail-on-resolved-known-difference"));
    }
}

#[test]
fn oracle_lane_help_lists_fixture_filters() {
    for lane in ["lane-a", "lane-b"] {
        let output = run_difftest(&[lane, "--help"]);

        assert!(output.status.success());
        let stdout = stdout_text(&output);
        assert!(stdout.contains("--source"));
        assert!(stdout.contains("--category"));
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

    let sample_kind = run_difftest(&["lane-a", "--issue-sample-kind", "unknown"]);
    assert!(!sample_kind.status.success());
    assert!(stderr_text(&sample_kind).contains("--issue-sample-kind expects one of"));

    let sample_source = run_difftest(&["lane-a", "--issue-sample-source", "unknown"]);
    assert!(!sample_source.status.success());
    assert!(stderr_text(&sample_source).contains("--issue-sample-source expects one of"));
}

#[test]
fn lane_a_filters_default_fixtures_by_source_and_category() {
    let temp_dir = unique_temp_dir("lane-a-filter");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    let output = run_difftest(&[
        "lane-a",
        "--oracle-command",
        "/bin/true",
        "--source",
        "curated-core",
        "--category",
        "context-namespace",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("cases seen: 14"));
    assert!(stdout.contains("cases compared: 14"));
    assert!(stdout.contains("oracle failures: 14"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_u64(&summary_json, "/counts/items_seen"), 14);
    assert_eq!(json_str(&summary_json, "/options/source"), "curated-core");
    assert_eq!(
        json_str(&summary_json, "/options/category"),
        "context-namespace"
    );
    assert_eq!(json_u64(&summary_json, "/counts/items_compared"), 14);
    assert_eq!(json_u64(&summary_json, "/counts/oracle_failures"), 14);
    let source_counts = json_array(&summary_json, "/issue_source_counts");
    assert_eq!(source_counts.len(), 1);
    assert_eq!(
        json_str(&source_counts[0], "/fixture_source"),
        "curated-core"
    );
    assert!(json_bool(&source_counts[0], "/project_owned_fixture"));
    assert_eq!(json_u64(&source_counts[0], "/total"), 14);
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn lane_a_filter_that_matches_no_fixtures_fails() {
    let output = run_difftest(&[
        "lane-a",
        "--oracle-command",
        "/bin/true",
        "--source",
        "curated-core",
        "--category",
        "not-a-category",
    ]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("lane-a filters matched no fixtures"),
        "stderr: {}",
        stderr_text(&output)
    );
}

#[test]
fn lane_b_filters_default_fixtures_by_source_and_category() {
    let temp_dir = unique_temp_dir("lane-b-filter");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    let output = run_difftest(&[
        "lane-b",
        "--oracle-command",
        "/bin/true",
        "--source",
        "stateful",
        "--category",
        "mutation-updates",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("steps seen: 54"));
    assert!(stdout.contains("steps compared: 54"));
    assert!(stdout.contains("oracle failures: 54"));
    assert!(stdout.contains("oracle session batches: 1"));
    assert!(stdout.contains("oracle session steps: 54"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_u64(&summary_json, "/counts/items_seen"), 54);
    assert_eq!(json_str(&summary_json, "/options/source"), "stateful");
    assert_eq!(
        json_str(&summary_json, "/options/category"),
        "mutation-updates"
    );
    assert_eq!(json_u64(&summary_json, "/counts/items_compared"), 54);
    assert_eq!(json_u64(&summary_json, "/counts/oracle_failures"), 54);
    assert_eq!(json_u64(&summary_json, "/counts/oracle_session_batches"), 1);
    assert_eq!(json_u64(&summary_json, "/counts/oracle_session_steps"), 54);
    let source_counts = json_array(&summary_json, "/issue_source_counts");
    assert_eq!(source_counts.len(), 1);
    assert_eq!(json_str(&source_counts[0], "/fixture_source"), "stateful");
    assert!(json_bool(&source_counts[0], "/project_owned_fixture"));
    assert_eq!(json_u64(&source_counts[0], "/total"), 54);
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn lane_b_filter_that_matches_no_fixtures_fails() {
    let output = run_difftest(&[
        "lane-b",
        "--oracle-command",
        "/bin/true",
        "--source",
        "stateful",
        "--category",
        "not-a-category",
    ]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("lane-b filters matched no fixtures"),
        "stderr: {}",
        stderr_text(&output)
    );
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

#[cfg(unix)]
#[test]
fn oracle_lanes_filter_summary_json_issue_samples_by_kind() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-issue-kind"));
        let oracle_path = temp_dir.join("fake-oracle.sh");
        let fixture_path = temp_dir.join("issue-kind.txt");
        let seed_drift_summary_path = temp_dir.join("seed-drift-summary.json");
        let non_match_summary_path = temp_dir.join("non-match-summary.json");
        fs::create_dir_all(&temp_dir).unwrap();
        write_fixed_value_batch_oracle(&oracle_path);
        fs::write(
            &fixture_path,
            "1 + 1 ||| stale-seed\n1 + 2 ||| stale-seed\n",
        )
        .unwrap();

        let seed_drift = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            seed_drift_summary_path.to_str().unwrap(),
            "--issue-sample-kind",
            "seed-drift",
        ]);

        assert!(
            seed_drift.status.success(),
            "stderr: {}",
            stderr_text(&seed_drift)
        );
        let seed_drift_summary = fs::read_to_string(&seed_drift_summary_path).unwrap();
        let seed_drift_json = parse_summary_json(&seed_drift_summary);
        assert_eq!(
            json_str(&seed_drift_json, "/options/issue_sample_kind"),
            "seed-drift"
        );
        assert_eq!(json_u64(&seed_drift_json, "/issue_samples_total"), 1);
        assert_eq!(json_u64(&seed_drift_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&seed_drift_json, "/issue_counts/non_match"), 1);
        assert_eq!(
            json_u64(&seed_drift_json, "/issue_counts/seed_snapshot_drift"),
            1
        );
        let seed_drift_source_counts = json_array(&seed_drift_json, "/issue_source_counts");
        assert_eq!(seed_drift_source_counts.len(), 1);
        assert!(json_str(&seed_drift_source_counts[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(
            &seed_drift_source_counts[0],
            "/project_owned_fixture"
        ));
        assert_eq!(json_u64(&seed_drift_source_counts[0], "/total"), 2);
        assert_eq!(json_u64(&seed_drift_source_counts[0], "/non_match"), 1);
        assert_eq!(
            json_u64(&seed_drift_source_counts[0], "/seed_snapshot_drift"),
            1
        );
        let seed_drift_samples = json_array(&seed_drift_json, "/issue_samples");
        assert_eq!(seed_drift_samples.len(), 1);
        assert_eq!(
            json_str(&seed_drift_samples[0], "/kind"),
            "seed_snapshot_drift"
        );
        assert!(json_str(&seed_drift_samples[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(&seed_drift_samples[0], "/project_owned_fixture"));

        let non_match = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            non_match_summary_path.to_str().unwrap(),
            "--issue-sample-kind",
            "non-match",
        ]);

        assert!(
            non_match.status.success(),
            "stderr: {}",
            stderr_text(&non_match)
        );
        let non_match_summary = fs::read_to_string(&non_match_summary_path).unwrap();
        let non_match_json = parse_summary_json(&non_match_summary);
        assert_eq!(
            json_str(&non_match_json, "/options/issue_sample_kind"),
            "non-match"
        );
        assert_eq!(json_u64(&non_match_json, "/issue_samples_total"), 1);
        assert_eq!(json_u64(&non_match_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&non_match_json, "/issue_counts/non_match"), 1);
        assert_eq!(
            json_u64(&non_match_json, "/issue_counts/seed_snapshot_drift"),
            1
        );
        let non_match_samples = json_array(&non_match_json, "/issue_samples");
        assert_eq!(non_match_samples.len(), 1);
        assert_eq!(json_str(&non_match_samples[0], "/kind"), "non_match");
        assert!(json_str(&non_match_samples[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(&non_match_samples[0], "/project_owned_fixture"));
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[test]
fn oracle_lanes_filter_summary_json_issue_samples_by_source() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-issue-source"));
        let fixture_path = temp_dir.join("issue-source.txt");
        let matching_summary_path = temp_dir.join("matching-summary.json");
        let empty_summary_path = temp_dir.join("empty-summary.json");
        let custom_source = if lane == "lane-a" {
            "lane-a-custom"
        } else {
            "lane-b-custom"
        };
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&fixture_path, "1 + 1 ||| 2\n1 + 2 ||| 3\n").unwrap();

        let matching = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            matching_summary_path.to_str().unwrap(),
            "--issue-sample-source",
            custom_source,
        ]);

        assert!(
            matching.status.success(),
            "stderr: {}",
            stderr_text(&matching)
        );
        let matching_summary = fs::read_to_string(&matching_summary_path).unwrap();
        let matching_json = parse_summary_json(&matching_summary);
        assert_eq!(
            json_str(&matching_json, "/options/issue_sample_source"),
            custom_source
        );
        assert_eq!(json_u64(&matching_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&matching_json, "/issue_samples_total"), 2);
        let matching_samples = json_array(&matching_json, "/issue_samples");
        assert_eq!(matching_samples.len(), 2);
        assert!(
            matching_samples
                .iter()
                .all(|sample| json_str(sample, "/fixture_source") == custom_source)
        );

        let empty = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            empty_summary_path.to_str().unwrap(),
            "--issue-sample-source",
            "seed-golden",
        ]);

        assert!(empty.status.success(), "stderr: {}", stderr_text(&empty));
        let empty_summary = fs::read_to_string(&empty_summary_path).unwrap();
        let empty_json = parse_summary_json(&empty_summary);
        assert_eq!(
            json_str(&empty_json, "/options/issue_sample_source"),
            "seed-golden"
        );
        assert_eq!(json_u64(&empty_json, "/issue_counts/total"), 2);
        assert_eq!(json_u64(&empty_json, "/issue_samples_total"), 0);
        assert!(json_array(&empty_json, "/issue_samples").is_empty());
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn oracle_lanes_can_fail_on_resolved_known_differences() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-resolved-known-diff"));
        let oracle_path = temp_dir.join("fake-oracle.sh");
        let fixture_path = temp_dir.join("resolved-known-diff.txt");
        let summary_path = temp_dir.join("summary.json");
        let failure_summary_path = temp_dir.join("failure-summary.json");
        fs::create_dir_all(&temp_dir).unwrap();
        write_fixed_value_batch_oracle(&oracle_path);
        fs::write(
            &fixture_path,
            "1 + 1 !!! id=resolved_fixture_case | type=temporary | reason=test fixture is intentionally resolved\n",
        )
        .unwrap();

        let output = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--include-known-diffs",
            "--summary-json",
            summary_path.to_str().unwrap(),
        ]);

        assert!(output.status.success(), "stderr: {}", stderr_text(&output));
        let stdout = stdout_text(&output);
        assert!(stdout.contains("resolved known differences: 1"));
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert!(json_bool(&summary_json, "/clean"));
        assert!(json_bool(
            &summary_json,
            "/options/include_known_differences"
        ));
        assert!(!json_bool(
            &summary_json,
            "/options/fail_on_resolved_known_difference"
        ));
        assert_eq!(
            json_u64(&summary_json, "/counts/resolved_known_differences"),
            1
        );
        assert_eq!(
            json_u64(&summary_json, "/issue_counts/resolved_known_difference"),
            1
        );
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(json_str(&samples[0], "/kind"), "resolved_known_difference");
        assert_eq!(
            json_str(&samples[0], "/known_difference_id"),
            "resolved_fixture_case"
        );

        let failure = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--include-known-diffs",
            "--summary-json",
            failure_summary_path.to_str().unwrap(),
            "--fail-on-resolved-known-difference",
        ]);

        assert!(
            !failure.status.success(),
            "resolved known differences should be fatal when requested"
        );
        let failure_summary = fs::read_to_string(&failure_summary_path).unwrap();
        let failure_summary_json = parse_summary_json(&failure_summary);
        assert!(json_bool(
            &failure_summary_json,
            "/options/fail_on_resolved_known_difference"
        ));
        assert!(!json_bool(
            &failure_summary_json,
            "/options/fail_on_mismatch"
        ));
        assert!(!json_bool(
            &failure_summary_json,
            "/options/fail_on_seed_drift"
        ));
        assert_eq!(
            json_u64(&failure_summary_json, "/counts/resolved_known_differences"),
            1
        );
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
        let project_failure_summary_path = temp_dir.join("project-failure-summary.json");
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
        assert!(stdout.contains("project seed snapshots drifted from oracle: 1"));
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert!(json_bool(&summary_json, "/clean"));
        assert_eq!(
            json_u64(&summary_json, "/counts/seed_snapshots_drifted_from_oracle"),
            1
        );
        assert_eq!(
            json_u64(
                &summary_json,
                "/counts/project_seed_snapshots_drifted_from_oracle"
            ),
            1
        );
        let samples = json_array(&summary_json, "/issue_samples");
        assert_eq!(json_str(&samples[0], "/kind"), "seed_snapshot_drift");
        assert!(json_str(&samples[0], "/fixture_source").ends_with("-custom"));
        assert!(json_bool(&samples[0], "/project_owned_fixture"));
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

        let project_failure = run_difftest(&[
            lane,
            "--oracle-command",
            oracle_path.to_str().unwrap(),
            "--fixture",
            fixture_path.to_str().unwrap(),
            "--summary-json",
            project_failure_summary_path.to_str().unwrap(),
            "--fail-on-project-seed-drift",
        ]);

        assert!(
            !project_failure.status.success(),
            "project seed drift should be fatal when requested"
        );
        assert!(
            stdout_text(&project_failure).contains("project seed snapshots drifted from oracle: 1")
        );
        let project_failure_summary = fs::read_to_string(&project_failure_summary_path).unwrap();
        let project_failure_summary_json = parse_summary_json(&project_failure_summary);
        assert!(json_bool(
            &project_failure_summary_json,
            "/options/fail_on_project_seed_drift"
        ));
        assert!(!json_bool(
            &project_failure_summary_json,
            "/options/fail_on_seed_drift"
        ));
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn oracle_lanes_can_fail_on_project_mismatch_only() {
    for lane in ["lane-a", "lane-b"] {
        let temp_dir = unique_temp_dir(&format!("{lane}-project-mismatch"));
        let imported_summary_path = temp_dir.join("imported-summary.json");
        let project_fixture_path = temp_dir.join("project-fixture.txt");
        let project_summary_path = temp_dir.join("project-summary.json");
        let imported_fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("seed/ham-import/golden/arithmetic.txt");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&project_fixture_path, "1 + 1 ||| 2\n").unwrap();

        let imported = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            imported_fixture_path.to_str().unwrap(),
            "--limit",
            "1",
            "--summary-json",
            imported_summary_path.to_str().unwrap(),
            "--fail-on-project-mismatch",
        ]);

        assert!(
            imported.status.success(),
            "imported seed non-match should not trigger the project mismatch gate: {}",
            stderr_text(&imported)
        );
        let imported_stdout = stdout_text(&imported);
        assert!(imported_stdout.contains("project non-matches: 0"));
        let imported_summary = fs::read_to_string(&imported_summary_path).unwrap();
        let imported_json = parse_summary_json(&imported_summary);
        assert!(json_bool(
            &imported_json,
            "/options/fail_on_project_mismatch"
        ));
        assert_eq!(json_u64(&imported_json, "/counts/project_non_matches"), 0);
        assert_eq!(
            json_str(
                &json_array(&imported_json, "/issue_source_counts")[0],
                "/fixture_source"
            ),
            "seed-golden"
        );

        let project = run_difftest(&[
            lane,
            "--oracle-command",
            "/bin/true",
            "--fixture",
            project_fixture_path.to_str().unwrap(),
            "--summary-json",
            project_summary_path.to_str().unwrap(),
            "--fail-on-project-mismatch",
        ]);

        assert!(
            !project.status.success(),
            "project-owned non-match should trigger the project mismatch gate"
        );
        let project_stdout = stdout_text(&project);
        assert!(project_stdout.contains("project non-matches: 1"));
        let project_summary = fs::read_to_string(&project_summary_path).unwrap();
        let project_json = parse_summary_json(&project_summary);
        assert!(json_bool(
            &project_json,
            "/options/fail_on_project_mismatch"
        ));
        assert_eq!(json_u64(&project_json, "/counts/project_non_matches"), 1);
        assert!(json_bool(
            &json_array(&project_json, "/issue_source_counts")[0],
            "/project_owned_fixture"
        ));
        fs::remove_dir_all(temp_dir).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn lane_a_project_seed_drift_gate_ignores_imported_seed_snapshots() {
    let temp_dir = unique_temp_dir("lane-a-imported-seed-drift");
    let oracle_path = temp_dir.join("fake-oracle.sh");
    let summary_path = temp_dir.join("summary.json");
    let fixture_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("seed/ham-import/golden/strings.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    write_fixed_value_batch_oracle(&oracle_path);

    let output = run_difftest(&[
        "lane-a",
        "--oracle-command",
        oracle_path.to_str().unwrap(),
        "--fixture",
        fixture_path.to_str().unwrap(),
        "--limit",
        "1",
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--fail-on-project-seed-drift",
    ]);

    assert!(
        output.status.success(),
        "imported seed drift should not trigger the project seed gate: {}",
        stderr_text(&output)
    );
    let stdout = stdout_text(&output);
    assert!(stdout.contains("seed snapshots drifted from oracle: 1"));
    assert!(stdout.contains("project seed snapshots drifted from oracle: 0"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(json_bool(
        &summary_json,
        "/options/fail_on_project_seed_drift"
    ));
    assert_eq!(
        json_u64(&summary_json, "/counts/seed_snapshots_drifted_from_oracle"),
        1
    );
    assert_eq!(
        json_u64(
            &summary_json,
            "/counts/project_seed_snapshots_drifted_from_oracle"
        ),
        0
    );
    let source_counts = json_array(&summary_json, "/issue_source_counts");
    assert_eq!(source_counts.len(), 1);
    assert_eq!(
        json_str(&source_counts[0], "/fixture_source"),
        "seed-golden"
    );
    assert!(!json_bool(&source_counts[0], "/project_owned_fixture"));
    assert_eq!(json_u64(&source_counts[0], "/total"), 1);
    assert_eq!(json_u64(&source_counts[0], "/non_match"), 1);
    let samples = json_array(&summary_json, "/issue_samples");
    assert_eq!(json_str(&samples[0], "/fixture_source"), "seed-golden");
    assert!(!json_bool(&samples[0], "/project_owned_fixture"));
    fs::remove_dir_all(temp_dir).unwrap();
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
