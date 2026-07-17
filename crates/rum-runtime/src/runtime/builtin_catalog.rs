use std::collections::HashMap;
use std::sync::OnceLock;

use super::*;

#[path = "builtin_catalog/associations.rs"]
mod association_symbols;
mod debug;
mod functional;
mod inert_heads;
mod message_templates;
mod namespace;
mod rules;
mod scoping;
mod stdlib_attributes;
#[path = "builtin_catalog/strings.rs"]
mod string_symbols;
mod structure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinAttributeSpec {
    pub name: &'static str,
    pub attributes: AttributeSet,
}

impl BuiltinAttributeSpec {
    pub const fn new(name: &'static str, attributes: AttributeSet) -> Self {
        Self { name, attributes }
    }
}

pub fn core_builtin_specs() -> &'static [BuiltinSpec] {
    static CORE_BUILTINS: OnceLock<Vec<BuiltinSpec>> = OnceLock::new();

    CORE_BUILTINS
        .get_or_init(|| {
            let mut specs = Vec::with_capacity(
                string_symbols::STRING_BUILTINS.len()
                    + structure::STRUCTURE_BUILTINS.len()
                    + association_symbols::ASSOCIATION_BUILTINS.len()
                    + debug::DEBUG_BUILTINS.len()
                    + functional::FUNCTIONAL_BUILTINS.len()
                    + inert_heads::INERT_HEAD_BUILTINS.len()
                    + scoping::SCOPING_BUILTINS.len()
                    + rules::RULE_BUILTINS.len()
                    + namespace::NAMESPACE_BUILTINS.len(),
            );
            specs.extend_from_slice(string_symbols::STRING_BUILTINS);
            specs.extend_from_slice(structure::STRUCTURE_BUILTINS);
            specs.extend_from_slice(association_symbols::ASSOCIATION_BUILTINS);
            specs.extend_from_slice(debug::DEBUG_BUILTINS);
            specs.extend_from_slice(functional::FUNCTIONAL_BUILTINS);
            specs.extend_from_slice(inert_heads::INERT_HEAD_BUILTINS);
            specs.extend_from_slice(scoping::SCOPING_BUILTINS);
            specs.extend_from_slice(rules::RULE_BUILTINS);
            specs.extend_from_slice(namespace::NAMESPACE_BUILTINS);
            specs
        })
        .as_slice()
}

pub(super) fn default_builtin_dispatch_spec(name: &str) -> Option<BuiltinSpec> {
    static CORE_BUILTINS_BY_NAME: OnceLock<HashMap<&'static str, BuiltinSpec>> = OnceLock::new();

    CORE_BUILTINS_BY_NAME
        .get_or_init(|| {
            core_builtin_specs()
                .iter()
                .map(|spec| (spec.name, *spec))
                .collect()
        })
        .get(name)
        .copied()
}

pub(super) fn default_builtin_dispatch_spec_for_symbol(symbol: &Symbol) -> Option<BuiltinSpec> {
    if !symbol.is_system() {
        return None;
    }

    static CORE_BUILTINS_BY_SYMBOL: OnceLock<HashMap<Symbol, BuiltinSpec>> = OnceLock::new();

    CORE_BUILTINS_BY_SYMBOL
        .get_or_init(|| {
            core_builtin_specs()
                .iter()
                .map(|spec| (Symbol::system(spec.name), *spec))
                .collect()
        })
        .get(symbol)
        .copied()
}

pub(super) fn default_system_attribute_set(symbol: &Symbol) -> AttributeSet {
    // rum's private Information display wrapper holds symbols that would
    // otherwise evaluate while an InformationData expression reaches a fixed point.
    if symbol.context() == "Information`" && symbol.name() == "InformationValueForm" {
        return AttributeSet::from_hold(HoldAttribute::HoldAll);
    }

    if !symbol.is_system() {
        return AttributeSet::empty();
    }

    default_builtin_dispatch_spec_for_symbol(symbol)
        .map(|spec| spec.attributes)
        .or_else(|| stdlib_attributes::default_stdlib_attribute_set_for_symbol(symbol))
        .unwrap_or_else(AttributeSet::empty)
}

pub fn stdlib_builtin_attribute_specs() -> &'static [BuiltinAttributeSpec] {
    stdlib_attributes::stdlib_builtin_attribute_specs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_catalog_drives_both_dispatch_and_default_attributes() {
        let module = Symbol::new(SYSTEM_CONTEXT, "Module");
        let spec = default_builtin_dispatch_spec("Module").expect("Module should be cataloged");

        assert_eq!(spec.dispatch, BuiltinDispatch::SpecialForm);
        assert_eq!(
            spec.attributes,
            AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected)
        );
        assert_eq!(default_system_attribute_set(&module), spec.attributes);
    }

    #[test]
    fn moved_stdlib_system_attributes_remain_kernel_known_without_registration() {
        let plus = Symbol::new(SYSTEM_CONTEXT, "Plus");
        let attributes = default_system_attribute_set(&plus);

        assert!(attributes.contains(Attribute::Listable));
        assert!(attributes.contains(Attribute::Orderless));
        assert!(attributes.contains(Attribute::Flat));
        assert!(attributes.contains(Attribute::OneIdentity));
    }
}
