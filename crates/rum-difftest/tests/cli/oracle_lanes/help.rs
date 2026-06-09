use super::super::{run_difftest, stderr_text, stdout_text};

#[test]
fn oracle_lane_help_lists_timeout_and_sample_controls() {
    for lane in ["lane-a", "lane-b"] {
        let output = run_difftest(&[lane, "--help"]);

        assert!(output.status.success());
        let stdout = stdout_text(&output);
        assert!(stdout.contains("--oracle-timeout-seconds"));
        assert!(stdout.contains("--summary-json"));
        assert!(stdout.contains("--max-issue-samples"));
        assert!(stdout.contains("--issue-sample-kind"));
        assert!(stdout.contains("--issue-sample-source"));
        assert!(stdout.contains("--fail-on-seed-drift"));
        assert!(stdout.contains("--fail-on-project-mismatch"));
        assert!(stdout.contains("--fail-on-project-seed-drift"));
        assert!(stdout.contains("--fail-on-resolved-known-difference"));
    }
}

#[test]
fn oracle_lane_help_lists_fixture_filters() {
    for lane in ["lane-a", "lane-b"] {
        let output = run_difftest(&[lane, "--help"]);

        assert!(output.status.success());
        let stdout = stdout_text(&output);
        assert!(stdout.contains("--source"));
        assert!(stdout.contains("--category"));
    }
}

#[test]
fn oracle_lane_rejects_invalid_numeric_controls() {
    let timeout = run_difftest(&[
        "lane-a",
        "--oracle-command",
        "/bin/true",
        "--oracle-timeout-seconds",
        "0",
        "--limit",
        "1",
    ]);

    assert!(!timeout.status.success());
    assert!(
        stderr_text(&timeout)
            .contains("--oracle-timeout-seconds expects a positive integer, got `0`")
    );

    let invalid_timeout = run_difftest(&["lane-a", "--oracle-timeout-seconds", "soon"]);
    assert!(!invalid_timeout.status.success());
    assert!(
        stderr_text(&invalid_timeout)
            .contains("--oracle-timeout-seconds expects a positive integer, got `soon`")
    );

    let zero_limit = run_difftest(&["lane-a", "--limit", "0"]);
    assert!(!zero_limit.status.success());
    assert!(stderr_text(&zero_limit).contains("--limit expects a positive integer, got `0`"));

    let invalid_limit = run_difftest(&["lane-b", "--limit", "many"]);
    assert!(!invalid_limit.status.success());
    assert!(stderr_text(&invalid_limit).contains("--limit expects a positive integer, got `many`"));

    let samples = run_difftest(&["lane-b", "--max-issue-samples", "0"]);
    assert!(!samples.status.success());
    assert!(stderr_text(&samples).contains("--max-issue-samples expects a positive integer"));

    let sample_kind = run_difftest(&["lane-a", "--issue-sample-kind", "unknown"]);
    assert!(!sample_kind.status.success());
    assert!(stderr_text(&sample_kind).contains("--issue-sample-kind expects one of"));

    let sample_source = run_difftest(&["lane-a", "--issue-sample-source", "unknown"]);
    assert!(!sample_source.status.success());
    assert!(stderr_text(&sample_source).contains("--issue-sample-source expects one of"));
}
