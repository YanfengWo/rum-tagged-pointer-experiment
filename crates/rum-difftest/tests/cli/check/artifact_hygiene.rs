use std::fs;

use super::super::{
    json_array, json_bool, json_str, json_u64, parse_summary_json, run_difftest, stdout_text,
    unique_temp_dir,
};
use super::{check_entry, issue_check_count_entry, issue_kind_count_entry};

#[test]
fn check_reports_artifact_hygiene_dirty_root() {
    let temp_dir = unique_temp_dir("check-artifact-hygiene-root");
    let hygiene_root = temp_dir.join("hygiene-root");
    let stateful_dir = hygiene_root.join("cases/stateful");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&stateful_dir).unwrap();
    fs::write(hygiene_root.join("case-0001.wl"), "1 + 1").unwrap();
    fs::write(stateful_dir.join("probe.tmp"), "temporary probe").unwrap();

    let output = run_difftest(&[
        "check",
        "--artifact-hygiene-root",
        hygiene_root.to_str().unwrap(),
        "--summary-json",
        summary_path.to_str().unwrap(),
        "--issue-sample-kind",
        "stray_fixture_file",
    ]);

    assert!(!output.status.success());
    let stdout = stdout_text(&output);
    assert!(stdout.contains("check\tartifact-hygiene\tdirty"));
    assert!(stdout.contains(&format!(
        "artifact hygiene root: {}",
        hygiene_root.display()
    )));
    assert!(stdout.contains("artifact hygiene issues: 2"));
    assert!(stdout.contains("artifact hygiene oracle artifacts: 1"));
    assert!(stdout.contains("artifact hygiene stray fixture files: 1"));
    assert!(stdout.contains(
        "artifact-hygiene-issue\toracle_artifact\tcase-0001.wl\tgenerated oracle input file"
    ));
    assert!(stdout.contains(
        "artifact-hygiene-issue\tstray_fixture_file\tcases/stateful/probe.tmp\tunexpected fixture tree file"
    ));
    assert!(stdout.contains("checks passed: 2"));
    assert!(stdout.contains("checks failed: 1"));
    assert!(stdout.contains("checks skipped: 2"));

    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert!(!json_bool(&summary_json, "/clean"));
    assert_eq!(json_u64(&summary_json, "/checks_passed"), 2);
    assert_eq!(json_u64(&summary_json, "/checks_failed"), 1);
    assert_eq!(json_u64(&summary_json, "/checks_skipped"), 2);
    assert_eq!(
        json_str(&summary_json, "/issue_sample_kind"),
        "stray_fixture_file"
    );
    assert_eq!(
        json_u64(&summary_json, "/issue_samples_unfiltered_total"),
        2
    );
    assert_eq!(json_u64(&summary_json, "/issue_samples_total"), 1);
    assert_eq!(json_u64(&summary_json, "/issue_samples_emitted"), 1);
    assert!(!json_bool(&summary_json, "/issue_samples_truncated"));

    let artifact_hygiene = check_entry(&summary_json, "artifact-hygiene");
    assert!(!json_bool(artifact_hygiene, "/clean"));
    assert_eq!(
        json_str(artifact_hygiene, "/root"),
        hygiene_root.display().to_string()
    );
    assert_eq!(json_u64(artifact_hygiene, "/issues"), 2);
    assert_eq!(
        json_u64(artifact_hygiene, "/issue_kind_counts/oracle_artifact"),
        1
    );
    assert_eq!(
        json_u64(artifact_hygiene, "/issue_kind_counts/stray_fixture_file"),
        1
    );
    let details = json_array(artifact_hygiene, "/issue_details");
    assert_eq!(details.len(), 2);
    assert!(details.iter().any(|detail| {
        json_str(detail, "/kind") == "oracle_artifact"
            && json_str(detail, "/path") == "case-0001.wl"
            && json_str(detail, "/reason") == "generated oracle input file"
    }));
    assert!(details.iter().any(|detail| {
        json_str(detail, "/kind") == "stray_fixture_file"
            && json_str(detail, "/path") == "cases/stateful/probe.tmp"
            && json_str(detail, "/reason").starts_with("unexpected fixture tree file")
    }));
    assert_eq!(
        json_u64(
            issue_check_count_entry(&summary_json, "artifact-hygiene"),
            "/count"
        ),
        2
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "artifact-hygiene", "oracle_artifact"),
            "/count"
        ),
        1
    );
    assert_eq!(
        json_u64(
            issue_kind_count_entry(&summary_json, "artifact-hygiene", "stray_fixture_file"),
            "/count"
        ),
        1
    );
    let samples = json_array(&summary_json, "/issue_samples");
    assert_eq!(samples.len(), 1);
    assert_eq!(json_str(&samples[0], "/check"), "artifact-hygiene");
    assert_eq!(json_str(&samples[0], "/kind"), "stray_fixture_file");
    assert_eq!(
        json_str(&samples[0], "/location"),
        "cases/stateful/probe.tmp"
    );

    fs::remove_dir_all(temp_dir).unwrap();
}
