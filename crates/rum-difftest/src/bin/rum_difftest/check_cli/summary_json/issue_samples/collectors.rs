use super::super::super::LocalCheckReport;
use super::CheckIssueSample;

#[path = "collectors/artifact_hygiene.rs"]
mod artifact_hygiene;
#[path = "collectors/inventory.rs"]
mod inventory;
#[path = "collectors/lane_c.rs"]
mod lane_c;
#[path = "collectors/lane_d.rs"]
mod lane_d;

#[cfg(test)]
pub(super) use artifact_hygiene::collect_artifact_hygiene_issue_samples;

pub(super) fn collect_issue_samples(report: &LocalCheckReport) -> Vec<CheckIssueSample> {
    let mut samples = Vec::new();
    inventory::collect_inventory_samples(report, &mut samples);
    lane_c::collect_lane_c_samples(report, &mut samples);
    artifact_hygiene::collect_artifact_hygiene_samples(report, &mut samples);
    lane_d::collect_lane_d_smoke_samples(report, &mut samples);
    lane_d::collect_lane_d_readiness_samples(report, &mut samples);
    samples
}
