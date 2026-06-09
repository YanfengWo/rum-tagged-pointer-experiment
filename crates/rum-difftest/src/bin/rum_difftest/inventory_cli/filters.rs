use std::fs;
use std::path::Path;

use rum_difftest::inventory::{
    FixtureInventoryReport, InventoryBaseline, InventoryBaselineDimension, InventoryBaselineRecord,
    format_inventory_baseline, write_inventory_baseline,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct InventoryFilter {
    pub(super) lane: Option<String>,
    pub(super) source: Option<String>,
    pub(super) category: Option<String>,
}

impl InventoryFilter {
    pub(super) fn is_active(&self) -> bool {
        self.lane.is_some() || self.source.is_some() || self.category.is_some()
    }

    pub(super) fn description(&self) -> String {
        [
            self.lane.as_deref().map(|value| format!("lane={value}")),
            self.source
                .as_deref()
                .map(|value| format!("source={value}")),
            self.category
                .as_deref()
                .map(|value| format!("category={value}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
    }
}

pub(super) fn apply_inventory_filter(
    report: &mut FixtureInventoryReport,
    filter: &InventoryFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    if !filter.is_active() {
        return Ok(());
    }

    report.lanes.retain(|lane| {
        filter
            .lane
            .as_deref()
            .is_none_or(|value| value == lane.lane)
    });
    for lane in &mut report.lanes {
        lane.fixtures.retain(|fixture| {
            filter
                .source
                .as_deref()
                .is_none_or(|value| value == fixture.source)
                && filter
                    .category
                    .as_deref()
                    .is_none_or(|value| value == fixture.category)
        });
    }
    report.lanes.retain(|lane| !lane.fixtures.is_empty());

    if report.totals().fixtures == 0 {
        return Err(format!(
            "inventory filters matched no fixtures: {}",
            filter.description()
        )
        .into());
    }
    Ok(())
}

pub(super) fn filter_inventory_baseline(
    baseline: &InventoryBaseline,
    filter: &InventoryFilter,
) -> InventoryBaseline {
    if !filter.is_active() {
        return baseline.clone();
    }

    InventoryBaseline {
        records: baseline
            .records
            .iter()
            .filter(|record| baseline_record_matches_filter(record, filter))
            .cloned()
            .collect(),
    }
}

pub(super) fn write_inventory_baseline_with_filter_context(
    path: &Path,
    baseline: &InventoryBaseline,
    filter: &InventoryFilter,
) -> Result<(), Box<dyn std::error::Error>> {
    if !filter.is_active() {
        write_inventory_baseline(path, baseline)?;
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut text = format!(
        "# Filtered non-oracle fixture coverage baseline.\n\
         # Filters: {}\n\
         # Use an unfiltered `rum-difftest inventory --write-baseline` for the repository-wide checked baseline.\n",
        filter.description()
    );
    text.push_str(&format_inventory_baseline(baseline));
    fs::write(path, text)?;
    Ok(())
}

fn baseline_record_matches_filter(
    record: &InventoryBaselineRecord,
    filter: &InventoryFilter,
) -> bool {
    if filter
        .lane
        .as_deref()
        .is_some_and(|lane| lane != record.lane)
    {
        return false;
    }
    if let Some(category) = filter.category.as_deref() {
        return record.dimension == InventoryBaselineDimension::Category && record.name == category;
    }
    if let Some(source) = filter.source.as_deref() {
        return record.dimension == InventoryBaselineDimension::Source && record.name == source;
    }
    true
}
