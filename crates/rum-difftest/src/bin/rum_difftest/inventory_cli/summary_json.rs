use std::path::Path;

use rum_difftest::inventory::{
    FixtureInventoryReport, FixtureInventoryTotals, InventoryBaselineReport, LaneFixtureInventory,
    display_path,
};

use crate::json_cli::{SUMMARY_SCHEMA_VERSION, json_string, write_json_file};

use super::InventoryFilter;

pub(super) fn write_summary_json(
    path: &Path,
    report: &FixtureInventoryReport,
    filter: &InventoryFilter,
    baseline_path: Option<&Path>,
    baseline_report: Option<&InventoryBaselineReport>,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json_file(
        path,
        summary_json(report, filter, baseline_path, baseline_report),
    )
}

fn summary_json(
    report: &FixtureInventoryReport,
    filter: &InventoryFilter,
    baseline_path: Option<&Path>,
    baseline_report: Option<&InventoryBaselineReport>,
) -> String {
    let totals = report.totals();
    format!(
        concat!(
            "{{\n",
            "  \"lane\": \"inventory\",\n",
            "  \"schema_version\": {},\n",
            "  \"filters\": {},\n",
            "  \"totals\": {},\n",
            "  \"baseline\": {},\n",
            "  \"lanes\": [\n",
            "{}\n",
            "  ]\n",
            "}}\n"
        ),
        SUMMARY_SCHEMA_VERSION,
        filters_json(filter),
        totals_json(&totals, 2),
        baseline_json(baseline_path, baseline_report),
        report
            .lanes
            .iter()
            .map(lane_json)
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn filters_json(filter: &InventoryFilter) -> String {
    format!(
        concat!(
            "{{\n",
            "    \"lane\": {},\n",
            "    \"source\": {},\n",
            "    \"category\": {}\n",
            "  }}"
        ),
        json_string_option(filter.lane.as_deref()),
        json_string_option(filter.source.as_deref()),
        json_string_option(filter.category.as_deref()),
    )
}

fn json_string_option(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_string(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn baseline_json(path: Option<&Path>, report: Option<&InventoryBaselineReport>) -> String {
    let Some(report) = report else {
        return "null".to_string();
    };
    let path = path.map(display_path).unwrap_or_default();
    format!(
        concat!(
            "{{\n",
            "    \"path\": \"{}\",\n",
            "    \"clean\": {},\n",
            "    \"records_considered\": {},\n",
            "    \"regressions\": {},\n",
            "    \"regression_details\": [\n",
            "{}\n",
            "    ]\n",
            "  }}"
        ),
        json_string(&path),
        report.is_clean(),
        report.records_considered,
        report.regressions.len(),
        report
            .regressions
            .iter()
            .map(|regression| {
                format!(
                    concat!(
                        "      {{",
                        "\"lane\": \"{}\", ",
                        "\"dimension\": \"{}\", ",
                        "\"name\": \"{}\", ",
                        "\"issues\": [{}]",
                        "}}"
                    ),
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
            .join(",\n"),
    )
}

fn lane_json(lane: &LaneFixtureInventory) -> String {
    format!(
        concat!(
            "    {{\n",
            "      \"lane\": \"{}\",\n",
            "      \"totals\": {},\n",
            "      \"sources\": [\n",
            "{}\n",
            "      ],\n",
            "      \"categories\": [\n",
            "{}\n",
            "      ],\n",
            "      \"fixtures\": [\n",
            "{}\n",
            "      ]\n",
            "    }}"
        ),
        json_string(&lane.lane),
        totals_json(&lane.totals(), 6),
        lane.totals_by_source()
            .iter()
            .map(|(source, totals)| source_json(source, totals))
            .collect::<Vec<_>>()
            .join(",\n"),
        lane.totals_by_category()
            .iter()
            .map(|(category, totals)| category_json(category, totals))
            .collect::<Vec<_>>()
            .join(",\n"),
        lane.fixtures
            .iter()
            .map(|fixture| {
                format!(
                    concat!(
                        "        {{",
                        "\"path\": \"{}\", ",
                        "\"source\": \"{}\", ",
                        "\"category\": \"{}\", ",
                        "\"entries\": {}, ",
                        "\"normal_entries\": {}, ",
                        "\"known_difference_entries\": {}",
                        "}}"
                    ),
                    json_string(&display_path(&fixture.path)),
                    json_string(&fixture.source),
                    json_string(&fixture.category),
                    fixture.entries,
                    fixture.normal_entries,
                    fixture.known_difference_entries,
                )
            })
            .collect::<Vec<_>>()
            .join(",\n"),
    )
}

fn source_json(source: &str, totals: &FixtureInventoryTotals) -> String {
    format!(
        "        {{\"source\": \"{}\", \"totals\": {}}}",
        json_string(source),
        totals_json(totals, 8)
    )
}

fn category_json(category: &str, totals: &FixtureInventoryTotals) -> String {
    format!(
        "        {{\"category\": \"{}\", \"totals\": {}}}",
        json_string(category),
        totals_json(totals, 8)
    )
}

fn totals_json(totals: &FixtureInventoryTotals, indent: usize) -> String {
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
