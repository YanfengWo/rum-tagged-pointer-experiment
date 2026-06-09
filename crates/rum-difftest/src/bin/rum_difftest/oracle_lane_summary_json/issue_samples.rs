use std::collections::BTreeMap;

use crate::json_cli::{json_string, json_string_option, json_usize_option};

pub(crate) struct OracleLaneIssueSample {
    pub(crate) kind: &'static str,
    pub(crate) fixture: String,
    pub(crate) fixture_source: &'static str,
    pub(crate) project_owned_fixture: bool,
    pub(crate) index: Option<usize>,
    pub(crate) input: String,
    pub(crate) status: String,
    pub(crate) known_difference_id: Option<String>,
    pub(crate) known_difference_type: Option<String>,
    pub(crate) seed_drift: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OracleLaneIssueSampleKindFilter {
    All,
    NonMatch,
    SeedDrift,
    ResolvedKnownDifference,
}

impl OracleLaneIssueSampleKindFilter {
    pub(super) fn includes(self, kind: &str) -> bool {
        match self {
            Self::All => true,
            Self::NonMatch => kind == "non_match",
            Self::SeedDrift => kind == "seed_snapshot_drift",
            Self::ResolvedKnownDifference => kind == "resolved_known_difference",
        }
    }

    pub(super) fn as_json_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::NonMatch => "non-match",
            Self::SeedDrift => "seed-drift",
            Self::ResolvedKnownDifference => "resolved-known-difference",
        }
    }
}

pub(crate) struct OracleLaneIssueSamples {
    pub(super) limit: usize,
    pub(super) total: usize,
    pub(super) counts: OracleLaneIssueCounts,
    source_counts: Vec<OracleLaneIssueSourceCounts>,
    samples: Vec<OracleLaneIssueSample>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct OracleLaneIssueCounts {
    pub(super) non_match: usize,
    pub(super) seed_snapshot_drift: usize,
    pub(super) resolved_known_difference: usize,
}

struct OracleLaneIssueSourceCounts {
    source: &'static str,
    project_owned_fixture: bool,
    counts: OracleLaneIssueCounts,
}

impl OracleLaneIssueCounts {
    pub(super) fn total(self) -> usize {
        self.non_match + self.seed_snapshot_drift + self.resolved_known_difference
    }

    fn increment(&mut self, kind: &str) {
        match kind {
            "non_match" => self.non_match += 1,
            "seed_snapshot_drift" => self.seed_snapshot_drift += 1,
            "resolved_known_difference" => self.resolved_known_difference += 1,
            _ => {}
        }
    }
}

impl OracleLaneIssueSamples {
    pub(crate) fn new(
        limit: usize,
        issue_sample_kind: OracleLaneIssueSampleKindFilter,
        issue_sample_source: Option<&str>,
        samples: Vec<OracleLaneIssueSample>,
    ) -> Self {
        let counts = OracleLaneIssueCounts::from_samples(&samples);
        let source_counts = OracleLaneIssueSourceCounts::from_samples(&samples);
        let mut samples = samples
            .into_iter()
            .filter(|sample| issue_sample_kind.includes(sample.kind))
            .filter(|sample| {
                issue_sample_source.is_none_or(|source| sample.fixture_source == source)
            })
            .collect::<Vec<_>>();
        let total = samples.len();
        samples.truncate(limit);
        Self {
            limit,
            total,
            counts,
            source_counts,
            samples,
        }
    }

    pub(super) fn emitted(&self) -> usize {
        self.samples.len()
    }

    pub(super) fn truncated(&self) -> bool {
        self.emitted() < self.total
    }

    fn samples(&self) -> &[OracleLaneIssueSample] {
        &self.samples
    }

