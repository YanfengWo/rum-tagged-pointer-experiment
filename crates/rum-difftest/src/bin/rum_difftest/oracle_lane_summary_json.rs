use std::path::Path;

use rum_difftest::oracle::OracleKernel;

use crate::json_cli::{
    SUMMARY_SCHEMA_VERSION, json_path_option, json_string, json_string_option, json_usize_option,
    write_json_file,
};

pub(super) use crate::json_cli::parse_max_issue_samples;

pub(super) const DEFAULT_MAX_ISSUE_SAMPLES: usize = 5;

#[path = "oracle_lane_summary_json/counts.rs"]
mod counts;
#[path = "oracle_lane_summary_json/issue_samples.rs"]
mod issue_samples;

pub(super) use issue_samples::{
    OracleLaneIssueSample, OracleLaneIssueSampleKindFilter, OracleLaneIssueSamples,
    issue_samples_json, issue_source_counts_json, parse_issue_sample_kind,
    parse_issue_sample_source,
};

pub(super) struct OracleLaneSummaryJsonContext<'a> {
    pub(super) lane: &'a str,
    pub(super) oracle: &'a OracleKernel,
    pub(super) fixture: Option<&'a Path>,
    pub(super) source: Option<&'a str>,
    pub(super) category: Option<&'a str>,
    pub(super) limit: Option<usize>,
    pub(super) include_known_differences: bool,
    pub(super) fail_on_mismatch: bool,
    pub(super) fail_on_project_mismatch: bool,
    pub(super) fail_on_seed_drift: bool,
    pub(super) fail_on_project_seed_drift: bool,
    pub(super) fail_on_resolved_known_difference: bool,
    pub(super) issue_sample_kind: OracleLaneIssueSampleKindFilter,
    pub(super) issue_sample_source: Option<&'a str>,
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
pub(super) struct OracleLaneTransportCounts {
    pub(super) oracle_processes: usize,
    pub(super) oracle_batches: usize,
    pub(super) oracle_batch_inputs: usize,
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
    pub(super) project_non_matches: usize,
    pub(super) seed_snapshots_checked: usize,
    pub(super) seed_snapshots_matched_oracle: usize,
    pub(super) seed_snapshots_drifted_from_oracle: usize,
    pub(super) project_seed_snapshots_drifted_from_oracle: usize,
    pub(super) oracle_session_batches: Option<usize>,
    pub(super) oracle_session_steps: Option<usize>,
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
            "    \"source\": {},\n",
            "    \"category\": {},\n",
            "    \"limit\": {},\n",
            "    \"include_known_differences\": {},\n",
            "    \"fail_on_mismatch\": {},\n",
            "    \"fail_on_project_mismatch\": {},\n",
            "    \"fail_on_seed_drift\": {},\n",
            "    \"fail_on_project_seed_drift\": {},\n",
            "    \"fail_on_resolved_known_difference\": {},\n",
            "    \"issue_sample_kind\": \"{}\",\n",
            "    \"issue_sample_source\": {}\n",
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
            "    \"project_non_matches\": {},\n",
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
        json_string_option(context.source),
        json_string_option(context.category),
        json_usize_option(context.limit),
        context.include_known_differences,
        context.fail_on_mismatch,
        context.fail_on_project_mismatch,
        context.fail_on_seed_drift,
        context.fail_on_project_seed_drift,
        context.fail_on_resolved_known_difference,
        json_string(context.issue_sample_kind.as_json_value()),
        json_string_option(context.issue_sample_source),
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
        counts.project_non_matches,
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
        issue_source_counts_json(&context.issue_samples),
        context.issue_samples.limit,
        context.issue_samples.emitted(),
        context.issue_samples.total,
        context.issue_samples.truncated(),
        issue_samples_json(&context.issue_samples),
    )
}

fn json_session_batch_transport_name(value: Option<OracleLaneSessionBatchTransport>) -> String {
    json_string_option(value.map(|transport| transport.name))
}

fn json_session_batch_processes(value: Option<OracleLaneSessionBatchTransport>) -> String {
    json_usize_option(value.and_then(|transport| transport.process_count))
}
