use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

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
fn oracle_lane_help_lists_timeout_control() {
    for lane in ["lane-a", "lane-b"] {
        let output = run_difftest(&[lane, "--help"]);

        assert!(output.status.success());
        assert!(stdout_text(&output).contains("--oracle-timeout-seconds"));
    }
}

#[test]
fn oracle_lane_rejects_zero_timeout() {
    let output = run_difftest(&[
        "lane-a",
        "--oracle-command",
        "/bin/true",
        "--oracle-timeout-seconds",
        "0",
        "--limit",
        "1",
    ]);

    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("expects a positive integer"));
}

#[test]
fn lane_c_help_and_unknown_flag_paths_are_stable() {
    let help = run_difftest(&["lane-c", "--help"]);
    assert!(help.status.success());
    assert!(stdout_text(&help).contains("rum-difftest lane-c"));
    assert!(stdout_text(&help).contains("--summary-json"));

    let unknown = run_difftest(&["lane-c", "--unknown"]);
    assert!(!unknown.status.success());
    assert!(stderr_text(&unknown).contains("unknown lane-c flag `--unknown`"));
}

#[test]
fn lane_c_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("lane-c-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&["lane-c", "--summary-json", summary_path.to_str().unwrap()]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    assert!(summary.contains("\"lane\": \"lane-c\""));
    assert!(summary.contains("\"clean\": true"));
    assert!(summary.contains("\"active_ledger_entries\": 2"));
    assert!(summary.contains("\"fixture_known_differences\": 3"));
    assert!(summary.contains("\"tracking_target_issues\": 0"));
    fs::remove_dir_all(temp_dir).unwrap();
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

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
