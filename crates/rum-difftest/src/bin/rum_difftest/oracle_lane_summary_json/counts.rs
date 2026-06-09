use rum_difftest::{lane_a, lane_b};

use super::{OracleLaneSummaryCounts, OracleLaneTransportCounts};

impl OracleLaneSummaryCounts {
    pub(crate) fn from_lane_a(
        summary: &lane_a::LaneASummary,
        transport: OracleLaneTransportCounts,
        project_non_matches: usize,
        project_seed_snapshots_drifted_from_oracle: usize,
    ) -> Self {
        Self::from_common(
            OracleLaneCommonSummaryCounts {
                total_seen: summary.total_seen,
                compared: summary.compared,
                matches: summary.matches,
                mismatches: summary.mismatches,
                rum_failures: summary.rum_failures,
                oracle_failures: summary.oracle_failures,
                skipped_known_differences: summary.skipped_known_differences,
                confirmed_known_differences: summary.confirmed_known_differences,
                resolved_known_differences: summary.resolved_known_differences,
                seed_snapshots_checked: summary.seed_snapshots_checked,
                seed_snapshots_matched_oracle: summary.seed_snapshots_matched_oracle,
                seed_snapshots_drifted_from_oracle: summary.seed_snapshots_drifted_from_oracle,
            },
            transport,
            project_non_matches,
            project_seed_snapshots_drifted_from_oracle,
            None,
            None,
        )
    }

    pub(crate) fn from_lane_b(
        summary: &lane_b::LaneBSummary,
        transport: OracleLaneTransportCounts,
        project_non_matches: usize,
        project_seed_snapshots_drifted_from_oracle: usize,
    ) -> Self {
        Self::from_common(
            OracleLaneCommonSummaryCounts {
                total_seen: summary.total_seen,
                compared: summary.compared,
                matches: summary.matches,
                mismatches: summary.mismatches,
                rum_failures: summary.rum_failures,
                oracle_failures: summary.oracle_failures,
                skipped_known_differences: summary.skipped_known_differences,
                confirmed_known_differences: summary.confirmed_known_differences,
                resolved_known_differences: summary.resolved_known_differences,
                seed_snapshots_checked: summary.seed_snapshots_checked,
                seed_snapshots_matched_oracle: summary.seed_snapshots_matched_oracle,
                seed_snapshots_drifted_from_oracle: summary.seed_snapshots_drifted_from_oracle,
            },
            transport,
            project_non_matches,
            project_seed_snapshots_drifted_from_oracle,
            Some(summary.oracle_session_batches),
            Some(summary.oracle_session_steps),
        )
    }

    fn from_common(
        common: OracleLaneCommonSummaryCounts,
        transport: OracleLaneTransportCounts,
        project_non_matches: usize,
        project_seed_snapshots_drifted_from_oracle: usize,
        oracle_session_batches: Option<usize>,
        oracle_session_steps: Option<usize>,
    ) -> Self {
        Self {
            total_seen: common.total_seen,
            compared: common.compared,
            matches: common.matches,
            mismatches: common.mismatches,
            rum_failures: common.rum_failures,
            oracle_failures: common.oracle_failures,
            oracle_processes: transport.oracle_processes,
            oracle_batches: transport.oracle_batches,
            oracle_batch_inputs: transport.oracle_batch_inputs,
            skipped_known_differences: common.skipped_known_differences,
            confirmed_known_differences: common.confirmed_known_differences,
            resolved_known_differences: common.resolved_known_differences,
            project_non_matches,
            seed_snapshots_checked: common.seed_snapshots_checked,
            seed_snapshots_matched_oracle: common.seed_snapshots_matched_oracle,
            seed_snapshots_drifted_from_oracle: common.seed_snapshots_drifted_from_oracle,
            project_seed_snapshots_drifted_from_oracle,
            oracle_session_batches,
            oracle_session_steps,
        }
    }
}

#[derive(Clone, Copy)]
struct OracleLaneCommonSummaryCounts {
    total_seen: usize,
    compared: usize,
    matches: usize,
    mismatches: usize,
    rum_failures: usize,
    oracle_failures: usize,
    skipped_known_differences: usize,
    confirmed_known_differences: usize,
    resolved_known_differences: usize,
    seed_snapshots_checked: usize,
    seed_snapshots_matched_oracle: usize,
    seed_snapshots_drifted_from_oracle: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_from_lanes_preserve_shared_and_session_fields() {
        let transport = OracleLaneTransportCounts {
            oracle_processes: 2,
            oracle_batches: 3,
            oracle_batch_inputs: 4,
        };
        let lane_a_summary = lane_a::LaneASummary {
            total_seen: 10,
            compared: 9,
            matches: 8,
            mismatches: 1,
            rum_failures: 2,
            oracle_failures: 3,
            skipped_known_differences: 4,
            confirmed_known_differences: 5,
            resolved_known_differences: 6,
            seed_snapshots_checked: 7,
            seed_snapshots_matched_oracle: 8,
            seed_snapshots_drifted_from_oracle: 9,
            cases: Vec::new(),
        };

        let lane_a_counts =
            OracleLaneSummaryCounts::from_lane_a(&lane_a_summary, transport, 11, 12);

        assert_eq!(lane_a_counts.total_seen, 10);
        assert_eq!(lane_a_counts.oracle_processes, 2);
        assert_eq!(lane_a_counts.project_non_matches, 11);
        assert_eq!(lane_a_counts.project_seed_snapshots_drifted_from_oracle, 12);
        assert_eq!(lane_a_counts.oracle_session_batches, None);
        assert_eq!(lane_a_counts.oracle_session_steps, None);

        let lane_b_summary = lane_b::LaneBSummary {
            total_seen: 20,
            compared: 19,
            matches: 18,
            mismatches: 1,
            rum_failures: 2,
            oracle_failures: 3,
            skipped_known_differences: 4,
            confirmed_known_differences: 5,
            resolved_known_differences: 6,
            seed_snapshots_checked: 7,
            seed_snapshots_matched_oracle: 8,
            seed_snapshots_drifted_from_oracle: 9,
            oracle_session_batches: 13,
            oracle_session_steps: 14,
            steps: Vec::new(),
        };

        let lane_b_counts =
            OracleLaneSummaryCounts::from_lane_b(&lane_b_summary, transport, 21, 22);

        assert_eq!(lane_b_counts.total_seen, 20);
        assert_eq!(lane_b_counts.oracle_batch_inputs, 4);
        assert_eq!(lane_b_counts.project_non_matches, 21);
        assert_eq!(lane_b_counts.project_seed_snapshots_drifted_from_oracle, 22);
        assert_eq!(lane_b_counts.oracle_session_batches, Some(13));
        assert_eq!(lane_b_counts.oracle_session_steps, Some(14));
    }
}
