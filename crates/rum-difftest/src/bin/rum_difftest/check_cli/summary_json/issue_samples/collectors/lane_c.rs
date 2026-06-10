use super::super::{
    CHECK_LANE_C_LEDGER, KIND_FIXTURE_MISMATCH, KIND_METADATA_MISMATCH, KIND_MISSING_LEDGER_ENTRY,
    KIND_OPEN_KNOWN_DIFFERENCE, KIND_RETIRED_SEED_FIXTURE_MISMATCH, KIND_TRACKING_TARGET_ISSUE,
    KIND_UNREFERENCED_LEDGER_ENTRY, KIND_UNREFERENCED_RETIRED_SEED_ENTRY,
};
use super::{CheckIssueSample, LocalCheckReport};

pub(super) fn collect_lane_c_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
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
    collect_open_known_difference_samples(report, samples);
}

fn collect_open_known_difference_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
    if !report.fail_on_open_known_difference {
        return;
    }

    for record in &report.known_differences.records {
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
