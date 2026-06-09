use rum_difftest::divergences::KnownDifferenceLedgerReport;

use crate::json_cli::json_string;

pub fn active_ledger_field_counts_json(
    report: &KnownDifferenceLedgerReport,
    field: &str,
    json_key: &str,
    indent: usize,
) -> String {
    let spaces = " ".repeat(indent);
    report
        .active_ledger_field_counts(field)
        .into_iter()
        .map(|(value, entries)| {
            format!(
                "{spaces}{{\"{}\": \"{}\", \"entries\": {}}}",
                json_string(json_key),
                json_string(&value),
                entries
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rum_difftest::divergences::KnownDifferenceRecord;

    use super::*;

    #[test]
    fn active_ledger_field_counts_json_renders_sorted_escaped_counts() {
        let report = KnownDifferenceLedgerReport {
            records: vec![
                record("status", "open"),
                record("status", "deferred"),
                record("status", "open"),
                record("status", "needs \"quote\""),
            ],
            retired_seed_records: Vec::new(),
            fixture_references: Vec::new(),
            missing_ledger_entries: Vec::new(),
            unreferenced_ledger_entries: Vec::new(),
            unreferenced_retired_seed_entries: Vec::new(),
            fixture_mismatches: Vec::new(),
            metadata_mismatches: Vec::new(),
            retired_seed_fixture_mismatches: Vec::new(),
            tracking_target_issues: Vec::new(),
        };

        assert_eq!(
            active_ledger_field_counts_json(&report, "status", "status", 2),
            concat!(
                "  {\"status\": \"deferred\", \"entries\": 1},\n",
                "  {\"status\": \"needs \\\"quote\\\"\", \"entries\": 1},\n",
                "  {\"status\": \"open\", \"entries\": 2}"
            )
        );
    }

    fn record(field: &str, value: &str) -> KnownDifferenceRecord {
        KnownDifferenceRecord {
            id: format!("{field}-{value}"),
            fields: BTreeMap::from([(field.to_string(), value.to_string())]),
        }
    }
}
