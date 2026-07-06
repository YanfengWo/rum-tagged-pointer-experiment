use rum_runtime::kernel_api::BuiltinSpec;

use crate::json;

pub(crate) const JSON_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FromJson",
        "Parses an RFC 8259 JSON text string into exact rum values, failing closed on invalid JSON.",
        json::eval_from_json,
    ),
    BuiltinSpec::ordinary(
        "ToJson",
        "Renders the JSON-representable subset of rum values as a JSON text string.",
        json::eval_to_json,
    ),
];
