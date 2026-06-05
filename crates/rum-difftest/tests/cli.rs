use std::process::{Command, Output};

fn run_difftest(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rum-difftest"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn top_level_help_lists_lane_d_readiness_gate() {
    let output = run_difftest(&["--help"]);

    assert!(output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("rum-difftest lane-d"));
    assert!(stdout.contains("--check-baseline-readiness"));
}

#[test]
fn lane_d_help_lists_readiness_gate() {
    let output = run_difftest(&["lane-d", "--help"]);

    assert!(output.status.success());
    assert!(stdout_text(&output).contains("--check-baseline-readiness"));
}

#[test]
fn lane_d_readiness_rejects_limited_runs() {
    let output = run_difftest(&["lane-d", "--check-baseline-readiness", "--limit", "1"]);

    assert!(!output.status.success());
    assert!(stderr_text(&output).contains(
        "--check-baseline-readiness expects --suite all --profile smoke without --limit"
    ));
}

#[test]
fn unknown_subcommand_reports_structured_error() {
    let output = run_difftest(&["not-a-lane"]);

    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("unknown subcommand `not-a-lane`"));
}
