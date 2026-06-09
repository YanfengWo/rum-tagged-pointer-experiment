use std::path::Path;

use rum_difftest::oracle::OracleKernel;

use crate::json_cli::{json_string, write_json_file};

pub(super) const DEFAULT_MAX_ISSUE_SAMPLES: usize = 5;

pub(super) struct OracleLaneSummaryJsonContext<'a> {
    pub(super) lane: &'a str,
    pub(super) oracle: &'a OracleKernel,
    pub(super) fixture: Option<&'a Path>,
    pub(super) limit: Option<usize>,
    pub(super) include_known_differences: bool,
    pub(super) counts: OracleLaneSummaryCounts,
    pub(super) issue_samples: OracleLaneIssueSamples,
}

#[derive(Clone, Copy)]
pub(super) struct OracleLaneSummaryCounts {
    pub(super) total_seen: usize,
    pub(super) compared: usize,
    pub(super) matches: usize,
    pub(super) mismatches: usize,
    pub(super) rum_failures: usize,
    pub(super) oracle_failures: usize,
    pub(super) skipped_known_differences: usize,
    pub(super) confirmed_known_differences: usize,
    pub(super) resolved_known_differences: usize,
    pub(super) seed_snapshots_checked: usize,
    pub(super) seed_snapshots_matched_oracle: usize,
    pub(super) seed_snapshots_drifted_from_oracle: usize,
    pub(super) oracle_session_batches: Option<usize>,
    pub(super) oracle_session_steps: Option<usize>,
}

pub(super) struct OracleLaneIssueSample {
    pub(super) kind: &'static str,
    pub(super) fixture: String,
    pub(super) index: Option<usize>,
    pub(super) input: String,
    pub(super) status: String,
    pub(super) known_difference_id: Option<String>,
    pub(super) known_difference_type: Option<String>,
    pub(super) seed_drift: bool,
}

pub(super) struct OracleLaneIssueSamples {
    pub(super) limit: usize,
    pub(super) total: usize,
    samples: Vec<OracleLaneIssueSample>,
}

impl OracleLaneIssueSamples {
    pub(super) fn new(limit: usize, mut samples: Vec<OracleLaneIssueSample>) -> Self {
        let total = samples.len();
        samples.truncate(limit);
        Self {
            limit,
            total,
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
            "  \"clean\": {},\n",
            "  \"oracle\": {{\n",
            "    \"executable\": \"{}\",\n",
            "    \"version_policy\": \"user-provided/unpinned\",\n",
            "    \"timeout_seconds\": {},\n",
            "    \"output_limit_bytes_per_stream\": {}\n",
            "  }},\n",
            "  \"options\": {{\n",
            "    \"fixture\": {},\n",
            "    \"limit\": {},\n",
            "    \"include_known_differences\": {}\n",
            "  }},\n",
            "  \"counts\": {{\n",
            "    \"items_seen\": {},\n",
            "    \"items_compared\": {},\n",
            "    \"matches\": {},\n",
            "    \"mismatches\": {},\n",
            "    \"rum_failures\": {},\n",
            "    \"oracle_failures\": {},\n",
            "    \"skipped_known_differences\": {},\n",
            "    \"confirmed_known_differences\": {},\n",
            "    \"resolved_known_differences\": {},\n",
            "    \"seed_snapshots_checked\": {},\n",
            "    \"seed_snapshots_matched_oracle\": {},\n",
            "    \"seed_snapshots_drifted_from_oracle\": {},\n",
            "    \"oracle_session_batches\": {},\n",
            "    \"oracle_session_steps\": {}\n",
            "  }},\n",
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
        clean,
        json_string(&context.oracle.executable().display().to_string()),
        context.oracle.timeout().as_secs(),
        context.oracle.output_limit_bytes(),
        json_path_option(context.fixture),
        json_usize_option(context.limit),
        context.include_known_differences,
        counts.total_seen,
        counts.compared,
        counts.matches,
        counts.mismatches,
        counts.rum_failures,
        counts.oracle_failures,
        counts.skipped_known_differences,
        counts.confirmed_known_differences,
        counts.resolved_known_differences,
        counts.seed_snapshots_checked,
        counts.seed_snapshots_matched_oracle,
        counts.seed_snapshots_drifted_from_oracle,
        json_usize_option(counts.oracle_session_batches),
        json_usize_option(counts.oracle_session_steps),
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

fn issue_samples_json(samples: &[OracleLaneIssueSample]) -> String {
    samples
        .iter()
        .map(|sample| {
            format!(
                concat!(
                    "    {{",
                    "\"kind\": \"{}\", ",
                    "\"fixture\": \"{}\", ",
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

fn json_path_option(value: Option<&Path>) -> String {
    value
        .map(|path| format!("\"{}\"", json_string(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_string())
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
