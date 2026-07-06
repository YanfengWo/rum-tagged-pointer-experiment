use rum_runtime::kernel_api::BuiltinSpec;

use crate::toml;

pub(crate) const TOML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FromToml",
        "Parses rum's documented fail-closed TOML subset into exact rum values.",
        toml::eval_from_toml,
    ),
    BuiltinSpec::ordinary(
        "ToToml",
        "Renders the TOML-representable subset of rum Associations as TOML text.",
        toml::eval_to_toml,
    ),
];
