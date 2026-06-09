use std::collections::BTreeMap;
use std::path::Path;

use rum_difftest::oracle::OracleKernel;

use crate::json_cli::{SUMMARY_SCHEMA_VERSION, json_string, write_json_file};

pub(super) const DEFAULT_MAX_ISSUE_SAMPLES: usize = 5;

pub(super) struct OracleLaneSummaryJsonContext<'a> {
    pub(super) lane: &'a str,
    pub(super) oracle: &'a OracleKernel,
    pub(super) fixture: Option<&'a Path>,
    pub(super) limit: Option<usize>,
    pub(super) include_known_differences: bool,
    pub(super) fail_on_mismatch: bool,
    pub(super) fail_on_seed_drift: bool,
    pub(super) fail_on_project_seed_drift: bool,
    pub(super) fail_on_resolved_known_difference: bool,
    pub(super) issue_sample_kind: OracleLaneIssueSampleKindFilter,
    pub(super) session_batch_transport: Option<OracleLaneSessionBatchTransport>,
    pub(super) counts: OracleLaneSummaryCounts,
    pub(super) issue_samples: OracleLaneIssueSamples,
}

#[derive(Clone, Copy)]
pub(super) struct OracleLaneSessionBatchTransport {
    pub(super) name: &'static str,
    pub(super) process_count: Option<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct OracleLaneSummaryCounts {
    pub(super) total_seen: usize,
    pub(super) compared: usize,
    pub(super) matches: usize,
    pub(super) mismatches: usize,
    pub(super) rum_failures: usize,
    pub(super) oracle_failures: usize,
    pub(super) oracle_processes: usize,
    pub(super) oracle_batches: usize,
    pub(super) oracle_batch_inputs: usize,
    pub(super) skipped_known_differences: usize,
    pub(super) confirmed_known_differences: usize,
    pub(super) resolved_known_differences: usize,
    pub(super) seed_snapshots_checked: usize,
    pub(super) seed_snapshots_matched_oracle: usize,
    pub(super) seed_snapshots_drifted_from_oracle: usize,
    pub(super) project_seed_snapshots_drifted_from_oracle: usize,
    pub(super) oracle_session_batches: Option<usize>,
    pub(super) oracle_session_steps: Option<usize>,
}

pub(super) struct OracleLaneIssueSample {
    pub(super) kind: &'static str,
    pub(super) fixture: String,
    pub(super) fixture_source: &'static str,
    pub(super) project_owned_fixture: bool,
    pub(super) index: Option<usize>,
    pub(super) input: String,
    pub(super) status: String,
    pub(super) known_difference_id: Option<String>,
    pub(super) known_difference_type: Option<String>,
    pub(super) seed_drift: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OracleLaneIssueSampleKindFilter {
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

    fn as_json_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::NonMatch => "non-match",
            Self::SeedDrift => "seed-drift",
            Self::ResolvedKnownDifference => "resolved-known-difference",
        }
    }
}

