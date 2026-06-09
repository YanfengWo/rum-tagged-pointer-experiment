use std::collections::BTreeMap;

use super::{
    InventoryBaseline, InventoryBaselineDimension, InventoryBaselineError, InventoryBaselineRecord,
};

const REQUIRED_FIELDS: [&str; 5] = [
    "lane",
    "min_fixtures",
    "min_entries",
    "min_normal_entries",
    "max_known_difference_entries",
];

pub fn parse_inventory_baseline(text: &str) -> Result<InventoryBaseline, InventoryBaselineError> {
    let mut records = Vec::new();
    let mut seen = BTreeMap::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = parse_fields(line)?;
        for required in REQUIRED_FIELDS {
            if !fields.contains_key(required) {
                return Err(InventoryBaselineError::MissingField {
                    line: line.to_string(),
                    field: required.to_string(),
                });
            }
        }

        let record = InventoryBaselineRecord {
            lane: fields["lane"].clone(),
            dimension: parse_dimension(line, &fields)?,
            name: parse_dimension_name(line, &fields)?.to_string(),
            min_fixtures: parse_usize(line, "min_fixtures", &fields["min_fixtures"])?,
            min_entries: parse_usize(line, "min_entries", &fields["min_entries"])?,
            min_normal_entries: parse_usize(
                line,
                "min_normal_entries",
                &fields["min_normal_entries"],
            )?,
            max_known_difference_entries: parse_usize(
                line,
                "max_known_difference_entries",
                &fields["max_known_difference_entries"],
            )?,
        };
        let key = (record.lane.clone(), record.dimension, record.name.clone());
        if seen.insert(key.clone(), ()).is_some() {
            return Err(InventoryBaselineError::DuplicateRecord {
                lane: key.0,
                dimension: key.1.field_name().to_string(),
                dimension_name: key.2,
            });
        }
        records.push(record);
    }

    Ok(InventoryBaseline { records })
}

pub fn format_inventory_baseline(baseline: &InventoryBaseline) -> String {
    let mut output = String::from(
        "# Stable non-oracle fixture coverage baseline.\n\
         # Records target either source groups (`source=...`) or semantic categories\n\
         # (`category=...`).\n\
         # `min_*` fields are lower bounds. `max_known_difference_entries` is an upper\n\
         # bound so resolved known differences can decrease without failing the gate.\n",
    );

    for record in &baseline.records {
        output.push_str(&format_inventory_baseline_record(record));
        output.push('\n');
    }

    output
}

fn format_inventory_baseline_record(record: &InventoryBaselineRecord) -> String {
    format!(
        "lane={} {}={} min_fixtures={} min_entries={} min_normal_entries={} max_known_difference_entries={}",
        record.lane,
        record.dimension.field_name(),
        record.name,
        record.min_fixtures,
        record.min_entries,
        record.min_normal_entries,
        record.max_known_difference_entries
    )
}

fn parse_fields(line: &str) -> Result<BTreeMap<String, String>, InventoryBaselineError> {
    let mut fields = BTreeMap::new();
    for part in line.split_whitespace() {
        let Some((key, value)) = part.split_once('=') else {
            return Err(InventoryBaselineError::InvalidLine {
                line: line.to_string(),
                message: format!("field `{part}` is not key=value"),
            });
        };
        fields.insert(key.to_string(), value.to_string());
    }
    Ok(fields)
}

fn parse_dimension(
    line: &str,
    fields: &BTreeMap<String, String>,
) -> Result<InventoryBaselineDimension, InventoryBaselineError> {
    match (
        fields.contains_key("source"),
        fields.contains_key("category"),
    ) {
        (true, false) => Ok(InventoryBaselineDimension::Source),
        (false, true) => Ok(InventoryBaselineDimension::Category),
        _ => Err(InventoryBaselineError::InvalidDimension {
            line: line.to_string(),
        }),
    }
}

fn parse_dimension_name<'a>(
    line: &str,
    fields: &'a BTreeMap<String, String>,
) -> Result<&'a str, InventoryBaselineError> {
    match parse_dimension(line, fields)? {
        InventoryBaselineDimension::Source => Ok(&fields["source"]),
        InventoryBaselineDimension::Category => Ok(&fields["category"]),
    }
}

fn parse_usize(line: &str, field: &str, value: &str) -> Result<usize, InventoryBaselineError> {
    value
        .parse::<usize>()
        .map_err(|_| InventoryBaselineError::InvalidInteger {
            line: line.to_string(),
            field: field.to_string(),
            value: value.to_string(),
        })
}
