use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::fixtures::{load_text, parse_metadata_fields};

use super::{
    LaneDBaseline, LaneDBaselineError, LaneDBaselineKey, LaneDBaselineRecord, LaneDThresholdProfile,
};
use crate::lane_d::{
    parse_profile_name, parse_threshold_profile_name, profile_name, threshold_profile_name,
};

const REQUIRED_BASELINE_FIELDS: [&str; 6] = [
    "profile",
    "suite",
    "input",
    "baseline_elapsed_micros",
    "max_elapsed_micros",
    "max_value_nodes",
];

pub fn load_baseline(path: impl AsRef<Path>) -> Result<LaneDBaseline, LaneDBaselineError> {
    let path = path.as_ref();
    let text = load_text(path).map_err(|source| LaneDBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_baseline(&text)
}

pub fn write_baseline(
    path: impl AsRef<Path>,
    baseline: &LaneDBaseline,
) -> Result<(), LaneDBaselineError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LaneDBaselineError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, format_baseline(baseline)).map_err(|source| LaneDBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn format_baseline(baseline: &LaneDBaseline) -> String {
    let mut output = String::from(
        "# Lane D captured performance baseline.\n\
         # Format:\n\
         # profile=... | threshold_profile=... | suite=... | category=... | input=... | baseline_elapsed_micros=... | max_elapsed_micros=... | max_value_nodes=...\n\
         # `baseline_elapsed_micros` is measurement data from the capture run. `max_elapsed_micros` and `max_value_nodes` are checked guardrails.\n\n",
    );

    for record in &baseline.records {
        output.push_str(&format_baseline_record(record));
        output.push('\n');
    }

    output
}

pub fn parse_baseline(text: &str) -> Result<LaneDBaseline, LaneDBaselineError> {
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields = parse_metadata_fields(line, line).map_err(|source| {
            LaneDBaselineError::InvalidLine {
                line: line.to_owned(),
                message: source.to_string(),
            }
        })?;
        for required in REQUIRED_BASELINE_FIELDS {
            if !fields.contains_key(required) {
                return Err(LaneDBaselineError::MissingField {
                    line: line.to_owned(),
                    field: required.to_owned(),
                });
            }
        }

        let profile_text = required_field(&fields, "profile").to_owned();
        let profile = parse_profile_name(&profile_text).ok_or_else(|| {
            LaneDBaselineError::UnknownProfile {
                line: line.to_owned(),
                profile: profile_text.clone(),
            }
        })?;
        let threshold_profile_text = fields
            .get("threshold_profile")
            .map(String::as_str)
            .unwrap_or_else(|| threshold_profile_name(LaneDThresholdProfile::LocalDebug));
        let threshold_profile =
            parse_threshold_profile_name(threshold_profile_text).ok_or_else(|| {
                LaneDBaselineError::UnknownThresholdProfile {
                    line: line.to_owned(),
                    profile: threshold_profile_text.to_owned(),
                }
            })?;
        let suite = required_field(&fields, "suite").to_owned();
        let input = required_field(&fields, "input").to_owned();
        let baseline_elapsed_micros =
            parse_unsigned_field::<u128>(&fields, line, "baseline_elapsed_micros")?;
        let max_elapsed_micros = parse_unsigned_field::<u128>(&fields, line, "max_elapsed_micros")?;
        let max_value_nodes = parse_unsigned_field::<usize>(&fields, line, "max_value_nodes")?;
        let key = LaneDBaselineKey {
            profile,
            threshold_profile,
            suite: suite.clone(),
            input: input.clone(),
        };
        if !seen.insert(key) {
            return Err(LaneDBaselineError::DuplicateRecord {
                profile: profile_name(profile).to_owned(),
                threshold_profile: threshold_profile_name(threshold_profile).to_owned(),
                suite,
                input,
            });
        }

        records.push(LaneDBaselineRecord {
            profile,
            threshold_profile,
            suite,
            category: fields.get("category").cloned(),
            input,
            baseline_elapsed_micros,
            max_elapsed_micros,
            max_value_nodes,
            note: fields.get("note").cloned(),
        });
    }

    Ok(LaneDBaseline { records })
}

fn format_baseline_record(record: &LaneDBaselineRecord) -> String {
    let mut fields = vec![
        format!("profile={}", profile_name(record.profile)),
        format!(
            "threshold_profile={}",
            threshold_profile_name(record.threshold_profile)
        ),
        format!("suite={}", record.suite),
    ];
    if let Some(category) = &record.category {
        fields.push(format!("category={category}"));
    }
    fields.extend([
        format!("input={}", record.input),
        format!("baseline_elapsed_micros={}", record.baseline_elapsed_micros),
        format!("max_elapsed_micros={}", record.max_elapsed_micros),
        format!("max_value_nodes={}", record.max_value_nodes),
    ]);
    if let Some(note) = &record.note {
        fields.push(format!("note={note}"));
    }

    fields.join(" | ")
}

fn required_field<'a>(
    fields: &'a std::collections::BTreeMap<String, String>,
    field: &str,
) -> &'a str {
    fields
        .get(field)
        .expect("required baseline field was already validated")
}

fn parse_unsigned_field<T>(
    fields: &std::collections::BTreeMap<String, String>,
    line: &str,
    field: &str,
) -> Result<T, LaneDBaselineError>
where
    T: std::str::FromStr,
{
    let value = required_field(fields, field);
    value
        .parse::<T>()
        .map_err(|_| LaneDBaselineError::InvalidInteger {
            line: line.to_owned(),
            field: field.to_owned(),
            value: value.to_owned(),
        })
}
