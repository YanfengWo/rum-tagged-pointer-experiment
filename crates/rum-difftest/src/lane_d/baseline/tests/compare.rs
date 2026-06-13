use super::super::super::{LaneDOptions, LaneDSuite, run};
use super::*;

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
fn default_smoke_baseline_covers_current_smoke_lane() {
    let handle = std::thread::Builder::new()
        .name("lane-d-smoke-baseline".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(assert_default_smoke_baseline_covers_current_smoke_lane)
        .expect("lane-d smoke baseline test thread should spawn");
    handle
        .join()
        .expect("lane-d smoke baseline test thread should finish");
}

fn assert_default_smoke_baseline_covers_current_smoke_lane() {
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
