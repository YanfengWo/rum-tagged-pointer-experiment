use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "cli/check.rs"]
mod check;
#[path = "cli/oracle_lanes.rs"]
mod oracle_lanes;

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

fn parse_summary_json(summary: &str) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(summary)
        .unwrap_or_else(|err| panic!("invalid summary JSON: {err}\n{summary}"));
    assert_eq!(
        value.get("schema_version").and_then(|value| value.as_u64()),
        Some(1)
    );
    value
}

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
    assert!(stdout.contains("--check-baseline-readiness"));
    assert!(stdout.contains("--write-baseline"));
    assert!(stdout.contains("--summary-json"));
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
fn inventory_reports_default_fixture_coverage() {
    let output = run_difftest(&["inventory"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("fixture inventory"));
    assert!(stdout.contains("lane-a fixtures: 26"));
    assert!(stdout.contains("lane-a source seed-golden: fixtures=9"));
    assert!(stdout.contains("lane-a source curated-core: fixtures=17"));
    assert!(stdout.contains("lane-a category associations-core: fixtures=1"));
    assert!(stdout.contains("lane-a category functional-combinators: fixtures=1"));
    assert!(stdout.contains("lane-a category structure-selectors: fixtures=1"));
    assert!(stdout.contains("lane-a category threading: fixtures=1"));
    assert!(stdout.contains("lane-b source stateful: fixtures=3"));
    assert!(stdout.contains("lane-b category definitions-messages: fixtures=1"));
    assert!(stdout.contains("lane-b category zz-fixture-isolation: fixtures=1"));
}

#[test]
fn inventory_help_lists_baseline_gate() {
    let output = run_difftest(&["inventory", "--help"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("--baseline"));
    assert!(stdout.contains("--check-baseline"));
    assert!(stdout.contains("--fail-on-regression"));
    assert!(stdout.contains("--write-baseline"));
}

#[test]
fn inventory_checked_baseline_is_clean_for_current_fixtures() {
    let output = run_difftest(&["inventory", "--check-baseline", "--fail-on-regression"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory baseline records considered: 34"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
}

#[test]
fn inventory_fail_on_regression_exits_nonzero_for_reduced_coverage() {
    let temp_dir = unique_temp_dir("inventory-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();
    let output = run_difftest(&[
        "inventory",
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--fail-on-regression",
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory baseline regressions: 1"));
    assert!(stdout.contains("inventory-baseline-regression\tlane-a\tsource\tcurated-core"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_writes_reusable_baseline() {
    let temp_dir = unique_temp_dir("inventory-write-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let output = run_difftest(&[
        "inventory",
        "--write-baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("wrote inventory baseline:"));
    let baseline = fs::read_to_string(&baseline_path).unwrap();
    assert!(baseline.contains("lane=lane-a source=all min_fixtures=26"));
    assert!(baseline.contains("lane=lane-a category=associations-core min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=structure-selectors min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=threading min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b source=stateful min_fixtures=3"));
    assert!(baseline.contains("lane=lane-b category=zz-fixture-isolation min_fixtures=1"));

    let check = run_difftest(&[
        "inventory",
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--fail-on-regression",
    ]);
    assert!(check.status.success(), "stderr: {}", stderr_text(&check));
    assert!(stdout_text(&check).contains("inventory baseline regressions: 0"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("inventory-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "inventory",
        "--check-baseline",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    parse_summary_json(&summary);
    assert!(summary.contains("\"lane\": \"inventory\""));
    assert!(summary.contains("\"schema_version\": 1"));
    assert!(summary.contains("\"baseline\": {"));
    assert!(summary.contains("\"clean\": true"));
    assert!(summary.contains("\"records_considered\": 34"));
    assert!(summary.contains("\"lane\": \"lane-a\""));
    assert!(summary.contains("\"source\": \"seed-golden\""));
    assert!(summary.contains("\"source\": \"curated-core\""));
    assert!(summary.contains("\"categories\": ["));
    assert!(summary.contains("\"category\": \"associations-core\""));
    assert!(summary.contains("\"category\": \"functional-combinators\""));
    assert!(summary.contains("\"fixtures\": 26"));
    assert!(summary.contains("\"known_difference_entries\": 3"));
    fs::remove_dir_all(temp_dir).unwrap();
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
    parse_summary_json(&summary);
    assert!(summary.contains("\"lane\": \"lane-c\""));
    assert!(summary.contains("\"schema_version\": 1"));
    assert!(summary.contains("\"clean\": true"));
    assert!(summary.contains("\"active_ledger_entries\": 2"));
    assert!(summary.contains("\"fixture_known_differences\": 3"));
    assert!(summary.contains("\"tracking_target_issues\": 0"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn lane_d_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("lane-d-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "lane-d",
        "--suite",
        "timing",
        "--limit",
        "2",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    parse_summary_json(&summary);
    assert!(summary.contains("\"lane\": \"lane-d\""));
    assert!(summary.contains("\"schema_version\": 1"));
    assert!(summary.contains("\"suite\": \"timing\""));
    assert!(summary.contains("\"profile\": \"smoke\""));
    assert!(summary.contains("\"threshold_profile\": \"local-debug\""));
    assert!(summary.contains("\"clean\": true"));
    assert!(summary.contains("\"cases_executed\": 2"));
    assert!(summary.contains("\"failures\": 0"));
    assert!(summary.contains("\"baseline\": null"));
    assert!(summary.contains("\"checked_baseline_readiness\": null"));
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
