use super::*;

#[test]
fn runs_limited_timing_lane() {
    let summary = run(&LaneDOptions {
        suite: LaneDSuite::Timing,
        profile: LaneDProfile::Smoke,
        limit: Some(3),
    })
    .unwrap();

    assert_eq!(summary.executed, 3);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.cases[0].suite, "timing");
    assert_eq!(summary.cases[0].category.as_deref(), Some("trivial"));
    assert_eq!(summary.cases[0].value_nodes, Some(1));
}

#[test]
fn runs_limited_core_language_lane() {
    let summary = run(&LaneDOptions {
        suite: LaneDSuite::CoreLanguage,
        profile: LaneDProfile::Smoke,
        limit: Some(2),
    })
    .unwrap();

    assert_eq!(summary.executed, 2);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.cases[0].suite, "core-language");
    assert_eq!(summary.cases[0].category.as_deref(), Some("core-loop"));
}

#[test]
fn parses_lane_d_suite_names() {
    assert_eq!(parse_suite_name("all"), Some(LaneDSuite::All));
    assert_eq!(parse_suite_name("timing"), Some(LaneDSuite::Timing));
    assert_eq!(parse_suite_name("memory"), Some(LaneDSuite::Memory));
    assert_eq!(
        parse_suite_name("core-language"),
        Some(LaneDSuite::CoreLanguage)
    );
    assert_eq!(parse_suite_name("core"), Some(LaneDSuite::CoreLanguage));
    assert_eq!(parse_suite_name("typo"), None);
}

#[test]
fn smoke_profile_skips_heavy_inputs() {
    let summary = run(&LaneDOptions {
        suite: LaneDSuite::Timing,
        profile: LaneDProfile::Smoke,
        limit: None,
    })
    .unwrap();

    assert_eq!(summary.failed, 0);
    assert_eq!(summary.executed, 15);
    assert_eq!(summary.skipped_by_profile, 5);
    assert!(summary.cases.iter().all(|case| {
        !matches!(
            case.category.as_deref(),
            Some("heavy" | "large_list" | "very_large")
        )
    }));
}

#[test]
fn parses_lane_d_profile_names() {
    assert_eq!(parse_profile_name("smoke"), Some(LaneDProfile::Smoke));
    assert_eq!(parse_profile_name("full"), Some(LaneDProfile::Full));
    assert_eq!(parse_profile_name("typo"), None);
}

#[test]
fn parses_lane_d_threshold_profile_names() {
    assert_eq!(
        parse_threshold_profile_name("local-debug"),
        Some(LaneDThresholdProfile::LocalDebug)
    );
    assert_eq!(
        parse_threshold_profile_name("ci-debug"),
        Some(LaneDThresholdProfile::CiDebug)
    );
    assert_eq!(
        parse_threshold_profile_name("ci-release"),
        Some(LaneDThresholdProfile::CiRelease)
    );
    assert_eq!(parse_threshold_profile_name("typo"), None);
}
