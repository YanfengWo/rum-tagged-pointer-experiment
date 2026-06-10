use std::process::ExitCode;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OracleLaneFailureGateOptions {
    pub(crate) fail_on_mismatch: bool,
    pub(crate) fail_on_project_mismatch: bool,
    pub(crate) fail_on_seed_drift: bool,
    pub(crate) fail_on_project_seed_drift: bool,
    pub(crate) fail_on_resolved_known_difference: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OracleLaneFailureGateCounts {
    pub(crate) mismatches: usize,
    pub(crate) rum_failures: usize,
    pub(crate) oracle_failures: usize,
    pub(crate) project_non_matches: usize,
    pub(crate) seed_snapshots_drifted_from_oracle: usize,
    pub(crate) project_seed_snapshots_drifted_from_oracle: usize,
    pub(crate) resolved_known_differences: usize,
}

pub(crate) fn oracle_lane_failure_exit_code(
    options: OracleLaneFailureGateOptions,
    counts: OracleLaneFailureGateCounts,
) -> ExitCode {
    let has_mismatch_issues =
        counts.mismatches > 0 || counts.rum_failures > 0 || counts.oracle_failures > 0;
    if options.fail_on_mismatch && has_mismatch_issues {
        return ExitCode::FAILURE;
    }
    if options.fail_on_project_mismatch && counts.project_non_matches > 0 {
        return ExitCode::FAILURE;
    }
    if options.fail_on_seed_drift && counts.seed_snapshots_drifted_from_oracle > 0 {
        return ExitCode::FAILURE;
    }
    if options.fail_on_project_seed_drift && counts.project_seed_snapshots_drifted_from_oracle > 0 {
        return ExitCode::FAILURE;
    }
    if options.fail_on_resolved_known_difference && counts.resolved_known_differences > 0 {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
