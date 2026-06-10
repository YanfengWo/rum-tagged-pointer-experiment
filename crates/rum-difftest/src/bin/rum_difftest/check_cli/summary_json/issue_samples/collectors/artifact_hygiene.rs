use rum_difftest::artifact_hygiene::ArtifactHygieneIssue;

use super::super::CHECK_ARTIFACT_HYGIENE;
use super::{CheckIssueSample, LocalCheckReport};

pub(super) fn collect_artifact_hygiene_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
    collect_artifact_hygiene_issue_samples(&report.artifact_hygiene_issues, samples);
}

pub(crate) fn collect_artifact_hygiene_issue_samples(
    issues: &[ArtifactHygieneIssue],
    samples: &mut Vec<CheckIssueSample>,
) {
    for issue in issues {
        samples.push(CheckIssueSample::new(
            CHECK_ARTIFACT_HYGIENE,
            issue.kind.as_str(),
            issue.path.display().to_string(),
            issue.reason.clone(),
        ));
    }
}
