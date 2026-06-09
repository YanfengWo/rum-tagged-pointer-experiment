use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stdout_text,
    unique_temp_dir,
};
use super::{issue_check_count_entry, issue_kind_count_entry};

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
    assert!(stdout.contains("checks passed: 2"));
    assert!(stdout.contains("checks failed: 1"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 1);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        4
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 4);
    assert!(json_bool(&summary_json, "/issue_samples_truncated"));
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "inventory-baseline"),
            "/count"
        ),
        4
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "inventory-baseline", "regression"),
            "/count"
        ),
        4
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert_eq!(samples.len(), 1);
    assert_eq!(json_str(&samples[0], "/check"), "inventory-baseline");
    assert_eq!(json_str(&samples[0], "/kind"), "regression");
    assert_eq!(
        json_str(&samples[0], "/location"),
        "lane-a/source/curated-core"
    );
    assert!(json_str(&samples[0], "/message").starts_with("fixtures "));
    fs::remove_dir_all(temp_dir).unwrap();
}
