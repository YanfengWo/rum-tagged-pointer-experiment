use super::super::{CHECK_INVENTORY_BASELINE, KIND_REGRESSION};
use super::{CheckIssueSample, LocalCheckReport};

pub(super) fn collect_inventory_samples(
    report: &LocalCheckReport,
    samples: &mut Vec<CheckIssueSample>,
) {
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
