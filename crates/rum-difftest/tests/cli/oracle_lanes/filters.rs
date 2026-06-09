use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};

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
