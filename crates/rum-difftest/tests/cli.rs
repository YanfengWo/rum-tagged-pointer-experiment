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

fn json_str<'a>(value: &'a serde_json::Value, pointer: &str) -> &'a str {
    value
        .pointer(pointer)
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| panic!("expected JSON string at {pointer}: {value}"))
}

fn json_u64(value: &serde_json::Value, pointer: &str) -> u64 {
    value
        .pointer(pointer)
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| panic!("expected JSON integer at {pointer}: {value}"))
}

fn json_bool(value: &serde_json::Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(|value| value.as_bool())
        .unwrap_or_else(|| panic!("expected JSON bool at {pointer}: {value}"))
}

fn json_array<'a>(value: &'a serde_json::Value, pointer: &str) -> &'a [serde_json::Value] {
    value
        .pointer(pointer)
        .and_then(|value| value.as_array())
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("expected JSON array at {pointer}: {value}"))
}

fn assert_json_null(value: &serde_json::Value, pointer: &str) {
    assert!(
        value
            .pointer(pointer)
            .is_some_and(serde_json::Value::is_null),
        "expected JSON null at {pointer}: {value}"
    );
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
fn inventory_reports_default_fixture_coverage() {
    let output = run_difftest(&["inventory"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("fixture inventory"));
    assert!(stdout.contains("lane-a fixtures: 38"));
    assert!(stdout.contains("lane-a source seed-golden: fixtures=9"));
    assert!(stdout.contains("lane-a source curated-core: fixtures=29"));
    assert!(stdout.contains("lane-a category associations-core: fixtures=1"));
    assert!(stdout.contains("lane-a category calculus-symbolic: fixtures=1"));
    assert!(stdout.contains("lane-a category collection-shape-predicates: fixtures=1"));
    assert!(stdout.contains("lane-a category context-namespace: fixtures=1"));
    assert!(stdout.contains("lane-a category exact-arithmetic: fixtures=1"));
    assert!(stdout.contains("lane-a category lists-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category messages-attributes: fixtures=1"));
    assert!(stdout.contains("lane-a category options-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category ordering-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category polynomial-rational: fixtures=1"));
    assert!(stdout.contains("lane-a category predicate-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category scoping-closures: fixtures=1"));
    assert!(stdout.contains("lane-a category functional-combinators: fixtures=1"));
    assert!(stdout.contains("lane-a category structure-selectors: fixtures=1"));
    assert!(stdout.contains("lane-a category threading: fixtures=1"));
    assert!(stdout.contains("lane-a category truth-comparison: fixtures=1"));
    assert!(stdout.contains("lane-b source stateful: fixtures=7"));
    assert!(stdout.contains("lane-b category definitions-messages: fixtures=1"));
    assert!(stdout.contains("lane-b category message-switches: fixtures=1"));
    assert!(stdout.contains("lane-b category mutation-updates: fixtures=1"));
    assert!(stdout.contains("lane-b category options-state: fixtures=1"));
    assert!(stdout.contains("lane-b category scoping-control: fixtures=1"));
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
    assert!(stdout.contains("--lane"));
    assert!(stdout.contains("--source"));
    assert!(stdout.contains("--category"));
}

#[test]
fn inventory_filters_text_json_and_written_baseline() {
    let temp_dir = unique_temp_dir("inventory-filter");
    let summary_path = temp_dir.join("summary.json");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let output = run_difftest(&[
        "inventory",
        "--lane",
        "lane-a",
        "--source",
        "curated-core",
        "--category",
        "threading",
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--check-baseline",
        "--fail-on-regression",
        "--write-baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("inventory filters: lane=lane-a source=curated-core category=threading")
    );
    assert!(stdout.contains("total fixtures: 1"));
    assert!(stdout.contains("total entries: 10"));
    assert!(stdout.contains("lane-a fixtures: 1"));
    assert!(stdout.contains("lane-a source curated-core: fixtures=1"));
    assert!(stdout.contains("lane-a category threading: fixtures=1"));
    assert!(!stdout.contains("lane-b fixtures:"));
    assert!(stdout.contains("inventory baseline records considered: 1"));
    assert!(stdout.contains("inventory baseline regressions: 0"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/filters/lane"), "lane-a");
    assert_eq!(json_str(&summary_json, "/filters/source"), "curated-core");
    assert_eq!(json_str(&summary_json, "/filters/category"), "threading");
    assert!(json_bool(&summary_json, "/baseline/clean"));
    assert_eq!(json_u64(&summary_json, "/baseline/records_considered"), 1);
    assert_eq!(json_u64(&summary_json, "/totals/fixtures"), 1);
    assert_eq!(json_u64(&summary_json, "/totals/entries"), 10);
    let lanes = json_array(&summary_json, "/lanes");
    assert_eq!(lanes.len(), 1);
    assert_eq!(json_str(&lanes[0], "/lane"), "lane-a");
    assert_eq!(json_u64(&lanes[0], "/totals/fixtures"), 1);
    assert_eq!(
        json_str(&json_array(&lanes[0], "/sources")[0], "/source"),
        "curated-core"
    );
    assert_eq!(
        json_str(&json_array(&lanes[0], "/categories")[0], "/category"),
        "threading"
    );

    let baseline = fs::read_to_string(&baseline_path).unwrap();
    assert!(baseline.contains("# Filtered non-oracle fixture coverage baseline."));
    assert!(baseline.contains("# Filters: lane=lane-a source=curated-core category=threading"));
    assert!(baseline.contains("lane=lane-a source=all min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a source=curated-core min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=threading min_fixtures=1"));
    assert!(!baseline.contains("lane=lane-b"));
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_filter_that_matches_no_fixtures_fails() {
    let output = run_difftest(&["inventory", "--category", "not-a-category"]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("inventory filters matched no fixtures"),
        "stderr: {}",
        stderr_text(&output)
    );
}

#[test]
fn inventory_filter_that_matches_no_baseline_records_fails() {
    let temp_dir = unique_temp_dir("inventory-filter-baseline");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a category=associations-core min_fixtures=1 min_entries=1 min_normal_entries=1 max_known_difference_entries=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "inventory",
        "--category",
        "threading",
        "--baseline",
        baseline_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("inventory filters matched no baseline records"),
        "stderr: {}",
        stderr_text(&output)
    );
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn inventory_checked_baseline_is_clean_for_current_fixtures() {
    let output = run_difftest(&["inventory", "--check-baseline", "--fail-on-regression"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory baseline records considered: 50"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
}

#[test]
fn inventory_checked_baseline_respects_lane_filter() {
    let output = run_difftest(&[
        "inventory",
        "--lane",
        "lane-a",
        "--check-baseline",
        "--fail-on-regression",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("inventory filters: lane=lane-a"));
    assert!(stdout.contains("inventory baseline records considered: 41"));
    assert!(stdout.contains("inventory baseline regressions: 0"));
    assert!(!stdout.contains("inventory-baseline-regression\tlane-b"));
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
    assert!(!baseline.contains("# Filters:"));
    assert!(baseline.contains("lane=lane-a source=all min_fixtures=38"));
    assert!(baseline.contains("lane=lane-a category=associations-core min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=calculus-symbolic min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=collection-shape-predicates min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=context-namespace min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=exact-arithmetic min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=lists-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=messages-attributes min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=options-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=ordering-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=polynomial-rational min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=predicate-surface min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=scoping-closures min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=structure-selectors min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=threading min_fixtures=1"));
    assert!(baseline.contains("lane=lane-a category=truth-comparison min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b source=stateful min_fixtures=7"));
    assert!(baseline.contains("lane=lane-b category=message-switches min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=mutation-updates min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=options-state min_fixtures=1"));
    assert!(baseline.contains("lane=lane-b category=scoping-control min_fixtures=1"));
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
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "inventory");
    assert_json_null(&summary_json, "/filters/lane");
    assert_json_null(&summary_json, "/filters/source");
    assert_json_null(&summary_json, "/filters/category");
    assert_eq!(json_u64(&summary_json, "/totals/fixtures"), 45);
    assert_eq!(json_u64(&summary_json, "/totals/entries"), 794);
    assert!(json_bool(&summary_json, "/baseline/clean"));
    assert_eq!(json_u64(&summary_json, "/baseline/records_considered"), 50);
    let lanes = json_array(&summary_json, "/lanes");
    let lane_a = lanes
        .iter()
        .find(|lane| json_str(lane, "/lane") == "lane-a")
        .expect("inventory summary should include lane-a");
    assert_eq!(json_u64(lane_a, "/totals/fixtures"), 38);
    assert_eq!(json_u64(lane_a, "/totals/entries"), 654);
    assert_eq!(json_u64(lane_a, "/totals/known_difference_entries"), 5);
    assert!(
        json_array(lane_a, "/sources")
            .iter()
            .any(|source| json_str(source, "/source") == "seed-golden")
    );
    assert!(
        json_array(lane_a, "/sources")
            .iter()
            .any(|source| json_str(source, "/source") == "curated-core")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "associations-core")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "calculus-symbolic")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "context-namespace")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "exact-arithmetic")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "lists-surface")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "messages-attributes")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "options-surface")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "polynomial-rational")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "scoping-closures")
    );
    assert!(
        json_array(lane_a, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "functional-combinators")
    );
    let lane_b = lanes
        .iter()
        .find(|lane| json_str(lane, "/lane") == "lane-b")
        .expect("inventory summary should include lane-b");
    assert_eq!(json_u64(lane_b, "/totals/fixtures"), 7);
    assert_eq!(json_u64(lane_b, "/totals/entries"), 140);
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "message-switches")
    );
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "mutation-updates")
    );
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "options-state")
    );
    assert!(
        json_array(lane_b, "/categories")
            .iter()
            .any(|category| json_str(category, "/category") == "scoping-control")
    );
    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn lane_c_help_and_unknown_flag_paths_are_stable() {
    let help = run_difftest(&["lane-c", "--help"]);
    assert!(help.status.success());
    assert!(stdout_text(&help).contains("rum-difftest lane-c"));
    assert!(stdout_text(&help).contains("--summary-json"));
    assert!(stdout_text(&help).contains("--fail-on-open-known-difference"));

    let unknown = run_difftest(&["lane-c", "--unknown"]);
    assert!(!unknown.status.success());
    assert!(stderr_text(&unknown).contains("unknown lane-c flag `--unknown`"));
}

