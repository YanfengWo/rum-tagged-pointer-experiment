use std::path::PathBuf;

use rum_difftest::artifact_hygiene::{ArtifactHygieneIssue, ArtifactHygieneIssueKind};

use super::checks::artifact_hygiene_issue_details_json;

#[test]
fn artifact_hygiene_issue_details_include_kind() {
    let details = artifact_hygiene_issue_details_json(&[
        ArtifactHygieneIssue {
            path: PathBuf::from("case-0001.wl"),
            kind: ArtifactHygieneIssueKind::OracleArtifact,
            reason: "generated oracle input file".to_string(),
        },
        ArtifactHygieneIssue {
            path: PathBuf::from("cases/stateful/probe.tmp"),
            kind: ArtifactHygieneIssueKind::StrayFixtureFile,
            reason: "unexpected fixture tree file".to_string(),
        },
    ]);

    assert!(details.contains(
        "{\"path\": \"case-0001.wl\", \"kind\": \"oracle_artifact\", \"reason\": \"generated oracle input file\"}"
    ));
    assert!(details.contains(
        "{\"path\": \"cases/stateful/probe.tmp\", \"kind\": \"stray_fixture_file\", \"reason\": \"unexpected fixture tree file\"}"
    ));
}