    fn source_counts(&self) -> &[OracleLaneIssueSourceCounts] {
        &self.source_counts
    }
}

impl OracleLaneIssueCounts {
    fn from_samples(samples: &[OracleLaneIssueSample]) -> Self {
        let mut counts = Self::default();
        for sample in samples {
            counts.increment(sample.kind);
        }
        counts
    }
}

impl OracleLaneIssueSourceCounts {
    fn from_samples(samples: &[OracleLaneIssueSample]) -> Vec<Self> {
        let mut by_source: BTreeMap<&'static str, (bool, OracleLaneIssueCounts)> = BTreeMap::new();
        for sample in samples {
            let (project_owned_fixture, counts) =
                by_source.entry(sample.fixture_source).or_insert((
                    sample.project_owned_fixture,
                    OracleLaneIssueCounts::default(),
                ));
            *project_owned_fixture |= sample.project_owned_fixture;
            counts.increment(sample.kind);
        }
        by_source
            .into_iter()
            .map(|(source, (project_owned_fixture, counts))| Self {
                source,
                project_owned_fixture,
                counts,
            })
            .collect()
    }
}

pub(crate) fn parse_issue_sample_kind(
    value: &str,
) -> Result<OracleLaneIssueSampleKindFilter, Box<dyn std::error::Error>> {
    match value {
        "all" => Ok(OracleLaneIssueSampleKindFilter::All),
        "non-match" | "non_match" => Ok(OracleLaneIssueSampleKindFilter::NonMatch),
        "seed-drift" | "seed_snapshot_drift" => Ok(OracleLaneIssueSampleKindFilter::SeedDrift),
        "resolved-known-difference" | "resolved_known_difference" => {
            Ok(OracleLaneIssueSampleKindFilter::ResolvedKnownDifference)
        }
        _ => Err(format!(
            "--issue-sample-kind expects one of all, non-match, seed-drift, resolved-known-difference; got `{value}`"
        )
        .into()),
    }
}

pub(crate) fn parse_issue_sample_source(
    value: &str,
) -> Result<Option<&'static str>, Box<dyn std::error::Error>> {
    match value {
        "all" => Ok(None),
        "seed-golden" => Ok(Some("seed-golden")),
        "curated-core" => Ok(Some("curated-core")),
        "lane-a-custom" => Ok(Some("lane-a-custom")),
        "stateful" => Ok(Some("stateful")),
        "lane-b-custom" => Ok(Some("lane-b-custom")),
        _ => Err(format!(
            "--issue-sample-source expects one of all, seed-golden, curated-core, lane-a-custom, stateful, lane-b-custom; got `{value}`"
        )
        .into()),
    }
}

