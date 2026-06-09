use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stderr_text,
    stdout_text, unique_temp_dir,
};

#[test]
fn inventory_filters_text_json_and_written_baseline() {
    let temp_dir = unique_temp_dir("inventory-filter");
    let summary_path = temp_dir.join("summary.json");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let output = run_difftest(&[
        "inventory",
        "--lane",
        "lane-a",
        "--source",
        "curated-core",
        "--category",
        "threading",
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--check-baseline",
        "--fail-on-regression",
        "--write-baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("inventory filters: lane=lane-a source=curated-core category=threading")
    );
    assert!(stdout.contains("total fixtures: 1"));
    assert!(stdout.contains("total entries: 10"));
    assert!(stdout.contains("lane-a fixtures: 1"));
    assert!(stdout.contains("lane-a source curated-core: fixtures=1"));
    assert!(stdout.contains("lane-a category threading: fixtures=1"));
    assert!(!stdout.contains("lane-b fixtures:"));
    assert!(stdout.contains("inventory baseline records considered: 1"));
    assert!(stdout.contains("inventory baseline regressions: 0"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/filters/lane"), "lane-a");
    assert_eq!(json_str(&summary_json, "/filters/source"), "curated-core");
    assert_eq!(json_str(&summary_json, "/filters/category"), "threading");
    assert!(json_bool(&summary_json, "/baseline/clean"));
    assert_eq!(json_u64(&summary_json, "/baseline/records_considered"), 1);
    assert_eq!(json_u64(&summary_json, "/totals/fixtures"), 1);
    assert_eq!(json_u64(&summary_json, "/totals/entries"), 10);
    let lanes = json_array(&summary_json, "/lanes");
    assert_eq!(lanes.len(), 1);
    assert_eq!(json_str(&lanes[0], "/lane"), "lane-a");
    assert_eq!(json_u64(&lanes[0], "/totals/fixtures"), 1);
    assert_eq!(
        json_str(&json_array(&lanes[0], "/sources")[0], "/source"),
        "curated-core"
    );
    assert_eq!(
        json_str(&json_array(&lanes[0], "/categories")[0], "/category"),
        "threading"
    );

    let baseline = fs::read_to_string(&baseline_path).unwrap();
    assert!(baseline.contains("# Filtered non-oracle fixture coverage baseline."));
    assert!(baseline.contains("# Filters: lane=lane-a source=curated-core category=threading"));
    assert!(baseline.contains("lane=lane-a source=all min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a source=curated-core min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=threading min_fixtures=1"));
    assert!(!baseline.contains("lane=lane-b"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_filter_that_matches_no_fixtures_fails() {
    let output = run_difftest(&["inventory", "--category", "not-a-category"]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("inventory filters matched no fixtures"),
        "stderr: {}",
        stderr_text(&output)
    );
}

#[test]
fn inventory_filter_that_matches_no_baseline_records_fails() {
    let temp_dir = unique_temp_dir("inventory-filter-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a category=associations-core min_fixtures=1 min_entries=1 min_normal_entries=1 max_known_difference_entries=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "inventory",
        "--category",
        "threading",
        "--baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("inventory filters matched no baseline records"),
        "stderr: {}",
        stderr_text(&output)
    );
    fs::remove_dir_all(temp_dir).unwrap();
}
