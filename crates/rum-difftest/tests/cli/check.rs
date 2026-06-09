use std::fs;

use super::{
    assert_json_null, json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest,
    stderr_text, stdout_text, unique_temp_dir,
};

#[test]
fn check_runs_fast_non_oracle_gates() {
    let output = run_difftest(&["check"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("rum-difftest check"));
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("inventory baseline records considered: 43"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("lane-c issues: 0"));
    assert!(stdout.contains("check\tartifact-hygiene\tclean"));
    assert!(stdout.contains("oracle artifact issues: 0"));
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

#[test]
fn check_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("check-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&["check", "--summary-json", summary_path.to_str().unwrap()]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    assert!(stdout_text(&output).contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/command"), "check");
    assert!(json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 3);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 0);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 2);
    assert_json_null(&summary_json, "/issue_sample_check");
    assert_json_null(&summary_json, "/issue_sample_kind");

    let inventory = check_entry(&summary_json, "inventory-baseline");
    assert!(json_bool(inventory, "/clean"));
    assert_eq!(json_u64(inventory, "/records_considered"), 43);
    let lane_c = check_entry(&summary_json, "lane-c-ledger");
    assert!(json_bool(lane_c, "/clean"));
    assert_eq!(json_u64(lane_c, "/counts/active_ledger_entries"), 4);
    assert!(!json_bool(
        lane_c,
        "/failure_gates/fail_on_open_known_difference"
    ));
    assert_eq!(json_u64(lane_c, "/failure_gates/open_known_differences"), 0);
    assert_eq!(
        json_str(
            &json_array(lane_c, "/counts/active_ledger_status_counts")[0],
            "/status"
        ),
        "deferred"
    );
    assert_eq!(
        json_u64(
            &json_array(lane_c, "/counts/active_ledger_status_counts")[0],
            "/entries"
        ),
        4
    );
    let type_counts = json_array(lane_c, "/counts/active_ledger_type_counts");
    assert_eq!(type_counts.len(), 2);
    assert_eq!(json_str(&type_counts[0], "/type"), "intentional");
    assert_eq!(json_u64(&type_counts[0], "/entries"), 1);
    assert_eq!(json_str(&type_counts[1], "/type"), "temporary");
    assert_eq!(json_u64(&type_counts[1], "/entries"), 3);
    assert_eq!(json_u64(lane_c, "/issues/tracking_target_issues"), 0);
    let artifact_hygiene = check_entry(&summary_json, "artifact-hygiene");
    assert!(json_bool(artifact_hygiene, "/clean"));
    assert_eq!(json_u64(artifact_hygiene, "/issues"), 0);
    assert!(json_array(artifact_hygiene, "/issue_details").is_empty());

    assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 5);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 0);
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        0
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 0);
    assert!(!json_bool(&summary_json, "/issue_samples_truncated"));
    assert!(json_array(&summary_json, "/issue_check_counts").is_empty());
    assert!(json_array(&summary_json, "/issue_kind_counts").is_empty());
    assert!(json_array(&summary_json, "/issue_samples").is_empty());
    assert_eq!(json_array(&summary_json, "/skipped_checks").len(), 2);
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-smoke"),
            "/reason"
        ),
        "opt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    );
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-readiness"),
            "/reason"
        ),
        "opt-in with --include-lane-d-readiness"
    );
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_can_fail_on_open_known_differences_when_requested() {
    let temp_dir = unique_temp_dir("check-open-known-diffs-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "check",
        "--fail-on-open-known-difference",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("open known-difference gate: clean open=0"));
    assert!(stdout.contains("checks failed: 0"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(json_bool(&summary_json, "/clean"));
    let lane_c = check_entry(&summary_json, "lane-c-ledger");
    assert!(json_bool(lane_c, "/clean"));
    assert!(json_bool(
        lane_c,
        "/failure_gates/fail_on_open_known_difference"
    ));
    assert_eq!(json_u64(lane_c, "/failure_gates/open_known_differences"), 0);
    assert!(json_array(&summary_json, "/issue_samples").is_empty());
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_can_include_lane_d_smoke_gate() {
    let output = run_difftest(&["check", "--include-lane-d-smoke"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("check\tartifact-hygiene\tclean"));
    assert!(stdout.contains("check\tlane-d-smoke\tclean"));
    assert!(
        stdout.contains("check\tlane-d-readiness\tskipped\topt-in with --include-lane-d-readiness")
    );
    assert!(stdout.contains("lane-d smoke baseline: perf/baselines/smoke.txt"));
    assert!(stdout.contains("lane-d baseline issues: 0"));
    assert!(stdout.contains("checks passed: 4"));
    assert!(stdout.contains("checks failed: 0"));
    assert!(stdout.contains("checks skipped: 1"));
}

#[test]
fn check_writes_lane_d_smoke_summary_when_included() {
    let temp_dir = unique_temp_dir("check-lane-d-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "check",
        "--include-lane-d-smoke",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 4);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 0);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 1);
    let smoke = check_entry(&summary_json, "lane-d-smoke");
    assert!(json_bool(smoke, "/clean"));
    assert_eq!(
        json_str(smoke, "/baseline/path"),
        "perf/baselines/smoke.txt"
    );
    assert_eq!(json_str(smoke, "/run_options/suite"), "all");
    assert_eq!(json_str(smoke, "/run_options/profile"), "smoke");
    assert_eq!(
        json_str(smoke, "/run_options/threshold_profile"),
        "local-debug"
    );
    assert_eq!(json_u64(smoke, "/baseline/missing_records"), 0);
    assert_eq!(json_u64(smoke, "/baseline/elapsed_regressions"), 0);
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-readiness"),
            "/reason"
        ),
        "opt-in with --include-lane-d-readiness"
    );
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_can_include_lane_d_readiness_gate() {
    let temp_dir = unique_temp_dir("check-lane-d-readiness-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "check",
        "--include-lane-d-readiness",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tclean"));
    assert!(stdout.contains("check\tlane-c-ledger\tclean"));
    assert!(stdout.contains("check\tartifact-hygiene\tclean"));
    assert!(stdout.contains("check\tlane-d-readiness\tdirty"));
    assert!(stdout.contains(
        "check\tlane-d-smoke\tskipped\topt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    ));
    assert!(stdout.contains("lane-d readiness artifacts: 3"));
    assert!(stdout.contains("lane-d readiness ready artifacts: 1"));
    assert!(stdout.contains("lane-d readiness missing artifacts: 2"));
    assert!(stdout.contains("lane-d-readiness-missing"));
    assert!(stdout.contains("--threshold-profile ci-debug"));
    assert!(stdout.contains("checks passed: 3"));
    assert!(stdout.contains("checks failed: 1"));
    assert!(stdout.contains("checks skipped: 1"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(!json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 3);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 1);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 1);
    assert_eq!(
        json_str(
            skipped_check_entry(&summary_json, "lane-d-smoke"),
            "/reason"
        ),
        "opt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH"
    );
    let readiness = check_entry(&summary_json, "lane-d-readiness");
    assert!(!json_bool(readiness, "/clean"));
    assert_eq!(json_u64(readiness, "/artifacts/total"), 3);
    assert_eq!(json_u64(readiness, "/artifacts/ready"), 1);
    assert_eq!(json_u64(readiness, "/artifacts/missing"), 2);
    assert_eq!(json_u64(readiness, "/artifacts/invalid"), 0);
    assert_eq!(json_u64(readiness, "/artifacts/not_clean"), 0);
    let artifact_entries = json_array(readiness, "/artifacts/entries");
    assert_eq!(artifact_entries.len(), 3);
    assert!(
        artifact_entries.iter().any(|entry| {
            json_str(entry, "/status") == "missing"
                && json_str(entry, "/threshold_profile") == "ci-release"
                && json_str(entry, "/path").contains("smoke-ci-release.txt")
                && json_str(entry, "/suggested_capture_command")
                    .contains("--threshold-profile ci-release")
        }),
        "expected a ci-release missing artifact entry: {artifact_entries:?}"
    );
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "lane-d-readiness"),
            "/count"
        ),
        2
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "lane-d-readiness", "missing_artifact"),
            "/count"
        ),
        2
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert!(samples.iter().any(|sample| {
        json_str(sample, "/check") == "lane-d-readiness"
            && json_str(sample, "/kind") == "missing_artifact"
            && json_str(sample, "/location").contains("smoke-ci-debug.txt")
    }));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_lane_d_smoke_baseline_regression_exits_nonzero() {
    let temp_dir = unique_temp_dir("check-lane-d-baseline");
    let baseline_path = temp_dir.join("lane-d-baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "profile=smoke | suite=timing | category=trivial | input=1 + 1 | baseline_elapsed_micros=0 | max_elapsed_micros=0 | max_value_nodes=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "check",
        "--lane-d-smoke-baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tlane-d-smoke\tdirty"));
    assert!(stdout.contains("lane-d baseline missing records:"));
    assert!(stdout.contains("lane-d baseline issues:"));
    assert!(stdout.contains("checks passed: 3"));
    assert!(stdout.contains("checks failed: 1"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_exits_nonzero_for_inventory_baseline_regression() {
    let temp_dir = unique_temp_dir("check-inventory-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "check",
        "--inventory-baseline",
        baseline_path.to_str().unwrap(),
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--max-issue-samples",
        "1",
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tinventory-baseline\tdirty"));
    assert!(stdout.contains("inventory baseline regressions: 1"));
    assert!(stdout.contains("checks passed: 2"));
    assert!(stdout.contains("checks failed: 1"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 1);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        4
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 4);
    assert!(json_bool(&summary_json, "/issue_samples_truncated"));
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "inventory-baseline"),
            "/count"
        ),
        4
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "inventory-baseline", "regression"),
            "/count"
        ),
        4
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert_eq!(samples.len(), 1);
    assert_eq!(json_str(&samples[0], "/check"), "inventory-baseline");
    assert_eq!(json_str(&samples[0], "/kind"), "regression");
    assert_eq!(
        json_str(&samples[0], "/location"),
        "lane-a/source/curated-core"
    );
    assert!(json_str(&samples[0], "/message").starts_with("fixtures "));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn check_filters_summary_json_issue_samples_by_check_and_kind() {
    let temp_dir = unique_temp_dir("check-filtered-issues");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "check",
        "--inventory-baseline",
        baseline_path.to_str().unwrap(),
        "--include-lane-d-readiness",
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--max-issue-samples",
        "1",
        "--issue-sample-check",
        "lane-d-readiness",
        "--issue-sample-kind",
        "missing_artifact",
    ]);

    assert!(!output.status.success());
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(
        json_str(&summary_json, "/issue_sample_check"),
        "lane-d-readiness"
    );
    assert_eq!(
        json_str(&summary_json, "/issue_sample_kind"),
        "missing_artifact"
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_limit"), 1);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        6
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 2);
    assert!(json_bool(&summary_json, "/issue_samples_truncated"));
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "inventory-baseline"),
            "/count"
        ),
        4
    );
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "lane-d-readiness"),
            "/count"
        ),
        2
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "inventory-baseline", "regression"),
            "/count"
        ),
        4
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "lane-d-readiness", "missing_artifact"),
            "/count"
        ),
        2
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert_eq!(samples.len(), 1);
    assert_eq!(json_str(&samples[0], "/check"), "lane-d-readiness");
    assert_eq!(json_str(&samples[0], "/kind"), "missing_artifact");
    fs::remove_dir_all(temp_dir).unwrap();
}