pub(crate) fn issue_samples_json(samples: &OracleLaneIssueSamples) -> String {
    samples
        .samples()
        .iter()
        .map(|sample| {
            format!(
                concat!(
                    "    {{",
                    "\"kind\": \"{}\", ",
                    "\"fixture\": \"{}\", ",
                    "\"fixture_source\": \"{}\", ",
                    "\"project_owned_fixture\": {}, ",
                    "\"index\": {}, ",
                    "\"input\": \"{}\", ",
                    "\"status\": \"{}\", ",
                    "\"known_difference_id\": {}, ",
                    "\"known_difference_type\": {}, ",
                    "\"seed_drift\": {}",
                    "}}"
                ),
                json_string(sample.kind),
                json_string(&sample.fixture),
                json_string(sample.fixture_source),
                sample.project_owned_fixture,
                json_usize_option(sample.index),
                json_string(&sample.input),
                json_string(&sample.status),
                json_string_option(sample.known_difference_id.as_deref()),
                json_string_option(sample.known_difference_type.as_deref()),
                sample.seed_drift,
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

pub(crate) fn issue_source_counts_json(samples: &OracleLaneIssueSamples) -> String {
    samples
        .source_counts()
        .iter()
        .map(|source_counts| {
            format!(
                concat!(
                    "    {{",
                    "\"fixture_source\": \"{}\", ",
                    "\"project_owned_fixture\": {}, ",
                    "\"total\": {}, ",
                    "\"non_match\": {}, ",
                    "\"seed_snapshot_drift\": {}, ",
                    "\"resolved_known_difference\": {}",
                    "}}"
                ),
                json_string(source_counts.source),
                source_counts.project_owned_fixture,
                source_counts.counts.total(),
                source_counts.counts.non_match,
                source_counts.counts.seed_snapshot_drift,
                source_counts.counts.resolved_known_difference,
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(kind: &'static str, fixture_source: &'static str) -> OracleLaneIssueSample {
        OracleLaneIssueSample {
            kind,
            fixture: format!("{fixture_source}.txt"),
            fixture_source,
            project_owned_fixture: fixture_source != "seed-golden",
            index: Some(1),
            input: "1 + 1".to_string(),
            status: "mismatch".to_string(),
            known_difference_id: None,
            known_difference_type: None,
            seed_drift: kind == "seed_snapshot_drift",
        }
    }

    #[test]
    fn issue_sample_kind_parser_accepts_canonical_and_json_spellings() {
        assert_eq!(
            parse_issue_sample_kind("all").unwrap(),
            OracleLaneIssueSampleKindFilter::All
        );
        assert_eq!(
            parse_issue_sample_kind("non-match").unwrap(),
            OracleLaneIssueSampleKindFilter::NonMatch
        );
        assert_eq!(
            parse_issue_sample_kind("non_match").unwrap(),
            OracleLaneIssueSampleKindFilter::NonMatch
        );
        assert_eq!(
            parse_issue_sample_kind("seed-drift").unwrap(),
            OracleLaneIssueSampleKindFilter::SeedDrift
        );
        assert_eq!(
            parse_issue_sample_kind("seed_snapshot_drift").unwrap(),
            OracleLaneIssueSampleKindFilter::SeedDrift
        );
        assert_eq!(
            parse_issue_sample_kind("resolved-known-difference").unwrap(),
            OracleLaneIssueSampleKindFilter::ResolvedKnownDifference
        );
        assert_eq!(
            parse_issue_sample_kind("resolved_known_difference").unwrap(),
            OracleLaneIssueSampleKindFilter::ResolvedKnownDifference
        );
        assert!(
            parse_issue_sample_kind("unknown")
                .unwrap_err()
                .to_string()
                .contains("--issue-sample-kind expects one of")
        );
    }

    #[test]
    fn issue_sample_source_parser_accepts_known_fixture_sources() {
        assert_eq!(parse_issue_sample_source("all").unwrap(), None);
        assert_eq!(
            parse_issue_sample_source("seed-golden").unwrap(),
            Some("seed-golden")
        );
        assert_eq!(
            parse_issue_sample_source("curated-core").unwrap(),
            Some("curated-core")
        );
        assert_eq!(
            parse_issue_sample_source("lane-a-custom").unwrap(),
            Some("lane-a-custom")
        );
        assert_eq!(
            parse_issue_sample_source("stateful").unwrap(),
            Some("stateful")
        );
        assert_eq!(
            parse_issue_sample_source("lane-b-custom").unwrap(),
            Some("lane-b-custom")
        );
        assert!(
            parse_issue_sample_source("unknown")
                .unwrap_err()
                .to_string()
                .contains("--issue-sample-source expects one of")
        );
    }

    #[test]
    fn issue_samples_keep_unfiltered_counts_when_filtered_and_capped() {
        let issue_samples = OracleLaneIssueSamples::new(
            1,
            OracleLaneIssueSampleKindFilter::SeedDrift,
            Some("curated-core"),
            vec![
                sample("non_match", "curated-core"),
                sample("seed_snapshot_drift", "seed-golden"),
                sample("seed_snapshot_drift", "curated-core"),
                sample("seed_snapshot_drift", "curated-core"),
                sample("resolved_known_difference", "curated-core"),
            ],
        );

        assert_eq!(issue_samples.limit, 1);
        assert_eq!(issue_samples.total, 2);
        assert_eq!(issue_samples.emitted(), 1);
        assert!(issue_samples.truncated());
        assert_eq!(issue_samples.samples()[0].kind, "seed_snapshot_drift");
        assert_eq!(issue_samples.samples()[0].fixture_source, "curated-core");
        assert_eq!(issue_samples.counts.non_match, 1);
        assert_eq!(issue_samples.counts.seed_snapshot_drift, 3);
        assert_eq!(issue_samples.counts.resolved_known_difference, 1);

        let source_counts = issue_samples.source_counts();
        assert_eq!(source_counts.len(), 2);
        assert_eq!(source_counts[0].source, "curated-core");
        assert!(source_counts[0].project_owned_fixture);
        assert_eq!(source_counts[0].counts.total(), 4);
        assert_eq!(source_counts[0].counts.non_match, 1);
        assert_eq!(source_counts[0].counts.seed_snapshot_drift, 2);
        assert_eq!(source_counts[0].counts.resolved_known_difference, 1);
        assert_eq!(source_counts[1].source, "seed-golden");
        assert!(!source_counts[1].project_owned_fixture);
        assert_eq!(source_counts[1].counts.total(), 1);
        assert_eq!(source_counts[1].counts.seed_snapshot_drift, 1);

        assert!(
            issue_source_counts_json(&issue_samples)
                .contains("\"fixture_source\": \"curated-core\"")
        );
        assert!(issue_samples_json(&issue_samples).contains("\"kind\": \"seed_snapshot_drift\""));
    }
}
