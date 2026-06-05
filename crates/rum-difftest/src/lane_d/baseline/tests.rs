use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::{LaneDCaseStatus, LaneDOptions, LaneDSuite, run};
use super::*;

#[test]
fn parses_lane_d_baseline_records() {
    let baseline = parse_baseline(
        "profile=smoke | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=1",
    )
    .unwrap();

    assert_eq!(baseline.records.len(), 1);
    assert_eq!(baseline.records[0].profile, LaneDProfile::Smoke);
    assert_eq!(
        baseline.records[0].threshold_profile,
        LaneDThresholdProfile::LocalDebug
    );
    assert_eq!(baseline.records[0].suite, "timing");
    assert_eq!(baseline.records[0].category.as_deref(), Some("trivial"));
    assert_eq!(baseline.records[0].baseline_elapsed_micros, 10);
    assert_eq!(baseline.records[0].max_elapsed_micros, 100);
    assert_eq!(baseline.records[0].max_value_nodes, 1);
}

#[test]
fn baseline_comparison_reports_missing_and_regressions() {
    let baseline = parse_baseline(
        r#"
        profile=smoke | suite=timing | input=fast[] | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=2
        profile=smoke | suite=timing | input=stale[] | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=2
        "#,
    )
    .unwrap();
    let summary = LaneDSummary {
        total_seen: 2,
        executed: 2,
        skipped_by_profile: 0,
        succeeded: 2,
        failed: 0,
        total_elapsed_micros: 501,
        cases: vec![
            test_case(1, "timing", "fast[]", 500, Some(3)),
            test_case(2, "timing", "missing[]", 1, Some(1)),
        ],
    };

    let report = compare_baseline(
        &summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            require_all_records: true,
        },
    );

    assert!(!report.is_clean());
    assert_eq!(report.compared, 1);
    assert_eq!(report.missing_records.len(), 1);
    assert_eq!(report.unused_records.len(), 1);
    assert_eq!(report.elapsed_regressions.len(), 1);
    assert_eq!(report.value_node_regressions.len(), 1);
}

#[test]
fn captures_and_formats_baseline_from_successful_summary() {
    let summary = LaneDSummary {
        total_seen: 1,
        executed: 1,
        skipped_by_profile: 0,
        succeeded: 1,
        failed: 0,
        total_elapsed_micros: 42,
        cases: vec![test_case(1, "timing", "fast[]", 42, Some(3))],
    };

    let baseline = capture_baseline(
        &summary,
        &LaneDBaselineCaptureOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            elapsed_multiplier: 3,
            min_elapsed_micros: 100,
        },
    )
    .unwrap();
    let text = format_baseline(&baseline);
    let parsed = parse_baseline(&text).unwrap();

    assert_eq!(parsed.records.len(), 1);
    assert_eq!(parsed.records[0].baseline_elapsed_micros, 42);
    assert_eq!(parsed.records[0].max_elapsed_micros, 126);
    assert_eq!(parsed.records[0].max_value_nodes, 3);
}

#[test]
fn captures_multi_run_baseline_with_median_and_slowest_guardrail() {
    let summaries = vec![
        test_summary(vec![test_case(1, "timing", "fast[]", 10, Some(1))]),
        test_summary(vec![test_case(1, "timing", "fast[]", 30, Some(3))]),
        test_summary(vec![test_case(1, "timing", "fast[]", 20, Some(2))]),
    ];

    let baseline = capture_baseline_from_runs(
        &summaries,
        &LaneDBaselineCaptureOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            elapsed_multiplier: 2,
            min_elapsed_micros: 1,
        },
    )
    .unwrap();

    assert_eq!(baseline.records.len(), 1);
    assert_eq!(baseline.records[0].baseline_elapsed_micros, 20);
    assert_eq!(baseline.records[0].max_elapsed_micros, 60);
    assert_eq!(baseline.records[0].max_value_nodes, 3);
    assert_eq!(baseline.records[0].note.as_deref(), Some("capture_runs=3"));
}

