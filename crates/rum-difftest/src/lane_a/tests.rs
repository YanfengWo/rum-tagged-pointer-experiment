use std::path::Path;

use crate::oracle::OracleEvaluation;

use super::*;

#[path = "tests/batches.rs"]
mod batches;
#[path = "tests/filters.rs"]
mod filters;
#[path = "tests/known_differences.rs"]
mod known_differences;
#[path = "tests/runner.rs"]
mod runner;

fn empty_summary() -> LaneASummary {
    LaneASummary {
        total_seen: 0,
        compared: 0,
        matches: 0,
        mismatches: 0,
        rum_failures: 0,
        oracle_failures: 0,
        skipped_known_differences: 0,
        confirmed_known_differences: 0,
        resolved_known_differences: 0,
        seed_snapshots_checked: 0,
        seed_snapshots_matched_oracle: 0,
        seed_snapshots_drifted_from_oracle: 0,
        cases: Vec::new(),
    }
}

fn prepared_case(
    input: &str,
    expected_seed: Option<&str>,
    known_difference: Option<LaneAKnownDifference>,
) -> PreparedLaneACase {
    PreparedLaneACase {
        fixture: Path::new("fixture.txt").to_path_buf(),
        input: input.to_string(),
        expected_seed: expected_seed.map(str::to_string),
        known_difference,
    }
}

fn value(result: &str, stderr: &str, printed_output: &str) -> CaseValue {
    case_from_oracle_output(OracleEvaluation {
        result: result.to_string(),
        printed_output: printed_output.to_string(),
        stderr: stderr.to_string(),
    })
}
