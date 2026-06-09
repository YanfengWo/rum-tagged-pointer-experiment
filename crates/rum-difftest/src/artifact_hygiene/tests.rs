use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
fn current_tree_has_no_oracle_artifacts() {
    let issues = validate_artifact_hygiene().unwrap();
    assert!(
        issues.is_empty(),
        "generated artifacts and stray fixture files should stay out of the crate tree: {issues:?}"
    );
}

#[test]
fn detects_oracle_wrapper_artifacts() {
    assert_eq!(
        oracle_artifact_issue(Path::new("oracle_wrapper.wls")),
        Some(issue(
            Path::new("oracle_wrapper.wls"),
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle wrapper script"
        ))
    );
    assert_eq!(
        oracle_artifact_issue(Path::new("oracle_version.wls")),
        Some(issue(
            Path::new("oracle_version.wls"),
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle wrapper script"
        ))
    );
    assert_eq!(
        oracle_artifact_issue(Path::new("oracle_session_batch_wrapper.wls")),
        Some(issue(
            Path::new("oracle_session_batch_wrapper.wls"),
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle wrapper script"
        ))
    );
}

#[test]
fn artifact_hygiene_issue_kinds_have_stable_summary_names() {
    assert_eq!(
        ArtifactHygieneIssueKind::OracleArtifact.as_str(),
        "oracle_artifact"
    );
    assert_eq!(
        ArtifactHygieneIssueKind::StrayFixtureFile.as_str(),
        "stray_fixture_file"
    );
}

#[test]
fn counts_artifact_hygiene_issue_kinds() {
    assert_eq!(
        count_artifact_hygiene_issues(&[
            issue(
                Path::new("case-0001.wl"),
                ArtifactHygieneIssueKind::OracleArtifact,
                "generated oracle input file"
            ),
            issue(
                Path::new("cases/stateful/probe-a.tmp"),
                ArtifactHygieneIssueKind::StrayFixtureFile,
                "unexpected fixture tree file"
            ),
            issue(
                Path::new("cases/stateful/probe-b.tmp"),
                ArtifactHygieneIssueKind::StrayFixtureFile,
                "unexpected fixture tree file"
            ),
        ]),
        ArtifactHygieneIssueCounts {
            oracle_artifacts: 1,
            stray_fixture_files: 2,
        }
    );
}

#[test]
fn detects_oracle_case_and_stream_artifacts() {
    assert_eq!(
        oracle_artifact_issue(Path::new("case-0001.wl")),
        Some(issue(
            Path::new("case-0001.wl"),
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle input file"
        ))
    );
    assert_eq!(
        oracle_artifact_issue(Path::new("session-0002/messages-0001.txt")),
        Some(issue(
            Path::new("session-0002/messages-0001.txt"),
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated Lane B oracle session directory"
        ))
    );
    assert_eq!(
        oracle_artifact_issue(Path::new("session-batches-0002/batch-0001/manifest.txt")),
        Some(issue(
            Path::new("session-batches-0002/batch-0001/manifest.txt"),
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated Lane B oracle session directory"
        ))
    );
    assert!(oracle_artifact_issue(Path::new("cases/stateful/package_defaults.txt")).is_none());
}

#[test]
fn detects_stray_fixture_tree_files() {
    assert_eq!(
        artifact_hygiene_issue(Path::new("cases/stateful/probe_options_messages.tmp")),
        Some(issue(
            Path::new("cases/stateful/probe_options_messages.tmp"),
            ArtifactHygieneIssueKind::StrayFixtureFile,
            "unexpected fixture tree file; keep probes outside the crate or use a supported fixture/support extension",
        ))
    );
    assert_eq!(
        artifact_hygiene_issue(Path::new("cases/wolfram/core/scratch.wl")),
        Some(issue(
            Path::new("cases/wolfram/core/scratch.wl"),
            ArtifactHygieneIssueKind::StrayFixtureFile,
            "unexpected fixture tree file; keep probes outside the crate or use a supported fixture/support extension",
        ))
    );
    assert!(artifact_hygiene_issue(Path::new("cases/stateful/packages/LaneBDiff.rum")).is_none());
    assert!(artifact_hygiene_issue(Path::new("cases/wolfram/core/threading.txt")).is_none());
}

#[test]
fn validates_artifacts_under_an_isolated_root() {
    let root = unique_temp_root("rum-difftest-artifact-hygiene");
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::create_dir_all(root.join("session-batches-0002/batch-0001")).unwrap();
    fs::create_dir_all(root.join("cases/stateful")).unwrap();
    fs::create_dir_all(root.join("cases/stateful/packages")).unwrap();
    fs::create_dir_all(root.join("cases/wolfram/core")).unwrap();
    fs::write(root.join("oracle_wrapper.wls"), "").unwrap();
    fs::write(root.join("nested/case-0001.wl"), "").unwrap();
    fs::write(
        root.join("session-batches-0002/batch-0001/manifest.txt"),
        "",
    )
    .unwrap();
    fs::write(root.join("cases/stateful/package_defaults.txt"), "").unwrap();
    fs::write(root.join("cases/stateful/packages/Pkg.rum"), "").unwrap();
    fs::write(root.join("cases/stateful/probe.tmp"), "").unwrap();
    fs::write(root.join("cases/wolfram/core/scratch.wl"), "").unwrap();

    let issues = validate_artifact_hygiene_in(&root).unwrap();
    fs::remove_dir_all(&root).unwrap();

    assert_eq!(
        issues,
        vec![
            issue(
                Path::new("cases/stateful/probe.tmp"),
                ArtifactHygieneIssueKind::StrayFixtureFile,
                "unexpected fixture tree file; keep probes outside the crate or use a supported fixture/support extension"
            ),
            issue(
                Path::new("cases/wolfram/core/scratch.wl"),
                ArtifactHygieneIssueKind::StrayFixtureFile,
                "unexpected fixture tree file; keep probes outside the crate or use a supported fixture/support extension"
            ),
            issue(
                Path::new("nested/case-0001.wl"),
                ArtifactHygieneIssueKind::OracleArtifact,
                "generated oracle input file"
            ),
            issue(
                Path::new("oracle_wrapper.wls"),
                ArtifactHygieneIssueKind::OracleArtifact,
                "generated oracle wrapper script"
            ),
            issue(
                Path::new("session-batches-0002/batch-0001/manifest.txt"),
                ArtifactHygieneIssueKind::OracleArtifact,
                "generated Lane B oracle session directory"
            ),
        ]
    );
}

fn unique_temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
