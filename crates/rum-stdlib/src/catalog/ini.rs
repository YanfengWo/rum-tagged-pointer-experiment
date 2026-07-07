use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::ini;

pub(crate) const INI_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromIni",
        "Parses rum's fail-closed INI dialect into an Association of section Associations.",
        AttributeSet::empty().with(Attribute::Protected),
        ini::eval_from_ini,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToIni",
        "Renders an Association of section Associations as rum's INI dialect.",
        AttributeSet::empty().with(Attribute::Protected),
        ini::eval_to_ini,
    ),
];
