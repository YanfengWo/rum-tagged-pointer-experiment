use rum_runtime::kernel_api::BuiltinSpec;

use crate::ini;

pub(crate) const INI_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FromIni",
        "Parses rum's fail-closed INI dialect into an Association of section Associations.",
        ini::eval_from_ini,
    ),
    BuiltinSpec::ordinary(
        "ToIni",
        "Renders an Association of section Associations as rum's INI dialect.",
        ini::eval_to_ini,
    ),
];
