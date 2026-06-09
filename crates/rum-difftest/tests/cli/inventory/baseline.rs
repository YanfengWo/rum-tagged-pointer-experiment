use std::fs;

use super::super::{run_difftest, stderr_text, stdout_text, unique_temp_dir};

#[test]
fn inventory_rejects_fail_on_regression_without_baseline_before_collecting() {
    let temp_dir = unique_temp_dir("inventory-invalid-baseline-gate");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();

    let output = run_difftest(&[
        "inventory",
        "--fail-on-regression",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output)
            .contains("--fail-on-regression requires --baseline PATH or --check-baseline")
    );
    assert!(stdout_text(&output).is_empty());
    assert!(!summary_path.exists());

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_checked_baseline_is_clean_for_current_fixtures() {
    let output = run_difftest(&["inventory", "--check-baseline", "--fail-on-regression"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory baseline records considered: 50"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
}

#[test]
fn inventory_checked_baseline_respects_lane_filter() {
    let output = run_difftest(&[
        "inventory",
        "--lane",
        "lane-a",
        "--check-baseline",
        "--fail-on-regression",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory filters: lane=lane-a"));
    assert!(stdout.contains("inventory baseline records considered: 41"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
    assert!(!stdout.contains("inventory-baseline-regression\tlane-b"));
}

#[test]
fn inventory_fail_on_regression_exits_nonzero_for_reduced_coverage() {
    let temp_dir = unique_temp_dir("inventory-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();
    let output = run_difftest(&[
        "inventory",
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--fail-on-regression",
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory baseline regressions: 1"));
    assert!(stdout.contains("inventory-baseline-regression\tlane-a\tsource\tcurated-core"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_writes_reusable_baseline() {
    let temp_dir = unique_temp_dir("inventory-write-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let output = run_difftest(&[
        "inventory",
        "--write-baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("wrote inventory baseline:"));
    let baseline = fs::read_to_string(&baseline_path).unwrap();
    assert!(!baseline.contains("# Filters:"));
    assert!(baseline.contains("lane=lane-a source=all min_fixtures=38"));
    assert!(baseline.contains("lane=lane-a category=associations-core min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=calculus-symbolic min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=collection-shape-predicates min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=context-namespace min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=exact-arithmetic min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=lists-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=messages-attributes min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=options-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=ordering-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=polynomial-rational min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=predicate-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=scoping-closures min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=structure-selectors min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=threading min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=truth-comparison min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b source=stateful min_fixtures=7"));
    assert!(baseline.contains("lane=lane-b category=message-switches min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=mutation-updates min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=options-state min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=scoping-control min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=zz-fixture-isolation min_fixtures=1"));

    let check = run_difftest(&[
        "inventory",
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--fail-on-regression",
    ]);
    assert!(check.status.success(), "stderr: {}", stderr_text(&check));
    assert!(stdout_text(&check).contains("inventory baseline regressions: 0"));
    fs::remove_dir_all(temp_dir).unwrap();
}
