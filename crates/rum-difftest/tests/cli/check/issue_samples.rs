use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, unique_temp_dir,
};
use super::{issue_check_count_entry, issue_kind_count_entry};

#[test]
fn check_filters_summary_json_issue_samples_by_check_and_kind() {
    let temp_dir = unique_temp_dir("check-filtered-issues");
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
        "--include-lane-d-readiness",
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--max-issue-samples",
        "1",
        "--issue-sample-check",
        "lane-d-readiness",
        "--issue-sample-kind",
        "missing_artifact",
    ]);

    assert!(!output.status.success());
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(
        json_str(&summary_json, "/issue_sample_check"),
        "lane-d-readiness"
    );
    assert_eq!(
        json_str(&summary_json, "/issue_sample_kind"),
        "missing_artifact"
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 1);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        6
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 2);
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
            issue_check_count_entry(&summary_json, "lane-d-readiness"),
            "/count"
        ),
        2
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "inventory-baseline", "regression"),
            "/count"
        ),
        4
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "lane-d-readiness", "missing_artifact"),
            "/count"
        ),
        2
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert_eq!(samples.len(), 1);
    assert_eq!(json_str(&samples[0], "/check"), "lane-d-readiness");
    assert_eq!(json_str(&samples[0], "/kind"), "missing_artifact");
    fs::remove_dir_all(temp_dir).unwrap();
}
