use std::fs;

use super::super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};
use super::super::write_fixed_value_batch_oracle;

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