fn check_entry<'a>(summary: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    json_array(summary, "/checks")
        .iter()
        .find(|check| json_str(check, "/name") == name)
        .unwrap_or_else(|| panic!("missing check summary `{name}`: {summary}"))
}

fn skipped_check_entry<'a>(summary: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    json_array(summary, "/skipped_checks")
        .iter()
        .find(|check| json_str(check, "/name") == name)
        .unwrap_or_else(|| panic!("missing skipped check summary `{name}`: {summary}"))
}

fn issue_check_count_entry<'a>(
    summary: &'a serde_json::Value,
    check_name: &str,
) -> &'a serde_json::Value {
    json_array(summary, "/issue_check_counts")
        .iter()
        .find(|entry| json_str(entry, "/check") == check_name)
        .unwrap_or_else(|| panic!("missing issue check count `{check_name}`: {summary}"))
}

fn issue_kind_count_entry<'a>(
    summary: &'a serde_json::Value,
    check_name: &str,
    kind: &str,
) -> &'a serde_json::Value {
    json_array(summary, "/issue_kind_counts")
        .iter()
        .find(|entry| json_str(entry, "/check") == check_name && json_str(entry, "/kind") == kind)
        .unwrap_or_else(|| panic!("missing issue kind count `{check_name}`/`{kind}`: {summary}"))
}
