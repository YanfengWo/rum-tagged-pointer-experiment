use rum_runtime::kernel_api::BuiltinSpec;

use crate::yaml;

pub(crate) const YAML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FromYaml",
        "Parses rum's documented fail-closed YAML block subset into exact rum values.",
        yaml::eval_from_yaml,
    ),
    BuiltinSpec::ordinary(
        "ToYaml",
        "Renders the YAML-representable subset of rum values as YAML text.",
        yaml::eval_to_yaml,
    ),
];
