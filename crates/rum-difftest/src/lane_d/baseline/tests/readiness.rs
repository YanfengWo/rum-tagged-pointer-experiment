use std::fs;

use super::*;

#[test]
fn checked_baseline_readiness_reports_clean_and_missing_artifacts() {
    let temp_dir = unique_temp_dir("lane-d-readiness");
    fs::create_dir_all(&temp_dir).unwrap();
    let ready_path = temp_dir.join("smoke.txt");
    let missing_path = temp_dir.join("smoke-ci-debug.txt");
    fs::write(
        &ready_path,
        "profile=smoke | threshold_profile=local-debug | suite=timing | input=fast[] | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=2\n",
    )
    .unwrap();
    let summary = test_summary(vec![test_case(1, "timing", "fast[]", 50, Some(2))]);

    let report = check_checked_baseline_readiness_for(
        &summary,
        &[
            checked_artifact(LaneDThresholdProfile::LocalDebug, &ready_path),
            checked_artifact(LaneDThresholdProfile::CiDebug, &missing_path),
        ],
    );

    assert!(!report.is_ready());
    assert!(matches!(
        report.entries[0].status,
        LaneDCheckedBaselineReadinessStatus::Ready {
            records_considered: 1,
            compared: 1
        }
    ));
    assert!(matches!(
        report.entries[1].status,
        LaneDCheckedBaselineReadinessStatus::MissingArtifact
    ));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn checked_baseline_readiness_reports_dirty_artifacts() {
    let temp_dir = unique_temp_dir("lane-d-dirty-readiness");
    fs::create_dir_all(&temp_dir).unwrap();
    let dirty_path = temp_dir.join("smoke.txt");
    fs::write(
        &dirty_path,
        "profile=smoke | threshold_profile=local-debug | suite=timing | input=fast[] | baseline_elapsed_micros=10 | max_elapsed_micros=10 | max_value_nodes=1\n",
    )
    .unwrap();
    let summary = test_summary(vec![test_case(1, "timing", "fast[]", 50, Some(2))]);

    let report = check_checked_baseline_readiness_for(
        &summary,
        &[checked_artifact(
            LaneDThresholdProfile::LocalDebug,
            &dirty_path,
        )],
    );

    assert!(!report.is_ready());
    match &report.entries[0].status {
        LaneDCheckedBaselineReadinessStatus::NotClean { report } => {
            assert_eq!(report.elapsed_regressions.len(), 1);
            assert_eq!(report.value_node_regressions.len(), 1);
        }
        other => panic!("expected dirty readiness status, got {other:?}"),
    }
    fs::remove_dir_all(temp_dir).unwrap();
}
