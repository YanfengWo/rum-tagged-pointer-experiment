use super::{InventoryBaseline, InventoryBaselineDimension, InventoryBaselineRecord};
use crate::inventory::{FixtureInventoryReport, FixtureInventoryTotals};

pub fn capture_inventory_baseline(inventory: &FixtureInventoryReport) -> InventoryBaseline {
    let mut records = Vec::new();

    for lane in &inventory.lanes {
        push_record_from_totals(
            &mut records,
            &lane.lane,
            InventoryBaselineDimension::Source,
            "all",
            &lane.totals(),
        );
        for (source, totals) in lane.totals_by_source() {
            push_record_from_totals(
                &mut records,
                &lane.lane,
                InventoryBaselineDimension::Source,
                source,
                &totals,
            );
        }
        for (category, totals) in lane.totals_by_category() {
            push_record_from_totals(
                &mut records,
                &lane.lane,
                InventoryBaselineDimension::Category,
                category,
                &totals,
            );
        }
    }

    InventoryBaseline { records }
}

fn push_record_from_totals(
    records: &mut Vec<InventoryBaselineRecord>,
    lane: &str,
    dimension: InventoryBaselineDimension,
    name: &str,
    totals: &FixtureInventoryTotals,
) {
    records.push(InventoryBaselineRecord {
        lane: lane.to_string(),
        dimension,
        name: name.to_string(),
        min_fixtures: totals.fixtures,
        min_entries: totals.entries,
        min_normal_entries: totals.normal_entries,
        max_known_difference_entries: totals.known_difference_entries,
    });
}
