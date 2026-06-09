use super::super::{run_difftest, stderr_text, stdout_text};

#[test]
fn check_runs_fast_non_oracle_gates() {
    let output = run_difftest(&["check"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("rum-difftest check"));
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("inventory baseline records considered: 50"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("lane-c issues: 0"));
    assert!(stdout.contains("check\tartifact-hygiene\tclean"));
    assert!(stdout.contains("artifact hygiene root: ."));
    assert!(stdout.contains("artifact hygiene issues: 0"));
    assert!(stdout.contains("artifact hygiene oracle artifacts: 0"));
    assert!(stdout.contains("artifact hygiene stray fixture files: 0"));
    assert!(stdout.contains(
        "check\tlane-d-smoke\tskipped\topt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    ));
    assert!(
        stdout.contains("check\tlane-d-readiness\tskipped\topt-in with --include-lane-d-readiness")
    );
    assert!(stdout.contains("checks passed: 3"));
    assert!(stdout.contains("checks failed: 0"));
    assert!(stdout.contains("checks skipped: 2"));
}

#[test]
fn check_help_and_unknown_flag_paths_are_stable() {
    let help = run_difftest(&["check", "--help"]);
    assert!(help.status.success());
    assert!(stdout_text(&help).contains("rum-difftest check"));
    assert!(stdout_text(&help).contains("--inventory-baseline"));
    assert!(stdout_text(&help).contains("--include-lane-d-smoke"));
    assert!(stdout_text(&help).contains("--lane-d-smoke-baseline"));
    assert!(stdout_text(&help).contains("--include-lane-d-readiness"));
    assert!(stdout_text(&help).contains("--fail-on-open-known-difference"));
    assert!(stdout_text(&help).contains("--artifact-hygiene-root"));
    assert!(stdout_text(&help).contains("--summary-json"));
    assert!(stdout_text(&help).contains("--max-issue-samples"));
    assert!(stdout_text(&help).contains("--issue-sample-check"));
    assert!(stdout_text(&help).contains("--issue-sample-kind"));
    assert!(stdout_text(&help).contains("artifact hygiene"));

    let unknown = run_difftest(&["check", "--unknown"]);
    assert!(!unknown.status.success());
    assert!(stderr_text(&unknown).contains("unknown check flag `--unknown`"));

    let invalid_max = run_difftest(&["check", "--max-issue-samples", "0"]);
    assert!(!invalid_max.status.success());
    assert!(stderr_text(&invalid_max).contains("--max-issue-samples expects a positive integer"));

    let invalid_check = run_difftest(&["check", "--issue-sample-check", "lane-z"]);
    assert!(!invalid_check.status.success());
    assert!(
        stderr_text(&invalid_check).contains("--issue-sample-check expects one of all"),
        "stderr: {}",
        stderr_text(&invalid_check)
    );

    let invalid_kind = run_difftest(&["check", "--issue-sample-kind", "missing-artifact"]);
    assert!(!invalid_kind.status.success());
    assert!(
        stderr_text(&invalid_kind).contains("--issue-sample-kind expects one of all"),
        "stderr: {}",
        stderr_text(&invalid_kind)
    );
}
