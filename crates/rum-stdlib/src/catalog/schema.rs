use rum_runtime::kernel_api::{BuiltinSpec, HoldAttribute};

use crate::schema;

pub(crate) const SCHEMA_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "ConformSchema",
        "Validates a value against a symbolic schema and returns the normalized value with defaults filled.",
        HoldAttribute::HoldRest,
        schema::eval_conform_schema,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SchemaErrors",
        "Returns path-keyed validation errors for a value checked against a symbolic schema.",
        HoldAttribute::HoldRest,
        schema::eval_schema_errors,
    ),
];
