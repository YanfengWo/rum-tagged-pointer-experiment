use super::{run_difftest, stderr_text, stdout_text};

#[test]
fn top_level_help_lists_lane_d_readiness_gate() {
    let output = run_difftest(&["--help"]);

    assert!(output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("rum-difftest check"));
    assert!(stdout.contains("rum-difftest lane-d"));
    assert!(stdout.contains("rum-difftest inventory"));
    assert!(stdout.contains("--inventory-baseline"));
    assert!(stdout.contains("--include-lane-d-smoke"));
    assert!(stdout.contains("--include-lane-d-readiness"));
    assert!(stdout.contains("--artifact-hygiene-root"));
    assert!(stdout.contains("--check-baseline-readiness"));
    assert!(stdout.contains("--write-baseline"));
    assert!(stdout.contains("--summary-json"));
    assert!(stdout.contains("--lane"));
    assert!(stdout.contains("--source"));
    assert!(stdout.contains("--category"));
}

#[test]
fn lane_d_help_lists_readiness_gate() {
    let output = run_difftest(&["lane-d", "--help"]);

    assert!(output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("--check-baseline-readiness"));
    assert!(stdout.contains("--summary-json"));
}

#[test]
fn unknown_subcommand_reports_structured_error() {
    let output = run_difftest(&["not-a-lane"]);

    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("unknown subcommand `not-a-lane`"));
}
