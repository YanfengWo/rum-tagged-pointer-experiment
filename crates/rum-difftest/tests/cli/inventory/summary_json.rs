use std::fs;

use super::super::{
    assert_json_null, json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest,
    stderr_text, stdout_text, unique_temp_dir,
};

#[test]
fn inventory_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("inventory-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "inventory",
        "--check-baseline",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "inventory");
    assert!(json_bool(&summary_json, "/clean"));
    assert!(json_bool(&summary_json, "/baseline_checked"));
    assert_json_null(&summary_json, "/filters/lane");
    assert_json_null(&summary_json, "/filters/source");
    assert_json_null(&summary_json, "/filters/category");
    assert_eq!(json_u64(&summary_json, "/totals/fixtures"), 45);
    assert_eq!(json_u64(&summary_json, "/totals/entries"), 1079);
    assert!(json_bool(&summary_json, "/baseline/clean"));
    assert_eq!(json_u64(&summary_json, "/baseline/records_considered"), 50);
    let lanes = json_array(&summary_json, "/lanes");
    let lane_a = lanes
        .iter()
        .find(|lane| json_str(lane, "/lane") == "lane-a")
        .expect("inventory summary should include lane-a");
    assert_eq!(json_u64(lane_a, "/totals/fixtures"), 38);
    assert_eq!(json_u64(lane_a, "/totals/entries"), 776);
    assert_eq!(json_u64(lane_a, "/totals/known_difference_entries"), 5);
    assert!(
        json_array(lane_a, "/sources")
            .iter()
            .any(|source| json_str(source, "/source") == "seed-golden")
    );
    assert!(
        json_array(lane_a, "/sources")
            .iter()
            .any(|source| json_str(source, "/source") == "curated-core")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "associations-core")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "calculus-symbolic")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "context-namespace")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "exact-arithmetic")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "lists-surface")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "messages-attributes")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "options-surface")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "polynomial-rational")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "scoping-closures")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "functional-combinators")
    );
    let lane_b = lanes
        .iter()
        .find(|lane| json_str(lane, "/lane") == "lane-b")
        .expect("inventory summary should include lane-b");
    assert_eq!(json_u64(lane_b, "/totals/fixtures"), 7);
    assert_eq!(json_u64(lane_b, "/totals/entries"), 303);
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "message-switches")
    );
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "mutation-updates")
    );
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "options-state")
    );
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "scoping-control")
    );
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_summary_without_baseline_reports_unchecked_clean_state() {
    let temp_dir = unique_temp_dir("inventory-json-no-baseline");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "inventory",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "inventory");
    assert!(json_bool(&summary_json, "/clean"));
    assert!(!json_bool(&summary_json, "/baseline_checked"));
    assert_json_null(&summary_json, "/baseline");
    fs::remove_dir_all(temp_dir).unwrap();
}
