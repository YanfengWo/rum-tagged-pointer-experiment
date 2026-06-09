use rum_difftest::inventory::{FixtureInventoryTotals, InventoryBaselineRegression};

use crate::json_cli::json_string;

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

pub fn inventory_regression_details_json(
    regressions: &[InventoryBaselineRegression],
    indent: usize,
) -> String {
    let spaces = " ".repeat(indent);
    regressions
        .iter()
        .map(|regression| {
            format!(
                concat!(
                    "{}{{",
                    "\"lane\": \"{}\", ",
                    "\"dimension\": \"{}\", ",
                    "\"name\": \"{}\", ",
                    "\"issues\": [{}]",
                    "}}"
                ),
                spaces,
                json_string(&regression.record.lane),
                json_string(regression.record.dimension.field_name()),
                json_string(&regression.record.name),
                regression
                    .issues
                    .iter()
                    .map(|issue| format!("\"{}\"", json_string(issue)))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

#[cfg(test)]
mod tests {
    use rum_difftest::inventory::{
        InventoryBaselineDimension, InventoryBaselineRecord, InventoryBaselineRegression,
    };

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

    #[test]
    fn inventory_regression_details_json_renders_escaped_entries() {
        let regressions = vec![InventoryBaselineRegression {
            record: InventoryBaselineRecord {
                lane: "lane-a".to_string(),
                dimension: InventoryBaselineDimension::Source,
                name: "curated\"core".to_string(),
                min_fixtures: 1,
                min_entries: 1,
                min_normal_entries: 1,
                max_known_difference_entries: 0,
            },
            actual: None,
            issues: vec!["fixtures 0 < 1".to_string(), "bad \"quote\"".to_string()],
        }];

        assert_eq!(
            inventory_regression_details_json(&regressions, 2),
            concat!(
                "  {",
                "\"lane\": \"lane-a\", ",
                "\"dimension\": \"source\", ",
                "\"name\": \"curated\\\"core\", ",
                "\"issues\": [\"fixtures 0 < 1\", \"bad \\\"quote\\\"\"]",
                "}"
            )
        );
    }
}
