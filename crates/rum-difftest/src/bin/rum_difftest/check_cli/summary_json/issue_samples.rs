use std::collections::BTreeMap;

use crate::json_cli::json_string;

use super::{super::LocalCheckReport, CheckIssueSampleFilter};

#[path = "issue_samples/collectors.rs"]
mod collectors;

#[cfg(test)]
use collectors::collect_artifact_hygiene_issue_samples;
use collectors::collect_issue_samples;

const CHECK_INVENTORY_BASELINE: &str = "inventory-baseline";
const CHECK_LANE_C_LEDGER: &str = "lane-c-ledger";
const CHECK_ARTIFACT_HYGIENE: &str = "artifact-hygiene";
const CHECK_LANE_D_SMOKE: &str = "lane-d-smoke";
const CHECK_LANE_D_READINESS: &str = "lane-d-readiness";

pub(super) const ISSUE_SAMPLE_CHECK_VALUES: &[&str] = &[
    CHECK_INVENTORY_BASELINE,
    CHECK_LANE_C_LEDGER,
    CHECK_ARTIFACT_HYGIENE,
    CHECK_LANE_D_SMOKE,
    CHECK_LANE_D_READINESS,
];

const KIND_REGRESSION: &str = "regression";
const KIND_MISSING_LEDGER_ENTRY: &str = "missing_ledger_entry";
const KIND_UNREFERENCED_LEDGER_ENTRY: &str = "unreferenced_ledger_entry";
const KIND_UNREFERENCED_RETIRED_SEED_ENTRY: &str = "unreferenced_retired_seed_entry";
const KIND_FIXTURE_MISMATCH: &str = "fixture_mismatch";
const KIND_METADATA_MISMATCH: &str = "metadata_mismatch";
const KIND_RETIRED_SEED_FIXTURE_MISMATCH: &str = "retired_seed_fixture_mismatch";
const KIND_TRACKING_TARGET_ISSUE: &str = "tracking_target_issue";
const KIND_OPEN_KNOWN_DIFFERENCE: &str = "open_known_difference";
const KIND_ORACLE_ARTIFACT: &str = "oracle_artifact";
const KIND_STRAY_FIXTURE_FILE: &str = "stray_fixture_file";
const KIND_MISSING_RECORD: &str = "missing_record";
const KIND_UNUSED_RECORD: &str = "unused_record";
const KIND_ELAPSED_REGRESSION: &str = "elapsed_regression";
const KIND_VALUE_NODE_REGRESSION: &str = "value_node_regression";
const KIND_MISSING_ARTIFACT: &str = "missing_artifact";
const KIND_INVALID_ARTIFACT: &str = "invalid_artifact";
const KIND_NOT_CLEAN_ARTIFACT: &str = "not_clean_artifact";

pub(super) const ISSUE_SAMPLE_KIND_VALUES: &[&str] = &[
    KIND_REGRESSION,
    KIND_MISSING_LEDGER_ENTRY,
    KIND_UNREFERENCED_LEDGER_ENTRY,
    KIND_UNREFERENCED_RETIRED_SEED_ENTRY,
    KIND_FIXTURE_MISMATCH,
    KIND_METADATA_MISMATCH,
    KIND_RETIRED_SEED_FIXTURE_MISMATCH,
    KIND_TRACKING_TARGET_ISSUE,
    KIND_OPEN_KNOWN_DIFFERENCE,
    KIND_ORACLE_ARTIFACT,
    KIND_STRAY_FIXTURE_FILE,
    KIND_MISSING_RECORD,
    KIND_UNUSED_RECORD,
    KIND_ELAPSED_REGRESSION,
    KIND_VALUE_NODE_REGRESSION,
    KIND_MISSING_ARTIFACT,
    KIND_INVALID_ARTIFACT,
    KIND_NOT_CLEAN_ARTIFACT,
];

struct CheckIssueSample {
    check: &'static str,
    kind: &'static str,
    location: String,
    message: String,
}

pub(super) struct IssueSamplesJson {
    pub(super) unfiltered_total: usize,
    pub(super) total: usize,
    pub(super) emitted: usize,
    pub(super) samples: String,
}

impl IssueSamplesJson {
    pub(super) fn truncated(&self) -> bool {
        self.emitted < self.total
    }
}

pub(super) fn issue_samples_json(
    report: &LocalCheckReport,
    max_issue_samples: usize,
    filter: &CheckIssueSampleFilter,
) -> IssueSamplesJson {
    let samples = collect_issue_samples(report);
    let unfiltered_total = samples.len();
    let samples = samples
        .into_iter()
        .filter(|sample| filter_includes(filter, sample))
        .collect::<Vec<_>>();
    let total = samples.len();
    let rendered_samples = samples
        .into_iter()
        .take(max_issue_samples)
        .map(sample_json)
        .collect::<Vec<_>>();
    let emitted = rendered_samples.len();

    IssueSamplesJson {
        unfiltered_total,
        total,
        emitted,
        samples: rendered_samples.join(",\n"),
    }
}

pub(super) fn issue_kind_counts_json(report: &LocalCheckReport) -> String {
    issue_kind_counts_json_from_samples(collect_issue_samples(report))
}

