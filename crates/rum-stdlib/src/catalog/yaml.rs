use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::yaml;

pub(crate) const YAML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromYaml",
        "Parses rum's documented fail-closed YAML block subset into exact rum values.",
        AttributeSet::empty().with(Attribute::Protected),
        yaml::eval_from_yaml,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToYaml",
        "Renders the YAML-representable subset of rum values as YAML text.",
        AttributeSet::empty().with(Attribute::Protected),
        yaml::eval_to_yaml,
    ),
];
