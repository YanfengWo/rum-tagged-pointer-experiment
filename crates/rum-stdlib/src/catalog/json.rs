use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::json;

pub(crate) const JSON_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromJson",
        "Parses an RFC 8259 JSON text string into exact rum values, failing closed on invalid JSON.",
        AttributeSet::empty().with(Attribute::Protected),
        json::eval_from_json,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToJson",
        "Renders the JSON-representable subset of rum values as a JSON text string.",
        AttributeSet::empty().with(Attribute::Protected),
        json::eval_to_json,
    ),
];
