use std::collections::BTreeMap;

use super::{
    FixtureInventory, FixtureInventoryReport, FixtureInventoryTotals, LaneFixtureInventory,
};

impl FixtureInventoryReport {
    pub fn totals(&self) -> FixtureInventoryTotals {
        self.lanes
            .iter()
            .fold(FixtureInventoryTotals::default(), |mut totals, lane| {
                totals.add(&lane.totals());
                totals
            })
    }
}

impl LaneFixtureInventory {
    pub fn totals(&self) -> FixtureInventoryTotals {
        self.fixtures
            .iter()
            .fold(FixtureInventoryTotals::default(), |mut totals, fixture| {
                totals.fixtures += 1;
                totals.entries += fixture.entries;
                totals.normal_entries += fixture.normal_entries;
                totals.known_difference_entries += fixture.known_difference_entries;
                totals
            })
    }

    pub fn totals_by_source(&self) -> BTreeMap<&str, FixtureInventoryTotals> {
        let mut totals = BTreeMap::new();
        for fixture in &self.fixtures {
            add_fixture_totals(&mut totals, fixture.source.as_str(), fixture);
        }
        totals
    }

    pub fn totals_by_category(&self) -> BTreeMap<&str, FixtureInventoryTotals> {
        let mut totals = BTreeMap::new();
        for fixture in &self.fixtures {
            add_fixture_totals(&mut totals, fixture.category.as_str(), fixture);
        }
        totals
    }
}

impl FixtureInventoryTotals {
    fn add(&mut self, other: &FixtureInventoryTotals) {
        self.fixtures += other.fixtures;
        self.entries += other.entries;
        self.normal_entries += other.normal_entries;
        self.known_difference_entries += other.known_difference_entries;
    }
}

fn add_fixture_totals<'a>(
    totals: &mut BTreeMap<&'a str, FixtureInventoryTotals>,
    group: &'a str,
    fixture: &FixtureInventory,
) {
    let entry = totals.entry(group).or_default();
    entry.fixtures += 1;
    entry.entries += fixture.entries;
    entry.normal_entries += fixture.normal_entries;
    entry.known_difference_entries += fixture.known_difference_entries;
}
