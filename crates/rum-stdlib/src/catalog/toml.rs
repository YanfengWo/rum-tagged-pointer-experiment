use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::toml;

pub(crate) const TOML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromToml",
        "Parses rum's documented fail-closed TOML subset into exact rum values.",
        AttributeSet::empty().with(Attribute::Protected),
        toml::eval_from_toml,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToToml",
        "Renders the TOML-representable subset of rum Associations as TOML text.",
        AttributeSet::empty().with(Attribute::Protected),
        toml::eval_to_toml,
    ),
];