pub(super) struct OracleLaneIssueSamples {
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

pub(super) struct OracleLaneIssueSourceCounts {
    source: &'static str,
    project_owned_fixture: bool,
    counts: OracleLaneIssueCounts,
}

impl OracleLaneIssueCounts {
    fn total(self) -> usize {
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
    pub(super) fn new(
        limit: usize,
        issue_sample_kind: OracleLaneIssueSampleKindFilter,
        samples: Vec<OracleLaneIssueSample>,
    ) -> Self {
        let counts = OracleLaneIssueCounts::from_samples(&samples);
        let source_counts = OracleLaneIssueSourceCounts::from_samples(&samples);
        let mut samples = samples
            .into_iter()
            .filter(|sample| issue_sample_kind.includes(sample.kind))
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

pub(super) fn write_summary_json(
    path: &Path,
    context: &OracleLaneSummaryJsonContext<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_file(path, summary_json(context))
}

fn summary_json(context: &OracleLaneSummaryJsonContext<'_>) -> String {
    let counts = context.counts;
    let clean = counts.mismatches == 0 && counts.rum_failures == 0 && counts.oracle_failures == 0;
    format!(
        concat!(
            "{{\n",
            "  \"lane\": \"{}\",\n",
            "  \"schema_version\": {},\n",
            "  \"clean\": {},\n",
            "  \"oracle\": {{\n",
            "    \"executable\": \"{}\",\n",
            "    \"version_policy\": \"user-provided/unpinned\",\n",
            "    \"timeout_seconds\": {},\n",
            "    \"output_limit_bytes_per_stream\": {},\n",
            "    \"session_batch_transport\": {},\n",
            "    \"session_batch_processes\": {}\n",
            "  }},\n",
            "  \"options\": {{\n",
            "    \"fixture\": {},\n",
            "    \"limit\": {},\n",
            "    \"include_known_differences\": {},\n",
            "    \"fail_on_mismatch\": {},\n",
            "    \"fail_on_seed_drift\": {},\n",
            "    \"fail_on_project_seed_drift\": {},\n",
            "    \"fail_on_resolved_known_difference\": {},\n",
            "    \"issue_sample_kind\": \"{}\"\n",
            "  }},\n",
            "  \"counts\": {{\n",
            "    \"items_seen\": {},\n",
            "    \"items_compared\": {},\n",
            "    \"matches\": {},\n",
            "    \"mismatches\": {},\n",
            "    \"rum_failures\": {},\n",
            "    \"oracle_failures\": {},\n",
            "    \"oracle_processes\": {},\n",
            "    \"oracle_batches\": {},\n",
            "    \"oracle_batch_inputs\": {},\n",
            "    \"skipped_known_differences\": {},\n",
            "    \"confirmed_known_differences\": {},\n",
            "    \"resolved_known_differences\": {},\n",
            "    \"seed_snapshots_checked\": {},\n",
            "    \"seed_snapshots_matched_oracle\": {},\n",
            "    \"seed_snapshots_drifted_from_oracle\": {},\n",
            "    \"project_seed_snapshots_drifted_from_oracle\": {},\n",
            "    \"oracle_session_batches\": {},\n",
            "    \"oracle_session_steps\": {}\n",
            "  }},\n",
            "  \"issue_counts\": {{\n",
            "    \"total\": {},\n",
            "    \"non_match\": {},\n",
            "    \"seed_snapshot_drift\": {},\n",
            "    \"resolved_known_difference\": {}\n",
            "  }},\n",
            "  \"issue_source_counts\": [\n",
            "{}\n",
            "  ],\n",
            "  \"issue_samples_limit\": {},\n",
            "  \"issue_samples_emitted\": {},\n",
            "  \"issue_samples_total\": {},\n",
            "  \"issue_samples_truncated\": {},\n",
            "  \"issue_samples\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        json_string(context.lane),
        SUMMARY_SCHEMA_VERSION,
        clean,
        json_string(&context.oracle.executable().display().to_string()),
        context.oracle.timeout().as_secs(),
        context.oracle.output_limit_bytes(),
        json_session_batch_transport_name(context.session_batch_transport),
        json_session_batch_processes(context.session_batch_transport),
        json_path_option(context.fixture),
        json_usize_option(context.limit),
        context.include_known_differences,
        context.fail_on_mismatch,
        context.fail_on_seed_drift,
        context.fail_on_project_seed_drift,
        context.fail_on_resolved_known_difference,
        json_string(context.issue_sample_kind.as_json_value()),
        counts.total_seen,
        counts.compared,
        counts.matches,
        counts.mismatches,
        counts.rum_failures,
        counts.oracle_failures,
        counts.oracle_processes,
        counts.oracle_batches,
        counts.oracle_batch_inputs,
        counts.skipped_known_differences,
        counts.confirmed_known_differences,
        counts.resolved_known_differences,
        counts.seed_snapshots_checked,
        counts.seed_snapshots_matched_oracle,
        counts.seed_snapshots_drifted_from_oracle,
        counts.project_seed_snapshots_drifted_from_oracle,
        json_usize_option(counts.oracle_session_batches),
        json_usize_option(counts.oracle_session_steps),
        context.issue_samples.counts.total(),
        context.issue_samples.counts.non_match,
        context.issue_samples.counts.seed_snapshot_drift,
        context.issue_samples.counts.resolved_known_difference,
        issue_source_counts_json(context.issue_samples.source_counts()),
        context.issue_samples.limit,
        context.issue_samples.emitted(),
        context.issue_samples.total,
        context.issue_samples.truncated(),
        issue_samples_json(context.issue_samples.samples()),
    )
}

pub(super) fn parse_max_issue_samples(value: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let max_issue_samples = value
        .parse::<usize>()
        .map_err(|_| format!("--max-issue-samples expects a positive integer, got `{value}`"))?;
    if max_issue_samples == 0 {
        return Err("--max-issue-samples expects a positive integer, got `0`".into());
    }
    Ok(max_issue_samples)
}

pub(super) fn parse_issue_sample_kind(
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

fn issue_samples_json(samples: &[OracleLaneIssueSample]) -> String {
    samples
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

fn issue_source_counts_json(source_counts: &[OracleLaneIssueSourceCounts]) -> String {
    source_counts
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

fn json_path_option(value: Option<&Path>) -> String {
    value
        .map(|path| format!("\"{}\"", json_string(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_string())
}

fn json_session_batch_transport_name(value: Option<OracleLaneSessionBatchTransport>) -> String {
    value
        .map(|transport| format!("\"{}\"", json_string(transport.name)))
        .unwrap_or_else(|| "null".to_string())
}

fn json_session_batch_processes(value: Option<OracleLaneSessionBatchTransport>) -> String {
    json_usize_option(value.and_then(|transport| transport.process_count))
}

fn json_string_option(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_string(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn json_usize_option(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}