fn issue_kind_counts_json_from_samples(
    samples: impl IntoIterator<Item = CheckIssueSample>,
) -> String {
    let mut counts = BTreeMap::<(&'static str, &'static str), usize>::new();
    for sample in samples {
        *counts.entry((sample.check, sample.kind)).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|((check, kind), count)| {
            format!(
                "    {{\"check\": \"{}\", \"kind\": \"{}\", \"count\": {}}}",
                json_string(check),
                json_string(kind),
                count,
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

pub(super) fn issue_check_counts_json(report: &LocalCheckReport) -> String {
    issue_check_counts_json_from_samples(collect_issue_samples(report))
}

fn issue_check_counts_json_from_samples(
    samples: impl IntoIterator<Item = CheckIssueSample>,
) -> String {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for sample in samples {
        *counts.entry(sample.check).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(check, count)| {
            format!(
                "    {{\"check\": \"{}\", \"count\": {}}}",
                json_string(check),
                count,
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn filter_includes(filter: &CheckIssueSampleFilter, sample: &CheckIssueSample) -> bool {
    filter
        .check
        .as_deref()
        .is_none_or(|check| check == sample.check)
        && filter
            .kind
            .as_deref()
            .is_none_or(|kind| kind == sample.kind)
}

fn sample_json(sample: CheckIssueSample) -> String {
    format!(
        concat!(
            "    {{",
            "\"check\": \"{}\", ",
            "\"kind\": \"{}\", ",
            "\"location\": \"{}\", ",
            "\"message\": \"{}\"",
            "}}"
        ),
        json_string(sample.check),
        json_string(sample.kind),
        json_string(&sample.location),
        json_string(&sample.message),
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rum_difftest::artifact_hygiene::{ArtifactHygieneIssue, ArtifactHygieneIssueKind};

    use super::*;

    fn hygiene_issue(
        path: &str,
        kind: ArtifactHygieneIssueKind,
        reason: &str,
    ) -> ArtifactHygieneIssue {
        ArtifactHygieneIssue {
            path: PathBuf::from(path),
            kind,
            reason: reason.to_string(),
        }
    }

    fn sample(kind: &'static str, location: &str) -> CheckIssueSample {
        CheckIssueSample {
            check: CHECK_ARTIFACT_HYGIENE,
            kind,
            location: location.to_string(),
            message: format!("{kind} issue"),
        }
    }

    #[test]
    fn artifact_hygiene_samples_distinguish_issue_kinds() {
        let issues = vec![
            hygiene_issue(
                "case-0001.wl",
                ArtifactHygieneIssueKind::OracleArtifact,
                "generated oracle input file",
            ),
            hygiene_issue(
                "cases/stateful/probe.tmp",
                ArtifactHygieneIssueKind::StrayFixtureFile,
                "unexpected fixture tree file",
            ),
        ];

        let mut samples = Vec::new();
        collect_artifact_hygiene_issue_samples(&issues, &mut samples);

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].check, CHECK_ARTIFACT_HYGIENE);
        assert_eq!(samples[0].kind, KIND_ORACLE_ARTIFACT);
        assert_eq!(samples[0].location, issues[0].path.display().to_string());
        assert_eq!(samples[0].message, "generated oracle input file");
        assert_eq!(samples[1].check, CHECK_ARTIFACT_HYGIENE);
        assert_eq!(samples[1].kind, KIND_STRAY_FIXTURE_FILE);
        assert_eq!(samples[1].location, issues[1].path.display().to_string());
        assert_eq!(samples[1].message, "unexpected fixture tree file");
    }

    #[test]
    fn artifact_hygiene_stray_samples_can_be_filtered_and_rendered() {
        let filter = CheckIssueSampleFilter {
            check: Some(CHECK_ARTIFACT_HYGIENE.to_string()),
            kind: Some(KIND_STRAY_FIXTURE_FILE.to_string()),
        };
        let stray = sample(KIND_STRAY_FIXTURE_FILE, "cases/stateful/probe.tmp");
        let oracle = sample(KIND_ORACLE_ARTIFACT, "case-0001.wl");

        assert!(filter_includes(&filter, &stray));
        assert!(!filter_includes(&filter, &oracle));

        let rendered = sample_json(stray);
        assert!(rendered.contains("\"check\": \"artifact-hygiene\""));
        assert!(rendered.contains("\"kind\": \"stray_fixture_file\""));
        assert!(rendered.contains("\"location\": \"cases/stateful/probe.tmp\""));
    }

    #[test]
    fn artifact_hygiene_counts_include_stray_fixture_files() {
        let kind_counts = issue_kind_counts_json_from_samples(vec![
            sample(KIND_STRAY_FIXTURE_FILE, "cases/stateful/probe-a.tmp"),
            sample(KIND_ORACLE_ARTIFACT, "case-0001.wl"),
            sample(KIND_STRAY_FIXTURE_FILE, "cases/stateful/probe-b.tmp"),
        ]);
        assert!(kind_counts.contains(
            "{\"check\": \"artifact-hygiene\", \"kind\": \"oracle_artifact\", \"count\": 1}"
        ));
        assert!(kind_counts.contains(
            "{\"check\": \"artifact-hygiene\", \"kind\": \"stray_fixture_file\", \"count\": 2}"
        ));

        let check_counts = issue_check_counts_json_from_samples(vec![
            sample(KIND_STRAY_FIXTURE_FILE, "cases/stateful/probe-a.tmp"),
            sample(KIND_ORACLE_ARTIFACT, "case-0001.wl"),
            sample(KIND_STRAY_FIXTURE_FILE, "cases/stateful/probe-b.tmp"),
        ]);
        assert!(check_counts.contains("{\"check\": \"artifact-hygiene\", \"count\": 3}"));
    }
}
