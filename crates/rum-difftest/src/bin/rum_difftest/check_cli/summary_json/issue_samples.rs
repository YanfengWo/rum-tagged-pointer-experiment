use rum_difftest::inventory::display_path;
use rum_difftest::lane_d;

use crate::json_cli::json_string;

use super::super::LocalCheckReport;

struct CheckIssueSample {
    check: &'static str,
    kind: &'static str,
    location: String,
    message: String,
}

pub(super) struct IssueSamplesJson {
    pub(super) total: usize,
    pub(super) emitted: usize,
    pub(super) samples: String,
}

impl IssueSamplesJson {
    pub(super) fn truncated(&self) -> bool {
        self.emitted < self.total
    }
}

pub(super) fn issue_samples_json(
    report: &LocalCheckReport,
    max_issue_samples: usize,
) -> IssueSamplesJson {
    let mut samples = Vec::new();
    collect_inventory_samples(report, &mut samples);
    collect_lane_c_samples(report, &mut samples);
    collect_artifact_hygiene_samples(report, &mut samples);
    collect_lane_d_smoke_samples(report, &mut samples);
    collect_lane_d_readiness_samples(report, &mut samples);

    let total = samples.len();
    let rendered_samples = samples
        .into_iter()
        .take(max_issue_samples)
        .map(sample_json)
        .collect::<Vec<_>>();
    let emitted = rendered_samples.len();

    IssueSamplesJson {
        total,
        emitted,
        samples: rendered_samples.join(",\n"),
    }
}

fn collect_inventory_samples(report: &LocalCheckReport, samples: &mut Vec<CheckIssueSample>) {
    for regression in &report.inventory_baseline.regressions {
        for issue in &regression.issues {
            samples.push(CheckIssueSample {
                check: "inventory-baseline",
                kind: "regression",
                location: format!(
                    "{}/{}/{}",
                    regression.record.lane,
                    regression.record.dimension.field_name(),
                    regression.record.name
                ),
                message: issue.clone(),
            });
        }
    }
}

fn collect_lane_c_samples(report: &LocalCheckReport, samples: &mut Vec<CheckIssueSample>) {
    let ledger = &report.known_differences;
    for reference in &ledger.missing_ledger_entries {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "missing_ledger_entry",
            location: reference.fixture.display().to_string(),
            message: reference.id.clone(),
        });
    }
    for record in &ledger.unreferenced_ledger_entries {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "unreferenced_ledger_entry",
            location: record
                .fields
                .get("fixture")
                .cloned()
                .unwrap_or_else(|| "<unknown fixture>".to_string()),
            message: record.id.clone(),
        });
    }
    for record in &ledger.unreferenced_retired_seed_entries {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "unreferenced_retired_seed_entry",
            location: record.seed_fixture().display().to_string(),
            message: record.id.clone(),
        });
    }
    for mismatch in &ledger.fixture_mismatches {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "fixture_mismatch",
            location: mismatch.actual_fixture.display().to_string(),
            message: format!(
                "{} expected {}",
                mismatch.id,
                mismatch.ledger_fixture.display()
            ),
        });
    }
    for mismatch in &ledger.metadata_mismatches {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "metadata_mismatch",
            location: mismatch.id.clone(),
            message: format!(
                "{} expected {}, got {}",
                mismatch.field, mismatch.ledger_value, mismatch.fixture_value
            ),
        });
    }
    for mismatch in &ledger.retired_seed_fixture_mismatches {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "retired_seed_fixture_mismatch",
            location: mismatch.actual_fixture.display().to_string(),
            message: format!(
                "{} expected {}",
                mismatch.id,
                mismatch.ledger_fixture.display()
            ),
        });
    }
    for issue in &ledger.tracking_target_issues {
        samples.push(CheckIssueSample {
            check: "lane-c-ledger",
            kind: "tracking_target_issue",
            location: issue.target.clone(),
            message: format!("{}: {}", issue.id, issue.message),
        });
    }
}

fn collect_artifact_hygiene_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
    for issue in &report.artifact_hygiene_issues {
        samples.push(CheckIssueSample {
            check: "artifact-hygiene",
            kind: "oracle_artifact",
            location: issue.path.display().to_string(),
            message: issue.reason.clone(),
        });
    }
}

fn collect_lane_d_smoke_samples(report: &LocalCheckReport, samples: &mut Vec<CheckIssueSample>) {
    let Some(smoke) = &report.lane_d_smoke else {
        return;
    };

    for case in &smoke.baseline.missing_records {
        samples.push(CheckIssueSample {
            check: "lane-d-smoke",
            kind: "missing_record",
            location: format!("{}#{}", case.suite, case.index),
            message: case.input.clone(),
        });
    }
    for record in &smoke.baseline.unused_records {
        samples.push(CheckIssueSample {
            check: "lane-d-smoke",
            kind: "unused_record",
            location: record.suite.clone(),
            message: record.input.clone(),
        });
    }
    for regression in &smoke.baseline.elapsed_regressions {
        samples.push(CheckIssueSample {
            check: "lane-d-smoke",
            kind: "elapsed_regression",
            location: format!("{}#{}", regression.case.suite, regression.case.index),
            message: format!(
                "{} > max {}",
                regression.case.elapsed_micros, regression.max_elapsed_micros
            ),
        });
    }
    for regression in &smoke.baseline.value_node_regressions {
        samples.push(CheckIssueSample {
            check: "lane-d-smoke",
            kind: "value_node_regression",
            location: format!("{}#{}", regression.case.suite, regression.case.index),
            message: format!(
                "{} > max {}",
                regression.case.value_nodes.unwrap_or(0),
                regression.max_value_nodes
            ),
        });
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
        let location = format!(
            "{profile}/{threshold_profile}/{}",
            display_path(&entry.path)
        );
        match &entry.status {
            lane_d::LaneDCheckedBaselineReadinessStatus::Ready { .. } => {}
            lane_d::LaneDCheckedBaselineReadinessStatus::MissingArtifact => {
                samples.push(CheckIssueSample {
                    check: "lane-d-readiness",
                    kind: "missing_artifact",
                    location,
                    message: "checked baseline artifact is missing".to_string(),
                });
            }
            lane_d::LaneDCheckedBaselineReadinessStatus::InvalidArtifact { message } => {
                samples.push(CheckIssueSample {
                    check: "lane-d-readiness",
                    kind: "invalid_artifact",
                    location,
                    message: message.clone(),
                });
            }
            lane_d::LaneDCheckedBaselineReadinessStatus::NotClean { report } => {
                samples.push(CheckIssueSample {
                    check: "lane-d-readiness",
                    kind: "not_clean_artifact",
                    location,
                    message: format!(
                        "missing={} unused={} elapsed={} value_nodes={}",
                        report.missing_records.len(),
                        report.unused_records.len(),
                        report.elapsed_regressions.len(),
                        report.value_node_regressions.len()
                    ),
                });
            }
        }
    }
}

fn sample_json(sample: CheckIssueSample) -> String {
    format!(
        concat!(
            "    {{",
            "\"check\": \"{}\", ",
            "\"kind\": \"{}\", ",
            "\"location\": \"{}\", ",
            "\"message\": \"{}\"",
            "}}"
        ),
        json_string(sample.check),
        json_string(sample.kind),
        json_string(&sample.location),
        json_string(&sample.message),
    )
}
