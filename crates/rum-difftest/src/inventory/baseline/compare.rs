use std::collections::BTreeMap;

use super::{
    InventoryBaseline, InventoryBaselineDimension, InventoryBaselineRecord,
    InventoryBaselineRegression, InventoryBaselineReport,
};
use crate::inventory::{FixtureInventoryReport, FixtureInventoryTotals};

pub fn compare_inventory_baseline(
    inventory: &FixtureInventoryReport,
    baseline: &InventoryBaseline,
) -> InventoryBaselineReport {
    let actual = actual_totals_by_lane_dimension(inventory);
    let mut regressions = Vec::new();

    for record in &baseline.records {
        let key = (record.lane.clone(), record.dimension, record.name.clone());
        let actual_totals = actual.get(&key).cloned();
        let mut issues = Vec::new();
        match &actual_totals {
            Some(totals) => collect_regression_issues(record, totals, &mut issues),
            None => issues.push(format!(
                "missing lane/{} in current inventory",
                record.dimension.missing_message_name()
            )),
        }
        if !issues.is_empty() {
            regressions.push(InventoryBaselineRegression {
                record: record.clone(),
                actual: actual_totals,
                issues,
            });
        }
    }

    InventoryBaselineReport {
        records_considered: baseline.records.len(),
        regressions,
    }
}

fn actual_totals_by_lane_dimension(
    inventory: &FixtureInventoryReport,
) -> BTreeMap<(String, InventoryBaselineDimension, String), FixtureInventoryTotals> {
    let mut totals = BTreeMap::new();
    for lane in &inventory.lanes {
        totals.insert(
            (
                lane.lane.clone(),
                InventoryBaselineDimension::Source,
                "all".to_string(),
            ),
            lane.totals(),
        );
        for (source, source_totals) in lane.totals_by_source() {
            totals.insert(
                (
                    lane.lane.clone(),
                    InventoryBaselineDimension::Source,
                    source.to_string(),
                ),
                source_totals,
            );
        }
        for (category, category_totals) in lane.totals_by_category() {
            totals.insert(
                (
                    lane.lane.clone(),
                    InventoryBaselineDimension::Category,
                    category.to_string(),
                ),
                category_totals,
            );
        }
    }
    totals
}

fn collect_regression_issues(
    record: &InventoryBaselineRecord,
    actual: &FixtureInventoryTotals,
    issues: &mut Vec<String>,
) {
    push_min_issue("fixtures", actual.fixtures, record.min_fixtures, issues);
    push_min_issue("entries", actual.entries, record.min_entries, issues);
    push_min_issue(
        "normal_entries",
        actual.normal_entries,
        record.min_normal_entries,
        issues,
    );
    if actual.known_difference_entries > record.max_known_difference_entries {
        issues.push(format!(
            "known_difference_entries {} > max {}",
            actual.known_difference_entries, record.max_known_difference_entries
        ));
    }
}

fn push_min_issue(field: &str, actual: usize, min: usize, issues: &mut Vec<String>) {
    if actual < min {
        issues.push(format!("{field} {actual} < min {min}"));
    }
}
