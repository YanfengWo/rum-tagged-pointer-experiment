use rum_difftest::inventory::FixtureInventoryTotals;

pub fn inventory_totals_json(totals: &FixtureInventoryTotals, indent: usize) -> String {
    let spaces = " ".repeat(indent);
    format!(
        concat!(
            "{{\n",
            "{0}  \"fixtures\": {1},\n",
            "{0}  \"entries\": {2},\n",
            "{0}  \"normal_entries\": {3},\n",
            "{0}  \"known_difference_entries\": {4}\n",
            "{0}}}"
        ),
        spaces,
        totals.fixtures,
        totals.entries,
        totals.normal_entries,
        totals.known_difference_entries,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_totals_json_renders_stable_shape() {
        let totals = FixtureInventoryTotals {
            fixtures: 2,
            entries: 11,
            normal_entries: 8,
            known_difference_entries: 3,
        };

        assert_eq!(
            inventory_totals_json(&totals, 2),
            concat!(
                "{\n",
                "    \"fixtures\": 2,\n",
                "    \"entries\": 11,\n",
                "    \"normal_entries\": 8,\n",
                "    \"known_difference_entries\": 3\n",
                "  }"
            )
        );
    }
}
