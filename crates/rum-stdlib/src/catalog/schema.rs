use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, HoldAttribute,
};

use crate::schema;

pub(crate) const SCHEMA_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "ConformSchema",
        "Validates a value against a symbolic schema and returns the normalized value with defaults filled.",
        AttributeSet::from_hold(HoldAttribute::HoldRest).with(Attribute::Protected),
        schema::eval_conform_schema,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "schema",
            template: "Reports a malformed schema or conformance failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "SchemaErrors",
        "Returns path-keyed validation errors for a value checked against a symbolic schema.",
        AttributeSet::from_hold(HoldAttribute::HoldRest).with(Attribute::Protected),
        schema::eval_schema_errors,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "schema",
            template: "Reports a malformed schema or conformance failure.",
        },
    ]),
];