#[test]
fn lane_c_writes_machine_readable_summary_json() {
    let temp_dir = unique_temp_dir("lane-c-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "lane-c",
        "--fail-on-open-known-difference",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("open known-difference gate: clean open=0"));
    assert!(stdout.contains("summary json:"));
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "lane-c");
    assert!(json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/counts/active_ledger_entries"), 4);
    assert_eq!(
        json_str(
            &json_array(&summary_json, "/counts/active_ledger_status_counts")[0],
            "/status"
        ),
        "deferred"
    );
    assert_eq!(
        json_u64(
            &json_array(&summary_json, "/counts/active_ledger_status_counts")[0],
            "/entries"
        ),
        4
    );
    let type_counts = json_array(&summary_json, "/counts/active_ledger_type_counts");
    assert_eq!(type_counts.len(), 2);
    assert_eq!(json_str(&type_counts[0], "/type"), "intentional");
    assert_eq!(json_u64(&type_counts[0], "/entries"), 1);
    assert_eq!(json_str(&type_counts[1], "/type"), "temporary");
    assert_eq!(json_u64(&type_counts[1], "/entries"), 3);
    assert_eq!(
        json_u64(&summary_json, "/counts/fixture_known_differences"),
        5
    );
    assert!(json_bool(
        &summary_json,
        "/failure_gates/fail_on_open_known_difference"
    ));
    assert_eq!(
        json_u64(&summary_json, "/failure_gates/open_known_differences"),
        0
    );
    assert_eq!(json_u64(&summary_json, "/issues/tracking_target_issues"), 0);
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
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/lane"), "lane-d");
    assert_eq!(json_str(&summary_json, "/suite"), "timing");
    assert_eq!(json_str(&summary_json, "/profile"), "smoke");
    assert_eq!(json_str(&summary_json, "/threshold_profile"), "local-debug");
    assert!(json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/counts/cases_executed"), 2);
    assert_eq!(json_u64(&summary_json, "/counts/failures"), 0);
    assert_json_null(&summary_json, "/baseline");
    assert_json_null(&summary_json, "/checked_baseline_readiness");
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
fn lane_d_readiness_summary_lists_artifacts_and_capture_commands() {
    let temp_dir = unique_temp_dir("lane-d-readiness-json");
    let summary_path = temp_dir.join("summary.json");
    let output = run_difftest(&[
        "lane-d",
        "--check-baseline-readiness",
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("checked-baseline-missing"));
    assert!(stdout.contains("--threshold-profile ci-debug"));
    assert!(stdout.contains("--write-baseline"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(!json_bool(&summary_json, "/clean"));
    let readiness = summary_json
        .pointer("/checked_baseline_readiness")
        .expect("lane-d readiness summary should be present");
    assert!(!json_bool(readiness, "/ready"));
    assert_eq!(json_u64(readiness, "/artifacts"), 3);
    assert_eq!(json_u64(readiness, "/missing_artifacts"), 2);
    let entries = json_array(readiness, "/entries");
    assert_eq!(entries.len(), 3);
    assert!(
        entries.iter().any(|entry| {
            json_str(entry, "/status") == "missing"
                && json_str(entry, "/threshold_profile") == "ci-debug"
                && json_str(entry, "/path").contains("smoke-ci-debug.txt")
                && json_str(entry, "/suggested_capture_command")
                    .contains("--threshold-profile ci-debug")
        }),
        "expected a ci-debug missing artifact entry: {entries:?}"
    );
    fs::remove_dir_all(temp_dir).unwrap();
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
