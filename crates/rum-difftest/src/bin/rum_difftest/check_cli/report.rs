use std::path::{Path, PathBuf};

use rum_difftest::artifact_hygiene::ArtifactHygieneIssue;
use rum_difftest::divergences::KnownDifferenceLedgerReport;
use rum_difftest::inventory::{FixtureInventoryReport, InventoryBaselineReport, display_path};
use rum_difftest::lane_d::{
    LaneDBaselineReport, LaneDCheckedBaselineReadinessReport, LaneDSummary,
};

use crate::lane_d_json;

pub(super) struct LocalCheckReport {
    pub(super) inventory_baseline_path: PathBuf,
    pub(super) inventory: FixtureInventoryReport,
    pub(super) inventory_baseline: InventoryBaselineReport,
    pub(super) known_differences: KnownDifferenceLedgerReport,
    pub(super) fail_on_open_known_difference: bool,
    pub(super) artifact_hygiene_root: PathBuf,
    pub(super) artifact_hygiene_issues: Vec<ArtifactHygieneIssue>,
    pub(super) lane_d_smoke: Option<LaneDSmokeCheckReport>,
    pub(super) lane_d_readiness: Option<LaneDReadinessCheckReport>,
}

pub(super) struct LaneDSmokeCheckReport {
    pub(super) baseline_path: PathBuf,
    pub(super) summary: LaneDSummary,
    pub(super) baseline: LaneDBaselineReport,
}

pub(super) struct LaneDReadinessCheckReport {
    pub(super) summary: LaneDSummary,
    pub(super) readiness: LaneDCheckedBaselineReadinessReport,
}

#[derive(Clone, Copy)]
pub(super) struct SkippedCheck {
    pub(super) name: &'static str,
    pub(super) reason: &'static str,
}

impl LocalCheckReport {
    pub(super) fn is_clean(&self) -> bool {
        self.inventory_baseline.is_clean()
            && self.lane_c_check_clean()
            && self.artifact_hygiene_issues.is_empty()
            && self
                .lane_d_smoke
                .as_ref()
                .is_none_or(LaneDSmokeCheckReport::is_clean)
            && self
                .lane_d_readiness
                .as_ref()
                .is_none_or(LaneDReadinessCheckReport::is_clean)
    }

    pub(super) fn failed_checks(&self) -> usize {
        usize::from(!self.inventory_baseline.is_clean())
            + usize::from(!self.lane_c_check_clean())
            + usize::from(!self.artifact_hygiene_issues.is_empty())
            + self
                .lane_d_smoke
                .as_ref()
                .map_or(0, |report| usize::from(!report.is_clean()))
            + self
                .lane_d_readiness
                .as_ref()
                .map_or(0, |report| usize::from(!report.is_clean()))
    }

    pub(super) fn total_checks(&self) -> usize {
        3 + usize::from(self.lane_d_smoke.is_some()) + usize::from(self.lane_d_readiness.is_some())
    }

    pub(super) fn passed_checks(&self) -> usize {
        self.total_checks() - self.failed_checks()
    }

    pub(super) fn skipped_checks(&self) -> Vec<SkippedCheck> {
        let mut skipped = Vec::new();
        if self.lane_d_smoke.is_none() {
            skipped.push(SkippedCheck {
                name: "lane-d-smoke",
                reason: "opt-in with --include-lane-d-smoke or --lane-d-smoke-baseline PATH",
            });
        }
        if self.lane_d_readiness.is_none() {
            skipped.push(SkippedCheck {
                name: "lane-d-readiness",
                reason: "opt-in with --include-lane-d-readiness",
            });
        }
        skipped
    }

    pub(super) fn skipped_checks_count(&self) -> usize {
        usize::from(self.lane_d_smoke.is_none()) + usize::from(self.lane_d_readiness.is_none())
    }

    pub(super) fn open_known_differences(&self) -> usize {
        self.known_differences.active_ledger_status_count("open")
    }

    pub(super) fn lane_c_check_clean(&self) -> bool {
        self.known_differences
            .is_clean_with_open_known_difference_gate(self.fail_on_open_known_difference)
    }
}

impl LaneDSmokeCheckReport {
    pub(super) fn is_clean(&self) -> bool {
        self.summary.failed == 0 && self.baseline.is_clean()
    }

    pub(super) fn baseline_issue_count(&self) -> usize {
        self.baseline.missing_records.len()
            + self.baseline.unused_records.len()
            + self.baseline.elapsed_regressions.len()
            + self.baseline.value_node_regressions.len()
    }
}

impl LaneDReadinessCheckReport {
    pub(super) fn is_clean(&self) -> bool {
        self.summary.failed == 0 && self.readiness.is_ready()
    }

    pub(super) fn artifact_counts(&self) -> lane_d_json::LaneDReadinessArtifactCounts {
        lane_d_json::readiness_artifact_counts(&self.readiness)
    }
}

pub(super) fn display_check_path(path: &Path) -> String {
    let path = display_path(path);
    if path.is_empty() {
        ".".to_string()
    } else {
        path
    }
}
