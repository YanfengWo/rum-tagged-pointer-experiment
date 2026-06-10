use std::path::PathBuf;

use super::*;

fn parse_options(args: &[&str]) -> CheckCliOptions {
    match parse_check_cli_args(args.iter().map(|arg| arg.to_string()).collect()).unwrap() {
        CheckCliArgs::Run(options) => options,
        CheckCliArgs::Help => panic!("expected runnable check options"),
    }
}

fn parse_error(args: &[&str]) -> String {
    match parse_check_cli_args(args.iter().map(|arg| arg.to_string()).collect()) {
        Ok(_) => panic!("expected check CLI parse error"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn returns_help_for_help_flags() {
    assert!(matches!(
        parse_check_cli_args(vec!["--help".to_string()]).unwrap(),
        CheckCliArgs::Help
    ));
    assert!(matches!(
        parse_check_cli_args(vec!["-h".to_string()]).unwrap(),
        CheckCliArgs::Help
    ));
}

#[test]
fn parses_basic_check_options() {
    let options = parse_options(&[
        "--inventory-baseline",
        "coverage/custom.txt",
        "--fail-on-open-known-difference",
        "--artifact-hygiene-root",
        "artifacts",
    ]);

    assert_eq!(
        options.inventory_baseline_path,
        Some(PathBuf::from("coverage/custom.txt"))
    );
    assert!(options.fail_on_open_known_difference);
    assert_eq!(
        options.artifact_hygiene_root,
        Some(PathBuf::from("artifacts"))
    );
}

#[test]
fn parses_lane_d_smoke_baseline_selection() {
    let default_baseline = parse_options(&["--include-lane-d-smoke"]);
    assert!(matches!(
        default_baseline.lane_d_smoke_baseline,
        Some(LaneDSmokeBaseline::Default)
    ));

    let explicit_after_default = parse_options(&[
        "--include-lane-d-smoke",
        "--lane-d-smoke-baseline",
        "custom-smoke.txt",
    ]);
    assert!(matches!(
        explicit_after_default.lane_d_smoke_baseline,
        Some(LaneDSmokeBaseline::Path(path)) if path == PathBuf::from("custom-smoke.txt")
    ));

    let explicit_before_include = parse_options(&[
        "--lane-d-smoke-baseline",
        "custom-smoke.txt",
        "--include-lane-d-smoke",
    ]);
    assert!(matches!(
        explicit_before_include.lane_d_smoke_baseline,
        Some(LaneDSmokeBaseline::Path(path)) if path == PathBuf::from("custom-smoke.txt")
    ));
}

#[test]
fn parses_summary_json_issue_sample_controls() {
    let options = parse_options(&[
        "--summary-json",
        "summary.json",
        "--max-issue-samples",
        "7",
        "--issue-sample-check",
        "lane-d-smoke",
        "--issue-sample-kind",
        "missing_artifact",
    ]);

    assert_eq!(
        options.summary_json_path,
        Some(PathBuf::from("summary.json"))
    );
    assert_eq!(options.max_issue_samples, 7);
    assert_eq!(
        options.issue_sample_filter.check,
        Some("lane-d-smoke".to_string())
    );
    assert_eq!(
        options.issue_sample_filter.kind,
        Some("missing_artifact".to_string())
    );
}

#[test]
fn accepts_all_issue_sample_filters() {
    let options = parse_options(&[
        "--summary-json",
        "summary.json",
        "--issue-sample-check",
        "all",
        "--issue-sample-kind",
        "all",
    ]);

    assert_eq!(options.issue_sample_filter.check, None);
    assert_eq!(options.issue_sample_filter.kind, None);
}

#[test]
fn rejects_issue_sample_controls_without_summary_json() {
    assert_eq!(
        parse_error(&["--max-issue-samples", "1"]),
        "--max-issue-samples requires --summary-json PATH"
    );
    assert_eq!(
        parse_error(&["--issue-sample-check", "inventory-baseline"]),
        "--issue-sample-check requires --summary-json PATH"
    );
    assert_eq!(
        parse_error(&["--issue-sample-kind", "regression"]),
        "--issue-sample-kind requires --summary-json PATH"
    );
}

#[test]
fn rejects_invalid_issue_sample_filters() {
    assert!(
        parse_error(&[
            "--summary-json",
            "summary.json",
            "--issue-sample-check",
            "bad"
        ])
        .contains("--issue-sample-check expects one of all")
    );
    assert!(
        parse_error(&[
            "--summary-json",
            "summary.json",
            "--issue-sample-kind",
            "bad"
        ])
        .contains("--issue-sample-kind expects one of all")
    );
}

#[test]
fn rejects_unknown_flags_and_missing_values() {
    assert_eq!(
        parse_error(&["--unknown"]),
        "unknown check flag `--unknown`"
    );
    assert_eq!(
        parse_error(&["--summary-json"]),
        "--summary-json expects a value"
    );
}
