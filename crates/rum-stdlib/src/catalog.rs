#[path = "catalog_control.rs"]
mod catalog_control;
#[path = "catalog_definitions.rs"]
mod catalog_definitions;

use rum_runtime::BuiltinSpec;

pub(crate) fn core_builtins() -> impl Iterator<Item = &'static BuiltinSpec> {
    catalog_definitions::CORE_BUILTINS
        .iter()
        .chain(catalog_control::CORE_BUILTINS.iter())
}