#[test]
fn threshold_profile_filters_baseline_records() {
    let baseline = parse_baseline(
        r#"
        profile=smoke | suite=timing | input=fast[] | baseline_elapsed_micros=10 | max_elapsed_micros=10 | max_value_nodes=1
        profile=smoke | threshold_profile=ci-debug | suite=timing | input=fast[] | baseline_elapsed_micros=10 | max_elapsed_micros=100 | max_value_nodes=1
        "#,
    )
    .unwrap();
    let summary = LaneDSummary {
        total_seen: 1,
        executed: 1,
        skipped_by_profile: 0,
        succeeded: 1,
        failed: 0,
        total_elapsed_micros: 20,
        cases: vec![test_case(1, "timing", "fast[]", 20, Some(1))],
    };

    let local_report = compare_baseline(
        &summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            require_all_records: true,
        },
    );
    let ci_report = compare_baseline(
        &summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::CiDebug,
            require_all_records: true,
        },
    );

    assert_eq!(local_report.records_considered, 1);
    assert_eq!(local_report.elapsed_regressions.len(), 1);
    assert_eq!(ci_report.records_considered, 1);
    assert!(ci_report.is_clean());
}

#[test]
fn multi_run_capture_requires_matching_case_shapes() {
    let summaries = vec![
        test_summary(vec![test_case(1, "timing", "first[]", 10, Some(1))]),
        test_summary(vec![test_case(1, "timing", "second[]", 10, Some(1))]),
    ];

    let err = capture_baseline_from_runs(&summaries, &LaneDBaselineCaptureOptions::default())
        .unwrap_err();

    assert!(matches!(
        err,
        LaneDBaselineError::CaptureRunShapeMismatch {
            run_index: 2,
            case_index: 1,
            ..
        }
    ));
}

#[test]
fn refuses_to_capture_failed_runs() {
    let summary = LaneDSummary {
        total_seen: 1,
        executed: 1,
        skipped_by_profile: 0,
        succeeded: 0,
        failed: 1,
        total_elapsed_micros: 42,
        cases: vec![LaneDCaseReport {
            suite: "timing".to_owned(),
            source: PathBuf::from("test"),
            index: 1,
            category: None,
            threshold: None,
            input: "bad[]".to_owned(),
            status: LaneDCaseStatus::Failure {
                message: "bad".to_owned(),
            },
            elapsed_micros: 42,
            value_nodes: None,
            message_count: 0,
            print_count: 0,
        }],
    };

    let err = capture_baseline(&summary, &LaneDBaselineCaptureOptions::default()).unwrap_err();

    assert!(matches!(
        err,
        LaneDBaselineError::CaptureHasFailures { count: 1 }
    ));
}

#[test]
fn default_smoke_baseline_covers_current_smoke_lane() {
    let summary = run(&LaneDOptions {
        suite: LaneDSuite::All,
        profile: LaneDProfile::Smoke,
        limit: None,
    })
    .unwrap();
    let baseline = load_baseline(default_smoke_baseline_path()).unwrap();
    let report = compare_baseline(
        &summary,
        &baseline,
        &LaneDBaselineCompareOptions {
            profile: LaneDProfile::Smoke,
            threshold_profile: LaneDThresholdProfile::LocalDebug,
            require_all_records: true,
        },
    );

    assert!(
        report.is_clean(),
        "baseline comparison was not clean: {:?}",
        report
    );
}

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

fn test_case(
    index: usize,
    suite: &str,
    input: &str,
    elapsed_micros: u128,
    value_nodes: Option<usize>,
) -> LaneDCaseReport {
    LaneDCaseReport {
        suite: suite.to_owned(),
        source: PathBuf::from("test"),
        index,
        category: None,
        threshold: None,
        input: input.to_owned(),
        status: LaneDCaseStatus::Success,
        elapsed_micros,
        value_nodes,
        message_count: 0,
        print_count: 0,
    }
}

fn checked_artifact(
    threshold_profile: LaneDThresholdProfile,
    path: &Path,
) -> LaneDCheckedBaselineArtifact {
    LaneDCheckedBaselineArtifact {
        profile: LaneDProfile::Smoke,
        threshold_profile,
        path: path.to_path_buf(),
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn test_summary(cases: Vec<LaneDCaseReport>) -> LaneDSummary {
    LaneDSummary {
        total_seen: cases.len(),
        executed: cases.len(),
        skipped_by_profile: 0,
        succeeded: cases.len(),
        failed: 0,
        total_elapsed_micros: cases.iter().map(|case| case.elapsed_micros).sum(),
        cases,
    }
}
