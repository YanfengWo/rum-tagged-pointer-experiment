use super::{LaneBStepReport, LaneBStepStatus, LaneBSummary, PreparedLaneBFixture};

pub(super) fn record_step_result(summary: &mut LaneBSummary, step: &LaneBStepReport) {
    summary.compared += 1;
    match step.status {
        LaneBStepStatus::Match => summary.matches += 1,
        LaneBStepStatus::Mismatch => summary.mismatches += 1,
        LaneBStepStatus::RumFailure => summary.rum_failures += 1,
        LaneBStepStatus::OracleFailure => summary.oracle_failures += 1,
    }

    if step.known_difference.is_some() {
        match step.status {
            LaneBStepStatus::Match => summary.resolved_known_differences += 1,
            LaneBStepStatus::Mismatch
            | LaneBStepStatus::RumFailure
            | LaneBStepStatus::OracleFailure => summary.confirmed_known_differences += 1,
        }
    }

    match step.seed_matches_oracle {
        Some(true) => {
            summary.seed_snapshots_checked += 1;
            summary.seed_snapshots_matched_oracle += 1;
        }
        Some(false) => {
            summary.seed_snapshots_checked += 1;
            summary.seed_snapshots_drifted_from_oracle += 1;
        }
        None => {}
    }
}

pub(super) fn record_oracle_session_batches(
    summary: &mut LaneBSummary,
    fixtures: &[PreparedLaneBFixture],
) {
    for fixture in fixtures {
        if fixture.steps.is_empty() {
            continue;
        }
        summary.oracle_session_batches += 1;
        summary.oracle_session_steps += fixture.steps.len();
    }
}
