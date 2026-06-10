use crate::json_cli::json_string;

use super::super::CheckIssueSampleFilter;

pub(super) struct CheckIssueSample {
    pub(super) check: &'static str,
    pub(super) kind: &'static str,
    pub(super) location: String,
    pub(super) message: String,
    fields: Vec<CheckIssueSampleField>,
}

impl CheckIssueSample {
    pub(super) fn new(
        check: &'static str,
        kind: &'static str,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check,
            kind,
            location: location.into(),
            message: message.into(),
            fields: Vec::new(),
        }
    }

    pub(super) fn with_field(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.fields.push(CheckIssueSampleField {
            name,
            value: value.into(),
        });
        self
    }
}

struct CheckIssueSampleField {
    name: &'static str,
    value: String,
}

pub(crate) struct IssueSamplesJson {
    pub(crate) unfiltered_total: usize,
    pub(crate) total: usize,
    pub(crate) emitted: usize,
    pub(crate) samples: String,
}

impl IssueSamplesJson {
    pub(crate) fn truncated(&self) -> bool {
        self.emitted < self.total
    }
}

pub(super) fn filter_includes(filter: &CheckIssueSampleFilter, sample: &CheckIssueSample) -> bool {
    filter
        .check
        .as_deref()
        .is_none_or(|check| check == sample.check)
        && filter
            .kind
            .as_deref()
            .is_none_or(|kind| kind == sample.kind)
}

pub(super) fn sample_json(sample: CheckIssueSample) -> String {
    let mut fields = vec![
        format!("\"check\": \"{}\"", json_string(sample.check)),
        format!("\"kind\": \"{}\"", json_string(sample.kind)),
        format!("\"location\": \"{}\"", json_string(&sample.location)),
        format!("\"message\": \"{}\"", json_string(&sample.message)),
    ];
    fields.extend(sample.fields.into_iter().map(|field| {
        format!(
            "\"{}\": \"{}\"",
            json_string(field.name),
            json_string(&field.value)
        )
    }));

    format!("    {{{}}}", fields.join(", "))
}
