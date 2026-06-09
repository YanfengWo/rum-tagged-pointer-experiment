use std::path::Path;

use rum_difftest::oracle::OracleKernel;

use crate::json_cli::{json_string, write_json_file};

pub(super) struct OracleLaneSummaryJsonContext<'a> {
    pub(super) lane: &'a str,
    pub(super) oracle: &'a OracleKernel,
    pub(super) fixture: Option<&'a Path>,
    pub(super) limit: Option<usize>,
    pub(super) include_known_differences: bool,
    pub(super) counts: OracleLaneSummaryCounts,
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
            "    \"seed_snapshots_drifted_from_oracle\": {}\n",
            "  }}\n",
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
    )
}

fn json_path_option(value: Option<&Path>) -> String {
    value
        .map(|path| format!("\"{}\"", json_string(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_string())
}

fn json_usize_option(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}
