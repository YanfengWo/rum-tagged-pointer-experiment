use rum_difftest::inventory::display_path;
use rum_difftest::lane_d;

use super::super::{
    CHECK_LANE_D_READINESS, CHECK_LANE_D_SMOKE, KIND_ELAPSED_REGRESSION, KIND_INVALID_ARTIFACT,
    KIND_MISSING_ARTIFACT, KIND_MISSING_RECORD, KIND_NOT_CLEAN_ARTIFACT, KIND_UNUSED_RECORD,
    KIND_VALUE_NODE_REGRESSION,
};
use super::{CheckIssueSample, LocalCheckReport};

pub(super) fn collect_lane_d_smoke_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
    let Some(smoke) = &report.lane_d_smoke else {
        return;
    };

    for case in &smoke.baseline.missing_records {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_D_SMOKE,
            KIND_MISSING_RECORD,
            format!("{}#{}", case.suite, case.index),
            case.input.clone(),
        ));
    }
    for record in &smoke.baseline.unused_records {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_D_SMOKE,
            KIND_UNUSED_RECORD,
            record.suite.clone(),
            record.input.clone(),
        ));
    }
    for regression in &smoke.baseline.elapsed_regressions {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_D_SMOKE,
            KIND_ELAPSED_REGRESSION,
            format!("{}#{}", regression.case.suite, regression.case.index),
            format!(
                "{} > max {}",
                regression.case.elapsed_micros, regression.max_elapsed_micros
            ),
        ));
    }
    for regression in &smoke.baseline.value_node_regressions {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_D_SMOKE,
            KIND_VALUE_NODE_REGRESSION,
            format!("{}#{}", regression.case.suite, regression.case.index),
            format!(
                "{} > max {}",
                regression.case.value_nodes.unwrap_or(0),
                regression.max_value_nodes
            ),
        ));
    }
}

pub(super) fn collect_lane_d_readiness_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
    let Some(readiness) = &report.lane_d_readiness else {
        return;
    };

    for entry in &readiness.readiness.entries {
        let profile = lane_d::profile_name(entry.profile);
        let threshold_profile = lane_d::threshold_profile_name(entry.threshold_profile);
        let cargo_profile = lane_d::checked_baseline_cargo_profile(entry.threshold_profile);
        let location = format!(
            "{profile}/{threshold_profile}/{}",
            display_path(&entry.path)
        );
        match &entry.status {
            lane_d::LaneDCheckedBaselineReadinessStatus::Ready { .. } => {}
            lane_d::LaneDCheckedBaselineReadinessStatus::MissingArtifact => {
                samples.push(readiness_sample(
                    KIND_MISSING_ARTIFACT,
                    location,
                    "checked baseline artifact is missing",
                    profile,
                    threshold_profile,
                    cargo_profile,
                ));
            }
            lane_d::LaneDCheckedBaselineReadinessStatus::InvalidArtifact { message } => {
                samples.push(readiness_sample(
                    KIND_INVALID_ARTIFACT,
                    location,
                    message.clone(),
                    profile,
                    threshold_profile,
                    cargo_profile,
                ));
            }
            lane_d::LaneDCheckedBaselineReadinessStatus::NotClean { report } => {
                samples.push(readiness_sample(
                    KIND_NOT_CLEAN_ARTIFACT,
                    location,
                    format!(
                        "missing={} unused={} elapsed={} value_nodes={}",
                        report.missing_records.len(),
                        report.unused_records.len(),
                        report.elapsed_regressions.len(),
                        report.value_node_regressions.len()
                    ),
                    profile,
                    threshold_profile,
                    cargo_profile,
                ));
            }
        }
    }
}

fn readiness_sample(
    kind: &'static str,
    location: String,
    message: impl Into<String>,
    profile: &'static str,
    threshold_profile: &'static str,
    cargo_profile: &'static str,
) -> CheckIssueSample {
    CheckIssueSample::new(CHECK_LANE_D_READINESS, kind, location, message)
        .with_field("profile", profile)
        .with_field("threshold_profile", threshold_profile)
        .with_field("cargo_profile", cargo_profile)
}
