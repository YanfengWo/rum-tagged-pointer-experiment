use rum_difftest::artifact_hygiene::ArtifactHygieneIssue;
use rum_difftest::inventory::display_path;
use rum_difftest::lane_d;

use super::super::super::LocalCheckReport;
use super::{
    CHECK_ARTIFACT_HYGIENE, CHECK_INVENTORY_BASELINE, CHECK_LANE_C_LEDGER, CHECK_LANE_D_READINESS,
    CHECK_LANE_D_SMOKE, CheckIssueSample, KIND_ELAPSED_REGRESSION, KIND_FIXTURE_MISMATCH,
    KIND_INVALID_ARTIFACT, KIND_METADATA_MISMATCH, KIND_MISSING_ARTIFACT,
    KIND_MISSING_LEDGER_ENTRY, KIND_MISSING_RECORD, KIND_NOT_CLEAN_ARTIFACT,
    KIND_OPEN_KNOWN_DIFFERENCE, KIND_REGRESSION, KIND_RETIRED_SEED_FIXTURE_MISMATCH,
    KIND_TRACKING_TARGET_ISSUE, KIND_UNREFERENCED_LEDGER_ENTRY,
    KIND_UNREFERENCED_RETIRED_SEED_ENTRY, KIND_UNUSED_RECORD, KIND_VALUE_NODE_REGRESSION,
};

pub(super) fn collect_issue_samples(report: &LocalCheckReport) -> Vec<CheckIssueSample> {
    let mut samples = Vec::new();
    collect_inventory_samples(report, &mut samples);
    collect_lane_c_samples(report, &mut samples);
    collect_artifact_hygiene_samples(report, &mut samples);
    collect_lane_d_smoke_samples(report, &mut samples);
    collect_lane_d_readiness_samples(report, &mut samples);
    samples
}

fn collect_inventory_samples(report: &LocalCheckReport, samples: &mut Vec<CheckIssueSample>) {
    for regression in &report.inventory_baseline.regressions {
        for issue in &regression.issues {
            samples.push(CheckIssueSample::new(
                CHECK_INVENTORY_BASELINE,
                KIND_REGRESSION,
                format!(
                    "{}/{}/{}",
                    regression.record.lane,
                    regression.record.dimension.field_name(),
                    regression.record.name
                ),
                issue.clone(),
            ));
        }
    }
}

fn collect_lane_c_samples(report: &LocalCheckReport, samples: &mut Vec<CheckIssueSample>) {
    let ledger = &report.known_differences;
    for reference in &ledger.missing_ledger_entries {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_MISSING_LEDGER_ENTRY,
            reference.fixture.display().to_string(),
            reference.id.clone(),
        ));
    }
    for record in &ledger.unreferenced_ledger_entries {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_UNREFERENCED_LEDGER_ENTRY,
            record
                .fields
                .get("fixture")
                .cloned()
                .unwrap_or_else(|| "<unknown fixture>".to_string()),
            record.id.clone(),
        ));
    }
    for record in &ledger.unreferenced_retired_seed_entries {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_UNREFERENCED_RETIRED_SEED_ENTRY,
            record.seed_fixture().display().to_string(),
            record.id.clone(),
        ));
    }
    for mismatch in &ledger.fixture_mismatches {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_FIXTURE_MISMATCH,
            mismatch.actual_fixture.display().to_string(),
            format!(
                "{} expected {}",
                mismatch.id,
                mismatch.ledger_fixture.display()
            ),
        ));
    }
    for mismatch in &ledger.metadata_mismatches {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_METADATA_MISMATCH,
            mismatch.id.clone(),
            format!(
                "{} expected {}, got {}",
                mismatch.field, mismatch.ledger_value, mismatch.fixture_value
            ),
        ));
    }
    for mismatch in &ledger.retired_seed_fixture_mismatches {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_RETIRED_SEED_FIXTURE_MISMATCH,
            mismatch.actual_fixture.display().to_string(),
            format!(
                "{} expected {}",
                mismatch.id,
                mismatch.ledger_fixture.display()
            ),
        ));
    }
    for issue in &ledger.tracking_target_issues {
        samples.push(CheckIssueSample::new(
            CHECK_LANE_C_LEDGER,
            KIND_TRACKING_TARGET_ISSUE,
            issue.target.clone(),
            format!("{}: {}", issue.id, issue.message),
        ));
    }
    if report.fail_on_open_known_difference {
        for record in &ledger.records {
            if record
                .fields
                .get("status")
                .is_some_and(|status| status == "open")
            {
                samples.push(CheckIssueSample::new(
                    CHECK_LANE_C_LEDGER,
                    KIND_OPEN_KNOWN_DIFFERENCE,
                    record
                        .fields
                        .get("fixture")
                        .cloned()
                        .unwrap_or_else(|| "<unknown fixture>".to_string()),
                    record.id.clone(),
                ));
            }
        }
    }
}

fn collect_artifact_hygiene_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
    collect_artifact_hygiene_issue_samples(&report.artifact_hygiene_issues, samples);
}

pub(super) fn collect_artifact_hygiene_issue_samples(
    issues: &[ArtifactHygieneIssue],
    samples: &mut Vec<CheckIssueSample>,
) {
    for issue in issues {
        samples.push(CheckIssueSample::new(
            CHECK_ARTIFACT_HYGIENE,
            issue.kind.as_str(),
            issue.path.display().to_string(),
            issue.reason.clone(),
        ));
    }
}

fn collect_lane_d_smoke_samples(report: &LocalCheckReport, samples: &mut Vec<CheckIssueSample>) {
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

fn collect_lane_d_readiness_samples(
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
