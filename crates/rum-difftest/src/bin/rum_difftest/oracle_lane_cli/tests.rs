use std::path::PathBuf;
use std::process::ExitCode;

use super::*;

#[test]
fn parse_oracle_lane_cli_args_defaults_common_options() {
    let parsed = parse_oracle_lane_cli_args(Vec::new(), "lane-a").unwrap();
    let OracleLaneCliArgs::Run(options) = parsed else {
        panic!("expected run options");
    };

    assert!(options.config_path.is_none());
    assert!(options.oracle_kernel.is_none());
    assert!(options.fixture.is_none());
    assert_eq!(options.max_issue_samples, DEFAULT_MAX_ISSUE_SAMPLES);
    assert_eq!(
        options.issue_sample_kind,
        OracleLaneIssueSampleKindFilter::All
    );
    assert_eq!(options.issue_sample_source, None);
    assert!(!options.include_known_differences);
    assert!(!options.fail_on_mismatch);
}

#[test]
fn parse_oracle_lane_cli_args_parses_shared_filters_and_gates() {
    let parsed = parse_oracle_lane_cli_args(
        vec![
            "--config".into(),
            "difftest.toml".into(),
            "--oracle-command".into(),
            "WolframKernel".into(),
            "--fixture".into(),
            "cases/a.wl".into(),
            "--source".into(),
            "curated-core".into(),
            "--category".into(),
            "strings".into(),
            "--limit".into(),
            "7".into(),
            "--include-known-differences".into(),
            "--oracle-timeout-seconds".into(),
            "12".into(),
            "--summary-json".into(),
            "tmp/out.json".into(),
            "--max-issue-samples".into(),
            "3".into(),
            "--issue-sample-kind".into(),
            "seed-drift".into(),
            "--issue-sample-source".into(),
            "curated-core".into(),
            "--fail-on-mismatch".into(),
            "--fail-on-project-mismatch".into(),
            "--fail-on-seed-drift".into(),
            "--fail-on-project-seed-drift".into(),
            "--fail-on-resolved-known-difference".into(),
        ],
        "lane-b",
    )
    .unwrap();
    let OracleLaneCliArgs::Run(options) = parsed else {
        panic!("expected run options");
    };

    assert_eq!(options.config_path, Some(PathBuf::from("difftest.toml")));
    assert_eq!(options.oracle_kernel, Some(PathBuf::from("WolframKernel")));
    assert_eq!(options.fixture, Some(PathBuf::from("cases/a.wl")));
    assert_eq!(options.source.as_deref(), Some("curated-core"));
    assert_eq!(options.category.as_deref(), Some("strings"));
    assert_eq!(options.limit, Some(7));
    assert!(options.include_known_differences);
    assert_eq!(options.oracle_timeout_seconds, Some(12));
    assert_eq!(
        options.summary_json_path,
        Some(PathBuf::from("tmp/out.json"))
    );
    assert_eq!(options.max_issue_samples, 3);
    assert_eq!(
        options.issue_sample_kind,
        OracleLaneIssueSampleKindFilter::SeedDrift
    );
    assert_eq!(options.issue_sample_source, Some("curated-core"));
    assert!(options.fail_on_mismatch);
    assert!(options.fail_on_project_mismatch);
    assert!(options.fail_on_seed_drift);
    assert!(options.fail_on_project_seed_drift);
    assert!(options.fail_on_resolved_known_difference);
}

#[test]
fn parse_oracle_lane_cli_args_reports_help_and_lane_specific_unknown_flags() {
    assert!(matches!(
        parse_oracle_lane_cli_args(vec!["--help".into()], "lane-a").unwrap(),
        OracleLaneCliArgs::Help
    ));

    assert_eq!(
        parse_oracle_lane_cli_args(vec!["--bogus".into()], "lane-b")
            .unwrap_err()
            .to_string(),
        "unknown lane-b flag `--bogus`"
    );
}

#[test]
fn oracle_kernel_from_cli_options_uses_explicit_kernel_and_timeout() {
    let mut options = OracleLaneCliOptions {
        oracle_kernel: Some(PathBuf::from("fake-WolframKernel")),
        oracle_timeout_seconds: Some(7),
        ..OracleLaneCliOptions::default()
    };

    let oracle = oracle_kernel_from_cli_options(&mut options).unwrap();

    assert_eq!(oracle.executable(), PathBuf::from("fake-WolframKernel"));
    assert_eq!(oracle.timeout().as_secs(), 7);
    assert!(options.oracle_kernel.is_none());
}

#[test]
fn oracle_lane_failure_exit_code_honors_shared_gate_flags() {
    let mut options = OracleLaneCliOptions::default();
    let counts = OracleLaneFailureGateCounts {
        mismatches: 1,
        project_non_matches: 1,
        seed_snapshots_drifted_from_oracle: 1,
        project_seed_snapshots_drifted_from_oracle: 1,
        resolved_known_differences: 1,
        ..OracleLaneFailureGateCounts::default()
    };

    assert_eq!(
        oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
        ExitCode::SUCCESS
    );

    options.fail_on_mismatch = true;
    assert_eq!(
        oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
        ExitCode::FAILURE
    );

    options.fail_on_mismatch = false;
    options.fail_on_project_mismatch = true;
    assert_eq!(
        oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
        ExitCode::FAILURE
    );

    options.fail_on_project_mismatch = false;
    options.fail_on_seed_drift = true;
    assert_eq!(
        oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
        ExitCode::FAILURE
    );

    options.fail_on_seed_drift = false;
    options.fail_on_project_seed_drift = true;
    assert_eq!(
        oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
        ExitCode::FAILURE
    );

    options.fail_on_project_seed_drift = false;
    options.fail_on_resolved_known_difference = true;
    assert_eq!(
        oracle_lane_failure_exit_code(options.failure_gate_options(), counts),
        ExitCode::FAILURE
    );
}
